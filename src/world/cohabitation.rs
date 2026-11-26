use std::{
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    capabilities::module_catalog::ModuleCatalog,
    engine::{intelligence::Intelligence, learning_parameters::LearningParameters},
    living::{
        clock_mode::ClockMode, existence::Existence, existence_store::ExistenceStore, life::Life,
        life_event::LifeEvent, life_settings::LifeSettings,
    },
    resources::initial_resources::InitialResources,
    world::{environment::Environment, environment_store::EnvironmentStore, portal::Portal},
};

/// The common time of the existences present in an environment. It only
/// schedules: each existence lives its own life (its own activities,
/// sleep, choices); none is made to act, wait for or answer another.
///
/// - **Virtual time**: the existences advance in turn by windows of the
///   environment's latency. A message emitted during a window can be
///   received at the earliest at its emission time plus the latency, that
///   is never before the next window: nothing is received before it is
///   sent, and the order in which existences take their turn within a
///   window changes nothing. The whole is reproducible.
/// - **Real time**: each existence lives in its own thread, at its own
///   pace; the environment is shared under a lock.
///
/// Existences and the environment are saved together at checkpoints and
/// at the end, each in its own file.
pub(crate) struct Cohabitation {
    data: PathBuf,
    resources: PathBuf,
    parameters: LearningParameters,
    settings: LifeSettings,
}

impl Cohabitation {
    pub(crate) fn new(
        data: PathBuf,
        resources: PathBuf,
        parameters: LearningParameters,
        settings: LifeSettings,
    ) -> Self {
        Self {
            data,
            resources,
            parameters,
            settings,
        }
    }

    fn directory(&self, individual: &str) -> PathBuf {
        self.data.join("individuals").join(individual)
    }

    fn intelligence(&self, external: bool, portal: Portal) -> Result<Intelligence, Box<dyn Error>> {
        Ok(Intelligence::new(
            ModuleCatalog::in_environment(ModuleCatalog::standard(!external), portal),
            InitialResources::load(&self.resources)?,
            self.parameters.clone(),
        ))
    }

    pub(crate) fn live(&self, store: &EnvironmentStore) -> Result<(), Box<dyn Error>> {
        let environment = store.load()?;
        let name = environment.name().to_owned();
        let external = environment.external();
        let latency = environment.latency();
        let members: Vec<(String, String)> = environment
            .members()
            .iter()
            .filter(|member| member.is_present())
            .map(|member| (member.handle().to_owned(), member.individual().to_owned()))
            .collect();
        let shared = Arc::new(Mutex::new(environment));
        let mut stores = Vec::new();
        let mut existences = Vec::new();

        for (_, individual) in &members {
            let existence_store = ExistenceStore::open(&self.directory(individual))?;

            if !existence_store.exists() {
                return Err(format!("{individual} has no existence to live").into());
            }

            existences.push(existence_store.load()?);
            stores.push(existence_store);
        }

        match self.settings.clock() {
            ClockMode::Virtual => self.live_virtually(
                &name,
                external,
                latency,
                &members,
                &shared,
                &stores,
                &mut existences,
                store,
            )?,
            ClockMode::Real => {
                self.live_really(&name, external, &members, &shared, &stores, &mut existences)?
            }
        }

        store.save(&Self::lock(&shared))
    }

