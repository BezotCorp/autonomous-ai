# Génération « existence » : individus vivants, espace latent et cycle de vie

Ce document décrit la refonte qui fait de l'intelligence un **individu** qui
naît, existe en continu, évolue et persiste entre les exécutions. Il complète
les documents précédents (`cognitive-activities.md`, archives, README), qui
restent les traces des générations antérieures et ne sont pas modifiés.

Il présente :

- l'organisation et le cycle de vie ;
- la représentation mathématique du latent et ses lois ;
- la naissance et l'individualité ;
- les couplages entre latent et cognition ;
- la plasticité, la temporalité et la persistance ;
- les garanties, l'observation et l'expérimentation ;
- les choix faits, les alternatives écartées, les limites et les résultats
  réellement obtenus.

Les paramètres numériques introduits sont presque tous des **ordres de
grandeur choisis**, pas des constantes établies. Chacun est signalé, avec ce
qu'il contrôle et l'effet probable de sa modification.

## 1. Vue d'ensemble

```
                      ┌──────────────────────── Existence (existence.bin) ───────────────────────┐
 stimuli (inbox,      │  Identité + naissance ─► Constitution (lois nées)    Dispositions        │
 planning)  ────────► │                               │                    (lois acquises)       │
                      │                               ▼                         ▲                │
                      │          occurrences ─► Organisme : état latent z ──────┘ plasticité     │
                      │              ▲                 │ canaux (taux, températures, probabilités)│
                      │              │                 ▼                                          │
                      │   Moteur unique (Intelligence::reason_with) ◄── Cycle de vie (Life)      │
                      │   action · réflexion · introspection · simulation · rêverie · rêve       │
                      │              │                                                            │
                      │   Mémoire cognitive (connaissances, routage, expériences, rêves,          │
                      │   corrections, influences) · préoccupations · activités · chronologie     │
                      └──────────────────────────────────────────────────────────────────────────┘
                                         │ réponses (outbox)
```

Les couches sont les suivantes.

| Module | Responsabilité |
|---|---|
| `individuality/` | naissance (`BirthParameters`, `Identity`), lois nées (`Constitution`, `ConstitutionScales`, `ChannelLaw`, `ReactionLaw`), lois acquises (`Dispositions`), canaux (`Channel`, `ChannelForm`) |
| `latent/` | état latent (`LatentState`), dynamique (`LatentDynamics`), occurrences (`Occurrence`), registre des réactions (`ReactionCatalog`, `ReactionRecord`) |
| `engine/` | moteur cognitif unique. Nouveautés : le contrat `Disposition` (lire une `Modulation`, ressentir des occurrences) et `NeutralDisposition` (laboratoire) |
| `living/` | existence persistante (`Existence`, `ExistenceStore`), organisme (`Organism`, qui implémente `Disposition`), cycle de vie (`Life`), préoccupations, activités, stimuli, vigilance, chronologie, boîtes aux lettres, synthèse d'observation |
| `interface/` | commandes (`CommandLine`, `execution`), inspection (`Inspection`) |

Rien de cette génération ne remplace les mécanismes cognitifs existants :

- le moteur de raisonnement ;
- la sélection par échantillonnage de Thompson ;
- la planification ;
- l'imagination ;
- les influences ;
- les corrections réversibles.

La refonte les **module** par l'état interne, les **ordonne** dans une
existence continue, et leur fait **produire** des occurrences qui modifient cet
état.

### Ce qui a été retiré

- La séparation `run` / `idle`, ainsi que le planificateur `Autonomy`. Il
  choisissait les activités par échantillonnage de Thompson sur leur
  « utilité » passée, ce qui revenait à une fonction de récompense
  (contraire au §14 du mandat). Ses tests (rejeu, attribution, exploration,
  influences, imagination) ont été conservés et déplacés dans
  `engine/imagination.rs`.
- Le journal d'activités `ActivityLog` de la mémoire cognitive, remplacé par
  le registre d'activités de l'existence.
- Le stockage `memory.bin` (`MemoryStore`, format `AAIM` v5), remplacé par
  `existence.bin` (format `AAIX` v1).

## 2. Cycle de vie (`living/life.rs`)

### Principe

Il n'y a pas de workflow. À chaque instant :

1. les **activités en cours** avancent par **tranches** :
   - pour le raisonnement : un épisode du moteur, 12 invocations nominales ;
   - pour l'imagination : un déroulement ;
2. **chaque changement possible** arrive à un **taux** (par heure) lu dans
   l'état latent par un canal individuel. Ces changements sont :
   - agir sur un stimulus ;
   - commencer une activité ;
   - en reprendre une suspendue ;
   - s'endormir ou se réveiller ;
   - laisser une préoccupation devenir consciente ;
3. les changements sont en **compétition** (temps d'attente exponentiels
   concurrents, procédé de type Gillespie). Le premier à échoir se produit ;
4. après chaque tranche, l'activité peut :
   - être **interrompue** (taux `Interrupt`) ;
   - **dériver** vers une activité voisine (taux `Transform`) ;
   - être **abandonnée** par le raisonneur (probabilité `Abandonment` à
     chaque invocation) ;
5. quand rien n'est en cours, le temps passe jusqu'au prochain changement :
   - en temps réel, le processus dort et relève la boîte de réception chaque
     seconde ;
   - en temps virtuel, il saute ce délai.

   L'état latent continue d'évoluer pendant ce temps.

### Contraintes structurelles

Ce sont les seules règles imposées, et elles ne décident d'aucun
comportement :

- **une** activité consciente à la fois : en commencer une autre suspend
  celle en cours ;
- au plus `MAXIMUM_UNCONSCIOUS = 2` activités inconscientes en plus. Elles
  s'entrelacent tranche par tranche avec l'activité consciente ;
- le sommeil ferme le conscient :
  - les activités conscientes sont suspendues ;
  - les stimuli attendent et augmentent le taux de réveil ;
  - l'inconscient continue ;
  - le rêve n'est possible qu'endormi ;
- une activité est bornée techniquement à `MAXIMUM_SLICES = 64` tranches. La
  fin `bounded` est enregistrée comme limite technique, pas comme décision.

### Changements possibles

