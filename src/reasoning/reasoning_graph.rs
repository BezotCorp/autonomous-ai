use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;

use crate::{
    reasoning::{
        contradiction::Contradiction, epistemic_kind::EpistemicKind,
        invalidation_cause::InvalidationCause, node_id::NodeId, producer::Producer,
        reasoning_node::ReasoningNode, validity::Validity,
    },
    representation::{
        bindings::Bindings, content::Content, provenance::Provenance, statement::Statement,
    },
};

/// Directed acyclic graph of every result produced during a reasoning.
/// Each node keeps the premises it was derived from, so that invalidations
/// propagate to dependent results and only affected branches are revisited.
/// The same content may be held by several nodes, one per justification.
///
/// The graph also tracks which statement satisfied which need. When a need
/// gets satisfied, the needs it was formulated for are reactivated, so that
/// the modules able to supply them reconsider them now that more is known;
/// when the satisfying statement is invalidated, the need reopens.
#[derive(Debug, Default, Serialize)]
pub(crate) struct ReasoningGraph {
    nodes: Vec<ReasoningNode>,
    satisfactions: BTreeMap<NodeId, NodeId>,
    #[serde(skip)]
    dependents: Vec<Vec<NodeId>>,
    #[serde(skip)]
    by_content: HashMap<Content, Vec<NodeId>>,
    #[serde(skip)]
    by_predicate: BTreeMap<String, Vec<NodeId>>,
    #[serde(skip)]
    needs_by_predicate: BTreeMap<String, Vec<NodeId>>,
    revision: u64,
}

/// Result of an insertion attempt.
#[derive(Debug)]
pub(crate) struct Insertion {
    id: NodeId,
    created: bool,
    contradictions: Vec<(NodeId, NodeId)>,
}

impl Insertion {
    pub(crate) fn id(&self) -> NodeId {
        self.id
    }

    pub(crate) fn created(&self) -> bool {
        self.created
    }

    pub(crate) fn contradictions(&self) -> &[(NodeId, NodeId)] {
        &self.contradictions
    }
}

impl ReasoningGraph {
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn node(&self, id: NodeId) -> Option<&ReasoningNode> {
        self.nodes.get(id.index())
    }

    pub(crate) fn valid_nodes(&self) -> impl Iterator<Item = &ReasoningNode> {
        self.nodes.iter().filter(|node| node.is_valid())
    }

