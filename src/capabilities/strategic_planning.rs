use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    cognition::{
        capability_model::CapabilityModel, cognitive_module::CognitiveModule,
        module_context::ModuleContext, module_descriptor::ModuleDescriptor,
        module_output::ModuleOutput, proposal::Proposal,
    },
    mathematics::beta_distribution::BetaDistribution,
    planning::{plan::Plan, plan_estimate::PlanEstimate, plan_step::PlanStep},
    reasoning::{epistemic_kind::EpistemicKind, node_id::NodeId},
    representation::{
        bindings::Bindings, content::Content, content_kind::ContentKind, statement::Statement,
        term::Term,
    },
};

/// Anticipates what solving the open problems will require and plans it.
///
/// From the goals (open needs), it regresses through the known rules and
/// the self-knowledge of the intelligence (what each module declares it
/// supplies, what it learned to need first, what the memory holds) to the
/// information that will predictably be needed. Every combination of ways
/// of obtaining it is a strategy; strategies are compared on their
/// expected success, duration, consumption of scarce resources and, when
/// already tried, on their observed effectiveness. The best one becomes a
/// plan: the anticipated needs are formulated at once, so that a module
/// can acquire everything it will supply in one invocation, and the plan
/// informs, without dictating, the choice of the next modules.
pub(crate) struct StrategicPlanning {
    descriptor: ModuleDescriptor,
}

/// Information a plan must obtain, with the ways of obtaining it.
struct Requirement {
    patterns: Vec<Statement>,
    goal: NodeId,
    depth: usize,
    methods: Vec<Method>,
}

/// One way of obtaining the information of a requirement: from a module
/// declaring it can supply it (after its usual prerequisites), or by
/// applying a rule to further information.
#[derive(Clone)]
struct Method {
    module: String,
    remembered: Option<f64>,
    requires: Vec<Statement>,
}

/// A module of a strategy: depth of the deepest requirement it supplies,
/// signatures it supplies and signatures they require.
type ModuleRole = (usize, Vec<String>, BTreeSet<String>);

/// Evaluated combination of methods.
struct Strategy {
    key: String,
    assignment: BTreeMap<String, usize>,
    estimate: PlanEstimate,
    reused: bool,
}

