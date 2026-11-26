# Autonomous AI — Refonte de l'architecture cognitive

## État d'avancement (2026-11-26)

Branche `feature/new-mechanism`, code dans `product/src/` :

- `representation/` : termes (symboles, textes, rationnels exacts, versions,
  expressions, variables), énoncés avec négation explicite, règles, notation
  textuelle (`p(a, ?x)`, `not p(a)`, `c(?x) :- p(?x)`).
- `reasoning/` : graphe de raisonnement (provenance, validité, propagation des
  invalidations, circularité, contradictions) et mémoire de travail.
- `cognition/` : contrat `CognitiveModule`, descripteurs, registre.
- `capabilities/` : inférence logique (chaînage avant + arrière, besoins),
  mathématiques, analyse relationnelle, formation d'hypothèses (induction),
  recherche en mémoire, analyse linguistique, acquisitions GitHub / Cargo /
  releases / références de symboles / docs.rs.
- `engine/` : boucle cognitive (`Intelligence`), sélection par Thompson
  sampling, révision des croyances (changement minimal), réponses justifiées,
  consolidation, paramètres explicites.
- `memory/` : connaissances, routage, expériences dans `memory.bin`.
- `scripts/run-experiments.sh` : expériences A/B/C/D isolées sous `/tmp`.
- Ressources initiales dans `data/resources/`, mémoire dans `data/memory/`.

Usage : `autonomous-ai run --memory data/memory/memory.bin --resources
data/resources [--offline] [--detailed] < event.ron`, puis `inspect`,
`relearn`, `modules`.

### Corrections après l'expérience de 47 épisodes (aucune réponse obtenue)

- Continuité : le graphe suit quelle affirmation satisfait quel besoin ;
  satisfaire un besoin réveille les besoins pour lesquels il avait été
  formulé, et perdre l'affirmation qui le satisfaisait le rouvre.
- Attribution : nouveau résultat `advanced` (besoin satisfait, ou besoin
  formulé puis satisfait) compté comme succès même si l'épisode échoue.
- Blocages : diagnostic des besoins restés ouverts (`waiting_on`,
  `no_capable_module`, `not_attempted`, `suppliers_failed`,
  `suppliers_without_result`), agrégé dans `inspect --section statistics`.
- Stratégies : l'état de la mémoire (`remembered:<signature>`) décrit la
  situation ; une caractéristique jamais observée garde l'a priori.
- Arrêt : décision sur l'utilité espérée ; `--cost-weight` et
  `--minimum-utility` exposés et enregistrés par épisode.
- Format de `memory.bin` modifié : les mémoires antérieures sont à recréer.

Mesure (scénario réel, publication de serde, mémoire persistante) :
8/8 épisodes prouvés (11 dépôts) ; avec `--cost-weight 0.05
--minimum-utility 0.25`, dès le 2e épisode 0 s au lieu de 18 s et 6
invocations au lieu de 22, arrêt appris (`nothing_worth_trying`).

### Anticipation, ressources et validité temporelle (commit 5f8944a)

Après l'expérience de 47 épisodes (47/47 prouvées, quota GitHub épuisé) :

- [x] Ressources consommables génériques (`consumables/`) : état observé
  (limite, restant, réinitialisation), budgets imposés par expérience
  (`--resource-budget r=n`), coût de rareté `ln((R+1)/(R+1-q))` pondéré par
  `--scarcity-weight` ; nul si illimité, infini seulement si l'épuisement est
  observé (`Termination::ResourcesExhausted`). Consommation par module apprise
  (moyenne récente) et intégrée à l'utilité de chaque candidat.
- [x] Acquisition GitHub économe : quotas lus dans les en-têtes, cache
  adressé par contenu (arbres/blobs par SHA, persistant, borné LRU),
  requêtes conditionnelles ETag (304 gratuits), révisions de tous les dépôts
  en une requête GraphQL, blobs groupés ; choix REST/GraphQL par coût de
  rareté. Mesure réelle : 36 REST + 22 GraphQL au 1er épisode, puis 1 REST
  (304, 0 unité) + 1 point GraphQL, 112 acquisitions évitées, 3 s au lieu de 17.
