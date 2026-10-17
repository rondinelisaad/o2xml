use std::collections::{BTreeMap, BTreeSet, VecDeque};

use thiserror::Error;
use xml_schema::{
    AttributeDeclaration, ContentAutomaton, ContentModel, ElementDeclaration, ExpandedName,
    SchemaIndex, SchemaIndexError, SchemaLanguage, StateId,
};

const MAX_PARTICLE_DEPTH: usize = 128;
const MAX_NFA_STATES: usize = 100_000;
const MAX_DFA_STATES: usize = 100_000;
const MAX_DFA_TRANSITIONS: usize = 500_000;

/// A normalized DTD element declaration. libxml2 extraction is responsible for
/// resolving parameter entities before producing this representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DtdElementDeclaration {
    pub name: ExpandedName,
    pub content: DtdContent,
    pub attributes: Vec<AttributeDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DtdContent {
    Empty,
    Any,
    Mixed(BTreeSet<ExpandedName>),
    Children(DtdParticle),
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

fn compile_particle(particle: &DtdParticle) -> Result<ContentAutomaton, DtdCompileError> {
    let mut nfa = Nfa::default();
    let fragment = nfa.compile(particle, 0)?;
    determinize(&nfa, fragment)
}

#[derive(Default)]
struct Nfa {
    states: Vec<NfaState>,
}

#[derive(Default)]
struct NfaState {
    epsilon: Vec<usize>,
    transitions: Vec<(ExpandedName, usize)>,
}

#[derive(Clone, Copy)]
struct Fragment {
    start: usize,
    end: usize,
}

impl Nfa {
    fn state(&mut self) -> Result<usize, DtdCompileError> {
        if self.states.len() >= MAX_NFA_STATES {
            return Err(DtdCompileError::TooManyNfaStates {
                limit: MAX_NFA_STATES,
            });
        }
        let id = self.states.len();
        self.states.push(NfaState::default());
        Ok(id)
    }

    fn compile(
        &mut self,
        particle: &DtdParticle,
        depth: usize,
    ) -> Result<Fragment, DtdCompileError> {
        if depth > MAX_PARTICLE_DEPTH {
            return Err(DtdCompileError::ParticleTooDeep {
                limit: MAX_PARTICLE_DEPTH,
            });
        }
        match particle {
            DtdParticle::Element(name) => {
                let start = self.state()?;
                let end = self.state()?;
                self.states[start].transitions.push((name.clone(), end));
                Ok(Fragment { start, end })
            }
            DtdParticle::Sequence(items) => self.compile_sequence(items, depth + 1),
            DtdParticle::Choice(items) => self.compile_choice(items, depth + 1),
            DtdParticle::Optional(inner) => {
                let fragment = self.compile(inner, depth + 1)?;
                let start = self.state()?;
                let end = self.state()?;
                self.states[start].epsilon.extend([fragment.start, end]);
                self.states[fragment.end].epsilon.push(end);
                Ok(Fragment { start, end })
            }
            DtdParticle::ZeroOrMore(inner) => {
                let fragment = self.compile(inner, depth + 1)?;
                let start = self.state()?;
                let end = self.state()?;
                self.states[start].epsilon.extend([fragment.start, end]);
                self.states[fragment.end]
                    .epsilon
                    .extend([fragment.start, end]);
                Ok(Fragment { start, end })
            }
            DtdParticle::OneOrMore(inner) => {
                let fragment = self.compile(inner, depth + 1)?;
                let start = self.state()?;
                let end = self.state()?;
                self.states[start].epsilon.push(fragment.start);
                self.states[fragment.end]
                    .epsilon
                    .extend([fragment.start, end]);
                Ok(Fragment { start, end })
            }
        }
    }

    fn compile_sequence(
        &mut self,
        items: &[DtdParticle],
        depth: usize,
    ) -> Result<Fragment, DtdCompileError> {
        let start = self.state()?;
        let mut tail = start;
        for item in items {
            let fragment = self.compile(item, depth)?;
            self.states[tail].epsilon.push(fragment.start);
            tail = fragment.end;
        }
        let end = self.state()?;
        self.states[tail].epsilon.push(end);
        Ok(Fragment { start, end })
    }

    fn compile_choice(
        &mut self,
        items: &[DtdParticle],
        depth: usize,
    ) -> Result<Fragment, DtdCompileError> {
        if items.is_empty() {
            return Err(DtdCompileError::EmptyChoice);
        }
        let start = self.state()?;
        let end = self.state()?;
        for item in items {
            let fragment = self.compile(item, depth)?;
            self.states[start].epsilon.push(fragment.start);
            self.states[fragment.end].epsilon.push(end);
        }
        Ok(Fragment { start, end })
    }
}

fn determinize(nfa: &Nfa, fragment: Fragment) -> Result<ContentAutomaton, DtdCompileError> {
    let start = epsilon_closure(nfa, BTreeSet::from([fragment.start]));
    let mut ids = BTreeMap::from([(start.clone(), 0_u32)]);
    let mut pending = VecDeque::from([start]);
    let mut accepting = BTreeSet::new();
    let mut transitions = BTreeMap::new();
    let mut transition_count = 0_usize;

    while let Some(states) = pending.pop_front() {
        let source = ids[&states];
        if states.contains(&fragment.end) {
            accepting.insert(source);
        }
        let mut targets = BTreeMap::<ExpandedName, BTreeSet<usize>>::new();
        for state in &states {
            for (name, target) in &nfa.states[*state].transitions {
                targets.entry(name.clone()).or_default().insert(*target);
            }
        }
        let mut deterministic = BTreeMap::new();
        for (name, target) in targets {
            transition_count = transition_count.saturating_add(1);
            if transition_count > MAX_DFA_TRANSITIONS {
                return Err(DtdCompileError::TooManyDfaTransitions {
                    limit: MAX_DFA_TRANSITIONS,
                });
            }
            let target = epsilon_closure(nfa, target);
            let target_id = if let Some(id) = ids.get(&target) {
                *id
            } else {
                if ids.len() >= MAX_DFA_STATES {
                    return Err(DtdCompileError::TooManyDfaStates {
                        limit: MAX_DFA_STATES,
                    });
                }
                let id = StateId::try_from(ids.len())
                    .map_err(|_| DtdCompileError::StateIdentifierOverflow)?;
                ids.insert(target.clone(), id);
                pending.push_back(target);
                id
            };
            deterministic.insert(name, target_id);
        }
        if !deterministic.is_empty() {
            transitions.insert(source, deterministic);
        }
    }

    Ok(ContentAutomaton {
        start_state: 0,
        accepting_states: accepting,
        transitions,
    })
}

fn epsilon_closure(nfa: &Nfa, mut states: BTreeSet<usize>) -> BTreeSet<usize> {
    let mut pending = states.iter().copied().collect::<Vec<_>>();
    while let Some(state) = pending.pop() {
        for target in &nfa.states[state].epsilon {
            if states.insert(*target) {
                pending.push(*target);
            }
        }
    }
    states
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DtdCompileError {
    #[error(transparent)]
    Schema(#[from] SchemaIndexError),
    #[error("DTD declares element {0:?} more than once")]
    DuplicateElement(ExpandedName),
    #[error("DTD content particle exceeds the depth limit of {limit}")]
    ParticleTooDeep { limit: usize },
    #[error("DTD choice must contain at least one particle")]
    EmptyChoice,
    #[error("DTD content model exceeds the {limit}-state NFA limit")]
    TooManyNfaStates { limit: usize },
    #[error("DTD content model exceeds the {limit}-state DFA limit")]
    TooManyDfaStates { limit: usize },
    #[error("DTD content model exceeds the {limit}-transition DFA limit")]
    TooManyDfaTransitions { limit: usize },
    #[error("DTD content model state identifier overflowed")]
    StateIdentifierOverflow,
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
