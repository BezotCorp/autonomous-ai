# Conscient, inconscient et rêve

Ce document décrit comment l'intelligence se partage entre trois activités
cognitives. Il présente les décisions d'architecture prises, les garanties
réellement apportées, les limites connues et les résultats expérimentaux
obtenus.

## Un seul moteur

Les trois activités sont des épisodes du même moteur (`Intelligence::reason`) :
mêmes capacités, même sélection apprise, même planification, même mémoire de
travail. Elles ne diffèrent que par trois choses :

- l'origine de leurs objectifs ;
- leur accès à l'environnement ;
- ce qu'elles ont le droit de modifier.

| Activité | Objectifs | Accès externe | Effets |
|---|---|---|---|
| Conscient (`action`) | événements reçus | réel, mesuré, contraint | connaissances, apprentissage |
| Inconscient (`reflection`) | questions sur la cognition elle-même | réel, aux mêmes coûts | corrections journalisées, influences, expériences |
| Rêve (`dream`) | aucun obligatoire | **aucun** : mode `Replay` | un registre des rêves séparé |

Aucun pipeline n'est imposé. Dans `idle`, l'intelligence choisit à chaque tour
entre réfléchir (et sur quoi), mener une expérience demandée, rêver ou
s'arrêter. Ce choix se fait par échantillonnage de Thompson, d'après ce que ces
activités ont apporté dans des situations semblables et en tenant compte de
leur coût.

## Rêve : imagination computationnelle (`engine/imagination.rs`)

Un rêve est une suite de scènes. Chaque scène est un épisode du moteur, exécuté
sur une copie isolée de la mémoire. Son décor est composé par des mécanismes
génériques, choisis au hasard, qui puisent dans la mémoire : faits, règles,
expériences, rêves précédents, impasses et représentation de soi.

- **Rappel** : des faits, ou le rejeu d'une expérience à partir des données
  enregistrées.
- **Association** : des souvenirs partageant une entité sont rapprochés, et un
  lien est conjecturé entre eux.
- **Transformation** : substitution d'une entité ou d'une relation, négation,
  suppression d'une condition.
- **Invention** : de nouvelles entités, relations et règles.
- **Capacité imaginée** : une spécification (signature fournie, forme des
  réponses) qu'un module générique simule dans le rêve, à partir des besoins
  que rien ne satisfait.
- **Anticipation** : une question jamais posée.
- **Contrainte** : des ressources illimitées, ou au contraire rares.
- **Représentation de soi** : telle quelle, ou autrement.

Les scènes s'enchaînent par continuation, association, saut, rejeu ou
anticipation, et le rêve peut s'interrompre à tout moment. Une scène n'a pas à
être cohérente, réaliste ou utile. Les besoins qui émergent sont les impasses
rencontrées dans la fiction.

Un rêve peut aussi commencer par une **expérience demandée** par l'inconscient :
le rejeu d'un épisode réel, historique ou réinterprété, avec une variante
(ordre des décisions, ablation d'une capacité, rareté, retrait d'une influence).
La scène est alors comparée à l'original.

### Frontière entre réel et fiction

Cette frontière est assurée par la provenance et l'isolation, pas par une
limitation de l'imagination.

| Élément | Représentation |
|---|---|
| Acquisition historique rejouée fidèlement | `Replay` : servie telle qu'enregistrée à la date de l'épisode |
| Information historique indisponible | erreur « left missing » : la donnée reste manquante |
| Acquisition volontairement inventée | `Imagined`, provenance `imagination` / `capability:<nom>` |
| Information transformée | `Imagined`, origine `transformation:<source>` |
| Souvenir réel mobilisé | `Recollection`, origine `recollection` / `association:<entité>` |
| Déduction en environnement fictif | `Deduction` dans le graphe isolé de la scène |

## Garanties effectives

### Aucune action sur l'environnement pendant un rêve

- En mode `Replay`, le client GitHub ne lance jamais `gh` : c'est le point de
  passage unique, et des tests le vérifient.
- La documentation n'est jamais téléchargée.
- Les capacités imaginées n'existent que dans les épisodes du rêve.

### Une donnée manquante n'est jamais inventée comme si elle était historique

- Ce qui n'a pas été enregistré à la date rejouée est refusé.
- Une scène d'invention peut ensuite inventer cette donnée, mais elle porte
  alors le genre `Imagined`.

### Aucun élément fictif ne devient un fait