- [x] Validité temporelle : historique des révisions par portée (source,
  périmètre), taux de changement appris (a priori 1 changement/jour), péremption
  des souvenirs propagée aux conclusions, confirmation par une observation de
  la même révision, supplantation par une autre révision ; réponses
  « actuelles » ou « historiques » (péremption, date `as_of`). Un besoin
  répondu par la mémoire reste ouvert aux modules observant le présent, avec
  pour enjeu la probabilité de péremption (vérification décidée par l'utilité).
- [x] Planification (`strategic_planning`, sélectionnée par le bandit comme
  tout module) : régression des objectifs via règles, déclarations des modules,
  prérequis appris et contenu mémorisé ; énumération et comparaison des
  stratégies (succès, durée, rareté, péremption, efficacité observée) ; besoins
  anticipés formulés d'un coup (regroupement) ; plan suivi par le moteur
  (caractéristiques `plan:ready/waiting/outside/active/failing/none`), révisé,
  appris par stratégie (`RoutingMemory::strategies`).
- [x] Observabilité : consommation par invocation et par épisode, requêtes
  évitées/redondantes/refusées, coûts prévus vs réels, plans
  adoptés/révisés/atteints, exploitation de la mémoire ; `run --summary`,
  `inspect --section summary|scopes|resources`.
- [x] Format mémoire versionné (en-tête `AAIM` v3) : les anciens fichiers sont
  refusés explicitement.

### Action, inconscient et rêve (commit 0138656)

- [x] Trois activités sur le même moteur (`Activity` : action, réflexion, rêve),
  distinguées dans les traces, statistiques et résumés.
- [x] Rêve (`engine/dreaming.rs`) : épisode ordinaire sur une copie isolée de
  la mémoire, accès en mode `Replay` (aucune requête ne sort ; données servies
  telles qu'enregistrées à la date de l'épisode ; donnée absente = manquante,
  jamais inventée ; consommation simulée). Modes historique (mémoire
  reconstituée à la date : preuves, obsolescences, réfutations et révisions
  datées) et réinterprétation. Variantes : réordonnancement, ablation d'une
  capacité, rareté simulée. Verdict comparé à l'original (consommation et
  invocations, jamais la durée). Genre épistémique `Replay`. Apprentissage
  simulé séparé (statistiques et stratégies, poids `simulation_weight`),
  hypothèses rêvées marquées `origin: dream`.
- [x] Inconscient : capacités réflexives `evidence_review`
  (`knowledge_concern/2` : contre-exemples, indépendance des corroborations,
  contradictions + vérification réelle, conclusions sans fondement, remise en
  cause de ses propres corrections) et `learning_review`
  (`learning_concern/2` : capacités improductives/sous-exploitées par bornes
  crédibles, erreurs d'attribution révélées par ablation, impasses
  récurrentes, besoins tardifs, stratégies mal calibrées, réponses simulées à
  vérifier, problèmes à réexaminer). Corrections justifiées, journalisées,
  réversibles (`revert`), réattributions appliquées au réapprentissage.
- [x] Autonomie (`idle`) : choix appris (Thompson) entre réfléchir, mener une
  expérience demandée, explorer un rêve ou s'arrêter (`--minimum-worth`).
- [x] Curiosité logique : un besoin non dirigé par un objectif n'est formulé
  que si une règle, une capacité ou la mémoire peut le fournir (impasse
  récurrente `package_published/3`).
- [x] Cache d'acquisition versionné et daté (date de première connaissance
  conservée) ; `run --without-cache` pour séparer gains techniques et
  cognitifs ; script : `AUTONOMOUS_AI_STUDY=activities` (M/U/R/F),
  `AUTONOMOUS_AI_TEST_EVENTS` (généralisation).
- [x] Format mémoire v4.

### Prochaines étapes identifiées

0. Inconscient/rêve : règles métacognitives déclaratives (ressources) au lieu
   de seuils codés dans les analyses ; validation statistique des corrections
   d'attribution sur plusieurs rêves ; rejeu conditionnel exact des données
   modifiables obtenues par GraphQL groupé ; exploration du bandit (moyenne
   des postérieurs peu exploratrice, constatée en test).

1. Négation existentielle (« aucune instance de… ») pour qu'une acquisition
   exhaustive (lockfile complet, documentation complète) prouve une absence
   sur un motif non clos.
2. Validité temporelle des conclusions dérivées rappelées directement : leur
   péremption vient de leurs prémisses, mais elles ne sont confirmées que si
   la conclusion est re-déduite ; relier la confirmation des prémisses aux
   conclusions mémorisées. Complétude des réponses non closes (nouvelles
   instances) non estimée.