impl StrategicPlanning {
    const MAXIMUM_DEPTH: usize = 6;
    const MAXIMUM_REQUIREMENTS: usize = 64;
    const MAXIMUM_STRATEGIES: usize = 64;
    const REPORTED_ALTERNATIVES: usize = 5;
    /// Success factor of information no known method can obtain: it may
    /// still arrive (from an event, from an unexpected module).
    const UNSUPPLIED_SUCCESS: f64 = 0.5;

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "strategic_planning",
                "Anticipates the information open problems require and plans how to obtain it.",
            )
            .consuming(&[ContentKind::Need])
            .producing(&[EpistemicKind::Need])
            .planning(),
        }
    }

    fn pattern_of(signature: &str) -> Option<Statement> {
        let (predicate, arity) = signature.rsplit_once('/')?;
        let arity: usize = arity.parse().ok()?;

        Some(Statement::new(
            predicate,
            (0..arity)
                .map(|index| Term::Variable(format!("p{index}")))
                .collect(),
        ))
    }

    /// Ways of obtaining a pattern with the current capabilities.
    fn methods(&self, context: &ModuleContext<'_>, pattern: &Statement) -> Vec<Method> {
        let capabilities = context.capabilities();
        let signature = pattern.signature();
        let mut methods = Vec::new();

        for descriptor in capabilities.descriptors() {
            if descriptor.plans() || !descriptor.consumes(ContentKind::Need) {
                continue;
            }

            if descriptor.declares(&signature) {
                let requires = capabilities
                    .prerequisites(descriptor.name())
                    .iter()
                    .filter(|prerequisite| **prerequisite != signature)
                    .filter_map(|prerequisite| Self::pattern_of(prerequisite))
                    .collect();

                methods.push(Method {
                    module: descriptor.name().to_owned(),
                    remembered: None,
                    requires,
                });
            } else if descriptor.supplies(&signature)
                && descriptor.produces(EpistemicKind::Recollection)
                && let Some(staleness) = capabilities.remembered(&signature)
            {
                methods.push(Method {
                    module: descriptor.name().to_owned(),
                    remembered: Some(staleness),
                    requires: Vec::new(),
                });
            }
        }

        let appliers: Vec<&ModuleDescriptor> = capabilities
            .descriptors()
            .iter()
            .filter(|descriptor| {
                descriptor.consumes(ContentKind::Rule)
                    && descriptor.produces(EpistemicKind::Deduction)
            })
            .collect();

        if let Some(applier) = appliers.first() {
            for node in context.graph().valid_nodes() {
                let Some(rule) = node.content().rule() else {
                    continue;
                };

                let Some(bindings) = Bindings::default().unify(rule.conclusion(), pattern) else {
                    continue;
                };

                methods.push(Method {
                    module: applier.name().to_owned(),
                    remembered: None,
                    requires: rule
                        .premises()
                        .iter()
                        .map(|premise| bindings.apply(premise))
                        .collect(),
                });
            }
        }

        methods
    }

    /// Regresses from the goals to every requirement, keyed by signature.
    fn requirements(
        &self,
        context: &ModuleContext<'_>,
        goals: &[(NodeId, &Statement)],
    ) -> BTreeMap<String, Requirement> {
        let graph = context.graph();
        let mut requirements: BTreeMap<String, Requirement> = BTreeMap::new();
        let mut pending: VecDeque<(Statement, NodeId, usize)> = goals
            .iter()
            .map(|(id, pattern)| ((*pattern).clone(), *id, 0))
            .collect();

        while let Some((pattern, goal, depth)) = pending.pop_front() {
            if graph.answers(&pattern) {
                continue;
            }

            let signature = pattern.signature();

            if let Some(requirement) = requirements.get_mut(&signature) {
                if !requirement.patterns.contains(&pattern) {
                    requirement.patterns.push(pattern);
                }

                continue;
            }

            if requirements.len() >= Self::MAXIMUM_REQUIREMENTS {
                break;
            }

            let methods = if depth < Self::MAXIMUM_DEPTH {
                self.methods(context, &pattern)
            } else {
                Vec::new()
            };

            for method in &methods {
                for required in &method.requires {
                    pending.push_back((required.clone(), goal, depth + 1));
                }
            }

            requirements.insert(
                signature,
                Requirement {
                    patterns: vec![pattern],
                    goal,
                    depth,
                    methods,
                },
            );
        }

        requirements
    }

    /// Requirements reached from the goals under an assignment of methods.
    fn active(
        requirements: &BTreeMap<String, Requirement>,
        goals: &BTreeSet<String>,
        assignment: &BTreeMap<String, usize>,
    ) -> Vec<String> {
        let mut active = Vec::new();
        let mut pending: Vec<String> = goals.iter().cloned().collect();

        while let Some(signature) = pending.pop() {
            let Some(requirement) = requirements.get(&signature) else {
                continue;
            };

            if active.contains(&signature) {
                continue;
            }

            active.push(signature.clone());

            if let Some(method) = assignment
                .get(&signature)
                .and_then(|choice| requirement.methods.get(*choice))
            {
                pending.extend(method.requires.iter().map(Statement::signature));
            }
        }

        active
    }

    fn evaluate(
        context: &ModuleContext<'_>,
        requirements: &BTreeMap<String, Requirement>,
        goals: &BTreeSet<String>,
        assignment: BTreeMap<String, usize>,
        goal_key: &str,
    ) -> Strategy {
        let capabilities: &CapabilityModel<'_> = context.capabilities();
        let meter = context.access().meter();
        let parameters = context.parameters();
        let active = Self::active(requirements, goals, &assignment);

        let mut supplied: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        let mut success = 1.0;
        let mut entries = Vec::new();

        for signature in &active {
            let requirement = &requirements[signature];

            match assignment
                .get(signature)
                .and_then(|choice| requirement.methods.get(*choice))
            {
                Some(method) => {
                    supplied
                        .entry(method.module.as_str())
                        .or_default()
                        .push(format!("need:{signature}"));

                    if let Some(staleness) = method.remembered {
                        success *= 1.0 - staleness;
                    }

                    entries.push(format!("{signature}<-{}", method.module));
                }
                None => {
                    success *= Self::UNSUPPLIED_SUCCESS;
                    entries.push(format!("{signature}<-?"));
                }
            }
        }

        entries.sort();

        let mut seconds = 0.0;
        let mut consumption: BTreeMap<String, f64> = BTreeMap::new();

        for (module, features) in &supplied {
            success *= capabilities.success(module, features);
            seconds += capabilities.seconds(module);

            for (resource, units) in capabilities.consumption(module) {
                *consumption.entry(resource).or_default() += units;
            }
        }

        let scarcity = meter.scarcity_cost(&consumption);
        let mut cost =
            parameters.cost_weight_per_second() * seconds + parameters.scarcity_weight() * scarcity;
        let key = entries.join(", ");
        let mut reused = false;

        // A strategy already tried is judged on its observed effectiveness;
        // one only tried in dreams, on its simulated effectiveness, which
        // counts less than real executions.
        if let Some(statistic) = capabilities.strategy(goal_key, &key) {
            let weight = parameters.simulation_weight();
            let simulated_failures =
                statistic.simulated_executions() - statistic.simulated_achieved();

            if let Some(observed_cost) = statistic.mean_cost().or_else(|| {
                (weight > 0.0)
                    .then_some(statistic.mean_simulated_cost())
                    .flatten()
            }) {
                success = BetaDistribution::posterior(
                    parameters.prior_successes() + weight * statistic.simulated_achieved() as f64,
                    parameters.prior_failures() + weight * simulated_failures as f64,
                    statistic.achieved(),
                    statistic.executions() - statistic.achieved(),
                )
                .mean();
                cost = observed_cost;
                reused = true;
            }
        }

        Strategy {
            key,
            assignment,
            estimate: PlanEstimate::new(success, seconds, consumption, scarcity, cost),
            reused,
        }
    }

    /// Every distinct strategy, up to a bound, by varying the method of
    /// each requirement that has several.
    fn strategies(
        context: &ModuleContext<'_>,
        requirements: &BTreeMap<String, Requirement>,
        goals: &BTreeSet<String>,
        goal_key: &str,
    ) -> Vec<Strategy> {
        let choices: Vec<(&String, usize)> = requirements
            .iter()
            .filter(|(_, requirement)| requirement.methods.len() > 1)
            .map(|(signature, requirement)| (signature, requirement.methods.len()))
            .collect();

        let base: BTreeMap<String, usize> = requirements
            .iter()
            .filter(|(_, requirement)| !requirement.methods.is_empty())
            .map(|(signature, _)| (signature.clone(), 0))
            .collect();

        let mut strategies: Vec<Strategy> = Vec::new();
        let mut counters = vec![0_usize; choices.len()];

        loop {
            let mut assignment = base.clone();

            for ((signature, _), choice) in choices.iter().zip(&counters) {
                assignment.insert((*signature).clone(), *choice);
            }

            let strategy = Self::evaluate(context, requirements, goals, assignment, goal_key);

            if !strategies.iter().any(|known| known.key == strategy.key) {
                strategies.push(strategy);
            }

            if strategies.len() >= Self::MAXIMUM_STRATEGIES {
                break;
            }

            // Next combination (mixed-radix counter).
            let mut position = 0;

            loop {
                if position == counters.len() {
                    return strategies;
                }

                counters[position] += 1;

                if counters[position] < choices[position].1 {
                    break;
                }

                counters[position] = 0;
                position += 1;
            }
        }

        strategies
    }

    /// Steps of a strategy: one per module, ordered so that a module comes
    /// after those supplying what it requires.
    fn steps(
        requirements: &BTreeMap<String, Requirement>,
        goals: &BTreeSet<String>,
        strategy: &Strategy,
    ) -> Vec<PlanStep> {
        let active = Self::active(requirements, goals, &strategy.assignment);
        let mut modules: BTreeMap<String, ModuleRole> = BTreeMap::new();

        for signature in &active {
            let requirement = &requirements[signature];

            if let Some(method) = strategy
                .assignment
                .get(signature)
                .and_then(|choice| requirement.methods.get(*choice))
            {
                let entry = modules.entry(method.module.clone()).or_default();

                entry.0 = entry.0.max(requirement.depth);
                entry.1.push(signature.clone());
                entry
                    .2
                    .extend(method.requires.iter().map(Statement::signature));
            }
        }

        // Deeper requirements first: a step only depends on earlier steps,
        // which keeps the plan acyclic.
        let mut ordered: Vec<(String, ModuleRole)> = modules.into_iter().collect();

        ordered.sort_by(|left, right| right.1.0.cmp(&left.1.0).then(left.0.cmp(&right.0)));

        let mut steps: Vec<PlanStep> = Vec::new();

        for (index, (module, (_, supplies, requires))) in ordered.iter().enumerate() {
            let depends_on = ordered[..index]
                .iter()
                .enumerate()
                .filter(|(_, (other, (_, other_supplies, _)))| {
                    other != module
                        && other_supplies
                            .iter()
                            .any(|supplied| requires.contains(supplied))
                })
                .map(|(position, _)| position)
                .collect();

            steps.push(PlanStep::new(module.clone(), supplies.clone(), depends_on));
        }

        steps
    }
}

