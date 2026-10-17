use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use quick_xml::{
    events::{BytesStart, Event},
    name::ResolveResult,
    reader::NsReader,
    XmlVersion,
};
use thiserror::Error;
use tracing::{debug, warn};
use url::Url;

const CATALOG_NAMESPACE: &[u8] = b"urn:oasis:names:tc:entity:xmlns:xml:catalog";
const XML_BASE: &[u8] = b"xml:base";

#[derive(Clone, Debug)]
pub struct ResourcePolicy {
    allowed_roots: Vec<PathBuf>,
    pub default_preference: IdentifierPreference,
    pub max_catalogs: usize,
    pub max_entries: usize,
    pub max_catalog_bytes: u64,
}

impl ResourcePolicy {
    pub fn offline(allowed_roots: impl IntoIterator<Item = PathBuf>) -> Result<Self, CatalogError> {
        let mut roots = Vec::new();
        for root in allowed_roots {
            roots.push(
                fs::canonicalize(&root)
                    .map_err(|source| CatalogError::Io { path: root, source })?,
            );
        }
        if roots.is_empty() {
            return Err(CatalogError::NoAllowedRoots);
        }
        Ok(Self {
            allowed_roots: roots,
            default_preference: IdentifierPreference::Public,
            max_catalogs: 32,
            max_entries: 10_000,
            max_catalog_bytes: 2 * 1024 * 1024,
        })
    }