- Un rêve ne touche jamais la mémoire de connaissances réelle : il travaille
  sur une copie.
- Les apprentissages simulés sont tenus à part. Ils ne sont pas utilisés comme
  preuves : la pondération de 0,5 a été supprimée, pas remplacée.
- Les conjectures rêvées ne deviennent des influences que si elles sont
  confirmées par des faits réels **indépendants de ceux que le rêve a
  utilisés**, sans aucun contre-exemple.

### Rien n'est jeté, tout reste borné

- Tous les rêves sont conservés : scènes, mécanismes, transitions, décor avec
  sa provenance, besoins apparus, conséquences imaginées, stratégies, coûts.
- Seules les traces détaillées des rêves anciens (au-delà des 50 derniers) sont
  allégées.
- Les ressources réellement consommées par un rêve sont mesurées (temps par
  scène et par rêve) et bornées : au plus 6 scènes, 24 étapes par scène,
  10 activités par `idle` par défaut.

### Pas de boucle de réinjection

- Les influences n'agissent que sur le conscient.
- Les rêves et la réflexion s'exécutent sans influence. Seul le rejeu qui teste
  une influence garde les autres.
- Le nombre d'influences actives est borné (64), et leur retrait est appris.

### Corrections explicables et réversibles

Chaque correction garde :

- sa base (preuve, expérience, résultats observés, incohérence) ;
- sa justification ;
- l'état qu'elle a remplacé.

`revert` la défait. L'inconscient revient de lui-même sur une réfutation dont
les faits fondateurs ont été retirés.

## Inconscient : remaniement et influence

- `evidence_review` examine les connaissances : contre-exemples, indépendance
  des corroborations, contradictions (doute injecté et vérification réelle
  demandée), conclusions sans fondement, remise en cause de ses propres
  corrections.
- `learning_review` examine l'apprentissage :
  - capacités improductives, jugées par bornes crédibles relatives et non par
    des seuils fixes ;
  - alternatives affamées par l'habitude, qui reçoivent une influence
    `Favour` ;
  - attribution des réussites, corrigée seulement sur une évidence
    **accumulée** d'ablations, sinon de nouvelles ablations sont demandées ;
  - impasses récurrentes, besoins tardifs, stratégies mal calibrées, réponses
    simulées à vérifier ;
  - examen de ses propres influences : retrait si elles sont crédiblement
    nuisibles, demande de démonstration sinon.
- `dream_interpretation` examine quelques rêves choisis au hasard parmi ceux
  qui ne l'ont pas encore été, sans obligation de les traiter tous. Il en tire :
  - des conjectures confirmées par la réalité ;
  - des besoins anticipés que la réalité peut satisfaire ;
  - des lacunes de capacités ;
  - des capacités imaginées qui auraient servi ;
  - des stratégies rêvées qui ont abouti ;
  - des opérations inutiles.

### Injection de contexte (`influences/`)

Une influence est un élément symbolique injecté par l'inconscient :
conjecture (hypothèse, association, reformulation), connaissance mise en avant,
besoin potentiel, doute, capacité à favoriser, choix habituel à questionner.
Elle s'applique quand la situation partage des prédicats avec son contexte.

- Les contenus entrent dans le raisonnement comme éléments réfutables, avec le
  producteur `unconscious:<id>`. La révision des croyances peut les retirer.
- Les préférences pèsent sur le choix sans l'imposer. Une influence tire dans
  la loi de son propre bilan réel (uniforme tant qu'elle n'a pas été testée).
  Une alternative obtient ainsi une vraie chance, et le conscient garde le
  dernier mot.
- Le moteur calcule aussi le choix qu'il aurait fait sans les influences. Une
  influence n'est **décisive** que si elle a changé ce choix.

Trois niveaux de preuve sont distingués, sans jamais conclure de la simple
présence :

1. **corrélation** : l'influence était présente ;
2. **contribution observée** : elle a changé une décision, avec l'issue de
   cette décision, ou son contenu a soutenu ou été réfuté par une réponse ;
3. **attribution démontrée** : un rejeu sans elle a changé le résultat, ou non.

## Exploration

Le sélecteur combinait les caractéristiques d'une situation en **moyennant**
plusieurs tirages, ce qui divisait l'incertitude par leur nombre. Une capacité
jamais essayée n'avait alors qu'environ 1 % de chances par décision face à une
capacité habituelle : c'est la cause de la capacité jamais essayée en
38 épisodes.

