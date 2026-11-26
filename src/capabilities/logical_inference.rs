use std::collections::BTreeSet;

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    reasoning::{epistemic_kind::EpistemicKind, node_id::NodeId, reasoning_graph::ReasoningGraph},
    representation::{
        bindings::Bindings, content::Content, content_kind::ContentKind, rule::Rule,
        statement::Statement,
    },
};

/// Deduction over implications with variables (generalised modus ponens).
///
/// Every valid rule is matched against valid statements, premises being
/// joined in order of selectivity. A complete match yields a deduction
/// justified by the rule and the matched statements. When a premise has no
/// match, the missing statement becomes a need — but only if it is focused
/// (some argument is known) or if it serves an open question or need
/// (backward chaining), so that unfocused curiosity does not explode.
pub(crate) struct LogicalInference {
    descriptor: ModuleDescriptor,
}

/// Remaining premise: its position, its instantiated pattern and its matches.
type PremiseMatches = (usize, Statement, Vec<(NodeId, Bindings)>);

struct Exploration<'a> {
    graph: &'a ReasoningGraph,
    rule_node: NodeId,
    rule: &'a Rule,
    goal: Option<NodeId>,
    budget: usize,
    deductions: BTreeSet<(Statement, Vec<NodeId>)>,
    needs: BTreeSet<(Statement, Vec<NodeId>)>,
    exhausted: bool,
}

impl LogicalInference {
    /// Upper bound on partial matches examined per invocation.
    const MAXIMUM_MATCHES: usize = 200_000;

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "logical_inference",
                "Applies implications to known statements and derives the information needed to apply them.",
            )
            .consuming(&[ContentKind::Statement, ContentKind::Rule, ContentKind::Need])
            .producing(&[EpistemicKind::Deduction, EpistemicKind::Need]),
        }
    }
}

impl CognitiveModule for LogicalInference {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let graph = context.graph();
        let mut output = ModuleOutput::default();
        let mut budget = Self::MAXIMUM_MATCHES;

        let rules: Vec<(NodeId, &Rule)> = graph
            .valid_nodes()
            .filter_map(|node| Some((node.id(), node.content().rule()?)))
            .collect();

        let goals: Vec<(NodeId, &Statement)> = graph
            .valid_nodes()
            .filter_map(|node| Some((node.id(), node.content().need()?)))
            .filter(|(_, pattern)| !graph.answers(pattern))
            .collect();

        let mut deductions = BTreeSet::new();
        let mut needs = BTreeSet::new();

        for (rule_node, rule) in &rules {
            let mut starts = vec![(Bindings::default(), None)];

            for (goal, pattern) in &goals {
                if let Some(bindings) = Bindings::default().unify(rule.conclusion(), pattern) {
                    starts.push((bindings, Some(*goal)));
                }
            }

            for (bindings, goal) in starts {
                let mut exploration = Exploration {
                    graph,
                    rule_node: *rule_node,
                    rule,
                    goal,
                    budget,
                    deductions: BTreeSet::new(),
                    needs: BTreeSet::new(),
                    exhausted: false,
                };

                let remaining: Vec<usize> = (0..rule.premises().len()).collect();
                exploration.explore(&bindings, &mut Vec::new(), &remaining);

                budget = exploration.budget;
                deductions.extend(exploration.deductions);
                needs.extend(exploration.needs);

                if exploration.exhausted {
                    output.note(format!(
                        "match budget of {} exhausted while applying {rule}",
                        Self::MAXIMUM_MATCHES
                    ));
                }
            }
        }

        for (statement, premises) in deductions {
            if graph.holding(&statement).next().is_none() {
                output.propose(Proposal::new(
                    Content::Statement(statement),
                    EpistemicKind::Deduction,
                    premises,
                ));
            }
        }

        for (pattern, premises) in needs {
            if !graph.answers(&pattern) {
                output.propose(Proposal::new(
                    Content::Need(pattern),
                    EpistemicKind::Need,
                    premises,
                ));
            }
        }

        Ok(output)
    }
}

