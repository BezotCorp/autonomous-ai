use std::{collections::BTreeMap, time::Instant};

use chrono::Utc;

use crate::{
    acquisition::access_mode::AccessMode,
    engine::{dreaming::Dreaming, intelligence::Intelligence},
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
    /// Chance that the dream stops after a scene.
    const INTERRUPTION: f64 = 0.3;
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
        }
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

    /// Dreams, starting from a requested experiment or from nothing.
    pub(crate) fn dream(mut self, request: Option<&ExperimentRequest>) -> DreamRecord {
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
            if !scenes.is_empty() && self.chance(Self::INTERRUPTION) {
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
                ));
                setting.questions = original.questions().to_vec();
                continue;
            }

            let (next, mechanisms) = self.compose(&setting, transition);
            let scene = self.enact(&mut sandbox, &next, mechanisms, transition, scenes.len());

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
            started_at,
            reason,
            scenes,
            hypotheses,
            clock.elapsed().as_secs_f64(),
        )
    }

    /// Composes the setting of the next scene.
    fn compose(&mut self, current: &Setting, transition: &str) -> (Setting, Vec<String>) {
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

        for _ in 0..extra {
            if let Some(mechanism) = Self::pick(&mut self.random, &Self::MECHANISMS) {
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
    ) -> DreamScene {
        let now = Utc::now();
        let configuration = EpisodeConfiguration::new(self.random.next_u64(), true, true)
            .with_activity(Activity::Dream)
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
        let report = self
            .intelligence
            .reason(&event, sandbox, configuration, true);
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