    /// Inserts a justified result. Premises must exist and be valid, and a
    /// result cannot be justified by itself, directly or transitively.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert(
        &mut self,
        content: Content,
        kind: EpistemicKind,
        producer: Producer,
        mut premises: Vec<NodeId>,
        provenance: Option<Provenance>,
        step: usize,
    ) -> Result<Insertion, String> {
        premises.sort_unstable();
        premises.dedup();

        for premise in &premises {
            match self.node(*premise) {
                Some(node) if node.is_valid() => {}
                Some(_) => return Err(format!("Premise {premise} is no longer valid")),
                None => return Err(format!("Unknown premise {premise}")),
            }
        }

        if let Some(existing) = self.by_content.get(&content).and_then(|ids| {
            ids.iter().copied().find(|id| {
                let node = &self.nodes[id.index()];

                node.is_valid()
                    && node.kind() == kind
                    && (kind == EpistemicKind::Need || node.premises() == premises.as_slice())
            })
        }) {
            return Ok(Insertion {
                id: existing,
                created: false,
                contradictions: Vec::new(),
            });
        }

        let ancestors = self.ancestors(&premises);

        if ancestors
            .iter()
            .any(|ancestor| self.nodes[ancestor.index()].content() == &content)
        {
            return Err(format!("Circular justification of {content}"));
        }

        self.revision += 1;

        let id = NodeId::new(self.nodes.len());

        for premise in &premises {
            self.dependents[premise.index()].push(id);
        }

        let contradictions = match &content {
            Content::Statement(statement) => self
                .holding(&statement.complement())
                .map(|other| {
                    if statement.is_negated() {
                        (other, id)
                    } else {
                        (id, other)
                    }
                })
                .collect(),
            _ => Vec::new(),
        };

        match &content {
            Content::Statement(statement) => self
                .by_predicate
                .entry(statement.predicate().to_owned())
                .or_default()
                .push(id),
            Content::Need(pattern) => self
                .needs_by_predicate
                .entry(pattern.predicate().to_owned())
                .or_default()
                .push(id),
            _ => {}
        }

        self.by_content.entry(content.clone()).or_default().push(id);
        self.dependents.push(Vec::new());
        self.nodes.push(ReasoningNode::new(
            id,
            content,
            kind,
            producer,
            premises,
            provenance,
            step,
            self.revision,
        ));

        match self.nodes[id.index()].content().clone() {
            Content::Statement(statement) => self.satisfy_needs(id, &statement),
            Content::Need(pattern) => {
                if let Some(answer) = self.answer_to(&pattern) {
                    self.satisfactions.insert(id, answer);
                    self.wake(id);
                }
            }
            _ => {}
        }

        Ok(Insertion {
            id,
            created: true,
            contradictions,
        })
    }

    /// Results directly justified by this node.
    pub(crate) fn dependents(&self, id: NodeId) -> &[NodeId] {
        &self.dependents[id.index()]
    }

    /// Statement that satisfied each need, for the needs satisfied so far.
    pub(crate) fn satisfactions(&self) -> &BTreeMap<NodeId, NodeId> {
        &self.satisfactions
    }

    /// Whether the statement answers the pattern (see `answers`).
    fn fulfils(pattern: &Statement, statement: &Statement) -> bool {
        let bindings = Bindings::default();

        bindings.unify(pattern, statement).is_some()
            || (pattern.is_ground() && bindings.unify(&pattern.complement(), statement).is_some())
    }

    /// A valid statement node answering the pattern, if any.
    fn answer_to(&self, pattern: &Statement) -> Option<NodeId> {
        self.by_predicate
            .get(pattern.predicate())
            .into_iter()
            .flatten()
            .copied()
            .find(|id| {
                let node = &self.nodes[id.index()];

                node.is_valid()
                    && node
                        .content()
                        .statement()
                        .is_some_and(|statement| Self::fulfils(pattern, statement))
            })
    }

    /// Records the open needs a new statement satisfies and wakes up the
    /// needs they were formulated for.
    fn satisfy_needs(&mut self, statement_id: NodeId, statement: &Statement) {
        let satisfied: Vec<NodeId> = self
            .needs_by_predicate
            .get(statement.predicate())
            .into_iter()
            .flatten()
            .copied()
            .filter(|need| {
                let node = &self.nodes[need.index()];

                node.is_valid()
                    && !self.satisfactions.contains_key(need)
                    && node
                        .content()
                        .need()
                        .is_some_and(|pattern| Self::fulfils(pattern, statement))
            })
            .collect();

        for need in satisfied {
            self.satisfactions.insert(need, statement_id);
            self.wake(need);
        }
    }

    /// Reactivates, transitively, the still open needs that motivated a
    /// need, by giving them a new revision.
    fn wake(&mut self, need: NodeId) {
        let mut pending: Vec<NodeId> = self.nodes[need.index()].premises().to_vec();
        let mut visited = BTreeSet::new();

        while let Some(id) = pending.pop() {
            let node = &self.nodes[id.index()];

            if !visited.insert(id) || !node.is_valid() || node.content().need().is_none() {
                continue;
            }

            pending.extend_from_slice(node.premises());

            if !self.satisfactions.contains_key(&id) {
                self.touch(id);
            }
        }
    }

    fn touch(&mut self, id: NodeId) {
        self.revision += 1;
        self.nodes[id.index()].touch(self.revision);
    }

    /// Whether this content was ever held, valid or not.
    pub(crate) fn contains(&self, content: &Content) -> bool {
        self.by_content.contains_key(content)
    }

    /// Valid nodes asserting exactly this ground statement.
    pub(crate) fn holding(&self, statement: &Statement) -> impl Iterator<Item = NodeId> + '_ {
        self.by_content
            .get(&Content::Statement(statement.clone()))
            .into_iter()
            .flatten()
            .copied()
            .filter(|id| self.nodes[id.index()].is_valid())
    }

    /// Valid statement nodes unifying with the pattern, with their bindings.
    pub(crate) fn matching<'a>(
        &'a self,
        pattern: &'a Statement,
        bindings: &'a Bindings,
    ) -> impl Iterator<Item = (NodeId, Bindings)> + 'a {
        self.by_predicate
            .get(pattern.predicate())
            .into_iter()
            .flatten()
            .filter_map(move |id| {
                let node = &self.nodes[id.index()];

                if !node.is_valid() {
                    return None;
                }

                let statement = node.content().statement()?;

                bindings
                    .unify(pattern, statement)
                    .map(|bindings| (*id, bindings))
            })
    }

    /// Whether a valid statement answers the pattern. A ground pattern is
    /// answered by the statement or by its negation; a pattern with
    /// variables only by a matching instance, since the negation of one
    /// instance says nothing about the others.
    pub(crate) fn answers(&self, pattern: &Statement) -> bool {
        let bindings = Bindings::default();

        self.matching(pattern, &bindings).next().is_some()
            || (pattern.is_ground()
                && self
                    .matching(&pattern.complement(), &bindings)
                    .next()
                    .is_some())
    }

    /// Transitive premises of the given nodes, the nodes themselves included.
    pub(crate) fn ancestors(&self, roots: &[NodeId]) -> BTreeSet<NodeId> {
        let mut visited = BTreeSet::new();
        let mut pending = roots.to_vec();

        while let Some(id) = pending.pop() {
            if visited.insert(id) {
                pending.extend_from_slice(self.nodes[id.index()].premises());
            }
        }

        visited
    }

    /// Valid results that would be lost if this node were invalidated,
    /// the node itself included.
    pub(crate) fn descendants(&self, root: NodeId) -> BTreeSet<NodeId> {
        let mut visited = BTreeSet::new();
        let mut pending = vec![root];

        while let Some(id) = pending.pop() {
            if self.nodes[id.index()].is_valid() && visited.insert(id) {
                pending.extend_from_slice(&self.dependents[id.index()]);
            }
        }

        visited
    }

    /// Invalidates a node and every result depending on it.
    pub(crate) fn invalidate(
        &mut self,
        id: NodeId,
        cause: InvalidationCause,
        step: usize,
    ) -> Vec<NodeId> {
        let mut invalidated = Vec::new();
        let mut pending = vec![(id, cause)];

        while let Some((id, cause)) = pending.pop() {
            if !self.nodes[id.index()].is_valid() {
                continue;
            }

            self.revision += 1;
            self.nodes[id.index()].invalidate(Validity::Invalidated { cause, step }, self.revision);
            invalidated.push(id);

            for dependent in &self.dependents[id.index()] {
                pending.push((*dependent, InvalidationCause::DependsOn { premise: id }));
            }
        }

        let lost: Vec<NodeId> = self
            .satisfactions
            .iter()
            .filter(|(need, by)| {
                !self.nodes[by.index()].is_valid() && self.nodes[need.index()].is_valid()
            })
            .map(|(need, _)| *need)
            .collect();

        for need in lost {
            self.satisfactions.remove(&need);

            let pattern = self.nodes[need.index()].content().need().cloned();

            match pattern.and_then(|pattern| self.answer_to(&pattern)) {
                Some(answer) => {
                    self.satisfactions.insert(need, answer);
                }
                None => self.touch(need),
            }
        }

        self.satisfactions
            .retain(|need, _| self.nodes[need.index()].is_valid());

        invalidated
    }

    /// Contradiction currently standing between two nodes, if both are valid.
    pub(crate) fn is_standing(&self, contradiction: &Contradiction) -> bool {
        contradiction
            .sides()
            .iter()
            .all(|side| self.nodes[side.index()].is_valid())
    }
}