| État | Changement | Canal | Contexte ajouté au log-taux |
|---|---|---|---|
| éveillé | agir sur le stimulus *s* | `Attend` | `ln(saillance de s)` |
| éveillé | introspection, rêverie | `Introspect`, `Reverie` | — |
| éveillé | simuler la préoccupation consciente *c* | `Simulate` | `ln(s_c / Σs · ln(1 + Σs))` |
| éveillé | rendre consciente la préoccupation *c* | `Surfacing` | idem, sur les inconscientes |
| éveillé | s'endormir | `FallAsleep` | — |
| endormi | se réveiller | `Wake` | `ln(1 + Σ saillances des stimuli en attente)` |
| endormi | rêver | `Dream` | — |
| toujours | réflexion inconsciente | `Reflect` | — |
| toujours | reprendre l'activité suspendue *a* | `Resume` | `ln(maintien de a + saillance de sa préoccupation)` |

Le taux est `exp(biais + terme latent + contexte)`, borné à 3600 par heure
(une fois par seconde, `Decision::MAXIMUM_RATE`). Chaque décision est
enregistrée avec cette décomposition (§9).

**Pourquoi `ln(1 + Σs)`.** Les premiers essais faisaient croître le taux total
de simulation linéairement avec le nombre de préoccupations. L'individu 2,
avec 120 préoccupations conscientes, ne faisait alors plus que simuler. La
saturation logarithmique est un choix : plus de préoccupations poussent
davantage, mais avec des rendements décroissants. L'inondation reste possible
pour un individu dont le canal `Simulate` est élevé ; elle n'est plus imposée
par la forme de la loi.

### Dérives (transformations)

| Activité | Peut dériver vers |
|---|---|
| action | simulation, rêverie |
| introspection | simulation, rêverie |
| simulation | introspection, rêverie |
| rêverie | introspection, simulation |
| réflexion (endormi) | rêve |
| rêve | réflexion |

La cible est tirée selon les taux de ces activités dans l'état courant. Le
sujet de l'activité est conservé : une action qui dérive en simulation simule
la question de l'action.

### Délibération (activité volontaire)

Une introspection perçoit les préoccupations conscientes. Avec la
probabilité du canal `Deliberation`, elle **en reprend une**, tirée selon sa
saillance, dans une simulation d'origine `voluntary`. Les autres activités
sont d'origine :

- `stimulus` ;
- `spontaneous` ;
- `concern` ;
- `transformation` ;
- `intervention` (expérimentateur).

### Ce que produisent les tranches

| Activité | Tranche | Fin |
|---|---|---|
| action | épisode sur l'événement, avec les influences sauf `--without-influences` | toutes les questions ont une réponse définie (une réponse est alors écrite dans `outbox/`) ; sinon elle continue |
| réflexion | épisode sur une question réflexive tirée au hasard, sans influences | elle continue tant qu'elle produit corrections, influences ou expériences |
| introspection | épisode avec perception de soi et toutes les questions réflexives (§5) | une tranche |
| simulation | déroulement d'imagination dirigé par le sujet (gardé comme question dans chaque scène) | une tranche |
| rêverie | déroulement libre, thèmes : préoccupations conscientes | une tranche |
| rêve | déroulement libre (reprend la première expérience demandée par l'inconscient, s'il y en a), thèmes : toutes les préoccupations | une tranche |

Une tranche dure au moins `MINIMUM_SLICE = 1 s` vécue, pour que le temps
avance toujours.

## 3. Conscient et inconscient

Un seul moteur, une seule mémoire. Conscient et inconscient diffèrent par leurs
**conditions d'accès et d'activité**, pas par une duplication des mécanismes :

| | Conscient | Inconscient |
|---|---|---|
| activités | action, introspection, simulation, rêverie | réflexion, rêve |
| influences de l'inconscient | admises, chacune avec la probabilité `InfluenceAdmission` | jamais (pas de boucle de réinjection) |
| exclusivité | une à la fois | jusqu'à deux, entrelacées avec le conscient |
| sommeil | fermé | continue |

**Inconscient vers conscient :**

- les influences (conjectures, croyances, besoins potentiels, doutes,
  préférences pour ou contre un module), injectées et admises selon l'état ;
- les préoccupations qui deviennent conscientes ;
- la modulation de tous les mécanismes par l'état latent, que le conscient ne
  lit jamais directement.

**Conscient vers inconscient :**

- les activités conscientes produisent les expériences que l'inconscient
  réexamine et rejoue ;
- l'attention consciente change la saillance des préoccupations, qui
  deviennent les thèmes des rêves ;
- les introspections exécutent les modules réflexifs, qui peuvent questionner
  ou retirer des influences de l'inconscient (corrections réversibles) ;
- l'introspection agit sur ses propres préoccupations : elle les ravive ou les
  apaise, et peut délibérément en reprendre une.

**Modèle d'exécution.** Le parallélisme est coopératif et monofil : les
tranches s'entrelacent. Ce choix garantit une mémoire cohérente sans verrous
et une existence reproductible. L'inconscient agit « pendant » une activité
consciente au sens de l'entrelacement, pas d'une exécution simultanée.

## 4. Activités : distinctions retenues

Chaque activité enregistrée porte les caractéristiques suivantes :

- son **type** (`memory::activity::Activity`) ;
- son **origine** (`ActivityOrigin`) ;
- son **sujet** (`ActivityFocus` : un événement, une préoccupation, ou rien) ;
- son **accessibilité** (consciente ou non, selon le type) ;
- son **degré de direction**, qui découle du type et du sujet :
  - action et simulation sont dirigées ;
  - rêverie et rêve sont libres ;
  - réflexion et introspection sont dirigées vers soi ;
- son **évolution** : tranches, suspensions, reprises, dérive ;
- sa **fin** (`ActivityEnding`) :
  - `completed` ;
  - `abandoned` ;
  - `interrupted` ;
  - `transformed` ;
  - `dissolved` (suspendue puis oubliée) ;
  - `bounded`.

Le **repos** n'est pas une activité : c'est l'absence d'activité, éveillé.
Le **sommeil** est une configuration globale (`Vigilance`). Il faut distinguer
plusieurs situations :

- l'**absence d'action extérieure** : il n'y a pas d'action, mais il peut y
  avoir introspection, rêverie ou réflexion ;
- l'**absence d'activité consciente** : sommeil, ou repos éveillé ;
- l'**absence d'activité intellectuelle** : rien en cours ; l'état latent et
  ses lois évoluent quand même ;
- l'**arrêt technique** : le processus est arrêté. Seule la relaxation
  déterministe est appliquée à la reprise (§8).

## 5. Introspection