3. Importeurs de ressources linguistiques (Open English WordNet CC-BY 4.0,
   Universal Dependencies) vers `data/resources/lexicon.ron`, puis analyse
   syntaxique (sujets, actions, objets) exploitant les arbres UD.
4. `source_reference_scanner` réanalyse encore le TOML brut au lieu de
   `CargoManifest`/`CargoLockfile` ; l'héritage de version de workspace n'est
   résolu que dans le manifeste lui-même.
5. Plusieurs justifications par conclusion (actuellement la première gagne) ;
   les initiales (`Premise`) ne sont jamais remises en cause.
6. Généralisation du routage : agrégation des caractéristiques plus fine que
   la moyenne des échantillons ; séquences de modules au-delà de `after:`.
7. Statistiques : réutilisation des connaissances et des stratégies,
   efficacité des corrections, capacité de généralisation.
8. Expériences : distinguer l'entraînement des mesures de D, comparer les
   conclusions et leur validité, convergence sur de nombreuses exécutions.

## 1. Objectif général

Développer une intelligence artificielle autonome, non neuronale, capable d'acquérir des connaissances, de construire ses propres raisonnements, de sélectionner ses capacités cognitives, d'apprendre de ses expériences et de corriger ses erreurs.

L'intelligence ne doit pas recevoir de règles spécifiques lui dictant la résolution d'un scénario particulier. Elle doit disposer de capacités générales et apprendre à les exploiter.

L'ensemble doit fonctionner sur CPU, notamment pour permettre des expérimentations gratuites sur GitHub Actions.

## 2. Reprise de l'existant

- [x] Identifier les composants réutilisables.
- [x] Conserver les mécanismes fonctionnels d'acquisition GitHub, Cargo et documentaire.
- [x] Conserver les mécanismes pertinents d'analyse syntaxique Rust.
- [x] Conserver la persistance binaire et les possibilités d'inspection.
- [x] Supprimer les anciens mécanismes cognitifs devenus incompatibles avec la nouvelle architecture.
- [x] Supprimer les règles métier qui prédéterminent les raisonnements.
- [x] Supprimer les associations lexicales artificielles et les pondérations arbitraires.
- [x] Revoir les points d'entrée et l'orchestration de l'intelligence.
- [ ] Corriger les incohérences existantes plutôt que les propager dans la nouvelle architecture.

La compatibilité avec les anciennes données n'est pas nécessaire. Le contenu du dossier `data` peut être entièrement remplacé.

## 3. Architecture cognitive modulaire

- [x] Concevoir un contrat commun pour les modules cognitifs.
- [x] Définir des entrées et sorties structurées et typées.
- [x] Permettre aux modules de déclarer leurs capacités, leurs contraintes et leurs besoins.
- [x] Créer un registre des modules disponibles.
- [x] Permettre l'ajout de nouveaux modules sans réécriture du moteur central.
- [x] Séparer les capacités cognitives des mécanismes de décision.
- [x] Permettre la composition dynamique des modules.
- [x] Permettre la réutilisation des résultats intermédiaires.
- [x] Ne pas imposer une séquence fixe d'exécution des modules.

Modules initiaux à envisager :

- NLP et compréhension linguistique.
- Raisonnement logique.
- Raisonnement mathématique.
- Analyse relationnelle.
- Acquisition de connaissances.
- Analyse syntaxique et structurelle.
- Vérification et validation.
- Recherche d'informations.

Les modules doivent fournir des capacités générales, pas des solutions préprogrammées.

## 4. NLP et connaissances linguistiques

- [ ] Étudier les ressources linguistiques existantes.
- [ ] Examiner notamment WordNet, Wiktionary, FrameNet et Universal Dependencies.
- [ ] Vérifier leurs licences, formats, tailles et possibilités d'intégration.
- [ ] Sélectionner les ressources réellement complémentaires.
- [ ] Prévoir leur acquisition et leur conversion dans des formats exploitables par Rust.
- [ ] Construire un lexique associant les mots à leurs définitions et significations.
- [x] Gérer les catégories grammaticales.
- [x] Gérer les différentes formes d'un mot.
- [ ] Gérer les synonymes, antonymes et relations sémantiques.
- [x] Gérer les ambiguïtés lexicales.
- [ ] Identifier les structures grammaticales.
- [ ] Identifier les sujets, actions, objets et relations.
- [ ] Prendre en compte les négations, conditions et modalités.
- [ ] Permettre l'acquisition de nouvelles connaissances linguistiques.
- [x] Conserver la provenance des connaissances.

