use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use xml_schema::{
    compile_content_particle, AttributeDeclaration, ContentAutomaton, ContentModel,
    ContentParticle, ElementDeclaration, ExpandedName, SchemaIndex, SchemaIndexError,
    SchemaLanguage,
};

mod engine;

pub use engine::{
    DtdIndexEngine, DtdIndexError, DtdIndexFuture, DtdIndexReport, DtdIndexRequest,
    Libxml2DtdIndexEngine,
};

/// A normalized DTD element declaration. libxml2 extraction is responsible for
/// resolving parameter entities before producing this representation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DtdElementDeclaration {
    pub name: ExpandedName,
    pub content: DtdContent,
    pub attributes: Vec<AttributeDeclaration>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DtdContent {
    Empty,
    Any,
    Mixed(BTreeSet<ExpandedName>),
    Children(DtdParticle),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DtdParticle {
    Element(ExpandedName),
    Sequence(Vec<DtdParticle>),
    Choice(Vec<DtdParticle>),
    Optional(Box<DtdParticle>),
    ZeroOrMore(Box<DtdParticle>),
    OneOrMore(Box<DtdParticle>),
}

pub fn compile_dtd(
    declarations: impl IntoIterator<Item = DtdElementDeclaration>,
) -> Result<SchemaIndex, DtdCompileError> {
    let mut index = SchemaIndex::new(SchemaLanguage::Dtd);
    for declaration in declarations {
        if index.element(&declaration.name).is_some() {
            return Err(DtdCompileError::DuplicateElement(declaration.name));
        }
        let attributes = declaration
            .attributes
            .into_iter()
            .map(|attribute| (attribute.name.clone(), attribute))
            .collect::<BTreeMap<_, _>>();
        let content = match declaration.content {
            DtdContent::Empty => ContentModel::Empty,
            DtdContent::Any => ContentModel::Any,
            DtdContent::Mixed(children) => {
                let transitions = children
                    .into_iter()
                    .map(|name| (name, 0))
                    .collect::<BTreeMap<_, _>>();
                ContentModel::Automaton(ContentAutomaton {
                    start_state: 0,
                    accepting_states: BTreeSet::from([0]),
                    transitions: BTreeMap::from([(0, transitions)]),
                })
            }
            DtdContent::Children(particle) => ContentModel::Automaton(compile_particle(&particle)?),
        };
        index.insert(ElementDeclaration {
            name: declaration.name,
            content,
            attributes,
        })?;
    }
    Ok(index)
}

fn compile_particle(
    particle: &DtdParticle,
) -> Result<xml_schema::ContentAutomaton, DtdCompileError> {
    compile_content_particle(&normalize_particle(particle)).map_err(Into::into)
}

fn normalize_particle(particle: &DtdParticle) -> ContentParticle {
    match particle {
        DtdParticle::Element(name) => ContentParticle::Element(name.clone()),
        DtdParticle::Sequence(items) => {
            ContentParticle::Sequence(items.iter().map(normalize_particle).collect())
        }
        DtdParticle::Choice(items) => {
            ContentParticle::Choice(items.iter().map(normalize_particle).collect())
        }
        DtdParticle::Optional(inner) => ContentParticle::Repeat {
            particle: Box::new(normalize_particle(inner)),
            min_occurs: 0,
            max_occurs: Some(1),
        },
        DtdParticle::ZeroOrMore(inner) => ContentParticle::Repeat {
            particle: Box::new(normalize_particle(inner)),
            min_occurs: 0,
            max_occurs: None,
        },
        DtdParticle::OneOrMore(inner) => ContentParticle::Repeat {
            particle: Box::new(normalize_particle(inner)),
            min_occurs: 1,
            max_occurs: None,
        },
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DtdCompileError {
    #[error(transparent)]
    Schema(#[from] SchemaIndexError),
    #[error("DTD declares element {0:?} more than once")]
    DuplicateElement(ExpandedName),
}

#[cfg(test)]
mod tests {
    use super::*;
    use xml_schema::{AttributeUse, CompletionResult};

    fn name(local_name: &str) -> ExpandedName {
        ExpandedName {
            namespace_uri: None,
            local_name: local_name.to_owned(),
        }
    }

    fn declaration(name_value: &str, content: DtdContent) -> DtdElementDeclaration {
        DtdElementDeclaration {
            name: name(name_value),
            content,
            attributes: Vec::new(),
        }
    }

    #[test]
    fn compiles_sequence_choice_and_occurrence_to_completion_states() -> Result<(), DtdCompileError>
    {
        let article = name("article");
        let front = name("front");
        let body = name("body");
        let back = name("back");
        let floats = name("floats-group");
        let content = DtdContent::Children(DtdParticle::Sequence(vec![
            DtdParticle::Element(front.clone()),
            DtdParticle::OneOrMore(Box::new(DtdParticle::Element(body.clone()))),
            DtdParticle::Optional(Box::new(DtdParticle::Choice(vec![
                DtdParticle::Element(back.clone()),
                DtdParticle::Element(floats.clone()),
            ]))),
        ]));
        let index = compile_dtd([declaration("article", content)])?;

        assert_eq!(
            index.complete_children(&article, std::slice::from_ref(&front)),
            Ok(CompletionResult {
                children: vec![body.clone()],
                can_close: false,
                accepts_any: false,
            })
        );
        assert_eq!(
            index.complete_children(&article, &[front.clone(), body.clone()]),
            Ok(CompletionResult {
                children: vec![back.clone(), body.clone(), floats.clone()],
                can_close: true,
                accepts_any: false,
            })
        );
        assert_eq!(
            index.complete_children(&article, &[front, body, back]),
            Ok(CompletionResult {
                children: Vec::new(),
                can_close: true,
                accepts_any: false,
            })
        );
        Ok(())
    }

    #[test]
    fn compiles_mixed_content_as_repeatable_children() -> Result<(), DtdCompileError> {
        let paragraph = name("p");
        let bold = name("bold");
        let italic = name("italic");
        let index = compile_dtd([declaration(
            "p",
            DtdContent::Mixed(BTreeSet::from([bold.clone(), italic.clone()])),
        )])?;
        assert_eq!(
            index.complete_children(&paragraph, &[bold, italic]),
            Ok(CompletionResult {
                children: vec![name("bold"), name("italic")],
                can_close: true,
                accepts_any: false,
            })
        );
        Ok(())
    }

    #[test]
    fn preserves_attribute_completion_metadata() -> Result<(), DtdCompileError> {
        let article = name("article");
        let article_type = name("article-type");
        let index = compile_dtd([DtdElementDeclaration {
            name: article.clone(),
            content: DtdContent::Any,
            attributes: vec![AttributeDeclaration {
                name: article_type.clone(),
                use_kind: AttributeUse::Required,
                allowed_values: vec!["research-article".to_owned()],
                value_constraint: None,
            }],
        }])?;
        let attributes = index.complete_attributes(&article, &BTreeSet::new());
        assert_eq!(
            attributes,
            Ok(vec![AttributeDeclaration {
                name: article_type,
                use_kind: AttributeUse::Required,
                allowed_values: vec!["research-article".to_owned()],
                value_constraint: None,
            }])
        );
        Ok(())
    }
}
