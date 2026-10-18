use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};
use thiserror::Error;

const MAX_TRANSFORM_SCENARIOS: usize = 128;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectManifest {
    pub project: ProjectSection,
    #[serde(default)]
    pub xml: XmlSection,
    #[serde(default)]
    pub validation: ValidationSection,
    #[serde(default)]
    pub transform: TransformSection,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSection {
    pub name: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XmlSection {
    pub catalog: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationSection {
    pub framework: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformSection {
    pub default: Option<String>,
    #[serde(default)]
    pub scenarios: Vec<TransformScenario>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformScenario {
    pub name: String,
    pub engine: TransformEngine,
    pub stylesheet: String,
    pub output: String,
    #[serde(default)]
    pub media_type: TransformMediaType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransformEngine {
    Libxslt,
    SaxonHe,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransformMediaType {
    Xml,
    #[default]
    Html,
    Text,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("invalid project manifest: {0}")]
    InvalidManifest(#[from] toml::de::Error),
    #[error("project has more than {limit} transformation scenarios")]
    TooManyTransformScenarios { limit: usize },
    #[error("transformation scenario names must be unique and contain 1 to 128 characters")]
    InvalidTransformScenarioName,
    #[error("transformation scenario path must be a safe project-relative path: {0}")]
    InvalidTransformPath(String),
    #[error("default transformation scenario is not defined: {0}")]
    UnknownDefaultTransform(String),
}

pub fn parse_manifest(source: &str) -> Result<ProjectManifest, ProjectError> {
    let manifest: ProjectManifest = toml::from_str(source)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_manifest(manifest: &ProjectManifest) -> Result<(), ProjectError> {
    if manifest.transform.scenarios.len() > MAX_TRANSFORM_SCENARIOS {
        return Err(ProjectError::TooManyTransformScenarios {
            limit: MAX_TRANSFORM_SCENARIOS,
        });
    }
    let mut names = BTreeSet::new();
    for scenario in &manifest.transform.scenarios {
        if scenario.name.is_empty()
            || scenario.name.len() > 128
            || !names.insert(scenario.name.as_str())
        {
            return Err(ProjectError::InvalidTransformScenarioName);
        }
        validate_relative_path(&scenario.stylesheet)?;
        validate_relative_path(&scenario.output)?;
    }
    if let Some(default) = manifest.transform.default.as_deref() {
        if !names.contains(default) {
            return Err(ProjectError::UnknownDefaultTransform(default.to_owned()));
        }
    }
    Ok(())
}

fn validate_relative_path(value: &str) -> Result<(), ProjectError> {
    let portable = value.replace('\\', "/");
    let invalid = value.is_empty()
        || value.len() > 4_096
        || Path::new(value).is_absolute()
        || portable.starts_with('/')
        || portable.contains("://")
        || portable
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."));
    if invalid {
        return Err(ProjectError::InvalidTransformPath(value.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_project() {
        let manifest = parse_manifest(
            r#"
                [project]
                name = "SciELO Articles"
                [xml]
                catalog = "catalog.xml"
                [validation]
                framework = "SPS-1.10"
                [transform]
                default = "jats-html"
                [[transform.scenarios]]
                name = "jats-html"
                engine = "saxon-he"
                stylesheet = "xslt/jats-html.xsl"
                output = "output/article.html"
                media_type = "html"
            "#,
        )
        .unwrap_or_default();
        assert_eq!(manifest.project.name, "SciELO Articles");
        assert_eq!(manifest.validation.framework.as_deref(), Some("SPS-1.10"));
        assert_eq!(manifest.transform.scenarios.len(), 1);
        assert_eq!(
            manifest.transform.scenarios[0].engine,
            TransformEngine::SaxonHe
        );
    }

    #[test]
    fn rejects_unsafe_and_inconsistent_transform_scenarios() {
        let unsafe_path = parse_manifest(
            r#"
                [project]
                name = "Unsafe"
                [[transform.scenarios]]
                name = "escape"
                engine = "libxslt"
                stylesheet = "../outside.xsl"
                output = "output/result.html"
            "#,
        );
        assert!(matches!(
            unsafe_path,
            Err(ProjectError::InvalidTransformPath(_))
        ));

        let unknown_default = parse_manifest(
            r#"
                [project]
                name = "Unknown"
                [transform]
                default = "missing"
            "#,
        );
        assert!(matches!(
            unknown_default,
            Err(ProjectError::UnknownDefaultTransform(_))
        ));
    }
}