Ne pas remplacer le NLP par une recherche de mots-clés associée à des conclusions prédéfinies.

Un lexique fournit des connaissances linguistiques ; le module NLP doit être capable de les exploiter.

## 5. Raisonnement mathématique

- [x] Concevoir un module mathématique indépendant.
- [x] Permettre la manipulation de valeurs et d'expressions mathématiques structurées.
- [x] Implémenter ou intégrer les opérations mathématiques nécessaires.
- [x] Permettre la comparaison et la transformation d'expressions.
- [ ] Permettre la manipulation de probabilités.
- [x] Permettre le calcul de statistiques à partir des expériences.
- [ ] Étudier les mécanismes d'inférence bayésienne.
- [ ] Étudier les méthodes d'optimisation et de décision.
- [ ] Étudier les ressources mathématiques existantes, notamment OpenMath.
- [x] Permettre au moteur cognitif de mobiliser les mathématiques lorsqu'elles sont pertinentes.

Ne pas réduire les mathématiques à un calcul de scores effectué après le raisonnement.

## 6. Raisonnement logique et relationnel

- [x] Concevoir une représentation structurée des faits, concepts et relations.
- [x] Permettre la manipulation de variables.
- [x] Permettre la formulation d'hypothèses.
- [x] Permettre la construction de déductions.
- [x] Permettre la manipulation des implications et équivalences.
- [x] Permettre la détection des contradictions.
- [x] Permettre la vérification des prémisses.
- [x] Permettre la construction de nouvelles relations.
- [x] Distinguer les observations, hypothèses, déductions et conclusions.
- [x] Distinguer l'absence de preuve d'une preuve d'absence.
- [x] Conserver la provenance des informations.
- [x] Permettre la révision des conclusions.
- [ ] Étudier les ressources conceptuelles existantes, notamment ConceptNet et Wikidata.

Les règles fondamentales de logique peuvent être fournies. Les conclusions propres aux situations rencontrées doivent être construites par l'intelligence.

## 7. Mémoire de travail — WorkingMemory

- [x] Créer une mémoire temporaire indépendante.
- [x] Stocker le contexte courant.
- [x] Stocker les objectifs en cours.
- [x] Stocker les informations nécessaires au raisonnement.
- [x] Stocker les hypothèses actives.
- [x] Stocker les résultats intermédiaires.
- [x] Stocker les contradictions détectées.
- [x] Stocker les besoins d'informations complémentaires.
- [x] Permettre l'invalidation et le remplacement des résultats.
- [x] Permettre la réutilisation des résultats encore valides.
- [x] Libérer les informations temporaires lorsque le raisonnement est terminé.

Cette mémoire n'a pas vocation à être persistée systématiquement.

## 8. Mémoire de connaissances — KnowledgeMemory

- [x] Séparer les connaissances initiales des connaissances acquises.
- [x] Intégrer les observations existantes.
- [x] Persister les faits acquis.
- [x] Persister les concepts et relations découverts.
- [x] Persister les conclusions suffisamment étayées.
- [x] Conserver les sources et éléments de preuve.
- [x] Conserver les contextes de validité.
- [x] Distinguer les connaissances confirmées des connaissances incertaines.
- [x] Permettre la correction des connaissances.
- [x] Permettre l'invalidation des connaissances obsolètes.
- [x] Éviter la duplication inutile.
- [x] Permettre la recherche et la réutilisation des connaissances.
- [x] Éviter qu'une hypothèse devienne automatiquement un fait.

La mémoire doit évoluer à partir des acquisitions et des raisonnements réellement effectués.

## 9. Mémoire de routage — RoutingMemory

- [x] Créer une mémoire dédiée aux stratégies cognitives.
- [x] Mémoriser les contextes dans lesquels les modules sont utilisés.
- [ ] Mémoriser les stratégies de sélection des modules.
- [x] Mémoriser les combinaisons de modules expérimentées.
- [x] Mémoriser les résultats observés.
- [x] Mémoriser les coûts d'exécution.
- [ ] Mémoriser les stratégies de correction efficaces.
- [ ] Permettre la généralisation à des situations similaires.
- [x] Permettre la réévaluation des stratégies apprises.
- [ ] Éviter de reproduire systématiquement les mêmes erreurs.
- [x] Ne pas confondre la pertinence d'un module avec la validité de toutes ses sorties.