    fn lock(shared: &Arc<Mutex<Environment>>) -> std::sync::MutexGuard<'_, Environment> {
        shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[allow(clippy::too_many_arguments)]
    fn live_virtually(
        &self,
        name: &str,
        external: bool,
        latency: f64,
        members: &[(String, String)],
        shared: &Arc<Mutex<Environment>>,
        stores: &[ExistenceStore],
        existences: &mut [Existence],
        environment: &EnvironmentStore,
    ) -> Result<(), Box<dyn Error>> {
        let mut portals = Vec::new();
        let mut intelligences = Vec::new();

        for ((handle, _), existence) in members.iter().zip(existences.iter()) {
            let portal = Portal::new(shared.clone(), handle.clone(), existence.clock().lived());

            intelligences.push(self.intelligence(external, portal.clone())?);
            portals.push(portal);
        }

        // Saves are made here, for all at once.
        let settings = self.settings.clone().with_checkpoint(f64::INFINITY);
        let lives: Vec<Life> = members
            .iter()
            .zip(&intelligences)
            .zip(&portals)
            .map(|(((_, individual), intelligence), portal)| {
                Life::new(
                    intelligence,
                    settings.clone().in_directory(self.directory(individual)),
                )
                .with_portal(portal.clone())
            })
            .collect();
        let mut sessions = Vec::new();

        for ((life, existence), (handle, _)) in lives.iter().zip(existences.iter_mut()).zip(members)
        {
            sessions.push(life.begin_session(existence));
            existence.record(LifeEvent::Present {
                environment: name.to_owned(),
                handle: handle.clone(),
            });
        }

        let start = Self::lock(shared).clock();
        let end = start + self.settings.duration().max(0.0);
        let mut time = start;
        let mut last_checkpoint = Instant::now();

        while time < end {
            let next = (time + latency).min(end);
            let mut going_on = false;

            for (((life, existence), (store, session)), portal) in lives
                .iter()
                .zip(existences.iter_mut())
                .zip(stores.iter().zip(sessions.iter_mut()))
                .zip(&portals)
            {
                going_on |= life.run(existence, store, session, portal.lived_at(next))?;
            }

            Self::lock(shared).advance_to(next);
            time = next;

            if last_checkpoint.elapsed().as_secs_f64() >= self.settings.checkpoint() {
                Self::save(stores, existences, shared, Some(environment))?;
                last_checkpoint = Instant::now();
            }

            if !going_on {
                break;
            }
        }

        for ((life, existence), store) in lives.iter().zip(existences.iter_mut()).zip(stores) {
            life.end_session(existence, store)?;
        }

        Ok(())
    }

    fn live_really(
        &self,
        name: &str,
        external: bool,
        members: &[(String, String)],
        shared: &Arc<Mutex<Environment>>,
        stores: &[ExistenceStore],
        existences: &mut [Existence],
    ) -> Result<(), Box<dyn Error>> {
        std::thread::scope(|scope| {
            let handles: Vec<_> = members
                .iter()
                .zip(existences.iter_mut())
                .zip(stores)
                .map(|(((handle, individual), existence), store)| {
                    let shared = shared.clone();
                    let settings = self
                        .settings
                        .clone()
                        .in_directory(self.directory(individual));

                    scope.spawn(move || -> Result<f64, String> {
                        let portal = Portal::new(shared, handle.clone(), existence.clock().lived());
                        let intelligence = self
                            .intelligence(external, portal.clone())
                            .map_err(|error| error.to_string())?;

                        existence.record(LifeEvent::Present {
                            environment: name.to_owned(),
                            handle: handle.clone(),
                        });
                        Life::new(&intelligence, settings)
                            .with_portal(portal.clone())
                            .live(existence, store)
                            .map_err(|error| error.to_string())?;

                        Ok(portal.now())
                    })
                })
                .collect();
            let mut reached = Self::lock(shared).clock();

            for handle in handles {
                let time = handle
                    .join()
                    .map_err(|_| "an existence's thread panicked".to_owned())??;

                reached = reached.max(time);
            }

            Self::lock(shared).advance_to(reached);
            Ok::<(), String>(())
        })?;

        Ok(())
    }