impl CognitiveModule for StrategicPlanning {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let graph = context.graph();
        let mut output = ModuleOutput::default();

        let open: Vec<(NodeId, &Statement)> = graph
            .valid_nodes()
            .filter_map(|node| Some((node.id(), node.content().need()?)))
            .filter(|(_, pattern)| !graph.answers(pattern))
            .collect();

        // Goals are the open needs not formulated for another need.
        let roots: Vec<(NodeId, &Statement)> = open
            .iter()
            .copied()
            .filter(|(id, _)| {
                graph.node(*id).is_some_and(|node| {
                    node.premises().iter().all(|premise| {
                        graph
                            .node(*premise)
                            .is_none_or(|premise| premise.content().need().is_none())
                    })
                })
            })
            .collect();

        let goals = if roots.is_empty() { open } else { roots };

        if goals.is_empty() {
            output.note("no open need to plan for");
            return Ok(output);
        }

        let goal_patterns: Vec<Statement> = goals
            .iter()
            .map(|(_, pattern)| (*pattern).clone())
            .collect();
        let goal_key = Plan::goal_key(&goal_patterns);
        let goal_signatures: BTreeSet<String> =
            goal_patterns.iter().map(Statement::signature).collect();
        let requirements = self.requirements(context, &goals);
        let mut strategies = Self::strategies(context, &requirements, &goal_signatures, &goal_key);