impl Exploration<'_> {
    /// A missing premise is focused when the data bound at least one of
    /// its variables, or when it has no variable at all. Constants written
    /// in the rule do not focus it: they say nothing about the situation.
    fn is_focused(premise: &Statement, instantiated: &Statement) -> bool {
        premise.variables().next().is_none()
            || premise
                .arguments()
                .iter()
                .zip(instantiated.arguments())
                .any(|(written, current)| written.is_variable() && !current.is_variable())
    }

    fn explore(&mut self, bindings: &Bindings, matched: &mut Vec<NodeId>, remaining: &[usize]) {
        if self.budget == 0 {
            self.exhausted = true;
            return;
        }

        if remaining.is_empty() {
            let conclusion = bindings.apply(self.rule.conclusion());

            if conclusion.is_ground() {
                let mut premises = vec![self.rule_node];
                premises.extend_from_slice(matched);
                self.deductions.insert((conclusion, premises));
            }

            return;
        }

        let premises = self.rule.premises();

        let mut candidates: Vec<PremiseMatches> = remaining
            .iter()
            .enumerate()
            .map(|(position, index)| {
                let pattern = bindings.apply(&premises[*index]);
                let matches = self.graph.matching(&premises[*index], bindings).collect();
                (position, pattern, matches)
            })
            .collect();

        // When a premise has no match, the rule cannot apply. Premises that
        // do have matches are still explored first if they can bind some
        // variable of the missing one, so that the resulting need is as
        // precise as possible; otherwise the missing premise becomes a need.
        let missing = candidates
            .iter()
            .position(|(_, _, matches)| matches.is_empty());

        let refinable = missing.is_some_and(|missing| {
            let unbound: BTreeSet<&str> = candidates[missing].1.variables().collect();

            candidates.iter().any(|(_, pattern, matches)| {
                !matches.is_empty() && pattern.variables().any(|name| unbound.contains(name))
            })
        });

        if let Some(missing) = missing.filter(|_| !refinable) {
            let (position, missing, _) = candidates.swap_remove(missing);
            let premise = &premises[remaining[position]];

            if self.goal.is_some() || Self::is_focused(premise, &missing) {
                let mut justification = vec![self.rule_node];
                justification.extend_from_slice(matched);
                justification.extend(self.goal);
                self.needs.insert((missing, justification));
            }

            return;
        }

        let missing_variables: BTreeSet<String> = missing
            .map(|missing| {
                candidates[missing]
                    .1
                    .variables()
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();

        let Some((position, _, matches)) = candidates
            .into_iter()
            .filter(|(_, pattern, matches)| {
                !matches.is_empty()
                    && (missing_variables.is_empty()
                        || pattern
                            .variables()
                            .any(|name| missing_variables.contains(name)))
            })
            .min_by_key(|(_, _, matches)| matches.len())
        else {
            return;
        };

        let rest: Vec<usize> = remaining
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != position)
            .map(|(_, premise)| *premise)
            .collect();

        for (node, next) in matches {
            if self.budget == 0 {
                self.exhausted = true;
                return;
            }

            self.budget -= 1;
            matched.push(node);
            self.explore(&next, matched, &rest);
            matched.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LogicalInference;
    use crate::{
        cognition::{cognitive_module::CognitiveModule, module_context::ModuleContext},
        memory::knowledge_memory::KnowledgeMemory,
        reasoning::{
            epistemic_kind::EpistemicKind, producer::Producer, reasoning_graph::ReasoningGraph,
        },
        representation::content::Content,
        resources::initial_resources::InitialResources,
    };

    #[test]
    fn deduces_and_expresses_focused_needs() {
        let mut graph = ReasoningGraph::default();

        for (content, kind) in [
            (
                Content::Rule("mortal(?x) :- human(?x)".parse().unwrap()),
                EpistemicKind::Premise,
            ),
            (
                Content::Rule("bird(?x) :- flies(?x), feathered(?x)".parse().unwrap()),
                EpistemicKind::Premise,
            ),
            (
                Content::Statement("human(socrates)".parse().unwrap()),
                EpistemicKind::Observation,
            ),
            (
                Content::Statement("flies(tweety)".parse().unwrap()),
                EpistemicKind::Observation,
            ),
        ] {
            graph
                .insert(content, kind, Producer::Perception, vec![], None, 0)
                .unwrap();
        }

        let knowledge = KnowledgeMemory::default();
        let resources = InitialResources::default();
        let module = LogicalInference::new();
        let output = module
            .apply(&ModuleContext::new(&graph, &knowledge, &resources, 0))
            .unwrap();

        let (proposals, _) = output.into_parts();
        let contents: Vec<String> = proposals
            .into_iter()
            .map(|proposal| proposal.into_parts().0.to_string())
            .collect();

        assert!(contents.contains(&"mortal(socrates)".to_owned()));
        assert!(contents.contains(&"need feathered(tweety)".to_owned()));
        assert_eq!(contents.len(), 2);
    }
}
