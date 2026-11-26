use std::{collections::BTreeMap, time::Instant};

use chrono::Utc;

use crate::{
    acquisition::access_mode::AccessMode,
    engine::{disposition::Disposition, dreaming::Dreaming, intelligence::Intelligence},
    events::event::Event,
    mathematics::random_source::RandomSource,
    memory::{
        activity::Activity, dream_mode::DreamMode, dream_record::DreamRecord,
        dream_scene::DreamScene, episode_configuration::EpisodeConfiguration,
        experiment_request::ExperimentRequest, imagined_capability_spec::ImaginedCapabilitySpec,
        imagined_element::ImaginedElement, persistent_memory::PersistentMemory,
    },
    reasoning::epistemic_kind::EpistemicKind,
    representation::{content::Content, rule::Rule, statement::Statement, term::Term},
};

/// Free unfolding of a dream. A dream is a succession of scenes, each one
/// an ordinary episode of the engine on an isolated copy of the memory,
/// whose setting the imagination composes with generic mechanisms over the
/// material it finds in memory (facts, rules, past experiences, previous
/// dreams, impasses, its representation of itself):
///
/// - recalling facts, or replaying a past experience from recorded data;
/// - associating memories that share an entity, and conjecturing a link;
/// - transforming a representation (substituting an entity or a relation,
///   negating, dropping a condition);
/// - inventing entities, relations and rules;
/// - imagining a capability it lacks, which is simulated in the dream;
/// - anticipating questions never asked;
/// - changing the constraints on resources;
/// - representing itself, possibly otherwise.
///
/// Scenes follow one another by continuation, association, jump, recall or
/// anticipation, and the dream may stop at any point. Chance drives the
/// choices, structure provides the material. Nothing is required to be
/// coherent, realistic or useful; everything is recorded with its
/// provenance, and nothing reaches the environment or the real memory.
pub(crate) struct Imagination<'a> {
    intelligence: &'a Intelligence,
    memory: &'a PersistentMemory,
    random: RandomSource,
    material: Material,
    invented: usize,
    seed: u64,
    activity: Activity,
    focus: Option<Statement>,
}

/// What the imagination draws from.
#[derive(Default)]
struct Material {
    facts: Vec<Statement>,
    rules: Vec<Rule>,
    predicates: Vec<(String, usize)>,
    entities: Vec<Term>,
    experiences: Vec<uuid::Uuid>,
    needs: Vec<Statement>,
    myself: Vec<Statement>,
    resources: Vec<String>,
}

/// Setting of a scene under construction.
#[derive(Clone, Default)]
struct Setting {
    elements: Vec<ImaginedElement>,
    capabilities: Vec<ImaginedCapabilitySpec>,
    questions: Vec<Statement>,
    budgets: BTreeMap<String, f64>,
}

impl<'a> Imagination<'a> {
    /// Bound on the scenes of one dream and on the reasoning steps of one
    /// scene: the computational constraints of dreaming.
    const MAXIMUM_SCENES: usize = 6;
    const SCENE_INVOCATIONS: usize = 24;
    const MATERIAL_FACTS: usize = 400;
    const SETTING_ELEMENTS: usize = 40;
    const ASSOCIATED_FACTS: usize = 6;
    const MECHANISMS: [&'static str; 8] = [
        "recollection",
        "association",
        "transformation",
        "invention",
        "imagined_capability",
        "anticipation",
        "constraint",
        "self_representation",
    ];

    pub(crate) fn new(
        intelligence: &'a Intelligence,
        memory: &'a PersistentMemory,
        seed: u64,
    ) -> Self {
        let mut random = RandomSource::new(seed);
        let material = Self::material(intelligence, memory, &mut random);

        Self {
            intelligence,
            memory,
            random,
            material,
            invented: 0,
            seed,
            activity: Activity::Dream,
            focus: None,
        }
    }

    /// The kind of imaginative activity, and what it is directed towards,
    /// if anything (a simulation keeps its focus as a question in every
    /// scene).
    pub(crate) fn directed(mut self, activity: Activity, focus: Option<Statement>) -> Self {
        self.activity = activity;
        self.focus = focus;
        self
    }

    /// Themes the imagination may draw on besides memory (for instance the
    /// concerns of the moment): they join the needs and relations it uses.
    pub(crate) fn themed(mut self, themes: &[Statement]) -> Self {
        for theme in themes {
            self.material
                .predicates
                .push((theme.predicate().to_owned(), theme.arguments().len()));
            self.material.needs.push(theme.clone());
        }

        self.material.predicates.sort();
        self.material.predicates.dedup();
        self
    }

