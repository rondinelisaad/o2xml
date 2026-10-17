use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_NAME_BYTES: usize = 8 * 1024;
const MAX_ELEMENTS: usize = 100_000;
const MAX_ATTRIBUTES_PER_ELEMENT: usize = 4_096;
const MAX_STATES_PER_ELEMENT: usize = 100_000;
const MAX_TRANSITIONS_PER_ELEMENT: usize = 500_000;

/// An XML expanded name. Prefixes are deliberately absent because schemas and
/// instance documents may bind different prefixes to the same namespace URI.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpandedName {
    pub namespace_uri: Option<String>,
    pub local_name: String,
}

impl ExpandedName {
    pub fn new(namespace_uri: Option<&str>, local_name: &str) -> Result<Self, SchemaIndexError> {
        let name = Self {
            namespace_uri: namespace_uri.map(str::to_owned),
            local_name: local_name.to_owned(),
        };
        validate_expanded_name(&name)?;
        Ok(name)
    }
}

fn validate_expanded_name(name: &ExpandedName) -> Result<(), SchemaIndexError> {
    if name.local_name.is_empty() {
        return Err(SchemaIndexError::EmptyLocalName);
    }
    if name.local_name.len() > MAX_NAME_BYTES
        || name
            .namespace_uri
            .as_ref()
            .is_some_and(|uri| uri.len() > MAX_NAME_BYTES)
    {
        return Err(SchemaIndexError::NameTooLarge {
            limit: MAX_NAME_BYTES,
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaLanguage {
    Dtd,
    Xsd10,
    RelaxNg,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeUse {
    Optional,
    Required,
    Prohibited,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ValueConstraint {
    Default(String),
    Fixed(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDeclaration {
    pub name: ExpandedName,
    pub use_kind: AttributeUse,
    pub allowed_values: Vec<String>,
    pub value_constraint: Option<ValueConstraint>,
}

pub type StateId = u32;

/// A deterministic content automaton produced by a DTD, XSD, or RNG adapter.
/// Each transition consumes one child element expanded name.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentAutomaton {
    pub start_state: StateId,
    pub accepting_states: BTreeSet<StateId>,
    pub transitions: BTreeMap<StateId, BTreeMap<ExpandedName, StateId>>,
}

impl ContentAutomaton {
    pub fn validate(&self) -> Result<(), SchemaIndexError> {
        let transition_count = self
            .transitions
            .values()
            .fold(0_usize, |count, transitions| {
                count.saturating_add(transitions.len())
            });
        if transition_count > MAX_TRANSITIONS_PER_ELEMENT {
            return Err(SchemaIndexError::TooManyTransitions {
                limit: MAX_TRANSITIONS_PER_ELEMENT,
            });
        }

        let mut states = BTreeSet::from([self.start_state]);
        states.extend(self.accepting_states.iter().copied());
        for (source, transitions) in &self.transitions {
            states.insert(*source);
            for (name, target) in transitions {
                validate_expanded_name(name)?;
                states.insert(*target);
            }
        }
        if states.len() > MAX_STATES_PER_ELEMENT {
            return Err(SchemaIndexError::TooManyStates {
                limit: MAX_STATES_PER_ELEMENT,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "model", rename_all = "snake_case")]
pub enum ContentModel {
    Empty,
    Any,
    Automaton(ContentAutomaton),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementDeclaration {
    pub name: ExpandedName,
    pub content: ContentModel,
    pub attributes: BTreeMap<ExpandedName, AttributeDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionResult {
    pub children: Vec<ExpandedName>,
    pub can_close: bool,
    pub accepts_any: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaIndex {
    pub language: SchemaLanguage,
    elements: BTreeMap<ExpandedName, ElementDeclaration>,
}

impl SchemaIndex {
    #[must_use]
    pub fn new(language: SchemaLanguage) -> Self {
        Self {
            language,
            elements: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, element: ElementDeclaration) -> Result<(), SchemaIndexError> {
        validate_expanded_name(&element.name)?;
        if self.elements.len() >= MAX_ELEMENTS && !self.elements.contains_key(&element.name) {
            return Err(SchemaIndexError::TooManyElements {
                limit: MAX_ELEMENTS,
            });
        }
        if element.attributes.len() > MAX_ATTRIBUTES_PER_ELEMENT {
            return Err(SchemaIndexError::TooManyAttributes {
                limit: MAX_ATTRIBUTES_PER_ELEMENT,
            });
        }
        if element.attributes.iter().any(|(name, declaration)| {
            name != &declaration.name || validate_expanded_name(name).is_err()
        }) {
            return Err(SchemaIndexError::InvalidAttributeDeclaration);
        }
        if let ContentModel::Automaton(automaton) = &element.content {
            automaton.validate()?;
        }
        self.elements.insert(element.name.clone(), element);
        Ok(())
    }

    #[must_use]
    pub fn element(&self, name: &ExpandedName) -> Option<&ElementDeclaration> {
        self.elements.get(name)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn complete_children(
        &self,
        parent: &ExpandedName,
        preceding_children: &[ExpandedName],
    ) -> Result<CompletionResult, SchemaIndexError> {
        let declaration = self
            .elements
            .get(parent)
            .ok_or_else(|| SchemaIndexError::UnknownElement(parent.clone()))?;

        match &declaration.content {
            ContentModel::Empty if preceding_children.is_empty() => Ok(CompletionResult {
                children: Vec::new(),
                can_close: true,
                accepts_any: false,
            }),
            ContentModel::Empty => Err(SchemaIndexError::UnexpectedChild {
                index: 0,
                child: preceding_children[0].clone(),
            }),
            ContentModel::Any => Ok(CompletionResult {
                children: self.elements.keys().cloned().collect(),
                can_close: true,
                accepts_any: true,
            }),
            ContentModel::Automaton(automaton) => {
                let mut state = automaton.start_state;
                for (index, child) in preceding_children.iter().enumerate() {
                    state = automaton
                        .transitions
                        .get(&state)
                        .and_then(|transitions| transitions.get(child))
                        .copied()
                        .ok_or_else(|| SchemaIndexError::UnexpectedChild {
                            index,
                            child: child.clone(),
                        })?;
                }
                let children = automaton
                    .transitions
                    .get(&state)
                    .map(|transitions| transitions.keys().cloned().collect())
                    .unwrap_or_default();
                Ok(CompletionResult {
                    children,
                    can_close: automaton.accepting_states.contains(&state),
                    accepts_any: false,
                })
            }
        }
    }

    pub fn complete_attributes(
        &self,
        element: &ExpandedName,
        present: &BTreeSet<ExpandedName>,
    ) -> Result<Vec<AttributeDeclaration>, SchemaIndexError> {
        let declaration = self
            .elements
            .get(element)
            .ok_or_else(|| SchemaIndexError::UnknownElement(element.clone()))?;
        Ok(declaration
            .attributes
            .values()
            .filter(|attribute| {
                attribute.use_kind != AttributeUse::Prohibited && !present.contains(&attribute.name)
            })
            .cloned()
            .collect())
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SchemaIndexError {
    #[error("expanded name local part cannot be empty")]
    EmptyLocalName,
    #[error("expanded name exceeds the {limit}-byte component limit")]
    NameTooLarge { limit: usize },
    #[error("schema index exceeds the {limit}-element limit")]
    TooManyElements { limit: usize },
    #[error("element exceeds the {limit}-attribute limit")]
    TooManyAttributes { limit: usize },
    #[error("content model exceeds the {limit}-state limit")]
    TooManyStates { limit: usize },
    #[error("content model exceeds the {limit}-transition limit")]
    TooManyTransitions { limit: usize },
    #[error("attribute declaration key and name differ or contain an invalid name")]
    InvalidAttributeDeclaration,
    #[error("schema index has no declaration for element {0:?}")]
    UnknownElement(ExpandedName),
    #[error("child {child:?} is not valid at content position {index}")]
    UnexpectedChild { index: usize, child: ExpandedName },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(local_name: &str) -> ExpandedName {
        ExpandedName {
            namespace_uri: None,
            local_name: local_name.to_owned(),
        }
    }

    #[test]
    fn completes_a_sequence_and_reports_when_the_parent_can_close() {
        let article = name("article");
        let front = name("front");
        let body = name("body");
        let transitions = BTreeMap::from([
            (0, BTreeMap::from([(front.clone(), 1)])),
            (1, BTreeMap::from([(body.clone(), 2)])),
        ]);
        let mut index = SchemaIndex::new(SchemaLanguage::Dtd);
        let inserted = index.insert(ElementDeclaration {
            name: article.clone(),
            content: ContentModel::Automaton(ContentAutomaton {
                start_state: 0,
                accepting_states: BTreeSet::from([2]),
                transitions,
            }),
            attributes: BTreeMap::new(),
        });
        assert_eq!(inserted, Ok(()));

        let start = index.complete_children(&article, &[]);
        assert_eq!(
            start,
            Ok(CompletionResult {
                children: vec![front.clone()],
                can_close: false,
                accepts_any: false,
            })
        );
        let after_front = index.complete_children(&article, &[front]);
        assert_eq!(
            after_front,
            Ok(CompletionResult {
                children: vec![body.clone()],
                can_close: false,
                accepts_any: false,
            })
        );
        let complete = index.complete_children(&article, &[name("front"), body]);
        assert_eq!(
            complete,
            Ok(CompletionResult {
                children: Vec::new(),
                can_close: true,
                accepts_any: false,
            })
        );
    }

    #[test]
    fn rejects_a_child_that_does_not_follow_the_content_model() {
        let article = name("article");
        let front = name("front");
        let body = name("body");
        let mut index = SchemaIndex::new(SchemaLanguage::Xsd10);
        let inserted = index.insert(ElementDeclaration {
            name: article.clone(),
            content: ContentModel::Automaton(ContentAutomaton {
                start_state: 0,
                accepting_states: BTreeSet::from([1]),
                transitions: BTreeMap::from([(0, BTreeMap::from([(front, 1)]))]),
            }),
            attributes: BTreeMap::new(),
        });
        assert_eq!(inserted, Ok(()));
        assert_eq!(
            index.complete_children(&article, std::slice::from_ref(&body)),
            Err(SchemaIndexError::UnexpectedChild {
                index: 0,
                child: body,
            })
        );
    }

    #[test]
    fn completes_only_missing_non_prohibited_attributes() {
        let article = name("article");
        let article_type = name("article-type");
        let language = ExpandedName {
            namespace_uri: Some("http://www.w3.org/XML/1998/namespace".to_owned()),
            local_name: "lang".to_owned(),
        };
        let legacy = name("legacy");
        let attributes = [
            AttributeDeclaration {
                name: article_type.clone(),
                use_kind: AttributeUse::Required,
                allowed_values: vec!["research-article".to_owned()],
                value_constraint: None,
            },
            AttributeDeclaration {
                name: language.clone(),
                use_kind: AttributeUse::Optional,
                allowed_values: Vec::new(),
                value_constraint: None,
            },
            AttributeDeclaration {
                name: legacy.clone(),
                use_kind: AttributeUse::Prohibited,
                allowed_values: Vec::new(),
                value_constraint: None,
            },
        ]
        .into_iter()
        .map(|attribute| (attribute.name.clone(), attribute))
        .collect();
        let mut index = SchemaIndex::new(SchemaLanguage::Xsd10);
        let inserted = index.insert(ElementDeclaration {
            name: article.clone(),
            content: ContentModel::Any,
            attributes,
        });
        assert_eq!(inserted, Ok(()));

        let completed = index.complete_attributes(&article, &BTreeSet::from([language]));
        assert_eq!(
            completed,
            Ok(vec![AttributeDeclaration {
                name: article_type,
                use_kind: AttributeUse::Required,
                allowed_values: vec!["research-article".to_owned()],
                value_constraint: None,
            }])
        );
    }
}