    fn save(
        stores: &[ExistenceStore],
        existences: &[Existence],
        shared: &Arc<Mutex<Environment>>,
        environment: Option<&EnvironmentStore>,
    ) -> Result<(), Box<dyn Error>> {
        // Existences first: if the process stops before the environment is
        // saved, the messages of the last period may be lost, never
        // duplicated (an existence ignores what it already received).
        for (store, existence) in stores.iter().zip(existences) {
            store.save(existence)?;
        }

        if let Some(environment) = environment {
            environment.save(&Self::lock(shared))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        sync::{Arc, Mutex},
    };

    use super::Cohabitation;
    use crate::{
        acquisition::access_mode::AccessMode,
        capabilities::module_catalog::ModuleCatalog,
        engine::{intelligence::Intelligence, learning_parameters::LearningParameters},
        events::event::Event,
        individuality::{birth_parameters::BirthParameters, channel::Channel},
        living::{
            clock_mode::ClockMode, existence::Existence, existence_store::ExistenceStore,
            life::Life, life_event::LifeEvent, life_settings::LifeSettings,
        },
        memory::{episode_configuration::EpisodeConfiguration, standing::Standing},
        resources::initial_resources::InitialResources,
        world::{
            environment::Environment, environment_store::EnvironmentStore, portal::Portal,
            transmission_status::TransmissionStatus,
        },
    };

    fn parameters() -> LearningParameters {
        LearningParameters::default().with_cost_weight_per_second(0.0)
    }

    fn born(data: &Path, name: &str, seed: u64) {
        let store = ExistenceStore::open(&data.join("individuals").join(name)).unwrap();

        store
            .save(&Existence::born(
                name.to_owned(),
                BirthParameters::new(seed, 8).unwrap(),
            ))
            .unwrap();
    }

    fn place(data: &Path, members: &[&str]) -> (EnvironmentStore, Vec<String>) {
        std::fs::create_dir_all(data.join("resources")).unwrap();

        let store = EnvironmentStore::open(&data.join("environments").join("place")).unwrap();
        let mut environment = Environment::new("place".to_owned(), 3, 60.0, 1.0, false).unwrap();
        let handles = members
            .iter()
            .map(|member| environment.admit((*member).to_owned()).unwrap())
            .collect();

        store.save(&environment).unwrap();
        (store, handles)
    }

    fn intelligence(portal: Portal) -> Intelligence {
        Intelligence::new(
            ModuleCatalog::in_environment(ModuleCatalog::standard(true), portal),
            InitialResources::default(),
            parameters(),
        )
    }

    /// Controlled validation (not a spontaneous interaction): a question
    /// is put to the sender to make it transmit.
    #[test]
    fn a_transmission_is_addressed_delayed_persisted_and_received_as_testimony() {
        let data = tempfile::tempdir().unwrap();

        born(data.path(), "a", 1);
        born(data.path(), "b", 2);

        let (environment_store, handles) = place(data.path(), &["a", "b"]);
        let shared = Arc::new(Mutex::new(environment_store.load().unwrap()));
        let a_store = ExistenceStore::open(&data.path().join("individuals/a")).unwrap();
        let mut a = a_store.load().unwrap();
        let a_portal = Portal::new(shared.clone(), handles[0].clone(), a.clock().lived());
        let event: Event = ron::de::from_str(&format!(
            r#"(id: "lab", source: "test", occurred_at: "2026-11-26T00:00:00Z",
                payload: assertions(statements: ["colour(sky, blue)"]),
                questions: ["transmitted({}, ?message)"])"#,
            handles[1]
        ))
        .unwrap();

        a_portal.follow(a.clock().lived());

        // In imagination nothing is transmitted.
        let now = chrono::Utc::now();

        intelligence(a_portal.clone()).reason(
            &event,
            a.memory_mut(),
            EpisodeConfiguration::new(1, true, true).with_access(AccessMode::Replay {
                as_of: now,
                since: now,
            }),
            false,
        );
        assert!(shared.lock().unwrap().transmissions().is_empty());

        // In reality it is.
        intelligence(a_portal.clone()).reason(
            &event,
            a.memory_mut(),
            EpisodeConfiguration::new(2, true, true),
            false,
        );

        let transmission = shared.lock().unwrap().transmissions()[0].clone();

        assert_eq!(transmission.from(), handles[0]);
        assert_eq!(transmission.to(), handles[1]);
        assert_eq!(transmission.status(), &TransmissionStatus::Pending);
        assert_eq!(transmission.deliver_at(), transmission.emitted_at() + 60.0);
        assert!(
            transmission
                .message()
                .statements()
                .iter()
                .any(|statement| statement.to_string() == "colour(sky, blue)")
        );

        // It survives a stop.
        environment_store.save(&shared.lock().unwrap()).unwrap();
        drop(environment_store);

        let environment_store =
            EnvironmentStore::open(&data.path().join("environments/place")).unwrap();
        let shared = Arc::new(Mutex::new(environment_store.load().unwrap()));

        assert_eq!(shared.lock().unwrap().transmissions().len(), 1);

        // The recipient receives it only once the latency has elapsed, as a
        // stimulus; acting on it, it records what was said as reported.
        let b_store = ExistenceStore::open(&data.path().join("individuals/b")).unwrap();
        let mut b = b_store.load().unwrap();

        b.constitution_mut().set_bias(Channel::Attend, 20.0);
        b.constitution_mut().set_bias(Channel::FallAsleep, -40.0);

        let b_portal = Portal::new(shared.clone(), handles[1].clone(), b.clock().lived());
        let b_intelligence = intelligence(b_portal.clone());
        let life = Life::new(
            &b_intelligence,
            LifeSettings::new(
                ClockMode::Virtual,
                3_600.0,
                data.path().join("individuals/b"),
            ),
        )
        .with_portal(b_portal.clone());
        let mut session = life.begin_session(&mut b);
        let received = |b: &Existence| {
            b.timeline().entries().iter().any(|entry| {
                matches!(entry.event(), LifeEvent::StimulusReceived { event, .. }
                    if *event == transmission.id().to_string())
            })
        };

        life.run(
            &mut b,
            &b_store,
            &mut session,
            b_portal.lived_at(transmission.deliver_at() - 1.0),
        )
        .unwrap();
        assert!(!received(&b));

        life.run(&mut b, &b_store, &mut session, b_portal.lived_at(1_800.0))
            .unwrap();
        assert!(received(&b));

        let entry = b
            .memory()
            .knowledge()
            .entries()
            .find(|entry| entry.statement().to_string() == "colour(sky, blue)")
            .expect("the testimony was integrated");

        assert_eq!(entry.standing(), &Standing::Reported);
    }

