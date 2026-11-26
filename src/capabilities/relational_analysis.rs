use std::collections::BTreeMap;

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    reasoning::{epistemic_kind::EpistemicKind, node_id::NodeId, reasoning_graph::ReasoningGraph},
    representation::{
        content::Content, content_kind::ContentKind, statement::Statement, term::Term,
    },
};

/// Reasoning about binary relations through their declared properties.
///
/// Rules cannot quantify over relations, so the general laws of relations
/// are carried by this module: whenever the knowledge states
/// `relation_property(r, transitive)`, `relation_property(r, symmetric)` or
/// `inverse_relation(r, s)`, the corresponding consequences are deduced.
/// Which relation has which property is knowledge, never assumed here.
pub(crate) struct RelationalAnalysis {
    descriptor: ModuleDescriptor,
}

impl RelationalAnalysis {
    /// Upper bound on deductions proposed per invocation.
    const MAXIMUM_DEDUCTIONS: usize = 50_000;

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "relational_analysis",
                "Derives consequences of declared relation properties (transitivity, symmetry, inverses).",
            )
            .consuming(&[ContentKind::Statement])
            .producing(&[EpistemicKind::Deduction]),
        }
    }

    fn pairs(graph: &ReasoningGraph, relation: &str) -> Vec<(NodeId, Term, Term)> {
        let pattern = Statement::new(
            relation,
            vec![
                Term::Variable("a".to_owned()),
                Term::Variable("b".to_owned()),
            ],
        );

        graph
            .matching(&pattern, &Default::default())
            .filter_map(|(id, bindings)| {
                Some((id, bindings.get("a")?.clone(), bindings.get("b")?.clone()))
            })
            .collect()
    }

    fn declarations(graph: &ReasoningGraph, predicate: &str) -> Vec<(NodeId, Vec<Term>)> {
        let pattern = Statement::new(
            predicate,
            vec![
                Term::Variable("r".to_owned()),
                Term::Variable("p".to_owned()),
            ],
        );

        graph
            .matching(&pattern, &Default::default())
            .filter_map(|(id, _)| {
                Some((
                    id,
                    graph.node(id)?.content().statement()?.arguments().to_vec(),
                ))
            })
            .collect()
    }
}

impl CognitiveModule for RelationalAnalysis {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let graph = context.graph();
        let mut output = ModuleOutput::default();
        let mut deductions: BTreeMap<Statement, Vec<NodeId>> = BTreeMap::new();

        let mut deduce = |statement: Statement, premises: Vec<NodeId>| {
            if deductions.len() < Self::MAXIMUM_DEDUCTIONS
                && graph.holding(&statement).next().is_none()
            {
                deductions.entry(statement).or_insert(premises);
            }
        };

        for (declaration, arguments) in Self::declarations(graph, "relation_property") {
            let [Term::Symbol(relation), Term::Symbol(property)] = arguments.as_slice() else {
                continue;
            };

            let pairs = Self::pairs(graph, relation);

            match property.as_str() {
                "symmetric" => {
                    for (id, left, right) in &pairs {
                        deduce(
                            Statement::new(relation.as_str(), vec![right.clone(), left.clone()]),
                            vec![declaration, *id],
                        );
                    }
                }
                "transitive" => {
                    let mut successors: BTreeMap<&Term, Vec<(NodeId, &Term)>> = BTreeMap::new();

                    for (id, left, right) in &pairs {
                        successors.entry(left).or_default().push((*id, right));
                    }

                    for (first, left, middle) in &pairs {
                        for (second, right) in successors.get(middle).into_iter().flatten() {
                            deduce(
                                Statement::new(
                                    relation.as_str(),
                                    vec![left.clone(), (*right).clone()],
                                ),
                                vec![declaration, *first, *second],
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        for (declaration, arguments) in Self::declarations(graph, "inverse_relation") {
            let [Term::Symbol(relation), Term::Symbol(inverse)] = arguments.as_slice() else {
                continue;
            };

            for (id, left, right) in Self::pairs(graph, relation) {
                deduce(
                    Statement::new(inverse.as_str(), vec![right, left]),
                    vec![declaration, id],
                );
            }
        }

        for (statement, premises) in deductions {
            output.propose(Proposal::new(
                Content::Statement(statement),
                EpistemicKind::Deduction,
                premises,
            ));
        }

        Ok(output)
    }
}