Une introspection est un épisode du moteur, sur des questions réflexives
(`knowledge_concern`, `learning_concern`, `possibility`), qui **perçoit
l'individu lui-même**. Les éléments perçus sont des observations datées de
source `introspection`. Chacune a son propre aspect (portée), si bien qu'une
perception plus récente remplace l'ancienne en mémoire :

- `sensation(sK, low|middle|high)` pour K = 1…6. Ce sont des **projections
  anonymes** de l'état latent, sur des directions tirées à la naissance et
  normalisées, avec des seuils à ±0,3. L'individu ne lit jamais ses dimensions
  ni ses lois ; il perçoit des manifestations grossières ;
- `concern(cXXXXXXXX, prédicat)` pour ses huit préoccupations conscientes les
  plus saillantes ;
- `recent_activity(rN, type, fin)` pour ses cinq dernières activités ;
- `vigilance(awake)`.

Ces observations entrent dans la mémoire comme les autres. Les modules
existants peuvent donc s'en servir, par exemple pour induire des régularités
reliant des sensations à des activités. **Rien n'impose que l'introspection
améliore quoi que ce soit** : elle peut tourner en rond, raviver une
préoccupation inutile, ou ne rien produire.

## 6. Imagination, rêverie, simulation, rêve

L'imagination de la génération précédente est conservée : scènes et mécanismes,
isolation, provenance, conjectures confrontées à la réalité. Elle sert
maintenant à trois activités, distinguées par `DreamRecord.activity` :

- **simulation** : dirigée. Son sujet reste une question dans chaque scène ;
- **rêverie** : libre, éveillée. Thèmes : les préoccupations conscientes ;
- **rêve** : libre, endormi. Thèmes : toutes les préoccupations. Il peut
  reprendre une expérience demandée par l'inconscient.

L'état latent module le déroulement :

- la probabilité de passer à une autre scène (`SceneContinuation`) ;
- le poids des huit mécanismes : rappel, association, transformation,
  invention, capacité imaginée, anticipation, contrainte, représentation de
  soi.

Chaque scène est un épisode du moteur **sous l'organisme**. Le rêve modifie
donc l'état latent, et un état modifié change la suite du rêve.

Les besoins apparus dans un déroulement forment jusqu'à 3 préoccupations
(origine `imagination:<type>`). Une possibilité imaginée peut ainsi orienter
l'attention et les décisions **sans avoir été confirmée**. En revanche, un
contenu imaginé ne devient jamais une observation du monde : la provenance
et l'isolation de la génération précédente sont intactes.

## 7. L'espace latent

### Représentation

L'état est un point `h ∈ [−6, 6]^D`. Les mécanismes lisent `z = tanh(h)`,
dans l'hypercube ouvert `(−1, 1)^D`.

- La dimension D est un paramètre de naissance : 12 par défaut, entre 2 et 64.
- Aucune dimension n'a de signification : seules les lois individuelles qui
  lisent et déplacent `z` en ont une.
- `tanh` borne la lecture sans écraser la dynamique. La borne de 6 sur `h`
  (tanh 6 = 0,999988) garantit des valeurs finies, même sous impulsions
  extrêmes.

### Dynamique (`latent/latent_dynamics.rs`)

```
dh/dt = (b + W · tanh(h) − h) / τ  +  fluctuations d'Ornstein–Uhlenbeck
```

- `τ_i` est le temps de relaxation de chaque dimension ;
- `b` est le point de repos effectif (né + acquis) ;
- `W` est le couplage interne effectif (né + acquis) ;
- les fluctuations ont l'intensité `σ_i`.

C'est un réseau récurrent à fuite, classique en dynamique non linéaire, choisi
pour quatre raisons :

1. selon le gain du couplage (`g` ≈ 1 et au-delà), il peut :
   - relaxer vers un point fixe ;
   - présenter plusieurs points fixes (bistabilité, « humeurs » durables
     sans nom) ;
   - osciller ;
   - devenir chaotique.

   Le régime n'est **pas choisi** : il découle de la naissance ;
2. ses interactions entre dimensions (`W`) produisent des conséquences
   croisées et contradictoires sans les programmer ;
3. il se prête à un intégrateur stable quel que soit le pas ;
4. ce n'est pas un réseau de neurones entraîné : aucune fonction n'est
   apprise par gradient. C'est un système dynamique aux lois tirées au
   hasard, modifiées lentement par la plasticité.

### Intégration

L'intégrateur est exponentiel. La cible `T = b + W·tanh(h)` est gelée sur un
pas, et l'on calcule `h ← T + (h − T)·e^(−dt/τ)`, plus un bruit de variance
exacte `σ²(1 − e^(−2dt/τ))`.

- Le pas vaut `max(τ_min/4, durée/2000)`. Une durée quelconque (un arrêt de
  plusieurs jours) s'intègre donc en au plus 2000 pas.
- Pour les très longs pas, l'intégration devient l'itération de point fixe
  `h ← b + W tanh h`. C'est une approximation assumée.
- Tout composant non fini est remplacé par le point de repos et **compté**
  (`repairs`), comme défaut logiciel. Aucun n'a été observé.

### Occurrences et réactions

Tout ce qui arrive est une **occurrence** à signature ouverte `famille:détail`,
avec une magnitude comprise entre 0 et 10.

Signatures produites par le moteur :

- `invoke:<module>`, `produce:<module>` (magnitude ln(1+n)),
  `barren:<module>`, `error:<module>` ;
- `consume:<ressource>` (magnitude ln(1+unités)), `refused:resource` ;
- `influenced:decision`, `revision:belief` ;
- `termination:<raison>`, `answer:<statut>`.

Signatures produites par la vie :

- `stimulus:<source>`, `respond:event`, `neglect:stimulus` ;
- `begin:<type>`, `end:<type>:<fin>`, `resume:<type>`, `suspend:activity` ;
- `vigilance:asleep|awake` ;
- `concern:<origine>`, `concern:surfaced|renewed|faded|resolved`,
  `revisit:concerns` ;
- `deliberate:<type>`, `imagine:<mécanisme>`.

La **loi de réaction** d'une signature est tirée d'un hachage stable
(FNV-1a) de la graine et de la signature. Une occurrence jamais rencontrée
reçoit donc une loi **nouvelle mais reproductible**, sans table de
comportements :

```
Δh = magnitude · force · gain · (directe + (gate_in · z) · gate_out)
```

- **directe** = `α·famille + (1−α)·propre` : les occurrences d'une même
  famille (`invoke:*`) ont des effets apparentés, dans la proportion α de
  l'individu ;