    #[test]
    fn a_shared_existence_in_virtual_time_is_reproducible() {
        let run = || {
            let data = tempfile::tempdir().unwrap();

            born(data.path(), "a", 5);
            born(data.path(), "b", 6);

            let (store, _) = place(data.path(), &["a", "b"]);

            Cohabitation::new(
                data.path().to_path_buf(),
                data.path().join("resources"),
                parameters(),
                LifeSettings::new(ClockMode::Virtual, 6.0 * 3_600.0, data.path().to_path_buf())
                    .with_checkpoint(f64::INFINITY),
            )
            .live(&store)
            .unwrap();

            let environment = store.load().unwrap();

            assert_eq!(environment.clock(), 6.0 * 3_600.0);

            ["a", "b"]
                .map(|name| {
                    let existence = ExistenceStore::read(&ExistenceStore::path(
                        &data.path().join("individuals").join(name),
                    ))
                    .unwrap();

                    existence
                        .timeline()
                        .entries()
                        .iter()
                        .map(|entry| format!("{:.0} {}", entry.lived(), entry.event().kind()))
                        .collect::<Vec<_>>()
                })
                .to_vec()
                .into_iter()
                .chain([vec![environment.transmissions().len().to_string()]])
                .collect::<Vec<_>>()
        };

        assert_eq!(run(), run());
    }
}
