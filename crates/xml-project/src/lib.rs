use serde::{Deserialize, Serialize};
use thiserror::Error;

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
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("invalid project manifest: {0}")]
    InvalidManifest(#[from] toml::de::Error),
}

pub fn parse_manifest(source: &str) -> Result<ProjectManifest, ProjectError> {
    Ok(toml::from_str(source)?)
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
            "#,
        )
        .unwrap_or_default();
        assert_eq!(manifest.project.name, "SciELO Articles");
        assert_eq!(manifest.validation.framework.as_deref(), Some("SPS-1.10"));
    }
}