- la partie **conditionnée** dépend de l'état. La même occurrence peut donc
  pousser dans des directions opposées selon l'état courant ;
- aucune occurrence n'est bonne ou mauvaise par construction. Un même appel
  d'API peut favoriser chez un individu les états qui augmentent le taux
  d'action, et chez un autre les états qui l'inhibent. Cela n'est pas
  programmé : cela découle de lois tirées au hasard.

Les impulsions de plusieurs occurrences simultanées s'additionnent. Leurs
interactions passent par la non-linéarité et le couplage.

### Temps

- Le **temps vécu** avance pendant que le processus tourne : temps réel, ou
  temps virtuel (2 s par invocation, et les attentes sont sautées).
- Le temps de l'**arrêt technique** est appliqué à la reprise, sans bruit,
  sans occurrence et sans plasticité (§8).

## 8. Naissance et individualité

### Tirages (`ConstitutionScales`, `Constitution`)

Une naissance est entièrement déterminée par :

- la **graine** ;
- la **dimension** ;
- la **version du générateur** (`constitution-1`).

Deux flux pseudo-aléatoires indépendants sont dérivés de la graine :
`scales`, puis `laws`. Un troisième, `existence`, sert au fonctionnement. Les
lois nées ne sont **jamais retirées** : elles changent seulement par
plasticité, enregistrée à part. Le hasard constitutif (naissance) et le hasard
de fonctionnement sont donc distincts.

**Échelles individuelles.** Elles sont tirées d'abord, ce qui permet à deux
individus de différer en nature et pas seulement en détail. Toutes suivent
une loi log-normale (médiane, écart du logarithme), sauf mention contraire.

| Échelle | Loi | Contrôle | Effet d'une modification |
|---|---|---|---|
| `time_scale` | LN(600 s, 0,7) | lenteur de l'état | plus grand : états plus durables, réactions plus inertes |
| `time_spread` | LN(0,8, 0,3) | hétérogénéité des τ_i | plus grand : coexistence de dimensions rapides et lentes |
| `coupling_gain` g | LN(0,9, 0,4) | force des interactions internes | > 1 : multistabilité, oscillations ou chaos |
| `baseline_scale` | LN(0,5, 0,5) | dispersion du point de repos | plus grand : dispositions de repos plus marquées |
| `noise` | LN(0,3, 0,6) | fluctuations spontanées | plus grand : comportement plus erratique |
| `impulse_scale` κ | LN(0,4, 0,5) | effet des occurrences | 0 : individu « inerte », que rien ne touche |
| `gating` ρ | LN(1, 0,5) | dépendance des réactions à l'état | plus grand : réactions plus contextuelles |
| `family_share` α | U(0,2, 0,9) | similitude des réactions apparentées | plus grand : généralisation plus forte |
| `sensitivity_scale` σ_s | LN(1,5, 0,5) | emprise du latent sur les mécanismes | 0 : latent **découplé** de la cognition |
| `bias_spread` | LN(1, 0,3) | dispersion des biais des canaux | plus grand : individus plus contrastés au repos |
| `gain_plasticity` | LN(0,05, 1) | sensibilisation ou habituation par occurrence | |
| `coupling_plasticity` | LN(0,02, 1) par heure | évolution du couplage | |
| `baseline_plasticity` | LN(0,01, 1) par heure | dérive du point de repos | |
| `habit_plasticity` | LN(0,1, 1) par engagement | formation d'habitudes | |
| `coupling_polarity` | U(−1, 1) | renforcement ou inhibition par coactivité | |

**Lois tirées ensuite :**

- `τ_i ~ LN(time_scale, time_spread)`, au moins 1 s ;
- `b_i ~ N(0, baseline_scale²)` ;
- `W_ij ~ N(0, g²/D)` (normalisation usuelle des réseaux aléatoires) ;
- `σ_i ~ LN(noise, 0,3)` ;
- état initial `h = b + σ·N(0, 1)` ;
- pour chaque canal : biais `= centre + bias_spread·N(0, 1)`, sensibilités
  `~ N(0, σ_s²/D)` ;
- 6 directions de sensations `~ N(0, 1)`.

**Profils.** Aucun profil n'est programmé. Les écarts log-normaux larges
produisent volontairement des individus extrêmes : très inertes ou très
réactifs, découplés ou hypersensibles. Le §15 montre la diversité observée.

### Canaux (`individuality/channel.rs`)

Les 30 canaux nomment des **paramètres de mécanismes**, jamais des qualités
psychologiques. Leur activation vaut `biais + (s_né + s_acquis) · z`. Elle est
bornée à ±25, puis mise en forme selon le type de canal :

- **taux** (par heure), `exp(activation)` : `Attend`, `Reflect`,
  `Introspect`, `Simulate`, `Reverie`, `Dream`, `FallAsleep`, `Wake`,
  `Resume`, `Interrupt`, `Transform`, `ConcernFading`, `Surfacing` ;
- **facteur** autour de 1, `exp(activation)` : `SamplingTemperature`,
  `CostSensitivity`, `SliceLength`, `StimulusGain`, `ConcernGain`, et les 8
  poids des mécanismes d'imagination ;
- **probabilité**, `logistique(activation)` : `InfluenceAdmission`,
  `Abandonment`, `SceneContinuation`, `Deliberation`.

Les centres des biais sont des ordres de grandeur. Ils ont été choisis pour
que chaque mécanisme puisse se manifester en quelques heures d'existence :

| Canal | Centre |
|---|---|
| `Attend` | 30/h |
| `Reflect` | 1,5/h |
| `Introspect` | 1/h |
| `Simulate` | 1/h |
| `Reverie` | 2/h |
| `Dream` | 4/h (endormi) |
| `FallAsleep` | 1/8 h |
| `Wake` | 1/2 h |
| `Resume` | 1/h |
| `Interrupt` | 0,5/h d'activité |
| `Transform` | 0,3/h |
| `ConcernFading` | 0,2/h |
| `Surfacing` | 1/h |
| `Abandonment` | ≈ 1 % par invocation |
| `InfluenceAdmission` | 75 % |
| `SceneContinuation` | 70 % |
| `Deliberation` | 50 % |
| facteurs | 1 |

## 9. Couplage latent → cognition

Le latent agit sur **l'ensemble** de l'organisation, pas seulement sur un
sélecteur d'activités.

