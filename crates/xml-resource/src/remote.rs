use std::time::Duration;

use reqwest::{
    header::{HeaderMap, LOCATION},
    redirect::Policy,
    Client, Method, Response, StatusCode,
};
use thiserror::Error;
use tokio::{net::lookup_host, time::timeout};
use url::Url;

use super::{RemotePolicyError, RemoteResourcePolicy};

const DNS_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const USER_AGENT: &str = "O2XML/0.1 remote-resource";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteResource {
    pub final_url: Url,
    pub bytes: Vec<u8>,
    pub media_type: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RemoteResourceClient {
    policy: RemoteResourcePolicy,
}

impl RemoteResourceClient {
    #[must_use]
    pub fn new(policy: RemoteResourcePolicy) -> Self {
        Self { policy }
    }

    pub async fn fetch(&self, url: &str) -> Result<RemoteResource, RemoteFetchError> {
        let result = self.fetch_inner(url).await;
        tracing::info!(
            action = "resource.remote.fetch",
            result = if result.is_ok() { "success" } else { "failure" },
            bytes = result.as_ref().map_or(0, |resource| resource.bytes.len()),
        );
        result
    }

    async fn fetch_inner(&self, url: &str) -> Result<RemoteResource, RemoteFetchError> {
        let mut current = self.policy.approve_url(url)?;
        if current.fragment().is_some() {
            return Err(RemoteFetchError::FragmentDenied);
        }
        for redirects in 0..=self.policy.max_redirects {
            let response = self
                .send_once(&current, Method::GET, HeaderMap::new(), None, None)
                .await?;

            if is_redirect(response.status()) {
                if redirects >= self.policy.max_redirects {
                    return Err(RemoteFetchError::TooManyRedirects);
                }
                let location = response
                    .headers()
                    .get(LOCATION)
                    .ok_or(RemoteFetchError::MissingRedirectLocation)?
                    .to_str()
                    .map_err(|_| RemoteFetchError::InvalidRedirect)?;
                current = current
                    .join(location)
                    .map_err(|_| RemoteFetchError::InvalidRedirect)?;
                current = self.policy.approve_url(current.as_str())?;
                if current.fragment().is_some() {
                    return Err(RemoteFetchError::FragmentDenied);
                }
                continue;
            }
            if !response.status().is_success() {
                return Err(RemoteFetchError::HttpStatus(response.status().as_u16()));
            }
            if response
                .content_length()
                .is_some_and(|length| length > self.policy.max_resource_bytes)
            {
                return Err(RemoteFetchError::ResourceTooLarge {
                    limit: self.policy.max_resource_bytes,
                });
            }
            let media_type = response_media_type(&response);
            let bytes = self.read_bounded(response).await?;
            return Ok(RemoteResource {
                final_url: current,
                bytes,
                media_type,
            });
        }
        Err(RemoteFetchError::TooManyRedirects)
    }

    pub(crate) async fn send_once(
        &self,
        url: &Url,
        method: Method,
        headers: HeaderMap,
        body: Option<Vec<u8>>,
        basic_auth: Option<(&str, &str)>,
    ) -> Result<Response, RemoteFetchError> {
        let approved_url = self.policy.approve_url(url.as_str())?;
        if approved_url.fragment().is_some() {
            return Err(RemoteFetchError::FragmentDenied);
        }
        let host = approved_url
            .host_str()
            .ok_or(RemotePolicyError::InvalidUrl)?
            .to_owned();
        let port = approved_url
            .port_or_known_default()
            .ok_or(RemotePolicyError::InvalidUrl)?;
        let addresses = timeout(DNS_TIMEOUT, lookup_host((host.as_str(), port)))
            .await
            .map_err(|_| RemoteFetchError::DnsTimeout)?
            .map_err(|error| RemoteFetchError::Dns(error.to_string()))?
            .collect::<Vec<_>>();
        let approved = self
            .policy
            .approve_addresses(approved_url.clone(), addresses)?;
        let pinned = approved.addresses.iter().copied().collect::<Vec<_>>();
        let client = Client::builder()
            .redirect(Policy::none())
            .no_proxy()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .resolve_to_addrs(&host, &pinned)
            .build()
            .map_err(|error| RemoteFetchError::Transport(error.to_string()))?;
        let mut request = client
            .request(method, approved_url)
            .headers(headers)
            .header(reqwest::header::USER_AGENT, USER_AGENT);
        if let Some(body) = body {
            request = request.body(body);
        }
        if let Some((username, password)) = basic_auth {
            request = request.basic_auth(username, Some(password));
        }
        request
            .send()
            .await
            .map_err(|error| RemoteFetchError::Transport(error.to_string()))
    }

    pub(crate) async fn read_bounded(
        &self,
        mut response: Response,
    ) -> Result<Vec<u8>, RemoteFetchError> {
        if response
            .content_length()
            .is_some_and(|length| length > self.policy.max_resource_bytes)
        {
            return Err(RemoteFetchError::ResourceTooLarge {
                limit: self.policy.max_resource_bytes,
            });
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| RemoteFetchError::Transport(error.to_string()))?
        {
            let next = u64::try_from(bytes.len())
                .unwrap_or(u64::MAX)
                .saturating_add(u64::try_from(chunk.len()).unwrap_or(u64::MAX));
            if next > self.policy.max_resource_bytes {
                return Err(RemoteFetchError::ResourceTooLarge {
                    limit: self.policy.max_resource_bytes,
                });
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }
}

fn response_media_type(response: &Response) -> Option<String> {
    response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn is_redirect(status: StatusCode) -> bool {
    matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RemoteFetchError {
    #[error(transparent)]
    Policy(#[from] RemotePolicyError),
    #[error("remote resource URL fragments are denied")]
    FragmentDenied,
    #[error("remote resource DNS resolution timed out")]
    DnsTimeout,
    #[error("remote resource DNS resolution failed: {0}")]
    Dns(String),
    #[error("remote resource transport failed: {0}")]
    Transport(String),
    #[error("remote resource redirect has no Location header")]
    MissingRedirectLocation,
    #[error("remote resource redirect target is invalid")]
    InvalidRedirect,
    #[error("remote resource exceeded the redirect limit")]
    TooManyRedirects,
    #[error("remote resource server returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("remote resource exceeds the {limit}-byte limit")]
    ResourceTooLarge { limit: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_only_explicit_get_redirect_statuses() {
        for status in [301, 302, 303, 307, 308] {
            assert!(is_redirect(
                StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            ));
        }
        assert!(!is_redirect(StatusCode::NOT_MODIFIED));
        assert!(!is_redirect(StatusCode::MULTIPLE_CHOICES));
    }

    #[test]
    fn rejects_fragments_before_dns_or_transport() -> Result<(), Box<dyn std::error::Error>> {
        let client = RemoteResourceClient::new(RemoteResourcePolicy::new([
            "https://example.com".to_owned()
        ])?);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        assert_eq!(
            runtime.block_on(client.fetch_inner("https://example.com/schema.xsd#part")),
            Err(RemoteFetchError::FragmentDenied)
        );
        Ok(())
    }
}