    fn approve_path(&self, path: &Path) -> Result<PathBuf, CatalogError> {
        let canonical = fs::canonicalize(path).map_err(|source| CatalogError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if self
            .allowed_roots
            .iter()
            .any(|root| canonical.starts_with(root))
        {
            Ok(canonical)
        } else {
            Err(CatalogError::PathDenied(canonical))
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum IdentifierPreference {
    System,
    #[default]
    Public,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedResource {
    pub uri: Url,
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("at least one allowed resource root is required")]
    NoAllowedRoots,
    #[error("cannot access {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("catalog exceeds the {limit}-byte limit: {path}")]
    CatalogTooLarge { path: PathBuf, limit: u64 },
    #[error("catalog limit exceeded ({limit})")]
    TooManyCatalogs { limit: usize },
    #[error("catalog entry limit exceeded ({limit})")]
    TooManyEntries { limit: usize },
    #[error("catalog cycle detected at {0}")]
    CatalogCycle(PathBuf),
    #[error("path is outside the allowed roots: {0}")]
    PathDenied(PathBuf),
    #[error("network or unsupported URI scheme is denied: {0}")]
    SchemeDenied(Url),
    #[error("invalid catalog XML in {path}: {message}")]
    InvalidXml { path: PathBuf, message: String },
    #[error("invalid URI reference '{value}' in {path}: {message}")]
    InvalidUri {
        path: PathBuf,
        value: String,
        message: String,
    },
    #[error("required attribute '{attribute}' is missing on '{element}' in {path}")]
    MissingAttribute {
        path: PathBuf,
        element: String,
        attribute: &'static str,
    },
    #[error("catalog resolution unexpectedly produced no resource for '{0}'")]
    UnresolvedReference(String),
}

#[derive(Clone, Debug)]
pub struct CatalogResolver {
    root: Catalog,
    root_path: PathBuf,
    policy: ResourcePolicy,
}

impl CatalogResolver {
    pub fn load(path: impl AsRef<Path>, policy: ResourcePolicy) -> Result<Self, CatalogError> {
        let mut state = LoadState::default();
        let root_path = policy.approve_path(path.as_ref())?;
        let root = Catalog::load(&root_path, &policy, &mut state)?;
        debug!(
            action = "catalog.load",
            result = "success",
            catalogs = state.catalogs,
            entries = state.entries
        );
        Ok(Self {
            root,
            root_path,
            policy,
        })
    }

    #[must_use]
    pub fn catalog_path(&self) -> &Path {
        &self.root_path
    }

    pub fn resolve_uri(&self, uri: &str) -> Result<Option<ResolvedResource>, CatalogError> {
        self.resolve(self.root.resolve_uri(uri))
    }

    pub fn resolve_external(
        &self,
        system_id: Option<&str>,
        public_id: Option<&str>,
    ) -> Result<Option<ResolvedResource>, CatalogError> {
        self.resolve(self.root.resolve_external(system_id, public_id))
    }

    pub fn resolve_reference(
        &self,
        base: &ResolvedResource,
        reference: &str,
    ) -> Result<ResolvedResource, CatalogError> {
        let absolute = base
            .uri
            .join(reference)
            .map_err(|error| CatalogError::InvalidUri {
                path: base.path.clone(),
                value: reference.to_owned(),
                message: error.to_string(),
            })?;
        let candidate = self.root.resolve_uri(absolute.as_str()).unwrap_or(absolute);
        self.resolve(Some(candidate))?
            .ok_or_else(|| CatalogError::UnresolvedReference(reference.to_owned()))
    }

    /// Resolves a URI through the catalog, falling back to the URI itself.
    ///
    /// The fallback is still subject to the offline scheme and allowed-root
    /// policy, so callers can safely broker relative resources requested by an
    /// isolated standards engine.
    pub fn resolve_uri_or_direct(&self, uri: &str) -> Result<ResolvedResource, CatalogError> {
        let parsed = Url::parse(uri).map_err(|error| CatalogError::InvalidUri {
            path: self.root_path.clone(),
            value: uri.to_owned(),
            message: error.to_string(),
        })?;
        let candidate = self.root.resolve_uri(uri).unwrap_or(parsed);
        self.resolve(Some(candidate))?
            .ok_or_else(|| CatalogError::UnresolvedReference(uri.to_owned()))
    }

    fn resolve(&self, candidate: Option<Url>) -> Result<Option<ResolvedResource>, CatalogError> {
        let Some(uri) = candidate else {
            debug!(action = "catalog.resolve", result = "not_found");
            return Ok(None);
        };
        if uri.scheme() != "file" {
            warn!(
                action = "catalog.resolve",
                result = "denied",
                reason = "scheme"
            );
            return Err(CatalogError::SchemeDenied(uri));
        }
        let path = uri
            .to_file_path()
            .map_err(|()| CatalogError::SchemeDenied(uri.clone()))?;
        let path = self.policy.approve_path(&path)?;
        let uri = Url::from_file_path(&path).map_err(|()| CatalogError::InvalidUri {
            path: path.clone(),
            value: path.display().to_string(),
            message: "cannot convert canonical path to file URI".to_owned(),
        })?;
        debug!(action = "catalog.resolve", result = "success");
        Ok(Some(ResolvedResource { uri, path }))
    }
}

#[derive(Clone, Debug)]
struct Catalog {
    entries: Vec<Entry>,
    next: Vec<Catalog>,
}

impl Catalog {
    fn load(
        path: &Path,
        policy: &ResourcePolicy,
        state: &mut LoadState,
    ) -> Result<Self, CatalogError> {
        let path = policy.approve_path(path)?;
        if state.ancestry.contains(&path) {
            return Err(CatalogError::CatalogCycle(path));
        }
        state.catalogs = state.catalogs.saturating_add(1);
        if state.catalogs > policy.max_catalogs {
            return Err(CatalogError::TooManyCatalogs {
                limit: policy.max_catalogs,
            });
        }
        let metadata = fs::metadata(&path).map_err(|source| CatalogError::Io {
            path: path.clone(),
            source,
        })?;
        if metadata.len() > policy.max_catalog_bytes {
            return Err(CatalogError::CatalogTooLarge {
                path,
                limit: policy.max_catalog_bytes,
            });
        }
        let source = fs::read_to_string(&path).map_err(|source| CatalogError::Io {
            path: path.clone(),
            source,
        })?;
        let base = Url::from_file_path(&path).map_err(|()| CatalogError::InvalidUri {
            path: path.clone(),
            value: path.display().to_string(),
            message: "cannot convert catalog path to file URI".to_owned(),
        })?;

        state.ancestry.insert(path.clone());
        let parsed = parse_catalog(&source, &path, &base, policy, state);
        state.ancestry.remove(&path);
        parsed
    }

    fn resolve_uri(&self, uri: &str) -> Option<Url> {
        self.entries
            .iter()
            .find_map(|entry| match entry {
                Entry::Uri { name, target } if name == uri => Some(target.clone()),
                _ => None,
            })
            .or_else(|| longest_rewrite(&self.entries, uri, false))
            .or_else(|| {
                self.next
                    .iter()
                    .find_map(|catalog| catalog.resolve_uri(uri))
            })
    }

    fn resolve_external(&self, system_id: Option<&str>, public_id: Option<&str>) -> Option<Url> {
        if let Some(system_id) = system_id {
            if let Some(target) = self.entries.iter().find_map(|entry| match entry {
                Entry::System { id, target } if id == system_id => Some(target.clone()),
                _ => None,
            }) {
                return Some(target);
            }
            if let Some(target) = longest_rewrite(&self.entries, system_id, true) {
                return Some(target);
            }
        }
        if let Some(public_id) = public_id {
            let normalized = normalize_public_id(public_id);
            if let Some(target) = self.entries.iter().find_map(|entry| match entry {
                Entry::Public {
                    id,
                    target,
                    preferred_with_system,
                } if id == &normalized && (system_id.is_none() || *preferred_with_system) => {
                    Some(target.clone())
                }
                _ => None,
            }) {
                return Some(target);
            }
        }
        self.next
            .iter()
            .find_map(|catalog| catalog.resolve_external(system_id, public_id))
    }
}

#[derive(Clone, Debug)]
enum Entry {
    System {
        id: String,
        target: Url,
    },
    Public {
        id: String,
        target: Url,
        preferred_with_system: bool,
    },
    Uri {
        name: String,
        target: Url,
    },
    RewriteSystem {
        start: String,
        prefix: Url,
    },
    RewriteUri {
        start: String,
        prefix: Url,
    },
}

#[derive(Default)]
struct LoadState {
    catalogs: usize,
    entries: usize,
    ancestry: HashSet<PathBuf>,
}

#[derive(Default)]
struct ParsedCatalog {
    entries: Vec<Entry>,
    next_uris: Vec<Url>,
}

fn parse_catalog(
    source: &str,
    path: &Path,
    document_base: &Url,
    policy: &ResourcePolicy,
    state: &mut LoadState,
) -> Result<Catalog, CatalogError> {
    let mut reader = NsReader::from_str(source);
    reader.config_mut().trim_text(true);
    let mut bases = vec![document_base.clone()];
    let mut preferences = vec![policy.default_preference];
    let mut output = ParsedCatalog::default();
    let mut saw_root = false;

    loop {
        let event = reader
            .read_event()
            .map_err(|error| CatalogError::InvalidXml {
                path: path.to_path_buf(),
                message: error.to_string(),
            })?;
        let resolver = reader.resolver().clone();
        match event {
            Event::Start(element) => {
                let in_catalog_namespace = matches!(
                    resolver.resolve_element(element.name()).0,
                    ResolveResult::Bound(namespace) if namespace.as_ref() == CATALOG_NAMESPACE
                );
                let parent = bases.last().unwrap_or(document_base);
                let base = element_base(&element, parent, path)?;
                if !saw_root {
                    if !in_catalog_namespace || element.local_name().as_ref() != b"catalog" {
                        return Err(CatalogError::InvalidXml {
                            path: path.to_path_buf(),
                            message:
                                "root element must be catalog in the OASIS XML Catalog namespace"
                                    .to_owned(),
                        });
                    }
                    saw_root = true;
                }
                let parent_preference = preferences
                    .last()
                    .copied()
                    .unwrap_or(policy.default_preference);
                let preference = if in_catalog_namespace
                    && matches!(element.local_name().as_ref(), b"catalog" | b"group")
                {
                    element_preference(&element, parent_preference, path)?
                } else {
                    parent_preference
                };
                bases.push(base.clone());
                preferences.push(preference);
                if in_catalog_namespace {
                    parse_entry(
                        &element,
                        &base,
                        path,
                        &mut output,
                        state,
                        policy,
                        preference,
                    )?;
                }
            }
            Event::Empty(element) => {
                let in_catalog_namespace = matches!(
                    resolver.resolve_element(element.name()).0,
                    ResolveResult::Bound(namespace) if namespace.as_ref() == CATALOG_NAMESPACE
                );
                if in_catalog_namespace {
                    let parent = bases.last().unwrap_or(document_base);
                    let base = element_base(&element, parent, path)?;
                    let preference = preferences
                        .last()
                        .copied()
                        .unwrap_or(policy.default_preference);
                    parse_entry(
                        &element,
                        &base,
                        path,
                        &mut output,
                        state,
                        policy,
                        preference,
                    )?;
                }
            }
            Event::End(_) => {
                if bases.len() > 1 {
                    bases.pop();
                    preferences.pop();
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !saw_root {
        return Err(CatalogError::InvalidXml {
            path: path.to_path_buf(),
            message: "missing catalog root in the OASIS XML Catalog namespace".to_owned(),
        });
    }

    let mut next = Vec::new();
    for uri in output.next_uris {
        if uri.scheme() != "file" {
            return Err(CatalogError::SchemeDenied(uri));
        }
        let next_path = uri
            .to_file_path()
            .map_err(|()| CatalogError::SchemeDenied(uri.clone()))?;
        next.push(Catalog::load(&next_path, policy, state)?);
    }
    Ok(Catalog {
        entries: output.entries,
        next,
    })
}

fn parse_entry(
    element: &BytesStart<'_>,
    base: &Url,
    path: &Path,
    output: &mut ParsedCatalog,
    state: &mut LoadState,
    policy: &ResourcePolicy,
    preference: IdentifierPreference,
) -> Result<(), CatalogError> {
    let local_name = element.local_name();
    let name = local_name.as_ref();
    let is_catalog_entry = matches!(
        name,
        b"system" | b"public" | b"uri" | b"rewriteSystem" | b"rewriteURI" | b"nextCatalog"
    );
    let entry = match name {
        b"system" => Some(Entry::System {
            id: required_attribute(element, b"systemId", "systemId", "system", path)?,
            target: target_url(element, b"uri", "uri", "system", base, path)?,
        }),
        b"public" => Some(Entry::Public {
            id: normalize_public_id(&required_attribute(
                element,
                b"publicId",
                "publicId",
                "public",
                path,
            )?),
            target: target_url(element, b"uri", "uri", "public", base, path)?,
            preferred_with_system: preference == IdentifierPreference::Public,
        }),
        b"uri" => Some(Entry::Uri {
            name: required_attribute(element, b"name", "name", "uri", path)?,
            target: target_url(element, b"uri", "uri", "uri", base, path)?,
        }),
        b"rewriteSystem" => Some(Entry::RewriteSystem {
            start: required_attribute(
                element,
                b"systemIdStartString",
                "systemIdStartString",
                "rewriteSystem",
                path,
            )?,
            prefix: target_url(
                element,
                b"rewritePrefix",
                "rewritePrefix",
                "rewriteSystem",
                base,
                path,
            )?,
        }),
        b"rewriteURI" => Some(Entry::RewriteUri {
            start: required_attribute(
                element,
                b"uriStartString",
                "uriStartString",
                "rewriteURI",
                path,
            )?,
            prefix: target_url(
                element,
                b"rewritePrefix",
                "rewritePrefix",
                "rewriteURI",
                base,
                path,
            )?,
        }),
        b"nextCatalog" => {
            output.next_uris.push(target_url(
                element,
                b"catalog",
                "catalog",
                "nextCatalog",
                base,
                path,
            )?);
            None
        }
        _ => None,
    };
    if is_catalog_entry {
        state.entries = state.entries.saturating_add(1);
        if state.entries > policy.max_entries {
            return Err(CatalogError::TooManyEntries {
                limit: policy.max_entries,
            });
        }
    }
    if let Some(entry) = entry {
        output.entries.push(entry);
    }
    Ok(())
}

fn element_base(element: &BytesStart<'_>, base: &Url, path: &Path) -> Result<Url, CatalogError> {
    match attribute(element, XML_BASE, path)? {
        Some(value) => join_uri(base, &value, path),
        None => Ok(base.clone()),
    }
}

fn element_preference(
    element: &BytesStart<'_>,
    inherited: IdentifierPreference,
    path: &Path,
) -> Result<IdentifierPreference, CatalogError> {
    match attribute(element, b"prefer", path)?.as_deref() {
        None => Ok(inherited),
        Some("system") => Ok(IdentifierPreference::System),
        Some("public") => Ok(IdentifierPreference::Public),
        Some(value) => Err(CatalogError::InvalidXml {
            path: path.to_path_buf(),
            message: format!("invalid prefer value '{value}'"),
        }),
    }
}

fn target_url(
    element: &BytesStart<'_>,
    key: &[u8],
    attribute_name: &'static str,
    element_name: &str,
    base: &Url,
    path: &Path,
) -> Result<Url, CatalogError> {
    let value = required_attribute(element, key, attribute_name, element_name, path)?;
    join_uri(base, &value, path)
}

fn join_uri(base: &Url, value: &str, path: &Path) -> Result<Url, CatalogError> {
    base.join(value).map_err(|error| CatalogError::InvalidUri {
        path: path.to_path_buf(),
        value: value.to_owned(),
        message: error.to_string(),
    })
}

fn required_attribute(
    element: &BytesStart<'_>,
    key: &[u8],
    attribute_name: &'static str,
    element_name: &str,
    path: &Path,
) -> Result<String, CatalogError> {
    attribute(element, key, path)?.ok_or_else(|| CatalogError::MissingAttribute {
        path: path.to_path_buf(),
        element: element_name.to_owned(),
        attribute: attribute_name,
    })
}

fn attribute(
    element: &BytesStart<'_>,
    key: &[u8],
    path: &Path,
) -> Result<Option<String>, CatalogError> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| CatalogError::InvalidXml {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        if attribute.key.as_ref() == key {
            return Ok(Some(
                attribute
                    .normalized_value(XmlVersion::Implicit1_0)
                    .map_err(|error| CatalogError::InvalidXml {
                        path: path.to_path_buf(),
                        message: error.to_string(),
                    })?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn longest_rewrite(entries: &[Entry], value: &str, system: bool) -> Option<Url> {
    entries
        .iter()
        .filter_map(|entry| match entry {
            Entry::RewriteSystem { start, prefix } if system && value.starts_with(start) => {
                Some((start, prefix))
            }
            Entry::RewriteUri { start, prefix } if !system && value.starts_with(start) => {
                Some((start, prefix))
            }
            _ => None,
        })
        .max_by_key(|(start, _)| start.len())
        .and_then(|(start, prefix)| {
            let rewritten = format!("{}{}", prefix.as_str(), &value[start.len()..]);
            Url::parse(&rewritten).ok()
        })
}

fn normalize_public_id(value: &str) -> String {
    value.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn resolver(name: &str) -> Result<CatalogResolver, CatalogError> {
        let fixtures = fixtures();
        let policy = ResourcePolicy::offline([fixtures.clone()])?;
        CatalogResolver::load(fixtures.join(name), policy)
    }

    fn required(
        result: Result<Option<ResolvedResource>, CatalogError>,
    ) -> Result<ResolvedResource, Box<dyn Error>> {
        result?.ok_or_else(|| std::io::Error::other("catalog entry did not resolve").into())
    }

    #[test]
    fn resolves_exact_system_public_and_uri_entries() -> Result<(), Box<dyn Error>> {
        let resolver = resolver("catalog.xml")?;
        let system =
            required(resolver.resolve_external(Some("https://example.org/article.dtd"), None))?;
        let public =
            required(resolver.resolve_external(None, Some("  -//O2XML//DTD   ARTICLE//EN ")))?;
        let uri = required(resolver.resolve_uri("https://example.org/article.xsd"))?;

        assert!(system.path.ends_with("article.dtd"));
        assert_eq!(system.path, public.path);
        assert!(uri.path.ends_with("article.xsd"));
        Ok(())
    }

    #[test]
    fn selects_longest_rewrite_and_follows_next_catalog_last() -> Result<(), Box<dyn Error>> {
        let resolver = resolver("catalog.xml")?;
        let rewritten =
            required(resolver.resolve_uri("https://example.org/schemas/nested/article.xsd"))?;
        let chained = required(resolver.resolve_uri("https://example.org/from-next.sch"))?;

        assert!(rewritten.path.ends_with("nested/article.xsd"));
        assert!(chained.path.ends_with("rules.sch"));
        Ok(())
    }

    #[test]
    fn denies_remote_targets_and_paths_outside_allowed_roots() -> Result<(), CatalogError> {
        let remote = resolver("remote.xml")?.resolve_uri("https://example.org/remote.xsd");
        assert!(matches!(remote, Err(CatalogError::SchemeDenied(_))));

        let escaped = resolver("escape.xml")?.resolve_uri("https://example.org/escape");
        assert!(matches!(escaped, Err(CatalogError::PathDenied(_))));
        Ok(())
    }

    #[test]
    fn rejects_next_catalog_cycles() -> Result<(), CatalogError> {
        let fixtures = fixtures();
        let policy = ResourcePolicy::offline([fixtures.clone()])?;
        let result = CatalogResolver::load(fixtures.join("cycle-a.xml"), policy);
        assert!(matches!(result, Err(CatalogError::CatalogCycle(_))));
        Ok(())
    }

    #[test]
    fn honors_scoped_public_identifier_preference() -> Result<(), CatalogError> {
        let resolver = resolver("prefer.xml")?;
        let ignored = resolver.resolve_external(
            Some("https://unmapped.example/article.dtd"),
            Some("-//O2XML//DTD SYSTEM-PREFERRED//EN"),
        )?;
        let selected = resolver.resolve_external(
            Some("https://unmapped.example/article.dtd"),
            Some("-//O2XML//DTD PUBLIC-PREFERRED//EN"),
        )?;

        assert!(ignored.is_none());
        assert!(selected.is_some());
        Ok(())
    }

    #[test]
    fn resolves_relative_references_under_the_same_policy() -> Result<(), CatalogError> {
        let resolver = resolver("catalog.xml")?;
        let base = resolver
            .resolve_uri("https://example.org/article.xsd")?
            .ok_or_else(|| CatalogError::UnresolvedReference("article.xsd".to_owned()))?;
        let nested = resolver.resolve_reference(&base, "schemas/nested/article.xsd")?;

        assert!(nested.path.ends_with("schemas/nested/article.xsd"));
        Ok(())
    }
}
