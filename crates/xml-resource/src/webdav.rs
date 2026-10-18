use std::fmt;

use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, ETAG, IF_MATCH, IF_NONE_MATCH},
    Method, StatusCode,
};
use thiserror::Error;
use url::Url;
use zeroize::Zeroizing;

use super::{RemoteFetchError, RemotePolicyError, RemoteResourceClient, RemoteResourcePolicy};

const MAX_CREDENTIAL_BYTES: usize = 4 * 1024;
const MAX_ENTRIES: usize = 10_000;
const MAX_XML_DEPTH: usize = 256;

pub struct WebDavCredentials {
    username: String,
    password: Zeroizing<String>,
}

impl WebDavCredentials {
    pub fn basic(username: String, password: String) -> Result<Self, WebDavError> {
        if username.is_empty()
            || username.len() > MAX_CREDENTIAL_BYTES
            || password.len() > MAX_CREDENTIAL_BYTES
            || username.contains('\0')
            || username.contains(':')
            || password.contains('\0')
        {
            return Err(WebDavError::InvalidCredentials);
        }
        Ok(Self {
            username,
            password: Zeroizing::new(password),
        })
    }

    fn pair(&self) -> (&str, &str) {
        (&self.username, self.password.as_str())
    }
}

impl fmt::Debug for WebDavCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDavCredentials")
            .field("username", &"[REDACTED]")
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct WebDavClient {
    remote: RemoteResourceClient,
    root: Url,
    allow_read: bool,
    allow_write: bool,
    max_upload_bytes: usize,
}

impl WebDavClient {
    pub fn new(
        remote_policy: RemoteResourcePolicy,
        root: &str,
        allow_read: bool,
        allow_write: bool,
    ) -> Result<Self, WebDavError> {
        let mut root = remote_policy.approve_url(root)?;
        if root.query().is_some() || root.fragment().is_some() {
            return Err(WebDavError::InvalidRoot);
        }
        if !root.path().ends_with('/') {
            let path = format!("{}/", root.path());
            root.set_path(&path);
        }
        Ok(Self {
            remote: RemoteResourceClient::new(remote_policy),
            root,
            allow_read,
            allow_write,
            max_upload_bytes: 16 * 1024 * 1024,
        })
    }