| Mécanisme | Effet de l'état (par canal individuel) |
|---|---|
| apparition, reprise, sommeil, réveil, surgissement des activités et préoccupations | taux (§2) |
| interruption, dérive d'une activité en cours | taux par heure d'activité |
| sélection des modules (Thompson) | **température** des postérieurs Beta : `Beta(α/T, β/T)` garde la moyenne et élargit l'incertitude si T > 1 (plus d'exploration) ou la resserre si T < 1 |
| poids des coûts dans la sélection | facteur `CostSensitivity` |
| influences de l'inconscient | chaque préférence n'est admise qu'avec la probabilité `InfluenceAdmission` |
| persistance du raisonnement | abandon possible avant chaque invocation (`Termination::Abandoned`) |
| longueur d'une tranche | facteur `SliceLength`, borné à [0,25 ; 4] |
| saillance des stimuli perçus | facteur `StimulusGain` |
| préoccupations | effacement, surgissement, gain à la formation, effet de l'attention (`s ← s · gain^0,5` : l'attention **amplifie ou apaise** selon l'état) |
| imagination | continuation des scènes, poids des 8 mécanismes |
| délibération | probabilité de reprendre une préoccupation |

### Lecture pendant l'épisode

La modulation est relue **avant chaque invocation**, et chaque invocation est
ressentie aussitôt. Le latent change donc pendant un épisode, et ses
changements modifient la suite de l'épisode, y compris la décision
d'abandonner.

### Neutralité du laboratoire

Sous `NeutralDisposition`, le moteur se comporte exactement comme avant : il
ne tire aucun nombre aléatoire supplémentaire, avec une température de 1, des
facteurs de 1 et une probabilité d'abandon nulle. Les 35 tests antérieurs
passent sans modification de leurs attentes.

### Causalité

Chaque décision enregistrée (`Decision`) porte la décomposition de son
log-taux : biais, terme latent et contexte. Le **terme latent est la
contribution causale exacte de l'état au taux du mécanisme** : avec un état
nul, ce taux serait multiplié par `exp(−terme)`.

Ce n'est pas une causalité sur le comportement, qui dépend aussi du hasard et
de la compétition. Pour celle-ci, il faut les expériences contrôlées du §14 :

- découplé (`sensitivity_scale = 0`) ;
- inerte (`impulse_scale = 0`) ;
- jumeau ;
- interventions.

## 10. Plasticité (`individuality/dispositions.rs`)

Quatre transformations lentes, chacune à un taux individuel. Aucune ne vise
une amélioration.