Les caractéristiques sont désormais un **mélange** : chaque décision tire dans
le postérieur d'une seule d'entre elles, ce qui préserve l'incertitude réelle.
Un test vérifie qu'une capacité jamais essayée finit par l'être. Les influences
`Favour` sont un second mécanisme, appris lui aussi.

## Résultats expérimentaux obtenus

Les essais ont été faits le 2026-11-26, sur un seul problème réel : la
publication de serde, et la recherche des dépôts BezotCorp qui verrouillent une
version antérieure. Ce n'est pas une évaluation statistique.

- **Exploration** : le rappel des connaissances, qui n'était presque jamais
  tenté, l'est désormais. Dès le 2e épisode, le conscient répond depuis la
  mémoire, sans aucune requête GitHub. La réponse est étiquetée « historique »,
  avec une péremption d'environ 0,0003, car la dernière vérification date de
  quelques secondes. Une vérification ne vaut alors pas son coût. C'est un
  changement de comportement par rapport à la version précédente, qui
  revérifiait à chaque épisode (2 requêtes) pour un risque négligeable.
- **Activités autonomes** : 15 activités en 0,07 s, sans aucune requête
  externe.
  - Elles ont produit 5 rêves : 15 scènes, 8 mécanismes différents, 67 besoins
    apparus.
  - Elles ont produit 5 influences, qui ont changé 10 décisions dans les
    épisodes suivants. Les 4 influences exposées en sont restées au niveau
    « contribution observée », aucune n'est encore démontrée ; la 5e n'a pas
    encore été exposée.
  - Une conjecture absurde, « confirmée » par les faits mêmes dont l'imagination
    l'avait tirée, a révélé un défaut. Il est corrigé : seules comptent
    désormais les confirmations indépendantes du rêve.
- **Comparaison des activités** : 5 configurations, 3 passes de 1 problème,
  8 activités autonomes par passe.
  - Toutes répondent juste à chaque fois.
  - Dans l'architecture complète, les influences ont changé 5 décisions.
  - Le nombre d'invocations varie entre 12 et 25 selon les épisodes et les
    configurations, ce qui reste dans le bruit du sélecteur. **Aucune
    amélioration attribuable à l'inconscient, au rêve ou aux influences n'est
    démontrée** sur ce problème unique, déjà maîtrisé dès le premier épisode.

Pour aller plus loin, le script `scripts/run-experiments.sh` permet :

- l'étude des activités, avec `AUTONOMOUS_AI_STUDY=activities` :
  - M : action seule ;
  - U : action + inconscient ;
  - R : action + rêve ;
  - F : architecture complète ;
  - N : architecture complète, mais conscient sourd aux influences, pour
    isoler leur efficacité ;
- la généralisation à des problèmes jamais vus, avec
  `AUTONOMOUS_AI_TEST_EVENTS` ;
- la séparation entre gain technique et gain cognitif, avec `--without-cache`.

Une conclusion exige plusieurs problèmes variés et de nombreuses exécutions.

## Limites connues

- **Mécanismes d'imagination** : ils sont génériques mais en nombre fini.
  L'émergence reste celle de leurs combinaisons. Les probabilités de
  transition et d'interruption (0,3), les bornes de taille des scènes et les
  quantités de matériau sont des réglages documentés, pas des grandeurs
  apprises.
- **Choix des mécanismes** : il n'est pas encore appris. On pourrait le relier
  à l'exploitation ultérieure des rêves par l'inconscient.
- **Valeur d'un rêve pour l'ordonnancement** : `idle` juge un rêve sur sa
  nouveauté immédiate, et non sur la valeur, connue plus tard, des influences
  qui en sont issues.
- **Critères métacognitifs** : les bornes crédibles (2 écarts-types) et le
  seuil « plus probable que non » (0,5) restent des choix de décision. Les
  analyses de l'inconscient sont codées comme des méthodes et non déclarées
  comme des règles.
- **Contexte d'une influence** : il est défini par des prédicats partagés. Une
  influence peut donc s'appliquer à des situations seulement voisines. Son
  bilan réel l'écartera si elle nuit.
- **Fidélité des rejeux** : les rejeux historiques sont fidèles pour les
  données adressées par empreinte. La mémoire reconstituée à une date ne
  sépare pas les vérifications successives d'une même portée.
- **Coût mémoire** : il est mesuré en temps, pas en octets. La taille de
  `memory.bin` augmente avec les rêves conservés (3,6 Mo après 6 épisodes et
  15 activités).
