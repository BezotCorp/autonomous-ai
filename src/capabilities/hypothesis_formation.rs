use std::collections::BTreeMap;

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    reasoning::{epistemic_kind::EpistemicKind, node_id::NodeId, reasoning_graph::ReasoningGraph},
    representation::{
        content::Content, content_kind::ContentKind, rule::Rule, statement::Statement, term::Term,
    },
};

/// Inductive formation of hypotheses.
///
/// For every pair of relations `p` and `q`, and every way of reading the
/// arguments of `q` among those of `p`, the module counts the instances of
/// `p` for which the corresponding `q` was independently established
/// (observed, recalled or given), and those for which its negation is held.
/// An implication `q(...) :- p(...)` is conjectured when it has enough
/// examples and no counterexample. It is justified by its examples and is
/// never proposed again once refuted.
pub(crate) struct HypothesisFormation {
    descriptor: ModuleDescriptor,
    minimum_support: usize,
}

impl HypothesisFormation {
    const MAXIMUM_ARITY: usize = 3;
    const MAXIMUM_HYPOTHESES: usize = 32;

    /// `minimum_support` is the number of distinct examples required before
    /// conjecturing a generalisation.
    pub(crate) fn new(minimum_support: usize) -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "hypothesis_formation",
                "Conjectures implications generalising regularities observed between relations.",
            )
            .consuming(&[ContentKind::Statement])
            .producing(&[EpistemicKind::Hypothesis]),
            minimum_support: minimum_support.max(1),
        }
    }

    fn is_independent(graph: &ReasoningGraph, id: NodeId) -> bool {
        graph.node(id).is_some_and(|node| {
            matches!(
                node.kind(),
                EpistemicKind::Observation | EpistemicKind::Recollection | EpistemicKind::Premise
            )
        })
    }

    /// Injective readings of `target` positions among `source` positions.
    fn mappings(source: usize, target: usize) -> Vec<Vec<usize>> {
        let mut mappings = vec![Vec::new()];

        for _ in 0..target {
            let mut extended = Vec::new();

            for mapping in &mappings {
                for position in 0..source {
                    if !mapping.contains(&position) {
                        let mut next = mapping.clone();
                        next.push(position);
                        extended.push(next);
                    }
                }
            }

            mappings = extended;
        }

        mappings
    }
}

impl CognitiveModule for HypothesisFormation {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let graph = context.graph();
        let mut output = ModuleOutput::default();

        let mut instances: BTreeMap<(String, usize), Vec<(NodeId, &Statement)>> = BTreeMap::new();

        for node in graph.valid_nodes() {
            if let Some(statement) = node.content().statement()
                && !statement.is_negated()
                && statement.is_ground()
                && (1..=Self::MAXIMUM_ARITY).contains(&statement.arguments().len())
                && Self::is_independent(graph, node.id())
            {
                instances
                    .entry((
                        statement.predicate().to_owned(),
                        statement.arguments().len(),
                    ))
                    .or_default()
                    .push((node.id(), statement));
            }
        }

        let mut proposed = 0;

        for ((source, source_arity), examples) in &instances {
            if examples.len() < self.minimum_support {
                continue;
            }

            for (target, target_arity) in instances.keys() {
                if target == source || target_arity > source_arity {
                    continue;
                }

                for mapping in Self::mappings(*source_arity, *target_arity) {
                    let mut support = Vec::new();
                    let mut counterexample = false;

                    for (node, statement) in examples {
                        let arguments = mapping
                            .iter()
                            .map(|position| statement.arguments()[*position].clone())
                            .collect();
                        let consequence = Statement::new(target.as_str(), arguments);

                        if graph.holding(&consequence.complement()).next().is_some() {
                            counterexample = true;
                            break;
                        }

                        if let Some(witness) = graph
                            .holding(&consequence)
                            .find(|id| Self::is_independent(graph, *id))
                        {
                            support.push((*node, witness));
                        }
                    }

                    if counterexample || support.len() < self.minimum_support {
                        continue;
                    }

                    let variables: Vec<Term> = (0..*source_arity)
                        .map(|position| Term::Variable(format!("a{position}")))
                        .collect();

                    let rule = Rule::new(
                        vec![Statement::new(source.as_str(), variables.clone())],
                        Statement::new(
                            target.as_str(),
                            mapping
                                .iter()
                                .map(|position| variables[*position].clone())
                                .collect(),
                        ),
                    )?;

                    let content = Content::Rule(rule);

                    if graph.contains(&content)
                        || content
                            .rule()
                            .is_some_and(|rule| context.knowledge().is_refuted(rule))
                    {
                        continue;
                    }

                    let premises = support
                        .iter()
                        .flat_map(|(example, witness)| [*example, *witness])
                        .collect();

                    output.note(format!(
                        "{content}: {} examples, no counterexample",
                        support.len()
                    ));
                    output.propose(Proposal::new(content, EpistemicKind::Hypothesis, premises));
                    proposed += 1;

                    if proposed >= Self::MAXIMUM_HYPOTHESES {
                        output.note("hypothesis budget reached");
                        return Ok(output);
                    }
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::HypothesisFormation;

    #[test]
    fn enumerates_injective_readings() {
        assert_eq!(
            HypothesisFormation::mappings(2, 2),
            vec![vec![0, 1], vec![1, 0]]
        );
        assert_eq!(HypothesisFormation::mappings(3, 1).len(), 3);
    }
}