    fn material(
        intelligence: &Intelligence,
        memory: &PersistentMemory,
        random: &mut RandomSource,
    ) -> Material {
        let mut material = Material::default();
        let mut seen = 0;

        // A uniform sample of what is believed.
        for entry in memory.knowledge().active_entries() {
            seen += 1;

            if material.facts.len() < Self::MATERIAL_FACTS {
                material.facts.push(entry.statement().clone());
            } else {
                let slot = (random.uniform() * seen as f64) as usize;

                if slot < Self::MATERIAL_FACTS {
                    material.facts[slot] = entry.statement().clone();
                }
            }
        }

        material
            .rules
            .extend(intelligence.resources().knowledge().rules().iter().cloned());
        material.rules.extend(
            memory
                .knowledge()
                .hypotheses()
                .map(|record| record.rule().clone()),
        );

        for dream in memory.dreams().dreams() {
            for scene in dream.scenes() {
                material.needs.extend(scene.emerged_needs().iter().cloned());

                for element in scene.elements() {
                    if let Content::Rule(rule) = element.content() {
                        material.rules.push(rule.clone());
                    }
                }
            }
        }

        for episode in memory.real_episodes() {
            if episode.event().is_some() && episode.activity() == Activity::Action {
                material.experiences.push(episode.id());
            }

            material.needs.extend(
                episode
                    .impasses()
                    .iter()
                    .filter(|impasse| impasse.kind().is_root())
                    .map(|impasse| impasse.need().clone()),
            );

            for question in episode.questions() {
                material
                    .predicates
                    .push((question.predicate().to_owned(), question.arguments().len()));
            }
        }

        for fact in &material.facts {
            material
                .predicates
                .push((fact.predicate().to_owned(), fact.arguments().len()));
            material.entities.extend(
                fact.arguments()
                    .iter()
                    .filter(|term| !term.is_variable())
                    .cloned(),
            );
        }

        for rule in &material.rules {
            for statement in rule.premises().iter().chain([rule.conclusion()]) {
                material.predicates.push((
                    statement.predicate().to_owned(),
                    statement.arguments().len(),
                ));
            }
        }

        for descriptor in intelligence.descriptors() {
            for signature in descriptor.supplied() {
                material.myself.push(Statement::new(
                    "capability",
                    vec![
                        Term::symbol(descriptor.name()),
                        Term::text(signature.clone()),
                    ],
                ));
            }
        }

        material.predicates.sort();
        material.predicates.dedup();
        material.entities.sort();
        material.entities.dedup();
        material.resources = memory.resources().keys().cloned().collect();
        material
    }