La mémoire de routage représente le savoir-faire de l'intelligence, indépendamment de ses connaissances déclaratives.

## 10. Mémoire des expériences — ExperienceMemory

- [x] Créer un registre persistant des expériences.
- [x] Enregistrer les essais effectués.
- [x] Enregistrer les modules sélectionnés.
- [x] Enregistrer les raisons de leur sélection.
- [ ] Enregistrer les entrées et sorties pertinentes.
- [x] Enregistrer les hypothèses formulées.
- [x] Enregistrer les résultats obtenus.
- [x] Enregistrer les erreurs techniques.
- [x] Enregistrer les contradictions logiques.
- [x] Enregistrer les hypothèses réfutées.
- [ ] Enregistrer les corrections tentées.
- [ ] Enregistrer les résultats des corrections.
- [ ] Enregistrer les éléments de validation.
- [x] Enregistrer les durées et coûts mesurables.
- [x] Conserver les expériences infructueuses.
- [x] Permettre de recalculer les statistiques lorsque les algorithmes d'apprentissage évoluent.

Le registre des expériences constitue la matière première de l'apprentissage. Il ne doit pas être réduit à des compteurs de succès et d'échecs.

## 11. Graphe de raisonnement — ReasoningGraph

- [x] Représenter le raisonnement sous forme de graphe.
- [x] Identifier chaque résultat intermédiaire.
- [x] Identifier le module ayant produit chaque résultat.
- [x] Identifier les entrées et prémisses utilisées.
- [x] Identifier les dépendances entre les résultats.
- [x] Identifier les hypothèses dont dépend une conclusion.
- [x] Associer un état de validité aux résultats.
- [x] Permettre la coexistence d'hypothèses concurrentes.
- [x] Permettre l'exploration de plusieurs branches.
- [x] Détecter les dépendances circulaires problématiques.
- [x] Permettre la propagation des invalidations.
- [x] Permettre le recalcul partiel du raisonnement.
- [x] Conserver la traçabilité des corrections.

Ne pas imposer un raisonnement strictement linéaire.

## 12. Gestion des erreurs et révision des croyances

- [x] Détecter les échecs et contradictions.
- [x] Identifier les résultats susceptibles d'être responsables.
- [x] Remonter les dépendances du graphe.
- [x] Examiner plusieurs causes possibles.
- [x] Ne pas considérer automatiquement le dernier module comme responsable.
- [x] Permettre la remise en question d'une hypothèse antérieure.
- [ ] Permettre la réexécution d'un module avec des entrées corrigées.
- [x] Permettre la sélection d'un autre module.
- [x] Permettre l'acquisition d'informations complémentaires.
- [x] Permettre le remplacement d'un résultat invalide dans la mémoire de travail.
- [x] Invalider les résultats qui en dépendent.
- [x] Recalculer uniquement les branches nécessaires.
- [x] Conserver l'expérience de l'erreur.
- [x] Réévaluer la stratégie de routage à partir des éléments disponibles.
- [ ] Permettre l'abandon d'une hypothèse impossible à confirmer.
- [x] Permettre à l'intelligence de reconnaître son incertitude.

Une erreur détectée tardivement doit pouvoir remettre en question une étape beaucoup plus ancienne.

## 13. Sélection adaptative des modules

- [x] Concevoir un mécanisme de sélection dynamique.
- [x] Identifier les capacités pertinentes pour un objectif.
- [x] Exploiter les connaissances de routage déjà acquises.
- [x] Permettre l'exploration de stratégies inconnues.
- [x] Équilibrer exploration et exploitation.
- [x] Étudier les bandits contextuels et les mécanismes bayésiens.
- [x] Choisir une première implémentation adaptée au CPU.
- [x] Évaluer les stratégies à partir de résultats vérifiables.
- [x] Prendre en compte les coûts mesurés.
- [ ] Prendre en compte les corrections nécessaires.
- [ ] Permettre l'apprentissage progressif de séquences de modules.
- [x] Éviter les boucles d'exécution infinies.
- [x] Prévoir des budgets de calcul et d'exploration.
- [x] Permettre l'arrêt lorsque les informations sont insuffisantes.

Les modules ne doivent pas décider seuls du moment où ils sont mobilisés. Le moteur doit apprendre à les sélectionner.

