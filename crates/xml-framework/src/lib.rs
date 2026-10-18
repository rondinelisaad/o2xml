use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_FRAMEWORKS: usize = 1_024;
const MAX_IDENTIFIER_BYTES: usize = 256;
const MAX_URI_BYTES: usize = 8 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FrameworkManifest {
    pub id: String,
    pub display_name: String,
    pub publication_standard: PublicationStandard,
    #[serde(default)]
    pub stability: FrameworkStability,
    pub grammar: ValidationStage,
    pub rules: Vec<ValidationStage>,
    pub assets: Vec<FrameworkAsset>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkStability {
    #[default]
    Stable,
    Candidate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PublicationStandard {
    pub family: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ValidationStage {
    pub id: String,
    pub source_label: String,
    pub language: ValidationLanguage,
    pub logical_uri: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationLanguage {
    Dtd,
    Xsd10,
    RelaxNg,
    Schematron,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FrameworkAsset {
    pub logical_uri: String,
    pub role: AssetRole,
    pub redistribution: Redistribution,
    pub sha256: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetRole {
    Grammar,
    Rules,
    Catalog,
    Documentation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Redistribution {
    Bundled,
    ExternalRequired,
}

#[derive(Clone, Debug, Default)]
pub struct FrameworkRegistry {
    manifests: BTreeMap<String, FrameworkManifest>,
}

impl FrameworkRegistry {
    pub fn with_builtin_scientific_frameworks() -> Result<Self, FrameworkError> {
        let mut registry = Self::default();
        registry.insert(sps_manifest(
            "SPS-1.9",
            "1.9",
            "1.1",
            FrameworkStability::Stable,
        ))?;
        registry.insert(sps_manifest(
            "SPS-1.10",
            "1.10",
            "1.3",
            FrameworkStability::Stable,
        ))?;
        registry.insert(sps_manifest(
            "SPS-1.11-candidate",
            "1.11",
            "1.4",
            FrameworkStability::Candidate,
        ))?;
        Ok(registry)
    }

    pub fn insert(&mut self, manifest: FrameworkManifest) -> Result<(), FrameworkError> {
        validate_manifest(&manifest)?;
        if self.manifests.contains_key(&manifest.id) {
            return Err(FrameworkError::DuplicateFramework);
        }
        if self.manifests.len() >= MAX_FRAMEWORKS {
            return Err(FrameworkError::TooManyFrameworks {
                limit: MAX_FRAMEWORKS,
            });
        }
        self.manifests.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&FrameworkManifest> {
        self.manifests.get(id)
    }

    pub fn validation_plan(&self, id: &str) -> Result<Vec<ValidationStage>, FrameworkError> {
        let manifest = self
            .get(id)
            .ok_or_else(|| FrameworkError::UnknownFramework(id.to_owned()))?;
        let mut stages = Vec::with_capacity(manifest.rules.len().saturating_add(1));
        stages.push(manifest.grammar.clone());
        stages.extend(manifest.rules.iter().cloned());
        Ok(stages)
    }
}

fn sps_manifest(
    id: &str,
    sps_version: &str,
    jats_version: &str,
    stability: FrameworkStability,
) -> FrameworkManifest {
    let grammar_uri = format!("urn:o2xml:framework:sps:{sps_version}:jats");
    let rules_uri = format!("urn:o2xml:framework:sps:{sps_version}:schematron");
    FrameworkManifest {
        id: id.to_owned(),
        display_name: format!("SciELO Publishing Schema {sps_version}"),
        publication_standard: PublicationStandard {
            family: "JATS".to_owned(),
            version: jats_version.to_owned(),
        },
        stability,
        grammar: ValidationStage {
            id: "jats-grammar".to_owned(),
            source_label: format!("JATS {jats_version}"),
            language: ValidationLanguage::Dtd,
            logical_uri: grammar_uri.clone(),
        },
        rules: vec![ValidationStage {
            id: "sps-rules".to_owned(),
            source_label: format!("SPS {sps_version}"),
            language: ValidationLanguage::Schematron,
            logical_uri: rules_uri.clone(),
        }],
        assets: vec![
            external_asset(grammar_uri, AssetRole::Grammar),
            external_asset(rules_uri, AssetRole::Rules),
        ],
    }
}

fn external_asset(logical_uri: String, role: AssetRole) -> FrameworkAsset {
    FrameworkAsset {
        logical_uri,
        role,
        redistribution: Redistribution::ExternalRequired,
        sha256: None,
    }
}

fn validate_manifest(manifest: &FrameworkManifest) -> Result<(), FrameworkError> {
    for value in [
        manifest.id.as_str(),
        manifest.display_name.as_str(),
        manifest.publication_standard.family.as_str(),
        manifest.publication_standard.version.as_str(),
    ] {
        validate_identifier(value)?;
    }
    if manifest.grammar.language == ValidationLanguage::Schematron {
        return Err(FrameworkError::GrammarCannotBeSchematron);
    }
    validate_stage(&manifest.grammar)?;
    for stage in &manifest.rules {
        validate_stage(stage)?;
        if stage.language != ValidationLanguage::Schematron {
            return Err(FrameworkError::RuleStageMustBeSchematron);
        }
    }
    for asset in &manifest.assets {
        validate_uri(&asset.logical_uri)?;
        if asset.sha256.as_ref().is_some_and(|value| {
            value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        }) {
            return Err(FrameworkError::InvalidChecksum);
        }
    }
    Ok(())
}

fn validate_stage(stage: &ValidationStage) -> Result<(), FrameworkError> {
    validate_identifier(&stage.id)?;
    validate_identifier(&stage.source_label)?;
    validate_uri(&stage.logical_uri)
}

fn validate_identifier(value: &str) -> Result<(), FrameworkError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES {
        return Err(FrameworkError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_uri(value: &str) -> Result<(), FrameworkError> {
    if value.is_empty() || value.len() > MAX_URI_BYTES {
        return Err(FrameworkError::InvalidLogicalUri);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum FrameworkError {
    #[error("framework identifier or label is empty or too large")]
    InvalidIdentifier,
    #[error("framework logical URI is empty or too large")]
    InvalidLogicalUri,
    #[error("framework checksum is not a 64-character hexadecimal SHA-256 value")]
    InvalidChecksum,
    #[error("framework registry exceeds the {limit}-entry limit")]
    TooManyFrameworks { limit: usize },
    #[error("framework identifier is already registered")]
    DuplicateFramework,
    #[error("unknown framework {0}")]
    UnknownFramework(String),
    #[error("the grammar stage cannot use Schematron")]
    GrammarCannotBeSchematron,
    #[error("framework rule stages must use Schematron")]
    RuleStageMustBeSchematron,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_jats_grammar_and_sps_rules_as_distinct_stages() -> Result<(), FrameworkError> {
        let registry = FrameworkRegistry::with_builtin_scientific_frameworks()?;
        let plan = registry.validation_plan("SPS-1.10")?;
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].source_label, "JATS 1.3");
        assert_eq!(plan[0].language, ValidationLanguage::Dtd);
        assert_eq!(plan[1].source_label, "SPS 1.10");
        assert_eq!(plan[1].language, ValidationLanguage::Schematron);
        assert_ne!(plan[0].logical_uri, plan[1].logical_uri);
        Ok(())
    }

    #[test]
    fn preserves_the_committed_sps_to_jats_mappings() -> Result<(), FrameworkError> {
        let registry = FrameworkRegistry::with_builtin_scientific_frameworks()?;
        assert_eq!(
            registry
                .get("SPS-1.9")
                .map(|manifest| manifest.publication_standard.version.as_str()),
            Some("1.1")
        );
        assert_eq!(
            registry
                .get("SPS-1.10")
                .map(|manifest| manifest.publication_standard.version.as_str()),
            Some("1.3")
        );
        assert!(registry.get("SPS-1.10").is_some_and(|manifest| manifest
            .assets
            .iter()
            .all(|asset| { asset.redistribution == Redistribution::ExternalRequired })));
        Ok(())
    }

    #[test]
    fn duplicate_registration_does_not_replace_the_original() -> Result<(), FrameworkError> {
        let mut registry = FrameworkRegistry::with_builtin_scientific_frameworks()?;
        let mut duplicate = registry
            .get("SPS-1.10")
            .cloned()
            .ok_or_else(|| FrameworkError::UnknownFramework("SPS-1.10".to_owned()))?;
        duplicate.display_name = "replacement".to_owned();

        assert_eq!(
            registry.insert(duplicate),
            Err(FrameworkError::DuplicateFramework)
        );
        assert_eq!(
            registry
                .get("SPS-1.10")
                .map(|manifest| manifest.display_name.as_str()),
            Some("SciELO Publishing Schema 1.10")
        );
        Ok(())
    }

    #[test]
    fn exposes_sps_1_11_only_as_an_explicit_candidate() -> Result<(), FrameworkError> {
        let registry = FrameworkRegistry::with_builtin_scientific_frameworks()?;
        let candidate = registry
            .get("SPS-1.11-candidate")
            .ok_or_else(|| FrameworkError::UnknownFramework("SPS-1.11-candidate".to_owned()))?;

        assert_eq!(candidate.stability, FrameworkStability::Candidate);
        assert_eq!(candidate.publication_standard.version, "1.4");
        assert!(candidate
            .assets
            .iter()
            .all(|asset| asset.redistribution == Redistribution::ExternalRequired));
        assert!(registry.get("SPS-1.11").is_none());
        Ok(())
    }
}