    fn pick<'b, T>(random: &mut RandomSource, items: &'b [T]) -> Option<&'b T> {
        (!items.is_empty()).then(|| {
            &items[((random.uniform() * items.len() as f64) as usize).min(items.len() - 1)]
        })
    }

    fn chance(&mut self, probability: f64) -> bool {
        self.random.uniform() < probability
    }

    fn fresh(&mut self, kind: &str) -> String {
        self.invented += 1;
        format!("imagined_{kind}_{}", self.invented)
    }

    /// Dreams without any internal state, from a requested experiment or
    /// from nothing.
    #[cfg(test)]
    pub(crate) fn dream(self, request: Option<&ExperimentRequest>) -> DreamRecord {
        self.unfold(
            request,
            &mut crate::engine::neutral_disposition::NeutralDisposition,
        )
    }

    /// Unfolds, from a requested experiment or from nothing, under the
    /// modulation of an internal state that every scene affects in turn.
    pub(crate) fn unfold(
        mut self,
        request: Option<&ExperimentRequest>,
        disposition: &mut dyn Disposition,
    ) -> DreamRecord {
        let started_at = Utc::now();
        let clock = Instant::now();
        let mut scenes = Vec::new();
        let mut setting = Setting::default();
        let reason = request
            .map(|request| request.reason().to_owned())
            .unwrap_or_else(|| "spontaneous".to_owned());

        let mut sandbox = match request
            .map(|request| (request, self.memory.experiences().find(request.episode())))
        {
            Some((request, Some(original))) => {
                let mut sandbox = match request.mode() {
                    DreamMode::Historical => self.memory.as_of(original.started_at()),
                    DreamMode::Reinterpretation => self.memory.clone(),
                };

                scenes.push(Dreaming::replay(
                    self.intelligence,
                    &mut sandbox,
                    original,
                    request.mode(),
                    request.variant().cloned(),
                    self.seed,
                    "requested".to_owned(),
                    disposition,
                ));
                setting.questions = original.questions().to_vec();
                sandbox
            }
            Some((request, None)) => {
                scenes.push(DreamScene::failed(
                    vec!["replay".to_owned()],
                    Some(request.episode()),
                    "the experience to replay is no longer recorded".to_owned(),
                ));
                self.memory.clone()
            }
            None => self.memory.clone(),
        };

        // The dream unfolds on its own from there.
        while scenes.len() < Self::MAXIMUM_SCENES {
            let interruption = 1.0 - disposition.modulation().scene_continuation();

            if !scenes.is_empty() && self.chance(interruption) {
                break;
            }

            let transition = if scenes.is_empty() {
                "beginning"
            } else {
                *Self::pick(
                    &mut self.random,
                    &[
                        "continuation",
                        "association",
                        "jump",
                        "recall",
                        "anticipation",
                    ],
                )
                .unwrap_or(&"continuation")
            };

            if transition == "recall"
                && let Some(episode) =
                    Self::pick(&mut self.random, &self.material.experiences).copied()
                && let Some(original) = self.memory.experiences().find(episode)
            {
                let seed = self.random.next_u64();

                scenes.push(Dreaming::replay(
                    self.intelligence,
                    &mut sandbox,
                    original,
                    DreamMode::Reinterpretation,
                    None,
                    seed,
                    transition.to_owned(),
                    disposition,
                ));
                setting.questions = original.questions().to_vec();
                continue;
            }

            let weights = *disposition.modulation().mechanism_weights();
            let (mut next, mechanisms) = self.compose(&setting, transition, &weights);

            if let Some(focus) = &self.focus
                && !next.questions.contains(focus)
            {
                next.questions.insert(0, focus.clone());
            }

            let scene = self.enact(
                &mut sandbox,
                &next,
                mechanisms,
                transition,
                scenes.len(),
                disposition,
            );

            setting = next;
            scenes.push(scene);
        }

        let hypotheses = sandbox
            .knowledge()
            .hypotheses()
            .filter(|record| self.memory.knowledge().hypothesis(record.rule()).is_none())
            .map(|record| record.rule().clone())
            .collect();

        DreamRecord::new(
            self.activity,
            self.focus.clone(),
            started_at,
            reason,
            scenes,
            hypotheses,
            clock.elapsed().as_secs_f64(),
        )
    }

    /// Composes the setting of the next scene.
    fn compose(
        &mut self,
        current: &Setting,
        transition: &str,
        weights: &[f64; 8],
    ) -> (Setting, Vec<String>) {
        let mut setting = match transition {
            "jump" | "beginning" => Setting::default(),
            _ => current.clone(),
        };
        let mut mechanisms = match transition {
            "association" => vec!["association".to_owned()],
            "anticipation" => vec!["anticipation".to_owned()],
            "jump" | "beginning" => vec!["recollection".to_owned()],
            _ => vec!["transformation".to_owned()],
        };

        let extra = 1 + (self.random.uniform() * 3.0) as usize;

        // Without modulation every mechanism is equally likely; an internal
        // state weighs them.
        let uniform = weights.iter().all(|weight| *weight == weights[0]);

        for _ in 0..extra {
            let mechanism = if uniform {
                Self::pick(&mut self.random, &Self::MECHANISMS)
            } else {
                self.random
                    .weighted(weights)
                    .map(|index| &Self::MECHANISMS[index])
            };

            if let Some(mechanism) = mechanism {
                mechanisms.push((*mechanism).to_owned());
            }
        }

        for mechanism in &mechanisms {
            match mechanism.as_str() {
                "recollection" => self.recollect(&mut setting),
                "association" => self.associate(&mut setting),
                "transformation" => self.transform(&mut setting),
                "invention" => self.invent(&mut setting),
                "imagined_capability" => self.imagine_capability(&mut setting),
                "anticipation" => self.anticipate(&mut setting),
                "constraint" => self.constrain(&mut setting),
                "self_representation" => self.represent_myself(&mut setting),
                _ => {}
            }
        }

        if setting.elements.is_empty() && setting.questions.is_empty() {
            self.invent(&mut setting);
            mechanisms.push("invention".to_owned());
        }

        let excess = setting
            .elements
            .len()
            .saturating_sub(Self::SETTING_ELEMENTS);

        setting.elements.drain(..excess);
        (setting, mechanisms)
    }

    /// Lets the engine reason in the imagined setting.
    fn enact(
        &mut self,
        sandbox: &mut PersistentMemory,
        setting: &Setting,
        mechanisms: Vec<String>,
        transition: &str,
        index: usize,
        disposition: &mut dyn Disposition,
    ) -> DreamScene {
        let now = Utc::now();
        let configuration = EpisodeConfiguration::new(self.random.next_u64(), true, true)
            .with_activity(self.activity)
            .with_access(AccessMode::Replay {
                as_of: now,
                since: now,
            })
            .without_influences()
            .with_maximum_invocations(Self::SCENE_INVOCATIONS)
            .with_resource_budgets(setting.budgets.clone())
            .imagining(setting.elements.clone(), setting.capabilities.clone());
        let event = Event::internal(format!("dream-scene-{index}"), setting.questions.clone());
        let clock = Instant::now();
        let report =
            self.intelligence
                .reason_with(&event, sandbox, configuration, true, disposition);
        let mut consequences: Vec<Statement> = report
            .episode()
            .answers()
            .iter()
            .flat_map(|answer| answer.answers().iter().cloned())
            .collect();

        // What was deduced from imagined elements: imagined consequences.
        if let Some(reasoning) = report.reasoning() {
            let graph = reasoning.graph();

            for node in graph.valid_nodes() {
                if node.kind() == EpistemicKind::Deduction
                    && let Some(statement) = node.content().statement()
                    && graph.ancestors(&[node.id()]).iter().any(|ancestor| {
                        graph
                            .node(*ancestor)
                            .is_some_and(|ancestor| ancestor.kind() == EpistemicKind::Imagined)
                    })
                    && consequences.len() < 2 * Self::ASSOCIATED_FACTS
                {
                    consequences.push(statement.clone());
                }
            }
        }

        DreamScene::new(
            mechanisms,
            transition.to_owned(),
            setting.elements.clone(),
            setting.capabilities.clone(),
            setting.questions.clone(),
            report.episode().clone(),
            consequences,
            clock.elapsed().as_secs_f64(),
        )
    }

    fn add(setting: &mut Setting, content: Content, kind: EpistemicKind, origin: String) {
        let element = ImaginedElement::new(content, kind, origin);

        if !setting.elements.contains(&element) {
            setting.elements.push(element);
        }
    }

    fn statements(setting: &Setting) -> Vec<Statement> {
        setting
            .elements
            .iter()
            .filter_map(|element| element.content().statement().cloned())
            .collect()
    }

    fn recollect(&mut self, setting: &mut Setting) {
        for _ in 0..3 {
            if let Some(fact) = Self::pick(&mut self.random, &self.material.facts).cloned() {
                Self::add(
                    setting,
                    Content::Statement(fact),
                    EpistemicKind::Recollection,
                    "recollection".to_owned(),
                );
            }
        }

        if self.chance(0.5)
            && let Some(rule) = Self::pick(&mut self.random, &self.material.rules).cloned()
        {
            Self::add(
                setting,
                Content::Rule(rule),
                EpistemicKind::Recollection,
                "recollection".to_owned(),
            );
        }
    }

    /// Brings memories sharing an entity together and conjectures a link
    /// between what is said of it.
    fn associate(&mut self, setting: &mut Setting) {
        let present: Vec<Term> = Self::statements(setting)
            .iter()
            .flat_map(|statement| statement.arguments().to_vec())
            .filter(|term| !term.is_variable())
            .collect();
        let Some(entity) = Self::pick(&mut self.random, &present)
            .or_else(|| Self::pick(&mut self.random, &self.material.entities))
            .cloned()
        else {
            return;
        };

        let related: Vec<Statement> = self
            .material
            .facts
            .iter()
            .filter(|fact| fact.arguments().contains(&entity))
            .take(Self::ASSOCIATED_FACTS)
            .cloned()
            .collect();
        let origin = format!("association:{entity}");

        for fact in &related {
            Self::add(
                setting,
                Content::Statement(fact.clone()),
                EpistemicKind::Recollection,
                origin.clone(),
            );
        }

        let generalise = |statement: &Statement| {
            Statement::new(
                statement.predicate(),
                statement
                    .arguments()
                    .iter()
                    .map(|term| {
                        if *term == entity {
                            Term::Variable("x".to_owned())
                        } else {
                            term.clone()
                        }
                    })
                    .collect(),
            )
        };

        if let (Some(first), Some(second)) = (
            Self::pick(&mut self.random, &related).cloned(),
            Self::pick(&mut self.random, &related).cloned(),
        ) && first.predicate() != second.predicate()
            && let Ok(rule) = Rule::new(vec![generalise(&first)], generalise(&second))
        {
            Self::add(
                setting,
                Content::Rule(rule),
                EpistemicKind::Imagined,
                origin,
            );
        }
    }

    /// Changes a representation: substitutes an entity or a relation,
    /// negates a statement, drops a condition of a rule.
    fn transform(&mut self, setting: &mut Setting) {
        let candidates: Vec<Content> = setting
            .elements
            .iter()
            .map(|element| element.content().clone())
            .chain(
                self.material
                    .facts
                    .iter()
                    .take(1)
                    .cloned()
                    .map(Content::Statement),
            )
            .collect();
        let Some(content) = Self::pick(&mut self.random, &candidates).cloned() else {
            return;
        };
        let origin = format!("transformation:{content}");

        let transformed = match content {
            Content::Statement(statement) => {
                let operation = (self.random.uniform() * 3.0) as usize;

                match operation {
                    0 => Some(Content::Statement(statement.complement())),
                    1 => {
                        let arity = statement.arguments().len();
                        let alternatives: Vec<String> = self
                            .material
                            .predicates
                            .iter()
                            .filter(|(predicate, other)| {
                                *other == arity && predicate != statement.predicate()
                            })
                            .map(|(predicate, _)| predicate.clone())
                            .collect();

                        Self::pick(&mut self.random, &alternatives).map(|predicate| {
                            Content::Statement(Statement::new(
                                predicate.clone(),
                                statement.arguments().to_vec(),
                            ))
                        })
                    }
                    _ => {
                        let substitute = Self::pick(&mut self.random, &self.material.entities)
                            .cloned()
                            .unwrap_or_else(|| Term::symbol(self.fresh("entity")));
                        let mut arguments = statement.arguments().to_vec();

                        if arguments.is_empty() {
                            None
                        } else {
                            let position = ((self.random.uniform() * arguments.len() as f64)
                                as usize)
                                .min(arguments.len() - 1);

                            arguments[position] = substitute;
                            Some(Content::Statement(Statement::new(
                                statement.predicate(),
                                arguments,
                            )))
                        }
                    }
                }
            }
            Content::Rule(rule) if rule.premises().len() > 1 => {
                let keep = rule.premises()[1..].to_vec();

                Rule::new(keep, rule.conclusion().clone())
                    .ok()
                    .map(Content::Rule)
            }
            Content::Rule(rule) => {
                let conclusion = rule.conclusion();
                let alternatives: Vec<String> = self
                    .material
                    .predicates
                    .iter()
                    .filter(|(predicate, arity)| {
                        *arity == conclusion.arguments().len()
                            && predicate != conclusion.predicate()
                    })
                    .map(|(predicate, _)| predicate.clone())
                    .collect();

                Self::pick(&mut self.random, &alternatives).and_then(|predicate| {
                    Rule::new(
                        rule.premises().to_vec(),
                        Statement::new(predicate.clone(), conclusion.arguments().to_vec()),
                    )
                    .ok()
                    .map(Content::Rule)
                })
            }
            _ => None,
        };

        if let Some(transformed) = transformed {
            Self::add(setting, transformed, EpistemicKind::Imagined, origin);
        }
    }

    /// Invents an entity, a relation, and a rule tying the invented
    /// relation to known ones.
    fn invent(&mut self, setting: &mut Setting) {
        let entity = Term::symbol(self.fresh("entity"));
        let relation = self.fresh("relation");
        let known = Self::pick(&mut self.random, &self.material.entities).cloned();
        let statement = match known {
            Some(known) if self.chance(0.5) => {
                Statement::new(relation.clone(), vec![entity.clone(), known])
            }
            _ => Statement::new(relation.clone(), vec![entity]),
        };

        Self::add(
            setting,
            Content::Statement(statement.clone()),
            EpistemicKind::Imagined,
            "invention".to_owned(),
        );

        if let Some((predicate, arity)) =
            Self::pick(&mut self.random, &self.material.predicates).cloned()
            && arity > 0
        {
            let variables: Vec<Term> = (0..arity)
                .map(|index| Term::Variable(format!("v{index}")))
                .collect();
            let premise = Statement::new(predicate, variables.clone());
            let conclusion = Statement::new(
                relation,
                variables[..statement.arguments().len().min(arity)].to_vec(),
            );

            if let Ok(rule) = Rule::new(vec![premise], conclusion) {
                Self::add(
                    setting,
                    Content::Rule(rule),
                    EpistemicKind::Imagined,
                    "invention".to_owned(),
                );
            }
        }
    }

    /// Imagines a capability able to supply what is lacking (a need nothing
    /// could provide, in reality or in a previous dream) or anything else.
    fn imagine_capability(&mut self, setting: &mut Setting) {
        let wanted: Vec<Statement> = setting
            .questions
            .iter()
            .cloned()
            .chain(self.material.needs.iter().cloned())
            .collect();
        let target = Self::pick(&mut self.random, &wanted).cloned().or_else(|| {
            Self::pick(&mut self.random, &self.material.predicates).map(|(predicate, arity)| {
                Statement::new(
                    predicate.clone(),
                    (0..*arity)
                        .map(|index| Term::Variable(format!("a{index}")))
                        .collect(),
                )
            })
        });

        let Some(target) = target else {
            return;
        };

        let name = self.fresh("capability");
        let template = Statement::new(
            target.predicate(),
            (0..target.arguments().len())
                .map(|index| Term::Variable(format!("a{index}")))
                .collect(),
        );

        setting.capabilities.push(ImaginedCapabilitySpec::new(
            name,
            target.signature(),
            vec![template],
            "imagined_capability".to_owned(),
        ));

        if !setting.questions.contains(&target) {
            setting.questions.push(target);
        }
    }

    /// Raises a question never asked: a relation, with an entity or not.
    fn anticipate(&mut self, setting: &mut Setting) {
        let Some((predicate, arity)) =
            Self::pick(&mut self.random, &self.material.predicates).cloned()
        else {
            return;
        };

        let mut arguments: Vec<Term> = (0..arity)
            .map(|index| Term::Variable(format!("q{index}")))
            .collect();

        if arity > 0
            && self.chance(0.5)
            && let Some(entity) = Self::pick(&mut self.random, &self.material.entities).cloned()
        {
            arguments[0] = entity;
        }

        let question = Statement::new(predicate, arguments);

        if !setting.questions.contains(&question) {
            setting.questions.push(question);
        }
    }

    /// Changes the constraints: unlimited resources, or a scarce one.
    fn constrain(&mut self, setting: &mut Setting) {
        if self.chance(0.5) {
            setting.budgets.clear();
        } else {
            let resource = Self::pick(&mut self.random, &self.material.resources)
                .cloned()
                .unwrap_or_else(|| self.fresh("resource"));
            let units = (self.random.uniform() * 4.0).floor();

            setting.budgets.insert(resource, units);
        }
    }

    /// Represents itself, as it is or otherwise.
    fn represent_myself(&mut self, setting: &mut Setting) {
        if let Some(statement) = Self::pick(&mut self.random, &self.material.myself).cloned() {
            Self::add(
                setting,
                Content::Statement(statement),
                EpistemicKind::Recollection,
                "self_representation".to_owned(),
            );
        }

        if self.chance(0.5) {
            let name = self.fresh("self");
            let signature = Self::pick(&mut self.random, &self.material.predicates)
                .map(|(predicate, arity)| format!("{predicate}/{arity}"))
                .unwrap_or_else(|| "imagined/1".to_owned());

            Self::add(
                setting,
                Content::Statement(Statement::new(
                    "capability",
                    vec![Term::symbol(name), Term::text(signature)],
                )),
                EpistemicKind::Imagined,
                "self_representation:otherwise".to_owned(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        capabilities::{
            dream_interpretation::DreamInterpretation, evidence_review::EvidenceReview,
            learning_review::LearningReview, logical_inference::LogicalInference,
        },
        cognition::{module_registry::ModuleRegistry, scripted_module::ScriptedModule},
        engine::{
            dreaming::Dreaming, imagination::Imagination, intelligence::Intelligence,
            learning_parameters::LearningParameters, memory_correction::MemoryCorrection,
        },
        events::event::Event,
        influences::{influence::Influence, influence_kind::InfluenceKind},
        memory::{
            activity::Activity, answer_status::AnswerStatus, correction::Correction,
            dream_mode::DreamMode, dream_record::DreamRecord, dream_variant::DreamVariant,
            dream_verdict::DreamVerdict, episode_configuration::EpisodeConfiguration,
            experiment_request::ExperimentRequest, hypothesis_status::HypothesisStatus,
            persistent_memory::PersistentMemory,
        },
        reasoning::epistemic_kind::EpistemicKind,
        resources::initial_resources::InitialResources,
    };

    fn problem(id: &str) -> Event {
        ron::de::from_str(&format!(
            r#"(id: "{id}", source: "test", occurred_at: "2026-11-26T00:00:00Z",
                payload: assertions(statements: ["context(x)"]), questions: ["fact(?which, 1)"])"#
        ))
        .unwrap()
    }

    fn intelligence(oracle: bool, reflective: bool) -> Intelligence {
        let mut registry = ModuleRegistry::default();

        registry.register(ScriptedModule::new(
            "gatherer",
            Some("item(?x)"),
            "fact(?x, 1)",
        ));
        registry.register(ScriptedModule::new("lister", None, "item(a)"));

        if oracle {
            registry.register(ScriptedModule::new("oracle", None, "fact(a, 1)"));
        }

        if reflective {
            registry.register(EvidenceReview::new());
            registry.register(LearningReview::new());
            registry.register(DreamInterpretation::new());
        }

        Intelligence::new(
            registry,
            InitialResources::default(),
            LearningParameters::default(),
        )
    }

    fn without(modules: &[&str]) -> BTreeSet<String> {
        modules.iter().map(|module| (*module).to_owned()).collect()
    }

    fn replay(
        intelligence: &Intelligence,
        memory: &PersistentMemory,
        episode: uuid::Uuid,
        mode: DreamMode,
        variant: Option<DreamVariant>,
        seed: u64,
    ) -> DreamRecord {
        Imagination::new(intelligence, memory, seed).dream(Some(&ExperimentRequest::new(
            episode,
            mode,
            variant,
            "test".to_owned(),
        )))
    }

    #[test]
    fn replays_recorded_data_and_leaves_the_rest_missing() {
        let intelligence = intelligence(false, false);
        let mut memory = PersistentMemory::default();

        intelligence.reason(
            &problem("recorded"),
            &mut memory,
            EpisodeConfiguration::new(1, true, true),
            false,
        );

        let original = memory.experiences().episodes()[0].clone();

        assert!(
            memory
                .as_of(original.started_at())
                .experiences()
                .episodes()
                .is_empty()
        );

        let dream = replay(
            &intelligence,
            &memory,
            original.id(),
            DreamMode::Historical,
            None,
            2,
        );
        let scene = dream.replays().next().unwrap();

        assert!(
            matches!(
                scene.verdict(),
                Some(DreamVerdict::Equivalent | DreamVerdict::Cheaper)
            ),
            "{dream:#?}"
        );
        assert_eq!(scene.episode().unwrap().activity(), Activity::Dream);

        // Without anything recorded, a replay cannot invent the data.
        let mut forgetful = PersistentMemory::default();

        intelligence.reason(
            &problem("unrecorded"),
            &mut forgetful,
            EpisodeConfiguration::new(3, true, true).without_acquisition_cache(),
            false,
        );

        let original = forgetful.experiences().episodes()[0].clone();
        let dream = replay(
            &intelligence,
            &forgetful,
            original.id(),
            DreamMode::Reinterpretation,
            None,
            4,
        );
        let scene = dream.replays().next().unwrap();

        assert_eq!(scene.verdict(), Some(DreamVerdict::Worse), "{dream:#?}");
        assert!(
            scene
                .findings()
                .iter()
                .any(|finding| finding.contains("left missing"))
        );
        assert_eq!(forgetful.experiences().episodes().len(), 1);
    }

    #[test]
    fn examination_refutes_then_questions_its_own_correction() {
        let intelligence = intelligence(false, true);
        let mut memory = PersistentMemory::default();
        let episode = uuid::Uuid::now_v7();
        let rule: crate::representation::rule::Rule = "mortal(?x) :- human(?x)".parse().unwrap();
        let negation: crate::representation::statement::Statement =
            "not mortal(zeus)".parse().unwrap();

        {
            let (knowledge, _, _) = memory.parts_mut();

            knowledge.record_observation("human(zeus)".parse().unwrap(), None, episode);
            knowledge.record_observation(negation.clone(), None, episode);
            knowledge.propose_hypothesis(rule.clone(), episode, 3);
        }

        let reflect = |memory: &mut PersistentMemory, seed| {
            intelligence.reason(
                &Event::internal(
                    format!("reflection-{seed}"),
                    vec!["knowledge_concern(?k, ?s)".parse().unwrap()],
                ),
                memory,
                EpisodeConfiguration::new(seed, true, true)
                    .with_activity(Activity::Reflection)
                    .without_influences(),
                false,
            )
        };

        let report = reflect(&mut memory, 1);

        assert_eq!(
            memory.knowledge().hypothesis(&rule).unwrap().status(),
            &HypothesisStatus::Refuted,
            "{report:#?}"
        );

        {
            let (knowledge, _, _) = memory.parts_mut();

            knowledge.withdraw(&negation, "turned out wrong".to_owned());
        }

        reflect(&mut memory, 2);

        assert_eq!(
            memory.knowledge().hypothesis(&rule).unwrap().status(),
            &HypothesisStatus::Open
        );
        assert!(!memory.corrections().records()[0].is_active());
    }

    #[test]
    fn corrects_attribution_only_on_accumulated_evidence_and_reversibly() {
        let intelligence = intelligence(true, true);
        let mut memory = PersistentMemory::default();

        // The gatherer and the lister record what they acquire.
        intelligence.reason(
            &problem("without-oracle"),
            &mut memory,
            EpisodeConfiguration::new(1, true, true).without_modules(without(&["oracle"])),
            false,
        );

        let reflect = |memory: &mut PersistentMemory, seed| {
            intelligence.reason(
                &Event::internal(
                    format!("reflection-{seed}"),
                    vec!["learning_concern(?k, ?s)".parse().unwrap()],
                ),
                memory,
                EpisodeConfiguration::new(seed, true, true)
                    .with_activity(Activity::Reflection)
                    .without_influences(),
                false,
            )
        };
        let reassessed = |memory: &PersistentMemory| {
            memory.corrections().records().iter().any(|record| {
                matches!(record.correction(), Correction::ReassessInvocation { module, .. } if module == "oracle")
            })
        };

        for round in 0..6_u64 {
            // The oracle alone answers, and is credited for it...
            intelligence.reason(
                &problem(&format!("oracle-{round}")),
                &mut memory,
                EpisodeConfiguration::new(10 + round, true, true)
                    .without_modules(without(&["gatherer", "lister"])),
                false,
            );

            let original = memory.experiences().episodes().last().unwrap().clone();

            // ...but the same answers are reached without it.
            let dream = replay(
                &intelligence,
                &memory,
                original.id(),
                DreamMode::Historical,
                Some(DreamVariant::Ablation {
                    module: "oracle".to_owned(),
                }),
                100 + round,
            );

            Dreaming::integrate(&mut memory, dream);
            reflect(&mut memory, 200 + round);

            if round == 0 {
                // One replay is no proof.
                assert!(!reassessed(&memory));
            }
        }

        assert!(reassessed(&memory));

        let correction = memory
            .corrections()
            .records()
            .iter()
            .find(|record| matches!(record.correction(), Correction::ReassessInvocation { .. }))
            .unwrap()
            .id();

        MemoryCorrection::revert(&mut memory, correction, "test".to_owned()).unwrap();

        assert_eq!(
            memory.corrections().reassessments().len() + 1,
            memory
                .corrections()
                .records()
                .iter()
                .filter(|record| matches!(
                    record.correction(),
                    Correction::ReassessInvocation { .. }
                ))
                .count()
        );
    }

    #[test]
    fn gives_never_tried_alternatives_a_chance() {
        let intelligence = intelligence(true, false);
        let mut memory = PersistentMemory::default();

        // Habit forms with the gatherer...
        for seed in 1..=5 {
            intelligence.reason(
                &problem(&format!("habit-{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true).without_modules(without(&["oracle"])),
                false,
            );
        }

        // ...yet the never-tried oracle still gets tried.
        let tried = (10..50).any(|seed| {
            intelligence.reason(
                &problem(&format!("p{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true),
                false,
            );

            memory
                .experiences()
                .episodes()
                .last()
                .unwrap()
                .invocations()
                .iter()
                .any(|invocation| invocation.module() == "oracle")
        });

        assert!(tried);
    }

    #[test]
    fn measures_and_retires_influences_reversibly() {
        let intelligence = intelligence(true, false);
        let mut memory = PersistentMemory::default();

        for seed in 1..=5 {
            intelligence.reason(
                &problem(&format!("habit-{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true).without_modules(without(&["oracle"])),
                false,
            );
        }

        let favour = Influence::new(
            InfluenceKind::Favour {
                module: "oracle".to_owned(),
            },
            BTreeSet::from(["fact".to_owned()]),
            vec!["test".to_owned()],
            "test".to_owned(),
        );
        let id = favour.id();

        assert!(memory.influences_mut().propose(favour));

        for seed in 10..20 {
            intelligence.reason(
                &problem(&format!("p{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true),
                false,
            );
        }

        let evidence = memory.influences().evidence()[&id];

        assert_eq!(evidence.exposures(), 10);
        assert!(
            evidence.outcomes().0 + evidence.outcomes().1 > 0,
            "{evidence:?}"
        );
        assert_eq!(evidence.attribution(), "observed_contribution");

        // The conscious alone is not influenced.
        intelligence.reason(
            &problem("alone"),
            &mut memory,
            EpisodeConfiguration::new(30, true, true).without_influences(),
            false,
        );
        assert_eq!(memory.influences().evidence()[&id].exposures(), 10);

        let correction = MemoryCorrection::apply(
            &mut memory,
            uuid::Uuid::now_v7(),
            Correction::RetireInfluence { influence: id },
            crate::memory::correction_basis::CorrectionBasis::Inconsistency {
                detail: "test".to_owned(),
            },
            "test".to_owned(),
        )
        .unwrap();

        assert!(memory.influences().active().next().is_none());
        MemoryCorrection::revert(&mut memory, correction, "test".to_owned()).unwrap();
        assert!(memory.influences().active().next().is_some());
    }

    #[test]
    fn imagines_freely_without_touching_reality() {
        let intelligence = intelligence(false, false);
        let mut memory = PersistentMemory::default();

        for seed in 1..=3 {
            intelligence.reason(
                &problem(&format!("p{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true),
                false,
            );
        }

        let facts_before = memory.knowledge().entries().count();
        let mut mechanisms = BTreeSet::new();
        let mut imagined = 0;

        for seed in 0..20 {
            let dream = Imagination::new(&intelligence, &memory, seed).dream(None);

            assert!(!dream.scenes().is_empty());

            for scene in dream.scenes() {
                mechanisms.extend(scene.mechanisms().iter().cloned());
                imagined += scene
                    .elements()
                    .iter()
                    .filter(|element| element.kind() == EpistemicKind::Imagined)
                    .count();

                if let Some(episode) = scene.episode() {
                    assert_eq!(episode.activity(), Activity::Dream);
                    assert!(matches!(
                        episode.configuration().access(),
                        crate::acquisition::access_mode::AccessMode::Replay { .. }
                    ));
                }
            }

            Dreaming::integrate(&mut memory, dream);
        }

        assert!(imagined > 0);
        assert!(mechanisms.len() >= 5, "{mechanisms:?}");
        assert_eq!(memory.dreams().dreams().len(), 20);
        // Dreams never become knowledge.
        assert_eq!(memory.knowledge().entries().count(), facts_before);
    }

    #[test]
    fn turns_a_dreamt_conjecture_confirmed_by_reality_into_an_influence() {
        let mut registry = ModuleRegistry::default();

        registry.register(LogicalInference::new());
        registry.register(DreamInterpretation::new());

        let intelligence = Intelligence::new(
            registry,
            InitialResources::default(),
            LearningParameters::default(),
        );
        let mut memory = PersistentMemory::default();
        let episode = uuid::Uuid::now_v7();

        {
            let (knowledge, _, _) = memory.parts_mut();

            for entity in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"] {
                knowledge.record_observation(
                    format!("bird({entity})").parse().unwrap(),
                    None,
                    episode,
                );
                knowledge.record_observation(
                    format!("flies({entity})").parse().unwrap(),
                    None,
                    episode,
                );
            }
        }

        // A dream in which flying was associated with being a bird.
        let dream = loop_until_conjecture(&intelligence, &memory);

        Dreaming::integrate(&mut memory, dream);

        intelligence.reason(
            &Event::internal(
                "reflection".to_owned(),
                vec!["possibility(?k, ?s)".parse().unwrap()],
            ),
            &mut memory,
            EpisodeConfiguration::new(1, true, true)
                .with_activity(Activity::Reflection)
                .without_influences(),
            false,
        );

        assert!(
            memory
                .influences()
                .active()
                .any(|influence| matches!(influence.kind(), InfluenceKind::Conjecture { .. }))
        );

        // The conscious, facing a new bird, can use it.
        let event: Event = ron::de::from_str(
            r#"(id: "tweety", source: "test", occurred_at: "2026-11-26T00:00:00Z",
                payload: assertions(statements: ["bird(tweety)"]), questions: ["flies(tweety)"])"#,
        )
        .unwrap();
        let report = intelligence.reason(
            &event,
            &mut memory,
            EpisodeConfiguration::new(2, true, true),
            false,
        );

        assert_eq!(
            report.episode().answers()[0].status(),
            AnswerStatus::Proven,
            "{report:#?}"
        );
        assert!(
            memory
                .influences()
                .exposures()
                .iter()
                .any(|exposure| exposure.supported_answer())
        );
    }

    /// Dreams until the imagination associates flying with being a bird.
    fn loop_until_conjecture(
        intelligence: &Intelligence,
        memory: &PersistentMemory,
    ) -> DreamRecord {
        for seed in 0..500 {
            let dream = Imagination::new(intelligence, memory, seed).dream(None);
            let associated = dream.scenes().iter().any(|scene| {
                scene.elements().iter().any(|element| {
                    element.kind() == EpistemicKind::Imagined
                        && element
                            .content()
                            .rule()
                            .is_some_and(|rule| rule.to_string() == "flies(?x) :- bird(?x)")
                })
            });

            if associated {
                return dream;
            }
        }

        panic!("the imagination never associated flying with being a bird");
    }
}
