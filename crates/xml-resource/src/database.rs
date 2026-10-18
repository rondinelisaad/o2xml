use std::{fmt, future::Future, pin::Pin};

use reqwest::{header::HeaderMap, Method, StatusCode};
use thiserror::Error;
use url::Url;
use zeroize::Zeroizing;

use super::{RemoteFetchError, RemoteResourceClient, RemoteResourcePolicy};

const MAX_CREDENTIAL_BYTES: usize = 4 * 1024;
const MAX_DATABASE_NAME_BYTES: usize = 256;
const MAX_RESOURCE_PATH_BYTES: usize = 8 * 1024;

/// Transient HTTP Basic credentials for a database account with only BaseX
/// `read` permission. Credentials are deliberately not serializable.
pub struct BaseXCredentials {
    username: String,
    password: Zeroizing<String>,
}

impl BaseXCredentials {
    pub fn basic(username: String, password: String) -> Result<Self, XmlDatabaseReadError> {
        let normalized = username.to_ascii_lowercase();
        if username.is_empty()
            || username.len() > MAX_CREDENTIAL_BYTES
            || password.len() > MAX_CREDENTIAL_BYTES
            || username.contains(['\0', ':'])
            || password.contains('\0')
            || matches!(normalized.as_str(), "admin" | "administrator" | "root")
        {
            return Err(XmlDatabaseReadError::InvalidCredentials);
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

impl fmt::Debug for BaseXCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BaseXCredentials")
            .field("username", &"[REDACTED]")
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseXDocument {
    pub url: Url,
    pub bytes: Vec<u8>,
    pub media_type: Option<String>,
}

/// Object-safe, read-only contract for XML database document retrieval.
/// Implementations intentionally expose neither arbitrary queries nor mutation.
pub trait XmlDatabaseReader: Send + Sync {
    fn read_document<'a>(
        &'a self,
        resource_path: &'a str,
        credentials: Option<&'a BaseXCredentials>,
    ) -> Pin<Box<dyn Future<Output = Result<BaseXDocument, XmlDatabaseReadError>> + Send + 'a>>;
}

#[derive(Clone, Debug)]
pub struct BaseXReadOnlyClient {
    remote: RemoteResourceClient,
    root: Url,
    database: String,
}

impl BaseXReadOnlyClient {
    /// Creates a reader scoped to one database below a configured BaseX REST
    /// root (for example, `https://db.example/rest/`).
    pub fn new(
        remote_policy: RemoteResourcePolicy,
        rest_root: &str,
        database: &str,
    ) -> Result<Self, XmlDatabaseReadError> {
        let mut root = remote_policy.approve_url(rest_root)?;
        if root.query().is_some() || root.fragment().is_some() {
            return Err(XmlDatabaseReadError::InvalidRestRoot);
        }
        if !root.path().ends_with('/') {
            let path = format!("{}/", root.path());
            root.set_path(&path);
        }
        validate_database_name(database)?;
        Ok(Self {
            remote: RemoteResourceClient::new(remote_policy),
            root,
            database: database.to_owned(),
        })
    }

    async fn read_inner(
        &self,
        resource_path: &str,
        credentials: Option<&BaseXCredentials>,
    ) -> Result<BaseXDocument, XmlDatabaseReadError> {
        let url = self.target(resource_path)?;
        let response = self
            .remote
            .send_once(
                &url,
                Method::GET,
                HeaderMap::new(),
                None,
                credentials.map(BaseXCredentials::pair),
            )
            .await?;
        if response.status().is_redirection() {
            return Err(XmlDatabaseReadError::RedirectDenied);
        }
        if response.status() != StatusCode::OK {
            return Err(XmlDatabaseReadError::HttpStatus(response.status().as_u16()));
        }
        let media_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = self.remote.read_bounded(response).await?;
        Ok(BaseXDocument {
            url,
            bytes,
            media_type,
        })
    }

    fn target(&self, resource_path: &str) -> Result<Url, XmlDatabaseReadError> {
        validate_resource_path(resource_path)?;
        self.root
            .join(&format!("{}/{resource_path}", self.database))
            .map_err(|_| XmlDatabaseReadError::InvalidResourcePath)
    }
}

impl XmlDatabaseReader for BaseXReadOnlyClient {
    fn read_document<'a>(
        &'a self,
        resource_path: &'a str,
        credentials: Option<&'a BaseXCredentials>,
    ) -> Pin<Box<dyn Future<Output = Result<BaseXDocument, XmlDatabaseReadError>> + Send + 'a>>
    {
        Box::pin(async move {
            let result = self.read_inner(resource_path, credentials).await;
            tracing::info!(
                action = "database.basex.read",
                resource_type = "xml_database_document",
                result = if result.is_ok() { "success" } else { "failure" },
                bytes = result.as_ref().map_or(0, |document| document.bytes.len()),
            );
            result
        })
    }
}

fn validate_database_name(database: &str) -> Result<(), XmlDatabaseReadError> {
    if database.is_empty()
        || database.len() > MAX_DATABASE_NAME_BYTES
        || database == "."
        || database == ".."
        || !database
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(XmlDatabaseReadError::InvalidDatabaseName);
    }
    Ok(())
}

