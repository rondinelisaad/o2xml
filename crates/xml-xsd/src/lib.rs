use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use xml_schema::{
    compile_content_particle, AttributeDeclaration, ContentModel, ContentParticle,
    ElementDeclaration, ExpandedName, SchemaIndex, SchemaIndexError, SchemaLanguage,
};

/// A normalized XSD 1.0 element declaration. An engine adapter is responsible
/// for resolving QName references, type derivation, groups, and schema imports
/// before emitting this bounded representation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XsdElementDeclaration {
    pub name: ExpandedName,
    pub content: XsdContent,
    pub attributes: Vec<AttributeDeclaration>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum XsdContent {
    Empty,
    Any,
    Particle(ContentParticle),
}

/// Compiles engine-normalized XSD 1.0 declarations into the common completion
/// index. This function does not parse XSD or attempt schema-validity checking.
pub fn compile_xsd(
    declarations: impl IntoIterator<Item = XsdElementDeclaration>,
) -> Result<SchemaIndex, XsdCompileError> {
    let mut index = SchemaIndex::new(SchemaLanguage::Xsd10);
    for declaration in declarations {
        if index.element(&declaration.name).is_some() {
            return Err(XsdCompileError::DuplicateElement(declaration.name));
        }
        let attributes = declaration
            .attributes
            .into_iter()
            .map(|attribute| (attribute.name.clone(), attribute))
            .collect::<BTreeMap<_, _>>();
        let content = match declaration.content {
            XsdContent::Empty => ContentModel::Empty,
            XsdContent::Any => ContentModel::Any,
            XsdContent::Particle(particle) => {
                ContentModel::Automaton(compile_content_particle(&particle)?)
            }
        };
        index.insert(ElementDeclaration {
            name: declaration.name,
            content,
            attributes,
        })?;
    }
    Ok(index)
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum XsdCompileError {
    #[error(transparent)]
    Schema(#[from] SchemaIndexError),
    #[error("XSD declares element {0:?} more than once after normalization")]
    DuplicateElement(ExpandedName),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use xml_schema::{AttributeUse, CompletionResult, ValueConstraint};

    const JATS: &str = "http://jats.nlm.nih.gov";

    fn name(local: &str) -> ExpandedName {
        ExpandedName::new(Some(JATS), local).unwrap_or_else(|error| {
            unreachable!("static test expanded name must be valid: {error}")
        })
    }

    #[test]
    fn compiles_namespaced_sequence_choice_and_occurrences() -> Result<(), XsdCompileError> {
        let article = name("article");
        let front = name("front");
        let body = name("body");
        let back = name("back");
        let floats = name("floats-group");
        let particle = ContentParticle::Sequence(vec![
            ContentParticle::Element(front.clone()),
            ContentParticle::Repeat {
                particle: Box::new(ContentParticle::Element(body.clone())),
                min_occurs: 1,
                max_occurs: None,
            },
            ContentParticle::Repeat {
                particle: Box::new(ContentParticle::Choice(vec![
                    ContentParticle::Element(back.clone()),
                    ContentParticle::Element(floats.clone()),
                ])),
                min_occurs: 0,
                max_occurs: Some(1),
            },
        ]);
        let index = compile_xsd([XsdElementDeclaration {
            name: article.clone(),
            content: XsdContent::Particle(particle),
            attributes: Vec::new(),
        }])?;

        assert_eq!(
            index.complete_children(&article, std::slice::from_ref(&front)),
            Ok(CompletionResult {
                children: vec![body.clone()],
                can_close: false,
                accepts_any: false,
            })
        );
        assert_eq!(
            index.complete_children(&article, &[front, body.clone()]),
            Ok(CompletionResult {
                children: vec![back, body, floats],
                can_close: true,
                accepts_any: false,
            })
        );
        Ok(())
    }

    #[test]
    fn preserves_attribute_uses_constraints_and_enumerations() -> Result<(), XsdCompileError> {
        let article = name("article");
        let article_type = ExpandedName::new(None, "article-type")?;
        let language = ExpandedName::new(Some("http://www.w3.org/XML/1998/namespace"), "lang")?;
        let obsolete = ExpandedName::new(None, "obsolete")?;
        let index = compile_xsd([XsdElementDeclaration {
            name: article.clone(),
            content: XsdContent::Any,
            attributes: vec![
                AttributeDeclaration {
                    name: article_type.clone(),
                    use_kind: AttributeUse::Required,
                    allowed_values: vec![
                        "research-article".to_owned(),
                        "review-article".to_owned(),
                    ],
                    value_constraint: None,
                },
                AttributeDeclaration {
                    name: language.clone(),
                    use_kind: AttributeUse::Optional,
                    allowed_values: Vec::new(),
                    value_constraint: Some(ValueConstraint::Default("en".to_owned())),
                },
                AttributeDeclaration {
                    name: obsolete,
                    use_kind: AttributeUse::Prohibited,
                    allowed_values: Vec::new(),
                    value_constraint: None,
                },
            ],
        }])?;

        let attributes = index.complete_attributes(&article, &BTreeSet::new())?;
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].name, article_type);
        assert_eq!(attributes[1].name, language);
        Ok(())
    }

    #[test]
    fn rejects_invalid_and_excessive_occurrence_ranges() {
        let declaration = |min_occurs, max_occurs| XsdElementDeclaration {
            name: name("article"),
            content: XsdContent::Particle(ContentParticle::Repeat {
                particle: Box::new(ContentParticle::Element(name("body"))),
                min_occurs,
                max_occurs,
            }),
            attributes: Vec::new(),
        };

        assert!(matches!(
            compile_xsd([declaration(2, Some(1))]),
            Err(XsdCompileError::Schema(
                SchemaIndexError::InvalidOccurrenceRange { .. }
            ))
        ));
        assert!(matches!(
            compile_xsd([declaration(4_097, None)]),
            Err(XsdCompileError::Schema(
                SchemaIndexError::OccurrenceLimitExceeded { .. }
            ))
        ));
    }
}