#[cfg(test)]
mod tests {
    use super::ReasoningGraph;
    use crate::{
        reasoning::{
            epistemic_kind::EpistemicKind, invalidation_cause::InvalidationCause,
            producer::Producer,
        },
        representation::content::Content,
    };

    fn statement(text: &str) -> Content {
        Content::Statement(text.parse().unwrap())
    }

    #[test]
    fn propagates_invalidation_and_detects_contradictions() {
        let mut graph = ReasoningGraph::default();
        let producer = Producer::Perception;

        let fact = graph
            .insert(
                statement("p(a)"),
                EpistemicKind::Observation,
                producer.clone(),
                vec![],
                None,
                0,
            )
            .unwrap()
            .id();

        let derived = graph
            .insert(
                statement("q(a)"),
                EpistemicKind::Deduction,
                producer.clone(),
                vec![fact],
                None,
                1,
            )
            .unwrap()
            .id();

        assert!(
            graph
                .insert(
                    statement("p(a)"),
                    EpistemicKind::Deduction,
                    producer.clone(),
                    vec![derived],
                    None,
                    2
                )
                .is_err()
        );

        let negation = graph
            .insert(
                statement("not q(a)"),
                EpistemicKind::Observation,
                producer,
                vec![],
                None,
                2,
            )
            .unwrap();

        assert_eq!(negation.contradictions(), &[(derived, negation.id())]);

        let invalidated =
            graph.invalidate(fact, InvalidationCause::Contested { contradiction: 0 }, 3);

        assert_eq!(invalidated.len(), 2);
        assert!(!graph.node(derived).unwrap().is_valid());
    }

    #[test]
    fn wakes_motivating_needs_and_reopens_lost_satisfactions() {
        let mut graph = ReasoningGraph::default();
        let producer = Producer::Perception;
        let need = |text: &str| Content::Need(text.parse().unwrap());

        let goal = graph
            .insert(
                need("fact(?x)"),
                EpistemicKind::Need,
                producer.clone(),
                vec![],
                None,
                0,
            )
            .unwrap()
            .id();
        let sub = graph
            .insert(
                need("item(?x)"),
                EpistemicKind::Need,
                producer.clone(),
                vec![goal],
                None,
                1,
            )
            .unwrap()
            .id();

        let before = graph.node(goal).unwrap().revision();
        let answer = graph
            .insert(
                statement("item(a)"),
                EpistemicKind::Recollection,
                producer,
                vec![],
                None,
                2,
            )
            .unwrap()
            .id();

        assert_eq!(graph.satisfactions().get(&sub), Some(&answer));
        assert!(graph.node(goal).unwrap().revision() > before);

        graph.invalidate(answer, InvalidationCause::Contested { contradiction: 0 }, 3);

        assert!(!graph.satisfactions().contains_key(&sub));
        assert!(!graph.answers(&"item(?x)".parse().unwrap()));
    }
}
