# Autonomous AI — Chronique de conception et d'expérimentation

**Archive de travail arrêtée au 26 novembre 2026.** Projet : [BezotCorp/autonomous-ai](https://github.com/BezotCorp/autonomous-ai). Langue de l'archive : français. Ce document n'est pas une publication scientifique évaluée par les pairs.

## 1. Genèse : un problème d'ingénierie

Le 25 novembre 2026, le projet prend forme comme produit Rust indépendant de Standardizer, dans `tools/rust/autonomous-ai/product`, avec des données externes au dépôt. L'objectif est de recevoir un événement, par exemple la publication d'une bibliothèque, puis de déterminer de manière autonome s'il faut intervenir, demander une décision humaine ou ne rien faire. Le premier périmètre est la maintenance des dépendances : identifier les consommateurs, comparer les versions et leurs usages, déterminer les modifications nécessaires, puis à terme les réaliser et les vérifier. La première phase est une projection soumise à l'humain, non une autorisation d'écriture autonome.

Le choix fondateur est de ne pas encoder la solution comme un workflow imposé. Le système doit apprendre à mobiliser des capacités génériques. Rust est retenu, sans réseau neuronal ni LLM dans le moteur cognitif. L'humain définit les objectifs et les contraintes, ChatGPT participe à la conception et à l'analyse, Claude réalise des évolutions du code. Cette répartition n'est pas une attribution exhaustive de chaque contribution.

## 2. Premier prototype — 25 novembre

Le socle initial traite des événements RON, observe GitHub et Cargo, conserve une mémoire BIN et déduplique les acquisitions. Les premières traces montrent des événements et des instantanés de dépôts. Une acquisition parallèle aurait ramené une observation d'environ 82 s à 20,05 s (chiffres rapportés dans la discussion, à revalider sur les traces originales). Le compte rendu mentionne 37 dépôts découverts, 35 observés et deux échecs. Une régression liée à la normalisation de crates.io apparaît ensuite.

Le cas d'étude est la publication de `standardizer-common` 2.2.0. Deux consommateurs sont identifiés : `standardizer` (1.1.0) et `standardizer-generate-data` (2.1.0). Les projections analysent les sources et les contrats publics. Les traces du 25 novembre contiennent notamment les conclusions `persistence_format_change_possible` et `serialization_impl_unchanged_in_observed_source`. La discussion met en évidence la différence entre changement de structure, changement d'API et changement effectif de format binaire. Un changement majeur SemVer n'est pas une preuve de rupture de sérialisation. La projection est jugée trop grossière ; la comparaison doit porter sur les types, leurs implémentations et les usages réels.

Le premier moteur comporte encore des règles de projection spécifiques. Le développeur refuse de confondre un ensemble de règles lexicales ou de conclusions prédéfinies avec une intelligence généraliste. Il souhaite une véritable représentation des connaissances, des relations et des significations, des capacités indépendantes et un apprentissage de leur sélection. Les ressources initiales doivent être séparées de la mémoire apprise. La mémoire persistante devient un fichier BIN inspectable, sans obligation de conserver une compatibilité avec les prototypes abandonnés.

## 3. Reconstruction cognitive — nuit du 25 au 26 novembre

Référence de reconstruction rapportée : `56f49d2391266bebe5bb3276db8750d81fa8b691` (`feat: rebuild the cognitive architecture`), branche `feature/new-mechanism`.

Le moteur représente des termes, variables, énoncés, règles, négations, expressions et versions. Il dispose d'un graphe de raisonnement avec justifications, invalidation et contradictions, de mémoires de connaissances, de routage et d'expériences, et d'une sélection probabiliste des modules inspirée du Thompson sampling. Les capacités incluent logique, mathématiques, relations, induction, rappel, analyse linguistique et acquisitions externes. La langue naturelle demeure rudimentaire ; le lexique est vide. Les règles initiales comprennent la transitivité, les relations inverses, la comparaison des versions et une règle de dépendance obsolète. Il ne faut donc pas prétendre que le système découvre seul la notion de dépendance obsolète.

Un premier compte rendu de Claude annonce des essais de réfutation d'hypothèses, 14 tests et une comparaison de configurations avec et sans connaissances/routage. Ces résultats sont rapportés, non reproduits dans cette archive. Les limites identifiées incluent la qualité des preuves, la propagation des invalidations, les preuves non indépendantes, la sélection contextuelle et le risque de confondre mémorisation et généralisation.

## 4. Génération 1 — 26 novembre, premier échec mesuré

Expérience : `/tmp/autonomous-ai.ZfeO9Y`, 47 épisodes, graine 42, mémoire cumulative. Même événement `2b669836-50dc-4472-9452-0e9582a5c0f1`, même question `outdated_dependency(?repository, standardizer-common, ?locked, v2.2.0)`. Les acquisitions utilisent les dépôts réels ; la répétition ne constitue pas 47 problèmes indépendants.

Résultat : aucune réponse prouvée. 552 invocations, 380 improductives, 77 observations, aucune déduction mémorisée, deux hypothèses ouvertes, 27,036703 s cumulées. Les 47 épisodes se terminent en quiescence. Le moteur découvre des dépôts mais ne réactive pas correctement les besoins qui ont motivé leur acquisition. Il apprend même à moins solliciter des capacités dont les contributions intermédiaires sont mal reconnues. L'échec n'est pas un défaut de motivation psychologique : c'est un problème de continuité et d'attribution du progrès.

Les hypothèses induites concernent notamment les relations entre propriétaire et dépôt. Leur répétition ne suffit pas à établir une loi générale. Le rappel permet néanmoins d'éviter certaines acquisitions. Cette génération montre pourquoi l'efficacité apparente (faible durée) ne suffit pas : elle ne résout pas le problème.

## 5. Génération 2 — continuité et diagnostic des impasses

Commit vérifié dans le rapport : `3ec592847f8a40f67a41b153b2a66426cdcc0373`, 26 novembre 10:51:22 +0100, `feat: introduce need-driven reasoning and impasse diagnosis`. Le diff précédent `d30a5d306779f344b50109a135ad7e20143649eb` porte le même chantier avant amendement.

Évolutions : satisfaction explicite des besoins, réveil des besoins dépendants, réouverture lors d'une invalidation, reconnaissance du progrès intermédiaire (`Advanced`), diagnostic des impasses, contexte de sélection enrichi, utilité attendue et seuil d'arrêt configurables. Le raisonnement logique formule des besoins plus ciblés. La mémoire change de format.

Expérience `/tmp/autonomous-ai.UwCDHG` : même événement, 47 épisodes. 47 réponses classées `proven`, 744 invocations dont 358 improductives, 3 044 observations, quatre déductions, trois hypothèses ouvertes, 643,001824 s. Les consommateurs identifiés sont les deux projets attendus. Le rapport détaillé atteint environ 88 Mio, ce qui rend son inspection difficile. Une erreur GitHub 403 apparaît en fin de série ; l'IA peut répondre à partir de sa mémoire, mais la fraîcheur n'est alors pas garantie. Le nombre exact de requêtes responsables de l'épuisement du quota n'est pas établi. La consommation de ressources devient une question cognitive et non seulement technique.

Limites : la satisfaction d'un besoin peut reposer sur un seul énoncé ; l'attribution du progrès peut récompenser des besoins non pertinents ; les impasses sont diagnostiquées mais pas nécessairement résolues. La classification `proven` n'est pas une validation indépendante de la complétude de la réponse.

## 6. Génération 3 — planification, économie et temporalité

Le développeur refuse une interdiction arbitraire des acquisitions. Une ressource illimitée ne doit pas subir de pénalité de rareté ; une ressource limitée doit devenir plus coûteuse à mesure qu'elle se raréfie. Il demande également pourquoi l'intelligence ne regroupe pas ses acquisitions au lieu de les découvrir successivement. La planification devient une capacité cognitive sélectionnable, non un orchestrateur obligatoire.

Claude rapporte l'introduction de la planification stratégique, de l'apprentissage des prérequis et des coûts, du regroupement des acquisitions, des empreintes SHA et du cache, de la revalidation conditionnelle, de la distinction entre connaissance actuelle et historique, et de statistiques synthétiques. La rareté est mesurée à partir des ressources observées ; les limites physiques restent contraignantes. Références rapportées : `5f8944a`, `ddc3318` ; tête de l'expérience vérifiée : `cee292758c6071ed9c9b25db493f743f759d79e8`, 26 novembre 13:03:25 +0100, `chore: print episode summaries in experiments`.

Expérience `/tmp/autonomous-ai.M9Jp9b` : même événement et graine, mémoire neuve, 47 épisodes. Configuration : connaissances et routage actifs, coût 0,01, rareté 1, utilité minimale 0. Les 47 réponses sont classées prouvées et actuelles. Premier épisode : 33 invocations, 10,21 s, 36 REST et 22 GraphQL. Les épisodes suivants utilisent généralement une requête REST sans consommation de quota et une GraphQL. Total : 82 REST, 68 GraphQL, 36 et 68 unités de quota respectivement, 5 152 requêtes évitées selon le compteur, zéro redondante ou refusée. 112 entrées de cache, 1 936 721 octets. 1 692 vérifications de portée sans changement.

| Mesure | G1 | G2 | G3 |
|---|---:|---:|---:|
| Épisodes | 47 | 47 | 47 |
| Réponses prouvées | 0 | 47 | 47 |
| Invocations | 552 | 744 | 855 |
| Improductives | 380 | 358 | 381 |
| Observations mémorisées | 77 | 3044 | 3044 |
| Déductions mémorisées | 0 | 4 | 4 |
| Hypothèses ouvertes | 2 | 3 | 3 |
| Contradictions | 0 | 0 | 0 |
| Temps cumulé (s) | 27,036703 | 643,001824 | 134,44981 |

G3 réduit le temps de G2 d'environ 79,1 %, mais augmente le nombre d'invocations. Le cache technique explique une part importante du gain ; il ne faut pas l'attribuer intégralement à l'apprentissage. Le module de planification est invoqué 59 fois, dont 4 contributions, 48 progrès intermédiaires et 7 invocations improductives. La stratégie combinant mathématiques et logique est exécutée 47 fois, avec 45 réussites. Le rappel des connaissances est invoqué 50 fois, toutes improductives. Une impasse `no_capable_module package_published/3` apparaît 47 fois. La question posée est résolue selon le moteur, mais ces anomalies demeurent.

## 7. Génération 4 — inconscient et rêve expérimental

Après ces expériences, le développeur réaffirme que l'objectif n'est pas l'AGI, mais une intelligence généraliste de résolution de problèmes. Il propose une méta-IA capable d'analyser et de corriger la mémoire de l'IA principale, puis l'analogie avec l'inconscient. Il ajoute le rêve : rejeu et réanalyse sans appel externe. La conception retient un moteur commun plutôt que trois moteurs indépendants. L'inconscient peut remettre en question ses propres corrections ; les preuves, et non la hiérarchie des activités, doivent faire autorité.

Claude rapporte le commit `0138656` sur `feature/new-mechanism`, 31 tests verts et Clippy strict, mémoire format 4 incompatible avec les anciennes. Une commande `idle` choisit entre réflexion, rêve et arrêt. Le rêve rejoue sur une copie isolée, en mode historique ou réinterprétation, avec variantes d'ablation, d'ordre et de rareté. Les acquisitions sont rejouées, jamais réellement effectuées. L'inconscient examine les connaissances, les attributions, les impasses et les stratégies ; les corrections sont journalisées et annulables (`revert`). La curiosité logique est restreinte aux besoins potentiellement satisfaisables, sauf ceux liés à la question.

Essai rapporté par Claude : trois épisodes serde, `idle`, deux épisodes. Douze activités autonomes en environ 0,5 s, aucune requête externe. Une ablation de `relational_analysis` conserve les réponses en 13 invocations contre 17 ; une variante sur un besoin tardif `repository/1` passe de 21 à 16. Les épisodes réels restent actuels et prouvés, deux requêtes chacun. Aucune correction effective dans cet essai ; le mécanisme est démontré par des tests selon Claude. Une capacité alternative n'est jamais explorée en 38 épisodes. Les corrections d'attribution reposent sur un seul rêve d'ablation et des seuils codés. Les apprentissages rêvés ont un poids fixe de 0,5.

Ces résultats n'ont pas encore été audités ni reproduits indépendamment dans cette archive. L'ablation n'établit pas l'inutilité générale du module.

## 8. Génération 5 — imagination libre (conception, implémentation en cours)

Le développeur conteste le poids fixe et l'idée de sélectionner les rêves selon leur réussite. Un rêve peut influencer le raisonnement sans être immédiatement jugé utile ; l'inconscient peut exploiter ses éléments et introduire de nouvelles voies ; les conséquences réelles permettent d'apprendre la pertinence de cette influence.

La réflexion élargit ensuite le rêve au-delà du rejeu. Il peut inventer une API inexistante, une capacité fictive, des ressources illimitées ou une identité différente ; anticiper une interaction jamais rencontrée ; mélanger des souvenirs, transformer des relations et produire des situations impossibles. Un même rêve peut commencer par anticipation, passer au rejeu puis à l'invention totale. Il n'est pas un ensemble de modes exclusifs, ni un pipeline, ni nécessairement orienté vers un objectif. Les exemples ne doivent pas devenir des scénarios codés. Le hasard est un mécanisme possible, pas l'unique source de créativité.

L'inconscient remanie les expériences réelles et imaginées, sans obligation d'analyser chaque rêve. Il peut injecter dans le contexte du conscient des hypothèses, associations, doutes, besoins, stratégies ou pistes d'exploration. Le conscient conserve la capacité de décider. L'influence doit être observable et son attribution évaluée, non déduite automatiquement d'une corrélation. La provenance demeure traçable sans figer la valeur cognitive des représentations. Une fiction peut inspirer une découverte, mais n'est pas une preuve du réel. Une capacité fictive n'est pas réellement exécutable ; une ressource fictivement illimitée ne supprime pas les contraintes physiques. Aucun effet externe pendant le rêve.

Un cahier des charges en 14 sections est transmis à Claude le 26 novembre : imagination, anticipation, isolation, mémoire, suppression de la pondération arbitraire, inconscient, injection de contexte, apprentissage des influences, autonomie, observabilité et réutilisation du moteur. Aucun résultat de cette nouvelle implémentation n'est disponible au point d'arrêt de l'archive.

## 9. Hypothèse de résilience cognitive

Le développeur exprime sa conviction que les rêves humains contribuent à la résilience mentale. Cette intuition inspire une hypothèse computationnelle : un système qui apprend doit pouvoir se libérer de ses apprentissages devenus inadéquats. Le rêve et l'inconscient pourraient limiter l'enfermement dans des stratégies habituelles, la généralisation excessive des échecs et la perte d'exploration. Il ne s'agit ni d'une preuve sur le sommeil humain, ni d'une attribution de conscience à un programme.

Expérience proposée, non réalisée : entraîner le système dans un environnement stable, changer ensuite les conditions et comparer sa vitesse de réadaptation, ses erreurs, sa qualité de réponse et ses coûts avec et sans rêve et inconscient. Comparer aussi des événements inconnus, plusieurs graines, des mémoires neuves ou figées, et des acquisitions identiques avec/sans cache. La réussite doit être vérifiée indépendamment des classifications du moteur.

## 10. Positionnement scientifique et limites

Les idées de mémoire épisodique, métacognition, révision des croyances, rejeu, raisonnement contrefactuel et imagination computationnelle ont des antécédents. La discussion évoque Soar, ACT-R, NARS, MIDCA et DAYDREAMER (Mueller et Dyer, 1985). Ces rapprochements sont des pistes bibliographiques, pas une revue exhaustive ni une preuve de nouveauté. Référence à vérifier : *Towards a Computational Theory of Human Daydreaming*, https://repositories.cdlib.org/uc/item/58g626jt. L'originalité éventuelle concerne l'intégration des mécanismes et leur évaluation, pas l'invention de chacun d'eux.

À ce stade, aucune mise à jour complète de dépendance avec modification et validation autonomes n'est démontrée dans les expériences conservées. Aucun résultat ne démontre une AGI, une conscience, une créativité générale ou une généralisation robuste. La croissance du code et le nombre de tests ne suffisent pas à prouver une amélioration cognitive.

## 11. Sources et conservation

Les rapports originaux disponibles dans cette conversation sont identifiés par leur nom d'upload :

- `Texte collé(20261125-143856).txt` : compilation initiale et avertissements.
- `Texte collé(20261125-152115).txt` : code initial.
- `Texte collé(20261125-152946).txt`, `Texte collé(20261125-153150).txt` : instantanés d'observation.
- `Texte collé(20261125-162340).txt`, `Texte collé(20261125-163943).txt`, `Texte collé(20261125-165012).txt`, `Texte collé(20261125-165905).txt` : projections successives.
- `Texte collé(20261125-183822).txt`, `Texte collé(20261125-184019).txt`, `Texte collé(20261125-184229).txt`, `Texte collé(20261125-185316).txt` : mémoire, code, initialisation et état avant reconstruction.
- `Texte collé(20261126-093441).txt` : premiers raisonnements G1.
- `Texte collé(20261126-093708).txt` : G1, épisodes 3 à 47 et statistiques cumulées.
- `Texte collé(20261126-095830).txt` : diff G2 avant amendement.
- `Texte collé(20261126-104539).txt` : G2, rapport détaillé et statistiques.
- `Texte collé(20261126-121045).txt` : G3, rapport synthétique et statistiques.

Les statistiques cumulées des trois générations ont été revérifiées dans les rapports originaux lors de la rédaction. Les autres détails proviennent des discussions contemporaines, des comptes rendus de Claude et des analyses antérieures ; ils ne constituent pas tous des vérifications directes du code. L'historique Git complet n'a pas été réexaminé. Les dates de certaines transitions sont approximatives. Les heures d'upload ne doivent pas être confondues avec les heures locales des commits ou des expériences. Préserver les originaux, les événements RON, les graines, les commandes, les versions du code et les mémoires compatibles si disponibles. Ne pas publier de données privées ou de secrets contenus dans les traces sans revue préalable.

## 12. Journal à poursuivre

Pour chaque évolution future, ajouter la date, le commit, le problème déclencheur, les hypothèses, les décisions humaines et contributions des assistants, les modifications effectivement réalisées, le protocole, les résultats positifs et négatifs, les limites et les sources. Ne pas réécrire les étapes anciennes pour les faire correspondre à une théorie ultérieure. Les corrections historiques doivent être datées et justifiées.

Titre de publication envisagé : *From Dependency Automation to Computational Imagination: The Evolution of a Cognitive Architecture*.


## 13. Annexe technique — protocole reproductible

Le dépôt produit est situé dans `tools/rust/autonomous-ai/product`. Les ressources sont dans `../data/resources` (knowledge.ron, lexicon.ron, documentation-queries.ron). Les mémoires des générations sont incompatibles lorsque leur format change. Ne pas écraser les originaux. Les chemins /tmp sont des références historiques locales et ne sont pas des archives pérennes.

Le protocole G3 utilise `cargo build --release --locked`, puis 47 exécutions de `cargo run --release --locked -- run --memory "$memory" --resources ../data/resources --seed 42 --summary < "$experiment/event.ron"`. Les inspections finales utilisent `inspect --memory "$memory" --section summary` et `--section statistics`. Les générations précédentes utilisaient des traces détaillées. Les sorties sont redirigées dans un rapport texte. Le même événement est copié depuis G1. La mémoire commence vide pour chaque génération, puis évolue au fil des épisodes.

La question porte sur standardizer-common 2.2.0, registre crates.io, propriétaire BezotCorp. Les réponses attendues concernent standardizer (1.1.0) et standardizer-generate-data (2.1.0). Les règles initiales définissent outdated_dependency à partir de package_published, locked_package et less_than. La comparaison des versions et les règles de compatibilité SemVer ne démontrent pas à elles seules la compatibilité des formats persistants.

Pour G4, Claude indique `AUTONOMOUS_AI_STUDY=activities`, `AUTONOMOUS_AI_IDLE`, `AUTONOMOUS_AI_TEST_EVENTS` et `--without-cache` dans `AUTONOMOUS_AI_OPTIONS`. Ces interfaces sont rapportées, non retestées ici. Comparer action seule, action avec inconscient, action avec rêve, et architecture complète. Utiliser des événements inconnus et des graines appariées ; séparer coûts externes, coûts internes et exactitude des réponses.

## 14. Registre des décisions et questions ouvertes

- Pas de workflow métier prédéterminé : les capacités sont sélectionnées selon les besoins et les expériences.
- Pas de réseau neuronal ni de LLM dans le moteur : l'assistance au développement est distincte de l'intelligence exécutée.
- La rareté influence les décisions sans interdiction arbitraire ; les limites physiques demeurent impératives.
- Une observation historique n'est pas nécessairement actuelle ; une répétition n'est pas une preuve indépendante.
- L'imagination est libre, mais sa provenance reste identifiable. Le rêve n'a aucun effet externe réel.
- L'inconscient peut corriger ses propres corrections et influencer le conscient sans imposer aveuglément ses décisions.
- La pertinence d'une influence doit être apprise à partir de ses conséquences, sans coefficient fixe de 0,5.
- La résilience cognitive est une hypothèse : mesurer la récupération après changement de contexte, pas seulement la rapidité sur un problème répété.

Questions ouvertes : comment générer des représentations inédites sans bruit stérile ? Comment attribuer causalement un progrès à une influence ? Comment préserver la diversité des stratégies ? Comment mesurer la complétude des réponses ? Comment gérer la croissance des traces et de la mémoire ? Comment démontrer la généralisation hors des scénarios connus ?

## 15. Préservation de l'archive

Ce dossier fige les informations disponibles au 26 novembre 2026, avant le retour de Claude sur l'imagination libre. Il ne remplace ni les sources Git ni les rapports bruts. Conserver ensemble les commits, événements, ressources, paramètres, rapports et mémoires compatibles. Les chiffres repris de l'archive précédente ne sont pas revérifiés lors de cette conversion. Les comptes rendus de Claude restent des déclarations d'implémentation, non un audit indépendant. Aucun dépôt n'est modifié par la création de ce document.