## 14. Statistiques et apprentissage

- [x] Calculer les statistiques à partir des expériences réelles.
- [x] Mesurer la fréquence d'utilisation des modules.
- [x] Mesurer les résultats confirmés, réfutés et indéterminés.
- [x] Mesurer la pertinence des modules selon le contexte.
- [ ] Mesurer l'efficacité des combinaisons de modules.
- [x] Mesurer la fréquence de réfutation des hypothèses.
- [ ] Mesurer l'efficacité des corrections.
- [x] Mesurer les coûts moyens.
- [ ] Mesurer la réutilisation des connaissances.
- [ ] Mesurer la réutilisation des stratégies.
- [ ] Mesurer la capacité de généralisation.
- [x] Permettre la révision des estimations probabilistes.
- [x] Distinguer fréquence, confiance et validité.
- [x] Éviter de renforcer des conclusions uniquement parce qu'elles sont répétées.
- [x] Éviter l'auto-confirmation des erreurs.

Aucun coefficient de fiabilité propre à un module ou à une connaissance ne doit être inventé sans justification. Les paramètres initiaux des algorithmes doivent être explicites et distincts des statistiques apprises.

## 15. Acquisition autonome des connaissances

- [x] Conserver les capacités d'acquisition existantes.
- [x] Exposer ces capacités au registre des modules.
- [x] Permettre à l'intelligence d'identifier ses besoins d'informations.
- [x] Permettre la sélection dynamique des sources.
- [x] Permettre l'acquisition de documentation.
- [x] Permettre l'acquisition de code source.
- [x] Permettre l'acquisition de métadonnées.
- [x] Conserver les versions et révisions exactes des sources.
- [x] Conserver la provenance des éléments acquis.
- [x] Distinguer les données brutes des connaissances interprétées.
- [x] Permettre la réutilisation des acquisitions persistées.
- [x] Permettre la réactualisation des connaissances.
- [x] Gérer les échecs d'acquisition sans invalider automatiquement l'ensemble du raisonnement.

L'acquisition GitHub constitue une capacité parmi d'autres. Elle ne doit pas définir l'architecture générale de l'intelligence.

## 16. Persistance et organisation des données

- [x] Repenser entièrement le contenu du dossier `data`.
- [x] Séparer les ressources initiales des connaissances acquises.
- [x] Conserver les ressources externes dans leurs formats appropriés lorsque nécessaire.
- [ ] Générer des représentations binaires optimisées lorsque cela présente un intérêt.
- [x] Persister les mémoires acquises dans un fichier `memory.bin`.
- [x] Séparer logiquement les différentes mémoires dans les structures Rust.
- [x] Conserver la sauvegarde atomique et le verrouillage de la mémoire.
- [x] Permettre l'initialisation sans mémoire acquise.
- [x] Permettre le chargement d'une mémoire existante.
- [x] Persister les acquisitions cognitives après les raisonnements.
- [x] Permettre l'inspection lisible de la mémoire binaire.
- [x] Ne pas imposer de fichier RON source pour la mémoire acquise.
- [x] Ne pas versionner les fichiers binaires générés.

Le RON peut rester un format d'inspection ou d'échange lorsqu'il est pertinent. Il ne doit pas devenir une contrainte artificielle.

## 17. Autonomie du moteur central

- [x] Revoir le rôle de `Intelligence`.
- [x] Séparer la réception d'un événement de son interprétation.
- [x] Permettre la formulation d'objectifs et de sous-objectifs.
- [x] Permettre l'identification des connaissances manquantes.
- [x] Permettre la sélection des modules nécessaires.
- [x] Permettre l'analyse des résultats.
- [x] Permettre la formulation et la vérification d'hypothèses.
- [x] Permettre la révision du raisonnement.
- [x] Permettre la production de conclusions justifiées.
- [ ] Permettre la décision de ne pas intervenir.
- [x] Permettre la demande d'informations complémentaires.
- [x] Permettre la reconnaissance d'une impossibilité à conclure.
- [x] Intégrer les expériences et connaissances acquises aux mémoires persistantes.

Le moteur doit être indépendant de GitHub Actions et de son environnement d'exécution.

## 18. Observabilité

