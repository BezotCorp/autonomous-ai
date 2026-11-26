use std::collections::BTreeSet;

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    reasoning::{epistemic_kind::EpistemicKind, reasoning_graph::ReasoningGraph},
    representation::{
        content::Content, content_kind::ContentKind, provenance::Provenance, statement::Statement,
        term::Term,
    },
    world::{message::Message, portal::Portal, transmission_status::TransmissionStatus},
};

/// Transmits: answers a need `transmitted(<destination>, ?message)` by
/// sending, to that destination of the environment, what the reasoning has
/// at hand: its other open questions, the statements related to them (or
/// else the latest ones), what it had itself been told (relayed, with the
/// chain of sources), and texts. The destination must be an address the
/// individual knows of (it appears in the reasoning); when it is left
/// open, the first known one is used, and without any, nothing is sent.
///
/// A transmission is an act: it happens only in reality. In imagination
/// the module sends nothing (the need stays open, an envisaged
/// interaction). It never reads anything but the current reasoning: no
/// file, no other memory, nothing of what the individual perceives of
/// itself.
pub(crate) struct Communication {
    descriptor: ModuleDescriptor,
    portal: Portal,
}

impl Communication {
    const STATEMENTS: usize = 16;
    const RELAYED: usize = 8;
    const QUESTIONS: usize = 4;
    const DOCUMENTS: usize = 2;
    /// Predicates of the reasoning about the act of communicating itself
    /// or about presences, never sent as content.
    const OWN: [&'static str; 2] = ["transmitted", "present"];

    pub(crate) fn new(portal: Portal) -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "communication",
                "Transmits what the reasoning has at hand to a known destination of the environment.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&["transmitted/2"])
            .producing(&[EpistemicKind::Observation]),
            portal,
        }
    }

    fn private(provenance: Option<&Provenance>) -> bool {
        provenance.is_some_and(|provenance| provenance.source() == "introspection")
    }

    fn compose(graph: &ReasoningGraph) -> Message {
        let own = |statement: &Statement| Self::OWN.contains(&statement.predicate());
        let questions: Vec<Statement> = graph
            .valid_nodes()
            .filter_map(|node| node.content().need())
            .filter(|need| !own(need) && !graph.answers(need))
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .take(Self::QUESTIONS)
            .collect();
        let related: BTreeSet<&str> = questions
            .iter()
            .map(|question| question.predicate())
            .collect();
        let candidates: Vec<(&Statement, EpistemicKind, Option<&Provenance>)> = graph
            .valid_nodes()
            .filter(|node| !Self::private(node.provenance()))
            .filter_map(|node| Some((node.content().statement()?, node.kind(), node.provenance())))
            .filter(|(statement, kind, _)| {
                !own(statement)
                    && matches!(
                        kind,
                        EpistemicKind::Observation
                            | EpistemicKind::Deduction
                            | EpistemicKind::Computation
                            | EpistemicKind::Recollection
                            | EpistemicKind::Testimony
                    )
            })
            .collect();
        let relevant: Vec<_> = if candidates
            .iter()
            .any(|(statement, _, _)| related.contains(statement.predicate()))
        {
            candidates
                .iter()
                .filter(|(statement, _, _)| related.contains(statement.predicate()))
                .collect()
        } else {
            candidates.iter().rev().collect()
        };
        let statements: Vec<Statement> = relevant
            .iter()
            .filter(|(_, kind, _)| *kind != EpistemicKind::Testimony)
            .map(|(statement, _, _)| (*statement).clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .take(Self::STATEMENTS)
            .collect();
        let relayed: Vec<(Statement, String)> = relevant
            .iter()
            .filter(|(_, kind, _)| *kind == EpistemicKind::Testimony)
            .map(|(statement, _, provenance)| {
                (
                    (*statement).clone(),
                    provenance.map_or_else(|| "unknown".to_owned(), |p| p.source().to_owned()),
                )
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .take(Self::RELAYED)
            .collect();
        let documents = graph
            .valid_nodes()
            .filter_map(|node| match node.content() {
                Content::Document(document) => Some(document.clone()),
                _ => None,
            })
            .take(Self::DOCUMENTS)
            .collect();

        Message::new(statements, relayed, questions, documents)
    }

    /// Destinations the individual knows of: those of the need, else the
    /// presences appearing in the reasoning.
    fn destination(graph: &ReasoningGraph, pattern: &Statement) -> Option<String> {
        let target = pattern.arguments().first()?;

        if !target.is_variable() {
            return Some(target.to_string());
        }

        graph
            .valid_nodes()
            .filter_map(|node| node.content().statement())
            .filter(|statement| statement.predicate() == "present")
            .filter_map(|statement| statement.arguments().first())
            .map(ToString::to_string)
            .min()
    }
}

impl CognitiveModule for Communication {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let mut output = ModuleOutput::default();
        let needs = context.open_needs(&self.descriptor);

        if needs.is_empty() {
            return Ok(output);
        }

        if !context.access().is_live() {
            output.note("nothing is transmitted in imagination");
            return Ok(output);
        }

        let graph = context.graph();

        for (_, pattern) in needs {
            let Some(destination) = Self::destination(graph, pattern) else {
                output.note(format!("{pattern}: no known destination"));
                continue;
            };
            let message = Self::compose(graph);
            let (id, status) = self.portal.transmit(&destination, message);

            match status {
                TransmissionStatus::Pending => {
                    output.propose(
                        Proposal::new(
                            Content::Statement(Statement::new(
                                "transmitted",
                                vec![
                                    Term::symbol(destination),
                                    Term::symbol(format!("m{}", id.simple())),
                                ],
                            )),
                            EpistemicKind::Observation,
                            Vec::new(),
                        )
                        .with_provenance(Provenance::new(
                            "environment",
                            None,
                            Some(format!("{:.0}", self.portal.now())),
                        )),
                    );
                }
                TransmissionStatus::Undeliverable { reason } => {
                    output.note(format!("{pattern}: {reason}"));
                }
                TransmissionStatus::Delivered { .. } => {}
            }
        }

        Ok(output)
    }
}