        strategies.sort_by(|left, right| {
            right
                .estimate
                .utility()
                .total_cmp(&left.estimate.utility())
                .then(left.key.cmp(&right.key))
        });

        let Some(best) = strategies.first() else {
            output.note("no strategy reaches the open needs");
            return Ok(output);
        };

        let alternatives = strategies
            .iter()
            .take(Self::REPORTED_ALTERNATIVES)
            .map(|strategy| (strategy.key.clone(), strategy.estimate.utility()))
            .collect();

        output.note(format!(
            "{} strategies compared for {goal_key}; chosen: {} (utility {:.3}{})",
            strategies.len(),
            best.key,
            best.estimate.utility(),
            if best.reused { ", from experience" } else { "" }
        ));

        // Anticipated needs: everything the chosen strategy will require.
        for signature in Self::active(&requirements, &goal_signatures, &best.assignment) {
            if goal_signatures.contains(&signature) {
                continue;
            }

            let requirement = &requirements[&signature];

            for pattern in &requirement.patterns {
                if !graph.contains(&Content::Need(pattern.clone())) {
                    output.propose(Proposal::new(
                        Content::Need(pattern.clone()),
                        EpistemicKind::Need,
                        vec![requirement.goal],
                    ));
                }
            }
        }

        let unchanged = context.active_plan().is_some_and(|plan| {
            plan.strategy() == best.key && plan.goals() == goal_patterns.as_slice()
        });

        if !unchanged {
            output.plan(Plan::new(
                goal_patterns,
                best.key.clone(),
                Self::steps(&requirements, &goal_signatures, best),
                best.estimate.clone(),
                alternatives,
                best.reused,
            ));
        }

        Ok(output)
    }
}