| Transformation | Loi | Borne |
|---|---|---|
| gains de réaction (sensibilisation ou habituation) | `ln g_σ += η_g · (d_σ · z)` à chaque occurrence. La direction `d_σ` est tirée avec la loi de réaction : c'est l'état au moment de l'occurrence qui décide si la répétition renforce ou affaiblit | `ln g ∈ [−4, 3]` |
| couplage interne | `ΔW_ij = polarité · η_W · x_i x_j · heures` (i ≠ j, `x = tanh h`) : règle de coactivité, renforçante ou inhibitrice selon l'individu | `‖ΔW‖ ≤ 2·max(‖W₀‖, 0,1)` |
| point de repos | se rapproche des états vécus d'une fraction `1 − e^(−η_b·heures)` | `|b| ≤ 3` |
| habitudes | quand un mécanisme est engagé (une tranche d'une activité), `Δs_canal += η_s · (z_après − z_avant)` : les états auxquels mène une activité finissent par la favoriser ou la défavoriser | `‖Δs‖ ≤ 3σ_s` |

### Temporalités distinctes

- le **latent** évolue en secondes ou en heures (τ) ;
- les **gains** changent à chaque occurrence ;
- le **couplage** et le **point de repos** changent en heures, voire en jours ;
- les **habitudes** changent à chaque engagement ;
- les **connaissances** et les **stratégies** (routage) sont apprises par les
  mécanismes existants. Elles interagissent avec les précédentes, puisque le
  latent module l'exploration, donc ce qui est appris.

La plasticité peut renforcer une mauvaise habitude. Elle peut aussi fixer un
individu dans un état extrême : le point de repos suit les états vécus. Elle
ne peut pas produire de valeur invalide.

## 11. Préoccupations (`living/concern*.rs`)

Une préoccupation a :

- un sujet (un énoncé, éventuellement avec variables) ;
- une origine générique ;
- une saillance, bornée à 50 ;
- un état : inconsciente ou consciente, puis active, résolue ou effacée.

### Origines et saillance initiale

La saillance initiale est multipliée par le canal `ConcernGain`.

| Origine | Saillance initiale | Remarque |
|---|---|---|
| impasse racine d'une **action** | 1 | |
| action non terminée | 1,5 | |
| stimulus négligé | 1,5 | |
| contradiction trouvée par l'examen | 0,5 | |
| besoin apparu en imagination | 0,3 | au plus 3 par déroulement |

### Évolution

- **Effacement** : `s ← s · e^(−ConcernFading · heures)`. Sous 0,05, la
  préoccupation est effacée.
- **Ravivement** : une préoccupation existante se ravive quand elle se forme
  à nouveau, ou par écho, quand un stimulus porte sur le même prédicat
  (gain de 0,2).
- **Attention** : la simulation de la préoccupation, ou son examen en
  introspection, donne `s ← s · gain^0,5`.
- **Résolution** : la préoccupation est résolue quand une réponse prouvée
  s'unifie avec son sujet.

### Bornes

- au plus 128 préoccupations actives : les moins saillantes sont
  déplacées ;
- au plus 2000 préoccupations conclues conservées.

### Rumination

La rumination n'est pas programmée. Elle apparaît quand un individu :

- efface lentement ses préoccupations ;
- les amplifie par l'attention ;
- simule beaucoup.

C'est le cas de l'individu 3 du §15. Une première version, où l'attention
ajoutait toujours de la saillance, la rendait quasi universelle. Cela a été
corrigé.

**Réflexion.** Une réflexion qui ne trouve rien n'est pas une impasse. Une
première version transformait l'absence de constat en préoccupation
(`knowledge_concern(?x, ?y)`, activée 245 fois), un artefact retiré.

## 12. Contraintes externes et fiabilité

- **Ressources** : les quotas, permissions et erreurs restent ceux du
  compteur de ressources existant. La modulation change le **poids** des
  coûts, jamais leur réalité : une ressource épuisée reste infaisable, quel
  que soit l'état.
- **Isolation de l'imaginaire** : un rêve ou une simulation n'atteint jamais
  l'environnement, comme dans la génération précédente.
- **Garanties numériques** :
  - `h` est fini et borné ;
  - les activations des canaux sont bornées à ±25 et les taux à 3600/h ;
  - les probabilités restent dans [0, 1] ;
  - toutes les valeurs de `Modulation` sont bornées, et une valeur non finie
    est ramenée à la neutralité ;
  - les magnitudes, gains, couplages et sensibilités sont bornés ;
  - les saillances sont bornées ;
  - les durées non finies ou négatives sont ignorées ;
  - les tirages exponentiels d'un taux nul ou non fini donnent l'infini.
- **Tests de robustesse** :
  - constitutions extrêmes (couplage ×1000 ou 0, bruit ×10⁶, τ ×10⁻⁹ ou
    ×10¹², point de repos ×10⁹), intégrées sur des durées allant jusqu'à
    10⁹ s, NaN et ∞ compris ;
  - dix variations extrêmes vivant 3 h :
    - couplage ×20, bruit ×50, impulsions ×100 ;
    - sensibilités ×30, dispersion des biais ×25 ;
    - les quatre plasticités ×1000 ;
    - τ ×10⁻⁶.

  Aucune réparation ni erreur n'est survenue.
- **Stabilité cognitive** : elle n'est pas garantie et n'est pas visée. Une
  existence peut rester endormie, ruminer, négliger les sollicitations ou ne
  rien faire. Rien ne corrige une configuration parce qu'elle est inhabituelle.

## 13. Persistance, données et temporalité

### Organisation de `product/data/`

- `resources/` contient les ressources initiales (RON, versionnées) :
  `knowledge.ron`, `documentation-queries.ron`, `lexicon.ron`. Ce sont des
  copies de `../data/resources/`, qui n'a pas été modifié et que le produit
  n'utilise plus.
- `individuals/<nom>/` contient les existences (générées, non versionnées) :
  - `existence.bin` : l'existence entière, au format binaire `AAIX` v1,
    compressé par bincode. Elle est sauvegardée atomiquement (fichier
    temporaire puis renommage, `fsync`), si bien que ses parties ne divergent
    jamais ;
  - `existence.bin.lock` : verrou exclusif d'un processus vivant ou
    modificateur ;
  - `inbox/` : stimuli à déposer. Ils sont déplacés dans `inbox/received/`,
    ou dans `inbox/rejected/` s'ils sont illisibles ;
  - `outbox/` : réponses ;
  - `stop` : demande d'arrêt technique.
- `experiments/` contient les sorties d'expériences (non versionnées).

### Contenu de `existence.bin`

- l'identité (identifiant, nom, naissance, lignée des duplications) ;
- la constitution et les dispositions ;
- l'état latent ;
- l'horloge (temps vécu, temps dormant, sessions) et la vigilance ;
- le flux aléatoire de fonctionnement : une existence reprise continue le même
  flux ;
- la mémoire cognitive (connaissances, routage, expériences, rêves,
  corrections, influences, cache d'acquisition) ;
- les préoccupations ;
- les activités en cours ou suspendues ;
- les stimuli en attente ;
- le registre des réactions ;
- la chronologie.

### Anciens formats

Les anciens `memory.bin` ne sont **ni lus ni modifiés** : le format est
reconnu et refusé avec un message explicite. Les données expérimentales
antérieures restent donc intactes.

### Arrêt et reprise

- La **sauvegarde** a lieu toutes les 60 s de temps réel (point de contrôle
  technique) et en fin de session.
- À l'ouverture d'une session, l'écart réel depuis la dernière sauvegarde est
  mesuré.
  - En temps réel, il est appliqué comme **dormance** : relaxation
    déterministe et couplage, sans bruit, sans occurrence, sans plasticité,
    sans activité simulée.
  - En temps virtuel, il est ignoré, sauf `--dormancy`.
- Le temps vécu et le temps dormant sont comptés séparément.
- Un arrêt technique n'est jamais une décision cognitive. Il se demande par
  `stop`, par la durée de la session, ou en arrêtant le processus. Dans ce
  dernier cas, le dernier point de contrôle fait foi.

**Alternatives étudiées pour la dormance :**

- geler l'état : le temps n'aurait aucun effet, ce que le mandat exclut ;
- simuler le bruit ou les activités pendant l'arrêt : on inventerait ce qui
  n'a pas eu lieu.

La relaxation déterministe découle des lois sans rien inventer.

## 14. Observation et expérimentation

### Commandes

`<data>` vaut `./data` par défaut.

```
autonomous-ai birth <nom> [--seed n] [--dimensions d]
autonomous-ai live <nom> [--for s] [--clock real|virtual] [--schedule f.ron]
                         [--without <activité>]... [--without-influences]
                         [--dormancy s] [--checkpoint s] [--offline] [--deterministic]
autonomous-ai stimulate <nom> < événement.ron      (dépôt dans inbox/)
autonomous-ai stop <nom>
autonomous-ai episode <nom> [--neutral] [--sandbox] [options de l'ancien `run`] < événement.ron
autonomous-ai duplicate <nom> <nouveau> [--reseed n]
autonomous-ai intervene <nom> [--latent i=v]... [--bias Canal=v]... [--scale échelle=f]... [--reason texte]
autonomous-ai inspect <nom> [--section s] [--last n] [--kind k] [--csv]
autonomous-ai revert <nom> --correction id | relearn <nom> | individuals | modules
```

**Remarques :**

- `--deterministic` annule le poids des durées mesurées dans les décisions.
  Avec `--clock virtual --offline`, une existence ne dépend alors que de sa
  graine et de ses stimuli (vérifié par test).
- `episode` est le laboratoire. Il exécute un épisode **sans** cycle de vie,
  sous l'organisme ou neutre (`--neutral`), sur la mémoire de l'individu. Il
  est enregistré comme intervention, et `--sandbox` ne sauvegarde rien.
- `duplicate` copie exactement un état cognitif, sous une nouvelle identité.
  La lignée est conservée et `--reseed` fait diverger le hasard futur.
  Dupliquer sert aussi à **sauvegarder et restaurer** une expérience.
- `intervene` permet les variations contrôlées :
  - de l'état (`--latent`) ;
  - d'un biais (`--bias`) ;
  - d'une échelle (`--scale`) : les lois sont retirées de la même graine
    avec l'échelle modifiée, si bien que seule cette échelle diffère.

  Chaque intervention est inscrite dans la chronologie. Aucune n'est
  disponible pendant l'existence normale.
- Les influences peuvent être ignorées pendant une session de vie
  (`live --without-influences`) ou un épisode (`episode --without-influences`).

### Sections d'inspection

L'inspection lit le dernier état sauvegardé **sans verrou** et ne touche jamais
l'individu.

| Section | Contenu |
|---|---|
| `summary` (texte ou `--csv`) | temps vécu, dormant et endormi ; activités par type, origine et fin ; stimuli ; préoccupations ; statistiques du latent ; canaux actuels (biais, terme latent, valeur) ; plasticité ; erreurs techniques ; interventions |
| `identity`, `constitution`, `dispositions` | paramètres de naissance, lois nées, lois acquises |
| `latent` | `h`, `z` et sensations perçues |
| `channels` | décomposition actuelle de chaque canal |
| `trajectory` | CSV de l'état, échantillonné toutes les 60 s vécues |
| `timeline` | chronologie, filtrable par type (`--kind`) et longueur (`--last`). Elle contient : naissance, sessions, dormance, stimuli reçus ou négligés, activités (début avec décision décomposée, suspension, reprise, fin, tranches), vigilance, préoccupations, occurrences (avec la norme de leur impulsion), réponses, interventions, erreurs techniques |
| `activities`, `concerns`, `stimuli` | registres |
| `reactions` | pour chaque signature : nombre, impulsion moyenne et sa norme, gain actuel |
| `sequences` | quelles occurrences ont suivi lesquelles : les relations observées entre actions et réactions |
| `episodes`, `statistics`, `knowledge`, `hypotheses`, `scopes`, `routing`, `resources`, `experiences`, `dreams`, `corrections`, `influences` | mémoire cognitive, comme avant |

**Ce que l'observation distingue :**

- les **observations** (chronologie, compteurs, états) ;
- les **décompositions de mécanisme** (décisions) ;
- les **hypothèses**, qui ne sont jamais produites par l'outil.

Aucune configuration latente n'est nommée.

### Scripts

- `scripts/run-life-experiments.sh <ressources> <planning.ron> [n] [heures]`
  enchaîne les expériences suivantes :
  - **population** ;
  - **découplée** (`sensitivity_scale = 0`) ;
  - **inerte** (`impulse_scale = 0`) ;
  - **jumeau** (même naissance) ;
  - **reclus** (sans stimulus) ;
  - **sans influences** ;
  - **dormance** (30 jours) contre **continuation**.

  Le script produit un CSV par individu.
- `scripts/run-experiments.sh` contient les comparaisons de laboratoire
  A/B/C/D et M/U/R/F/N, portées sur des individus de même naissance. Ses
  épisodes sont neutres par défaut, pour rester comparables avec les
  générations précédentes. `AUTONOMOUS_AI_LATENT=1` les fait moduler par
  l'état.

## 15. Résultats expérimentaux

Les résultats ci-dessous ont été réellement obtenus avec cette version. Tous
les essais ont été faits hors ligne, en temps virtuel, en mode
`--deterministic`. Le planning de stimuli compte 9 événements sur 24 h,
résolubles hors ligne : versions, syllogismes, contradiction.

### 15.1 Population : 8 naissances × 3 conditions × 24 h virtuelles

Commande : `scripts/run-life-experiments.sh data/resources schedule-day.ron 8 24`.
Ce sont 29 existences au total.

**Validité technique.** Sur les 29 existences, il n'y a eu aucune réparation
de valeur latente et aucune erreur technique. Il a fallu entre 0,2 s et
environ 30 s de calcul par journée virtuelle, selon l'individu.

**Reproductibilité.** Le jumeau (`twin-1`, même graine, mêmes stimuli)
présente une existence identique à `population-1` sur toutes les mesures :
activités, occurrences (22 657), longueur de trajectoire (1614,07),
plasticité.

**Diversité.** Mêmes lois d'architecture, mêmes stimuli, mêmes connaissances
initiales :

| Individu | Endormi | Réflexions | Introspections | Simulations | Rêveries | Rêves | Réponses / 9 stimuli |
|---|---|---|---|---|---|---|---|
| population-1 | 9 % | 696 | 28 | 226 | 84 | 3 | 7 |
| population-2 | 7 % | 77 | 168 | 382 | 54 | 6 | 7 |
| population-3 | 32 % | 109 | 50 | 261 | 78 | 26 | 9 |
| population-4 | **97 %** | 40 | 5 | 7 | 23 | 95 | 6 |
| population-5 | 8 % | 18 | 25 | 101 | 341 | 29 | 3 |
| population-6 | 5 % | 9 | 1 | 19 | 30 | 3 | **2** (7 négligés) |
| population-7 | 42 % | 44 | 47 | 12 | 671 | 289 | 9 |
| population-8 | 14 % | 13 | 236 | **1358** | 45 | 27 | 6 |

Les comportements observés, que rien ne programme, sont les suivants :

- un individu qui dort presque tout le temps et rêve (4) ;
- un individu qui **néglige 7 sollicitations sur 9**, alors qu'il disposait
  de toutes les capacités pour y répondre (6) ;
- un individu dont les simulations dirigées vers ses préoccupations
  s'enchaînent par centaines, avec 917 préoccupations formées. C'est une
  dynamique analogue à la rumination (8) ;
- des individus surtout réflexifs (1), introspectifs (2) ou rêveurs (5, 7) ;
- des reprises d'activités suspendues : 12 suspensions et 12 reprises chez
  l'individu 1, 19 et 18 chez l'individu 8.

Ce sont des **observations sur un seul planning et 24 h** : elles ne
caractérisent pas des « types ».

**Effet causal du couplage.** Mêmes naissances avec `sensitivity_scale = 0`
(le latent n'agit sur aucun mécanisme, les biais nés restent) :

| Mesure (8 individus) | Couplés : moyenne (CV) | Découplés : moyenne (CV) |
|---|---|---|
| réponses aux 9 stimuli | 6,1 (0,39) | 6,8 (0,06) |
| réflexions | 126 (1,73) | 45 (0,56) |
| simulations | 296 (1,42) | 108 (0,71) |
| rêveries | 166 (1,29) | 66 (0,69) |
| préoccupations formées | 243 (1,13) | 116 (0,59) |

Le couplage latent → cognition multiplie la dispersion entre individus et
l'activité intérieure. Il rend aussi la réponse aux sollicitations moins
fiable. C'est une **mesure contrôlée** : seules les sensibilités diffèrent.
Le découplage supprime aussi les habitudes, dont la borne est proportionnelle
à σ_s. Les deux effets ne sont donc pas séparés.

**Effet causal des réactions.** Mêmes naissances avec `impulse_scale = 0` :
rien de ce qui arrive ne déplace l'état. L'écart type moyen de l'état latent
tombe de 0,63 à 0,22, et la longueur de trajectoire de 1210 à 360. Les
comportements restent très différents : l'individu « inerte » 6 fait 1924
réflexions et ne répond à aucun stimulus. L'état, figé là où la naissance
l'a placé, maintient durablement un même biais.

**Sans sollicitation** (`secluded-1`, duplicata du nouveau-né 1, aucun
stimulus) : la vie intérieure continue, avec 427 réflexions, 87 rêveries, 31
simulations, 49 rêves et 23 % de sommeil, contre 9 % pour l'original
stimulé.

**Sans influences** (`unswayed-1`) : 551 réflexions contre 696 et 178
simulations contre 226, pour le même nombre de réponses (7). La dynamique
étant sensible aux conditions initiales, **cet écart ne mesure pas l'effet
propre des influences**. Il montre seulement que les deux existences
divergent.

**Dormance** : duplicatas de `population-1` après 24 h. L'un subit 30 jours
d'arrêt avant de revivre 24 h, l'autre revit directement. Ils ont le même
flux aléatoire, et seule la relaxation déterministe diffère. Les résultats
divergent : 1281 réflexions contre 1601, 324 simulations contre 433. Le
temps d'arrêt a donc une conséquence sur la suite, comme prévu par
construction. L'ampleur de l'écart, amplifiée par la divergence des
trajectoires, n'est pas interprétable.

### 15.2 Laboratoire (comparabilité avec les générations précédentes)

`AUTONOMOUS_AI_STUDY=activities scripts/run-experiments.sh data/resources
<4 événements hors ligne> 2 --offline` : configurations M/U/R/F/N, épisodes
neutres, une heure virtuelle de vie entre les tours.

Les épisodes de laboratoire se comportent comme avant :

- `e1` et `h2` sont résolus ;
- `h1` est prouvé au premier tour, puis indéterminé après `h2`, qui réfute
  l'hypothèse induite « humain → mortel » (révision correcte) ;
- les phases de vie intercalées produisent réflexions, introspections et
  rêves.

Sur ce petit ensemble, aucune différence de résolution attribuable aux
activités n'est démontrée.

### 15.3 Temps réel, en ligne

Un individu (graine 7) a vécu en temps réel, avec les modules en ligne (sans
jeton GitHub). Deux stimuli lui ont été déposés par `stimulate` pendant qu'il
vivait :

- `p1` : dépendances serde, semver ;
- `h1` : syllogisme.

**Ce qui s'est passé :**

- les deux stimuli ont été reçus et l'individu a agi sur chacun ;
- **il a abandonné les deux actions** (9 tranches au total) **sans
  répondre**. Ces abandons ont formé 19 préoccupations « inachevées » et
  « impasse », dont une a été résolue par la suite ;
- en dehors des tranches, le processus attend sans consommer de CPU (relevé
  de la boîte chaque seconde) ;
- `stop` a provoqué un arrêt technique propre, inscrit comme intervention ;
- à la reprise, l'écart réel de 9,4 s a été mesuré puis appliqué comme
  dormance (`session_opened { wall_gap: 9.384, dormancy_applied: 9.384 }`).

Cet individu, capable de répondre, ne l'a pas fait : c'est une issue
permise, et observée.

## 16. Limites connues

- **Les tranches repartent de la mémoire consolidée.** Une action reprise ou
  continuée ne retrouve pas sa mémoire de travail : les déductions et
  observations consolidées sont retrouvées, pas les besoins ouverts ni le
  plan en cours. Un individu à tranches courtes résout donc moins bien les
  problèmes à plusieurs étapes. La borne de 0,25 sur `SliceLength` limite cet
  effet sans le supprimer.
- **Concurrence entrelacée, pas simultanée.** En temps réel, un épisode long
  retarde la prise en compte des stimuli jusqu'à la fin de la tranche.
- **Taux supposés constants** pendant au plus 60 s, ou pendant une tranche
  (approximation de la compétition exponentielle).
- **Constantes choisies** (ordres de grandeur, non mesurés) :
  - centres et lois de naissance ;
  - saillances par origine ;
  - exposant d'attention 0,5 ;
  - 12 invocations par tranche ;
  - 2 inconscients au plus ;
  - 64 tranches au plus ;
  - seuils des sensations ±0,3 ;
  - 2 s virtuelles par invocation ;
  - bornes des préoccupations, de la chronologie et de la plasticité.

  Les modifier change la fréquence des phénomènes, pas leur possibilité.
- **Reproductibilité exacte** : elle exige le temps virtuel, le mode hors
  ligne et `--deterministic`. Même alors, l'actualité des connaissances
  dépend de l'horloge murale. C'est négligeable en temps virtuel, mais pas
  démontré nul en général. Le test de jumeaux porte sur 6 h et un petit
  registre.
- **Introspection grossière** : 6 sensations à trois niveaux. Ce que
  l'individu apprend de lui-même passe par les modules existants (induction,
  examen), sans mécanisme dédié de régulation.
- **Délibération rudimentaire.** Le seul acte volontaire est la reprise
  d'une préoccupation après introspection. Le conscient ne choisit pas encore
  de dormir, de s'arrêter, ni d'agir sur une sollicitation.
- **Attribution causale.** Elle est exacte au niveau des mécanismes
  (décisions). Au niveau du comportement, elle n'est établie que par
  expériences contrôlées, et sur peu d'individus et de scénarios.
- **Pas de migration des anciennes mémoires** `memory.bin`.
- **Temps virtuel et environnement réel.** En ligne, le temps virtuel ne
  reflète pas la latence réelle des API.

## 17. Hypothèses non démontrées

- Que la diversité observée entre individus reste stable sur des existences
  longues (semaines), au lieu de converger sous l'effet de la plasticité.
- Que l'introspection permette effectivement à un individu d'apprendre des
  régularités sur ses propres manifestations.
- Que les habitudes formées par plasticité soient discernables des effets
  des biais nés sur des durées plus longues.
- Qu'un régime chaotique de la dynamique latente (g > 1) se traduise par des
  comportements discernables d'un bruit accru.