- [x] Rendre les raisonnements inspectables.
- [x] Afficher les modules sélectionnés.
- [x] Afficher les raisons de leur sélection.
- [x] Afficher les connaissances mobilisées.
- [x] Afficher les nouvelles acquisitions.
- [x] Afficher les hypothèses formulées.
- [x] Afficher les dépendances entre les résultats.
- [x] Afficher les contradictions détectées.
- [x] Afficher les corrections effectuées.
- [x] Afficher les hypothèses abandonnées.
- [x] Afficher les conclusions et leurs justifications.
- [x] Afficher les incertitudes restantes.
- [x] Afficher les statistiques d'apprentissage.
- [x] Afficher les évolutions des mémoires.

L'objectif est de comprendre ce que fait réellement l'intelligence, et non de produire un rapport donnant artificiellement l'impression qu'elle raisonne.

## 19. Expérimentations comparatives

Préparer des expérimentations reproductibles utilisant les mêmes objectifs et, autant que possible, les mêmes données sources.

- [x] Expérience A : aucune connaissance acquise, aucun routage appris.
- [x] Expérience B : connaissances acquises, aucun routage appris.
- [x] Expérience C : routage appris, aucune connaissance acquise.
- [x] Expérience D : connaissances acquises et routage appris.
- [x] Répéter les expérimentations sur plusieurs exécutions.
- [ ] Comparer les conclusions.
- [ ] Comparer la validité des conclusions.
- [ ] Comparer les temps de raisonnement.
- [ ] Comparer les coûts d'acquisition.
- [ ] Comparer les stratégies employées.
- [ ] Comparer les erreurs et corrections.
- [ ] Observer la convergence ou la divergence.
- [ ] Détecter l'accumulation éventuelle d'erreurs.
- [ ] Évaluer la généralisation sur de nouvelles situations.

Utiliser des environnements temporaires sous `/tmp` pour les expérimentations isolées et une mémoire persistante pour les expérimentations cumulatives.

Le scénario de publication d'une dépendance Rust peut être réutilisé comme première expérience réelle, mais il ne doit pas influencer artificiellement la conception du moteur cognitif.

## 20. Contraintes d'architecture et de développement

- [x] Rust pour l'ensemble du produit.
- [x] Fonctionnement sur CPU.
- [x] Aucun réseau neuronal nécessaire.
- [x] Aucun service d'inférence IA externe nécessaire.
- [x] Une structure par fichier.
- [x] Champs privés et responsabilités clairement définies.
- [x] Aucun code métier dans les fichiers `mod.rs`.
- [x] Imports directs depuis les modules concernés.
- [x] Conserver un produit binaire sans créer artificiellement une bibliothèque.
- [ ] Éviter les fichiers fourre-tout.
- [ ] Éviter les abstractions inutiles.
- [x] Éviter les données métier codées en dur.
- [x] Éviter les scénarios de démonstration intégrés au moteur.
- [x] Ne pas masquer les erreurs.
- [x] Ne pas modifier la CI/CD existante.
- [x] Ne pas modifier les versions manuellement.
- [x] Travailler exclusivement sur `feature/new-mechanism`.
- [x] Utiliser uniquement des commits `chore:`, `feat:` ou `fix:`.
- [x] Utiliser `cargo check --locked` pendant le développement.
- [x] Effectuer la validation finale avec `cargo clippy --locked -- -D warnings`.
- [x] Laisser le hook de pré-commit gérer Standardizer.

## 21. Ordre d'implémentation recommandé

1. Audit et nettoyage de l'architecture existante.
2. Représentation structurée des informations et des résultats cognitifs.
3. Contrats et registre des modules.
4. Mémoires et persistance.
5. Graphe de raisonnement et invalidation.
6. Registre des expériences.
7. Routage adaptatif et statistiques.
8. Intégration des capacités d'acquisition existantes.
9. Modules logiques, mathématiques et linguistiques.
10. Intégration des ressources externes sélectionnées.
11. Boucle cognitive autonome.
12. Observabilité complète.
13. Expérimentations comparatives.
14. Corrections et améliorations fondées sur les résultats observés.

## Principe directeur

Donner à l'intelligence les moyens de raisonner, et non lui fournir les raisonnements attendus.

Accepter qu'elle se trompe, à condition de pouvoir observer ses erreurs, comprendre leurs causes, réviser ses conclusions et apprendre de ses expériences.

L'architecture est expérimentale. Les choix algorithmiques peuvent évoluer après plusieurs exécutions. Il n'est pas nécessaire d'atteindre une implémentation parfaite avant de commencer à expérimenter.