    pub async fn list(
        &self,
        relative: &str,
        credentials: Option<&WebDavCredentials>,
    ) -> Result<Vec<WebDavEntry>, WebDavError> {
        self.require_read()?;
        let url = self.target(relative)?;
        let mut headers = HeaderMap::new();
        headers.insert("depth", HeaderValue::from_static("1"));
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/xml; charset=utf-8"),
        );
        let body = br#"<?xml version="1.0" encoding="utf-8"?><propfind xmlns="DAV:"><prop><displayname/><resourcetype/><getetag/><getcontentlength/></prop></propfind>"#.to_vec();
        let response = self
            .remote
            .send_once(
                &url,
                Method::from_bytes(b"PROPFIND").map_err(|_| WebDavError::Protocol)?,
                headers,
                Some(body),
                credentials.map(WebDavCredentials::pair),
            )
            .await?;
        if response.status() != StatusCode::MULTI_STATUS {
            return Err(WebDavError::HttpStatus(response.status().as_u16()));
        }
        let bytes = self.remote.read_bounded(response).await?;
        parse_multistatus(&bytes)
    }

    pub async fn read(
        &self,
        relative: &str,
        credentials: Option<&WebDavCredentials>,
    ) -> Result<WebDavRead, WebDavError> {
        self.require_read()?;
        let url = self.target(relative)?;
        let response = self
            .remote
            .send_once(
                &url,
                Method::GET,
                HeaderMap::new(),
                None,
                credentials.map(WebDavCredentials::pair),
            )
            .await?;
        if !response.status().is_success() {
            return Err(WebDavError::HttpStatus(response.status().as_u16()));
        }
        let etag = header_text(response.headers(), ETAG)?;
        let bytes = self.remote.read_bounded(response).await?;
        Ok(WebDavRead { url, bytes, etag })
    }

    pub async fn write(
        &self,
        relative: &str,
        bytes: Vec<u8>,
        condition: WebDavWriteCondition,
        credentials: Option<&WebDavCredentials>,
    ) -> Result<WebDavWriteResult, WebDavError> {
        if !self.allow_write {
            return Err(WebDavError::WriteDenied);
        }
        if bytes.len() > self.max_upload_bytes {
            return Err(WebDavError::UploadTooLarge {
                limit: self.max_upload_bytes,
            });
        }
        let url = self.target(relative)?;
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/xml; charset=utf-8"),
        );
        match condition {
            WebDavWriteCondition::Match(etag) => {
                headers.insert(IF_MATCH, safe_etag(&etag)?);
            }
            WebDavWriteCondition::CreateOnly => {
                headers.insert(IF_NONE_MATCH, HeaderValue::from_static("*"));
            }
        }
        let response = self
            .remote
            .send_once(
                &url,
                Method::PUT,
                headers,
                Some(bytes),
                credentials.map(WebDavCredentials::pair),
            )
            .await?;
        if !matches!(response.status().as_u16(), 200 | 201 | 204) {
            return Err(WebDavError::HttpStatus(response.status().as_u16()));
        }
        let etag = header_text(response.headers(), ETAG)?;
        Ok(WebDavWriteResult { url, etag })
    }

    fn require_read(&self) -> Result<(), WebDavError> {
        if self.allow_read {
            Ok(())
        } else {
            Err(WebDavError::ReadDenied)
        }
    }

    fn target(&self, relative: &str) -> Result<Url, WebDavError> {
        if relative.starts_with('/')
            || relative.contains(['\\', '?', '#', '%', '\0'])
            || relative.split('/').any(|part| matches!(part, "." | ".."))
        {
            return Err(WebDavError::InvalidRelativePath);
        }
        self.root
            .join(relative)
            .map_err(|_| WebDavError::InvalidRelativePath)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebDavWriteCondition {
    Match(String),
    CreateOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebDavRead {
    pub url: Url,
    pub bytes: Vec<u8>,
    pub etag: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebDavWriteResult {
    pub url: Url,
    pub etag: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebDavEntry {
    pub href: String,
    pub display_name: Option<String>,
    pub etag: Option<String>,
    pub content_length: Option<u64>,
    pub collection: bool,
}

fn safe_etag(value: &str) -> Result<HeaderValue, WebDavError> {
    if value.is_empty()
        || value.len() > 1024
        || !value.ends_with('"')
        || (!value.starts_with('"') && !value.starts_with("W/\""))
    {
        return Err(WebDavError::InvalidEtag);
    }
    HeaderValue::from_str(value).map_err(|_| WebDavError::InvalidEtag)
}

fn header_text(
    headers: &HeaderMap,
    name: reqwest::header::HeaderName,
) -> Result<Option<String>, WebDavError> {
    headers
        .get(name)
        .map(|value| {
            value
                .to_str()
                .map(str::to_owned)
                .map_err(|_| WebDavError::Protocol)
        })
        .transpose()
}

fn parse_multistatus(bytes: &[u8]) -> Result<Vec<WebDavEntry>, WebDavError> {
    let mut reader = NsReader::from_reader(bytes);
    reader.config_mut().trim_text(true);
    let mut entries = Vec::new();
    let mut current: Option<WebDavEntry> = None;
    let mut field = None::<Vec<u8>>;
    let mut saw_multistatus = false;
    let mut depth = 0_usize;
    loop {
        let (namespace, event) = reader
            .read_resolved_event()
            .map_err(|_| WebDavError::Protocol)?;
        if matches!(namespace, ResolveResult::Unknown(_)) {
            return Err(WebDavError::Protocol);
        }
        let dav = matches!(namespace, ResolveResult::Bound(value) if value.as_ref() == b"DAV:");
        if matches!(event, Event::Start(_)) {
            depth = depth.saturating_add(1);
            if depth > MAX_XML_DEPTH {
                return Err(WebDavError::Protocol);
            }
        }
        match event {
            Event::Start(start) if dav => match start.local_name().as_ref() {
                b"multistatus" => saw_multistatus = true,
                b"response" if saw_multistatus => {
                    current = Some(WebDavEntry {
                        href: String::new(),
                        display_name: None,
                        etag: None,
                        content_length: None,
                        collection: false,
                    })
                }
                b"href" | b"displayname" | b"getetag" | b"getcontentlength" => {
                    field = Some(start.local_name().as_ref().to_vec())
                }
                b"collection" => {
                    if let Some(entry) = &mut current {
                        entry.collection = true;
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if let (Some(entry), Some(field)) = (&mut current, &field) {
                    let value = text
                        .decode()
                        .map_err(|_| WebDavError::Protocol)?
                        .into_owned();
                    match field.as_slice() {
                        b"href" => entry.href = value,
                        b"displayname" => entry.display_name = Some(value),
                        b"getetag" => entry.etag = Some(value),
                        b"getcontentlength" => {
                            entry.content_length =
                                Some(value.parse().map_err(|_| WebDavError::Protocol)?)
                        }
                        _ => {}
                    }
                }
            }
            Event::Empty(empty) if dav => {
                if empty.local_name().as_ref() == b"collection" {
                    if let Some(entry) = &mut current {
                        entry.collection = true;
                    }
                }
            }
            Event::End(end) => {
                if dav {
                    match end.local_name().as_ref() {
                        b"response" => {
                            let entry = current.take().ok_or(WebDavError::Protocol)?;
                            if entry.href.is_empty() || entries.len() >= MAX_ENTRIES {
                                return Err(WebDavError::Protocol);
                            }
                            entries.push(entry);
                        }
                        b"href" | b"displayname" | b"getetag" | b"getcontentlength" => field = None,
                        _ => {}
                    }
                }
                depth = depth.checked_sub(1).ok_or(WebDavError::Protocol)?;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if saw_multistatus {
        Ok(entries)
    } else {
        Err(WebDavError::Protocol)
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum WebDavError {
    #[error(transparent)]
    Policy(#[from] RemotePolicyError),
    #[error(transparent)]
    Remote(#[from] RemoteFetchError),
    #[error("WebDAV root URL is invalid")]
    InvalidRoot,
    #[error("WebDAV credentials are invalid")]
    InvalidCredentials,
    #[error("WebDAV read capability is denied")]
    ReadDenied,
    #[error("WebDAV write capability is denied")]
    WriteDenied,
    #[error("WebDAV relative path is invalid")]
    InvalidRelativePath,
    #[error("WebDAV ETag is invalid")]
    InvalidEtag,
    #[error("WebDAV upload exceeds the {limit}-byte limit")]
    UploadTooLarge { limit: usize },
    #[error("WebDAV server returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("WebDAV response violates the bounded protocol")]
    Protocol,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client(read: bool, write: bool) -> Result<WebDavClient, WebDavError> {
        WebDavClient::new(
            RemoteResourcePolicy::new(["https://dav.example.org".to_owned()])?,
            "https://dav.example.org/articles/",
            read,
            write,
        )
    }

    #[test]
    fn scopes_paths_and_redacts_credentials() -> Result<(), WebDavError> {
        let client = client(true, false)?;
        assert_eq!(
            client.target("2026/article.xml")?.as_str(),
            "https://dav.example.org/articles/2026/article.xml"
        );
        assert_eq!(
            client.target("../secret"),
            Err(WebDavError::InvalidRelativePath)
        );
        let credentials = WebDavCredentials::basic("editor".to_owned(), "secret".to_owned())?;
        let debug = format!("{credentials:?}");
        assert!(!debug.contains("editor"));
        assert!(!debug.contains("secret"));
        Ok(())
    }

    #[test]
    fn parses_bounded_multistatus() -> Result<(), WebDavError> {
        let xml = br#"<d:multistatus xmlns:d="DAV:"><d:response><d:href>/articles/a.xml</d:href><d:propstat><d:prop><d:displayname>A</d:displayname><d:getetag>"abc"</d:getetag><d:getcontentlength>42</d:getcontentlength><d:resourcetype/></d:prop></d:propstat></d:response><d:response><d:href>/articles/folder/</d:href><d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat></d:response></d:multistatus>"#;
        let entries = parse_multistatus(xml)?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].content_length, Some(42));
        assert!(entries[1].collection);
        Ok(())
    }

    #[test]
    fn requires_safe_write_preconditions() {
        assert!(safe_etag("\"abc\"").is_ok());
        assert_eq!(safe_etag("abc"), Err(WebDavError::InvalidEtag));
    }
}