fn validate_resource_path(path: &str) -> Result<(), XmlDatabaseReadError> {
    if path.is_empty()
        || path.len() > MAX_RESOURCE_PATH_BYTES
        || path.starts_with('/')
        || path.contains(['\\', '?', '#', '%', '\0'])
        || path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(XmlDatabaseReadError::InvalidResourcePath);
    }
    Ok(())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum XmlDatabaseReadError {
    #[error(transparent)]
    Policy(#[from] super::RemotePolicyError),
    #[error(transparent)]
    Remote(#[from] RemoteFetchError),
    #[error("BaseX REST root must not contain a query or fragment")]
    InvalidRestRoot,
    #[error("BaseX database name is invalid")]
    InvalidDatabaseName,
    #[error("BaseX resource path is invalid")]
    InvalidResourcePath,
    #[error("database credentials are invalid or name an administrative account")]
    InvalidCredentials,
    #[error("database redirects are denied to prevent credential forwarding")]
    RedirectDenied,
    #[error("database server returned HTTP status {0}")]
    HttpStatus(u16),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> Result<BaseXReadOnlyClient, XmlDatabaseReadError> {
        BaseXReadOnlyClient::new(
            RemoteResourcePolicy::new(["https://db.example".to_owned()])?,
            "https://db.example/rest",
            "articles-2026",
        )
    }

    #[test]
    fn builds_a_scoped_resource_url_without_query_capabilities() -> Result<(), XmlDatabaseReadError>
    {
        let client = client()?;
        assert_eq!(
            client.target("journal/123.xml")?.as_str(),
            "https://db.example/rest/articles-2026/journal/123.xml"
        );
        Ok(())
    }

    #[test]
    fn rejects_unsafe_database_names() -> Result<(), XmlDatabaseReadError> {
        let policy = RemoteResourcePolicy::new(["https://db.example".to_owned()])?;
        for database in ["", "../articles", "articles/news", "articles?query=."] {
            assert!(matches!(
                BaseXReadOnlyClient::new(policy.clone(), "https://db.example/rest/", database),
                Err(XmlDatabaseReadError::InvalidDatabaseName)
            ));
        }
        Ok(())
    }

    #[test]
    fn rejects_traversal_and_query_resource_paths() -> Result<(), XmlDatabaseReadError> {
        let client = client()?;
        for path in [
            "",
            "/article.xml",
            "../article.xml",
            "journal//article.xml",
            "article.xml?query=delete node .",
            "article%2exml",
        ] {
            assert_eq!(
                client.target(path),
                Err(XmlDatabaseReadError::InvalidResourcePath)
            );
        }
        Ok(())
    }

    #[test]
    fn rejects_administrative_accounts_and_redacts_credentials() -> Result<(), XmlDatabaseReadError>
    {
        for username in ["root", "ADMIN", "Administrator"] {
            assert!(matches!(
                BaseXCredentials::basic(username.to_owned(), "not-a-real-secret".to_owned()),
                Err(XmlDatabaseReadError::InvalidCredentials)
            ));
        }
        let credentials =
            BaseXCredentials::basic("o2xml-reader".to_owned(), "not-a-real-secret".to_owned())?;
        let rendered = format!("{credentials:?}");
        assert!(!rendered.contains("o2xml-reader"));
        assert!(!rendered.contains("not-a-real-secret"));
        assert!(rendered.contains("[REDACTED]"));
        Ok(())
    }
}
