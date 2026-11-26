# Environnements partagés et interactions autonomes

Ce document décrit l'évolution qui permet à plusieurs existences de vivre dans
un environnement commun, de le percevoir et d'y transmettre des informations,
sans jamais y être obligées. Il complète `living-architecture.md` (génération
« existence »), qui n'est pas modifié.

## 1. Audit : ce qui existait et ce qui a été réutilisé

| Besoin | Mécanisme existant réutilisé |
|---|---|
| perception d'événements | stimuli de la vie (`Life::receive`) : saillance individuelle, attention, négligence possible |
| réponse | `outbox/`, réservée désormais aux sollicitations extérieures |
| capacités | modules cognitifs (`CognitiveModule`, descripteurs, signatures fournies) choisis par l'échantillonnage de Thompson appris |
| découverte de capacités | modèle des capacités (`CapabilityModel`), imagination (mécanisme `self_representation`, anticipation de questions) |
| besoins, préoccupations, intentions | besoins des épisodes, impasses, préoccupations, délibération |
| imagination | scènes isolées en mode `Replay` : aucun effet réel |
| conscient / inconscient | activités, influences, vigilance |
| temporalité | temps vécu réel ou virtuel par existence |
| persistance | `existence.bin` atomique, verrou exclusif, inspection sans verrou |
| accès extérieur | `ExternalAccess`, compteur de ressources, quotas, mode hors ligne |
| provenance et révision | `Provenance`, révision des croyances, consolidation, corrections |

Aucun système parallèle n'a été créé : percevoir et transmettre sont des
**modules cognitifs ordinaires**, mobilisés seulement quand un besoin les
appelle ; recevoir est un **stimulus** comme un autre ; ce qui est reçu passe
par la **consolidation** existante, sous un nouveau statut épistémique.

## 2. Architecture

```
  data/environments/<env>/environment.bin          data/individuals/<nom>/existence.bin
  ┌─────────────────────────────────┐            ┌──────────────────────────────┐
  │ Environment                     │            │ Existence (inchangée)        │
  │  membres : poignée ↔ existence  │◄─Portal───►│  Life (sessions, fenêtres)   │
  │  transmissions datées           │            │  Intelligence + 2 capacités  │
  │  latence, perceptibilité,       │            │   environment_observation    │
  │  accès externe                  │            │   communication              │
  └─────────────────────────────────┘            └──────────────────────────────┘
                     ▲ Cohabitation : ordonnancement du temps commun (rien d'autre)
```

| Module | Rôle |
|---|---|
| `world/environment.rs` | `Environment` : membres, transmissions, latence, perceptibilité, accès externe. Il ne décide de rien |
| `world/member.rs` | `Member` : poignée opérationnelle (adresse opaque) ↔ répertoire de l'existence |
| `world/message.rs`, `transmission.rs`, `transmission_status.rs` | messages (énoncés, relais avec chaîne de sources, questions, textes), transmissions datées (émise, recevable à partir de, statut) |
| `world/portal.rs`, `portal_state.rs` | `Portal` : accès d'une existence à son environnement (temps commun, perception, émission, réception) |
| `world/cohabitation.rs` | `Cohabitation` : ordonnancement du temps commun |
| `world/environment_store.rs` | persistance de l'environnement (`AAIE` v2, migration de la v1) |
| `world/format_one/` | lecture du format 1 des environnements, pour migration uniquement |
| `living/period_series.rs` | résumés par périodes successives |
| `world/environment_report.rs` | synthèse observable de l'environnement |
| `capabilities/environment_observation.rs` | capacité `present/1` |
| `capabilities/communication.rs` | capacité `transmitted/2` |
| `living/session.rs` | session de vie reprenable par fenêtres |
| `living/period_report.rs` | rapport d'une existence sur une période |

### Ce que l'environnement n'est pas

- **Pas un orchestrateur** : il ne déclenche aucune activité, aucune
  perception, aucun message. Il contient des présences, transporte ce qui est
  émis et répond à ce qu'on lui demande.
- **Pas un espace** : aucun modèle spatial n'a été introduit, faute de
  nécessité. La « proximité » se réduit à la présence dans le même
  environnement, plus une perceptibilité.
- **Pas une mémoire partagée** : chaque existence garde ses fichiers, sa
  mémoire, son état latent et ses dispositions. Rien n'est copié de l'une à
  l'autre en dehors des messages qu'elles émettent elles-mêmes.

## 3. Existence objective, perceptibilité, perception, représentation, connaissance

| Niveau | Où il se trouve |
|---|---|
| **existence objective** | la liste des membres de l'environnement |
| **perceptibilité** | `perceptibility` ∈ [0, 1]. À chaque observation, chaque présence est perceptible avec cette probabilité, par un tirage reproductible (graine, observateur, observé, instant) |
| **perception effective** | le module `environment_observation` a été mobilisé, en réalité, et a rapporté `present(<poignée>)`. Les observations sont journalisées dans l'épisode |
| **représentation cognitive** | le nœud `present(<poignée>)` dans la mémoire de travail, puis tout ce que le raisonnement en tire |
| **connaissances acquises** | la consolidation de cette observation en mémoire, de provenance `environment` |

Aucune connaissance sur les voisins n'est injectée. Une existence ne sait même
pas qu'elle a ces capacités, sauf par les mécanismes qui le lui font
découvrir : le modèle de ses capacités, et sa représentation de soi en
imagination.

### Comment une observation peut survenir sans être imposée

- **En imagination**, les signatures que l'existence pourrait obtenir
  (`present/1`, `transmitted/2`, comme toutes les autres) font désormais
  partie du matériau des questions anticipées. Une rêverie peut donc
  « se demander » `present(?q0)`.

  Rien n'y est perçu, puisque l'imagination est isolée. Le besoin y reste
  ouvert, et devient une préoccupation d'origine `imagination`, comme tout
  besoin apparu en imagination.
- Une **préoccupation consciente** peut désormais être reprise **en réalité** :
  - soit par un taux de passage à l'action, porté par le canal `Attend` et la
    part de saillance de la préoccupation ;
  - soit par délibération après une introspection. Le choix entre simulation
    et action se fait en proportion des taux courants de simuler et d'agir.

  L'action est un épisode ordinaire dont la question est le sujet de la
  préoccupation. Si ce sujet est `present(?x)`, le module d'observation peut
  y répondre.

Ce chemin est générique : il vaut pour n'importe quelle préoccupation et
n'importe quelle capacité. Il est long, et peut ne jamais être parcouru.

## 4. Communication

Le module `communication` fournit `transmitted(<destination>, ?message)`. Il
n'est mobilisé que si un besoin de ce type apparaît, par le même chemin que
ci-dessus, ou par une question reçue.

### Destination

C'est celle que nomme le besoin, ou, si elle est laissée ouverte, la première
présence connue dans le raisonnement. L'existence ne peut donc s'adresser
qu'à ce qu'elle a perçu ou appris. Une destination absente donne une
transmission `undeliverable`, enregistrée, et le besoin reste ouvert.

### Contenu

Le contenu est composé de ce que la mémoire de travail contient :

- jusqu'à 4 **questions** : ses autres besoins ouverts ;
- jusqu'à 16 **énoncés** : ceux qui portent sur les prédicats de ces
  questions, sinon les plus récents ;
- jusqu'à 8 **énoncés relayés**, qu'elle avait elle-même reçus, avec leur
  source ;
- jusqu'à 4 **règles** qu'elle tient ou conjecture. Sont exclues ses
  ressources initiales, que tout individu possède, les injections de
  l'inconscient et les contenus imaginés. Le destinataire les reçoit comme
  **hypothèses** : proposées, éprouvées, réfutables, jamais établies ;
- jusqu'à 2 **textes**.

Un message peut être vide, sans question, sans attente de réponse.

Ces bornes sont choisies, et non établies. Le module ne lit rien d'autre que
le raisonnement en cours : ni fichier, ni autre partie de la mémoire, ni
**rien de ce que l'existence perçoit d'elle-même** (provenance
`introspection`).

### En imagination

Le module ne transmet rien. Le besoin reste ouvert et apparaît dans les rêves
comme une **interaction envisagée** (observable dans la section `dreams`). Un
test vérifie qu'un épisode en mode `Replay` ne crée aucune transmission.

### Réception

À partir de `émission + latence`, le message est remis au destinataire comme
un **stimulus** :

- de source `individual:<poignée>` ;
- avec une saillance qui dépend de son état ;
- il peut être négligé.

Les énoncés relayés arrivent comme un stimulus séparé, de source
`individual:<poignée> relaying <chaîne>`. La provenance des transmissions
successives est ainsi préservée.

### Absence de boucle automatique

- Une action qui répond aux questions d'un message n'envoie rien en retour.
- L'`outbox` est réservée aux sollicitations extérieures (humaines ou
  système).
- Répondre suppose un besoin de transmettre né de l'existence elle-même.

### Ce qui est reçu n'est pas une vérité (statut `Testimony`)

- Les énoncés d'un message entrent comme **témoignage** (`EpistemicKind::Testimony`) :
  - ils restent révisables (`is_defeasible`) ;
  - ils sont retirés si une observation les contredit ;
  - une déduction qui en dépend n'est **jamais établie**.
- La consolidation les enregistre avec le statut `Standing::Reported` : ce
  qui a été dit, avec sa provenance. Un énoncé déjà établi garde son statut ;
  le témoignage s'ajoute seulement à ses justifications.
- La remémoration les rappelle comme témoignage, pas comme souvenir de fait.
- Ce qui est affirmé, ce qui est compris et ce qui est établi restent donc
  distincts :

| Niveau | Où il se trouve |
|---|---|
| affirmé | le contenu de la transmission, dans l'environnement |
| compris | les nœuds et déductions de l'épisode du destinataire, dans ses expériences |
| établi | l'observation ou la preuve, dans la mémoire du destinataire |

- Les mémoires ne sont jamais fusionnées.

Les stimuli humains restent des observations, comme avant. Les traiter aussi
en témoignages changerait le passé expérimental des existences : c'est une
question ouverte.

## 5. Temporalité et ordonnancement

### Temps virtuel : fenêtres de latence

L'environnement a un temps commun. Le temps de l'environnement vaut, pour
chaque existence, son temps vécu moins un décalage fixé à l'ouverture de sa
session. Chaque existence garde ainsi son propre temps vécu.

Les existences avancent **à tour de rôle par fenêtres égales à la latence**
(60 s par défaut, au moins 1 s). Une transmission émise pendant une fenêtre
est datée de son émission et recevable à partir de `émission + latence`,
c'est-à-dire jamais avant la fenêtre suivante. Il en découle trois
garanties :

- **causalité** : rien n'est reçu avant d'avoir été émis ;
- **indépendance de l'ordre** : au sein d'une fenêtre, aucune existence ne
  peut affecter une autre, donc l'ordre de passage ne change rien ;
- **reproductibilité** : une cohabitation virtuelle déterministe est
  reproductible (test `a_shared_existence_in_virtual_time_is_reproducible`).

**Pas de synchronisation des activités.** Chaque existence suit sa propre
vie : l'une peut dormir pendant que l'autre réfléchit.

**Attente efficace.** Une existence au repos saute directement jusqu'au
prochain changement possible, à la fin de la fenêtre ou à l'arrivée du
prochain message connu. Elle ne sonde jamais à vide. 90 jours virtuels pour
deux existences de 105 jours ont pris 1,2 s.

**Conséquence expérimentale.** La latence borne la réactivité. Un message ne
peut susciter de réaction qu'après au moins une latence. Une latence plus
courte rend l'environnement plus réactif et l'ordonnancement plus coûteux.

### Temps réel : un fil par existence

Chaque existence vit dans son propre fil, à son rythme. L'environnement est
partagé sous verrou. Les messages sont relevés chaque seconde au plus tard.
Le temps de l'environnement suit le temps vécu, donc le temps réel.

### Sources de non-déterminisme restantes

Même en temps virtuel, `--deterministic`, hors ligne, il en reste trois :

- les identifiants (UUID v7) et les horodatages muraux ne pèsent sur aucune
  décision, sauf l'actualité des connaissances, calculée sur l'horloge
  murale. Elle reste négligeable en temps virtuel, sans être démontrée nulle ;
- en temps réel, l'ordre d'accès au verrou entre fils et les durées réelles
  font varier les existences ;
- en ligne, les réponses du réseau varient.

## 6. Identité historique et identité opérationnelle

L'**identité historique** est celle qui figure dans `existence.bin`.
Les six existences présentes ont **le même identifiant historique** : elles
ont été copiées par le système de fichiers.

Dans un environnement, une existence est désignée par une **poignée** (par
exemple `eab64783c`). Cette poignée :

- est dérivée de la graine de l'environnement et du rang d'admission ;
- n'est jamais un nom ;
- n'écrit rien dans l'existence.

Deux copies d'une même existence peuvent donc cohabiter sans collision.

`branch <nom> <nouveau>` crée une **branche exacte** :

- même fichier à l'octet près ;
- même identité ;
- même histoire ;
- même état aléatoire ;
- un fichier `branch.ron` décrivant l'origine, à côté.

`duplicate` reste disponible quand on veut au contraire une nouvelle identité,
avec sa lignée.

## 7. Persistance et migration

### Formats

| Fichier | Format |
|---|---|
| `existence.bin` | `AAIX` v1, inchangé |
| `environment.bin` | `AAIE` v2. Contient membres, transmissions (en attente, remises, impossibles), horloge commune, paramètres |

### Migration des environnements (v1 → v2)

Le format 2 ajoute les règles aux messages.

- Un fichier v1 est **lu** par des structures dédiées
  (`world/format_one/`), puis converti en mémoire : chaque champ est
  conservé, et les messages reçoivent une liste de règles vide.
- Une lecture seule (`environment inspect`) n'écrit rien.
- Avant qu'un fichier v1 ne soit réécrit (au premier chargement en vue de
  modification), l'original est copié à l'identique dans
  `environment.bin.v1`, qui n'est plus jamais touché.

Le test `migrates_format_one_without_touching_the_original` vérifie trois
choses : un fichier v1 construit champ par champ est intégralement relu, la
copie est identique à l'octet près, et elle survit à la sauvegarde en v2. Les
15 environnements présents sur disque, tous au format 1, ont été relus avec
le code final ; leurs fichiers sont inchangés.

### Pourquoi aucune migration des existences

Les ajouts au format des existences sont tous des **variantes ajoutées en fin
d'énumération** :

- `EpistemicKind::Testimony` ;
- `Standing::Reported` ;
- `ActivityEnding::Exhausted` ;
- les nouvelles variantes de `LifeEvent`.

Bincode encode une variante par son rang, si bien que les fichiers existants
se lisent sans changement.

### Vérification

Elle a été refaite avec le code final sur toutes les existences présentes
(les six originales et les branches d'expérience) et sur l'archive : toutes
sont identiques.
`verify <nom>` décode le fichier, le réencode et compare les octets :

```
…/individual-20261126-155111/existence.bin                         : identique (3 385 909 octets)
…/individual-20261126-155111-control-20261126-161603               : identique (4 989 868 octets)
…/individual-20261126-155111-control-day8                          : identique (4 989 893 octets)
…/individual-20261126-155111-control-long-20261126-162032          : identique (22 839 477 octets)
…/individual-20261126-155111-stimulated-20261126-161603            : identique (5 167 065 octets)
…/individual-20261126-155111-stimulated-long-20261126-162032       : identique (23 144 569 octets)
```

### Données d'origine

- Avant tout changement, une archive en lecture seule a été faite dans
  `data/archives/individuals-before-environment-20261126/`, avec ses sommes
  SHA-256 (`data/archives/…sha256`).
- Les expériences travaillent sur des branches. Les sommes des deux
  existences de 105 jours ont été revérifiées après chaque expérience : elles
  sont inchangées.

### Rétention

La chronologie était bornée à 200 000 entrées, et A et B en comptaient déjà
environ 159 000. Au-delà, leurs débuts, dont les 168 stimulations, auraient
été effacés. La borne a été relevée à 50 millions, ce qui la réduit à une
protection contre l'emballement. **Les rapports sélectionnent des périodes au
lieu de tronquer l'histoire.**

### Arrêt et reprise

- Les messages en attente sont dans `environment.bin` et survivent à un arrêt
  (test).
- Une existence ignore une transmission qu'elle a déjà reçue : elle ne la
  reçoit qu'une fois, même si l'environnement a été sauvegardé avant elle.
- Aux points de contrôle, les existences sont sauvegardées avant
  l'environnement. Un arrêt brutal entre les deux peut **perdre** les
  émissions de la dernière période (l'émetteur croit avoir transmis), mais
  jamais les dupliquer. Ce risque résiduel est documenté.
- Une existence se lit, se sauvegarde et se déplace sans son environnement.

## 8. Accès extérieur et isolements

Les situations suivantes sont distinguées :

| Situation | Configuration |
|---|---|
| isolement social | seule dans son environnement. Elle a les capacités d'observer et de transmettre, mais personne à percevoir |
| absence de stimulation humaine | aucun dépôt dans `inbox/`, aucun planning |
| absence d'accès aux autres individus | environnement d'un seul membre, ou perceptibilité nulle |
| absence d'accès aux ressources externes | environnement sans `--external` : catalogue hors ligne |
| isolation technique complète | hors de tout environnement, hors ligne. Elle n'a même pas les capacités d'observer ou de transmettre |

Un accès disponible n'implique pas qu'il soit connu ou exploité. Aucune
recherche périodique ni mission de découverte n'a été introduite.

## 9. Sécurité

- Les permissions (accès externe) appartiennent à la configuration de
  l'environnement. Aucune information reçue ne peut la modifier : les messages
  ne sont que des énoncés et des textes, perçus comme témoignages.
- Les quotas, budgets et compteurs de ressources existants s'appliquent
  inchangés.
- Le module de communication ne lit que la mémoire de travail de l'épisode.
  Il n'accède ni aux fichiers, ni aux secrets, ni à la mémoire persistante en
  dehors de ce que le raisonnement s'est déjà remémoré, ni aux perceptions de
  soi.

## 10. Influence des interactions

Percevoir, émettre et recevoir produisent des occurrences ordinaires :

- `invoke:environment_observation`, `produce:communication` ;
- `stimulus:individual:<poignée>`, qui porte une signature distincte par
  émetteur ;
- etc.

Ces occurrences passent par les lois de réaction individuelles, la
plasticité et les dispositions. Aucune réaction n'est universellement
positive ou négative, et aucune disposition n'a été retouchée.

## 11. Changements de comportement pour toutes les existences

Ces changements sont nécessaires et appliqués à tous de la même façon.

1. Une action **sans question** s'achève une fois l'événement perçu et
   raisonné. Avant, elle continuait jusqu'à l'abandon : les 22 abandons de A
   sur ses stimulations en langage naturel venaient de là.
2. Une préoccupation consciente peut être reprise **en action réelle** (§3).
3. L'imagination peut anticiper des questions sur **ce que l'existence pourrait
   obtenir**, c'est-à-dire les signatures de ses capacités.
4. Les canaux et la plasticité sont échantillonnés toutes les 10 minutes vécues
   (`LifeEvent::Plasticity`), pour permettre les rapports d'évolution.
5. La propension à **reprendre** une activité suspendue sature en
   `ln(1 + Σ maintiens)`, comme celle de simuler. Elle croissait auparavant
   linéairement avec le nombre d'activités suspendues : un nouveau-né a
   enchaîné 43 000 suspensions et reprises en 7 jours, un effet de la forme
   de la loi et non de l'individu.
6. Une action dont la tranche se termine en **quiescence** (aucune capacité
   n'a plus rien à faire) s'achève avec la fin `exhausted`. Elle ne répète
   plus la même tranche jusqu'à la borne technique : 452 fins `bounded` en
   7 jours chez ce même nouveau-né.

## 12. Observabilité et rapports

### Existence

`inspect <nom> --section report --from <s> --to <s>` donne, pour une période
du temps vécu :

- les compteurs d'événements ;
- les activités commencées (par origine) et terminées (par fin) ;
- les stimuli reçus, relayés et négligés ;
- les transmissions par destination et statut ;
- les préoccupations par origine ;
- les occurrences les plus fréquentes ;
- la longueur de trajectoire latente ;
- la plasticité et les canaux entre le premier et le dernier échantillon de
  la période ;
- les connaissances apparues dans la période, par statut, dont les
  rapportées ;
- les perceptions effectives d'autrui, les messages reçus d'autres individus
  et les scènes imaginées où percevoir ou transmettre était en question
  (interactions envisagées).

`inspect <nom> --section periods --every <s> [--from] [--to]` donne une ligne
CSV par période successive, en un seul passage sur la chronologie :

- activités commencées, abandonnées et interrompues ;
- stimuli reçus et négligés ;
- transmissions et messages reçus d'individus ;
- préoccupations ;
- occurrences ;
- changements de vigilance ;
- trajectoire latente.

C'est ce qui permet de suivre une évolution sans réimprimer l'histoire.

`inspect <nom> --section summary [--csv]` ajoute les cumuls d'environnement :
perceptions, transmissions, messages reçus et actions épuisées.

`--from` et `--to` filtrent aussi la chronologie, avec `--kind` et `--last`
pour des extractions ciblées. L'intégralité des données reste accessible par
les sections détaillées.

### Environnement

`environment inspect <env> [--from] [--to]` donne :

- les membres (présence effective) ;
- les transmissions émises, remises, en attente ou impossibles, par paire ;
- les transmissions adressées à un individu dont un message avait déjà été
  remis à l'émetteur. Ce sont des réponses possibles : l'ordre dans le temps
  est observé, le fait qu'elles répondent n'est pas établi ;
- le contenu (énoncés, règles, questions, relais, textes).

La section `transmissions` liste le détail.

### Ce qu'on peut distinguer

| À observer | Source |
|---|---|
| présence effective | membres de l'environnement |
| découvertes et perceptions effectives | `present(…)` consolidés, épisodes d'observation |
| interactions envisagées | scènes imaginées dont une question ou un besoin porte sur `transmitted`/`present` (rapport de période) |
| interactions engagées | `LifeEvent::Transmitted`, transmissions de l'environnement |
| informations reçues | stimuli de source `individual:` |
| intégration | entrées `Reported` |
| conséquences | occurrences, plasticité, canaux |

Aucune métrique psychologique n'est calculée.

## 13. Commandes exactes des expériences

Depuis `product/`, les données étant dans `data/` :

```sh
# Vérifier une existence (lecture + réencodage identique)
target/…/autonomous-ai verify <nom>

# Expérience principale, hors ligne, 90 jours virtuels (A = stimulated-long, B = control-long)
scripts/run-environment-experiment.sh data \
  individual-20261126-155111-stimulated-long-20261126-162032 \
  individual-20261126-155111-control-long-20261126-162032 \
  coexistence-3 90 > data/experiments/coexistence-3-offline-90d.txt

# Expérience distincte avec accès externe, 30 jours virtuels
scripts/run-environment-experiment.sh data <A> <B> coexistence-4-external 30 --external \
  > data/experiments/coexistence-4-external-30d.txt

# Exploration : 3 paires de nouveau-nés (graines 101 à 106), 7 jours, témoins isolés
scripts/run-environment-population.sh data <étiquette> 3 7 101

# Inspection
autonomous-ai environment inspect coexistence-3-shared
autonomous-ai inspect coexistence-3-shared-a --section report --from 9072000
autonomous-ai inspect coexistence-3-shared-a --section periods --every 86400 --from 9072000
autonomous-ai inspect coexistence-3-shared-a --section trajectory | sha256sum   # comparer isolé et partagé
```

Les noms et les durées de l'expérience ne figurent que dans les commandes,
jamais dans le code de production.

## 14. Résultats réellement obtenus

### 14.1 Validations techniques

Ce sont des tests contrôlés : un échange y est **provoqué**, il n'est pas
spontané.

- **Adressage et latence** : la transmission va de la bonne poignée à la
  bonne poignée, elle est en attente, recevable exactement à
  `émission + 60 s`, et remise une seule fois.
- **Imagination** : un épisode en mode `Replay` portant la même question ne
  crée **aucune** transmission.
- **Persistance** : la transmission en attente survit à la sauvegarde et au
  rechargement de l'environnement.
- **Réception** : rien n'est reçu une seconde avant l'échéance. Après, le
  message arrive comme stimulus, l'existence (rendue attentive par
  l'expérimentateur, dans le test uniquement) agit dessus, et l'énoncé reçu
  est consolidé avec le statut **`Reported`**, pas `Observed`.
- **Reproductibilité** : deux cohabitations virtuelles identiques donnent des
  chronologies identiques.

### 14.2 Expérience principale

C'est l'expérience du §19 du mandat : quatre branches des existences de
105 jours, pendant 90 jours virtuels, hors ligne. La copie de A et la copie
de B qui vivent ensemble disposent de toutes les possibilités d'interaction.

L'étiquette est `coexistence-3`, sortie dans
`data/experiments/coexistence-3-offline-90d.txt`. L'expérience a été relancée
après les corrections 5 et 6 du §11. Le premier passage (`coexistence-1`,
code du commit `7c87547`) est conservé dans `data/experiments/superseded/` :
il donne exactement les mêmes empreintes de trajectoire.

- **Aucune perception d'autrui, aucune transmission, aucun message**, dans
  aucun groupe.
- Les deux existences sont restées presque inactives sur 90 jours, endormies
  la plupart du temps :
  - A : 4 introspections, 3 réflexions, 1 rêve ;
  - B : 3 introspections, 2 réflexions.

  Leurs canaux d'activité étaient très bas au départ (par exemple `Attend`
  vers 0,0005 par heure, `Dream` vers 0,0005 par heure).
- **Les copies isolée et partagée sont identiques au bit près** :
  trajectoire latente et suite des activités, pour A comme pour B.

  Faute d'avoir jamais perçu l'autre, la coexistence n'a eu **aucun effet
  causal**. C'est un résultat valide, et un témoin net : il montre que la
  seule présence d'autrui ne produit rien par construction.
- Leur plasticité était **à ses bornes** avant l'expérience : variation du
  couplage 4,2287 et du point de repos 10,2451, identiques chez A et B. Les
  deux sont nés de la même graine, et leurs lois acquises ont saturé au cours
  des 105 jours.

### 14.3 Accès externe autorisé

C'est une expérience distincte, sur des branches des mêmes originaux, pendant
30 jours virtuels (`coexistence-4-external`).

- Aucune requête, aucune transmission.
- Une activité pour A, une pour B.
- Les copies isolée et partagée sont encore identiques.

### 14.4 Exploration sur des nouveau-nés (interrompue)

**Premier passage** (`exploration-1`, avant les corrections 5 et 6 du §11).
Il a été arrêté après la première paire, et ses données sont conservées dans
`data/experiments/aborted/`. Les deux nouveau-nés, très actifs, ont :

- entrepris 776 et 1093 actions à partir de leurs préoccupations ;
- rêvé de questions de présence ou de transmission (44 et 285 mentions dans
  leurs rêves) ;
- mais n'ont **perçu personne et rien transmis** en 7 jours.

Ce passage a surtout révélé les deux défauts structurels corrigés au §11.

**Second passage** (`exploration-2`, code final). Il a été **interrompu à la
demande des expérimentateurs**, pendant la troisième paire. Ses existences et
environnements partiels sont conservés dans `data/`. Aucun résultat n'en est
tiré ici : l'expérience sera menée par les expérimentateurs.

### 14.5 Temps réel

Deux nouveau-nés ont cohabité 20 s réelles, avec un fil par existence. Le
CPU était quasi nul pendant l'attente, les sauvegardes et l'horloge commune
(20 s) sont correctes, et rien ne s'est passé en si peu de temps. Ce n'est
pas une validation de l'échange en temps réel, qui n'a été validé qu'en temps
virtuel.

### 14.6 Arrêt des expérimentations

À la demande des expérimentateurs, toutes les simulations ont été arrêtées.
Les fonctionnalités ajoutées ensuite ont été validées par les tests unitaires
et par des lectures seules des données existantes, jamais par de nouvelles
simulations. Ce sont la transmission de règles, la migration v1 → v2, les
rapports `periods`, les interactions envisagées, les perceptions et les
réponses possibles.

## 15. Limites et problèmes non résolus

- **Chaîne d'initiative longue.** Pour communiquer, une existence doit :
  1. anticiper en imagination une question de présence ou de transmission ;
  2. en faire une préoccupation, la rendre consciente, la reprendre en
     action ;
  3. percevoir quelqu'un ;
  4. faire naître un besoin de transmettre à cette adresse.

  Ce chemin est générique et libre, mais rare. Des existences très peu
  actives, comme A et B, ne le parcourent pas en 90 jours.
- **Contenu des messages.** Il est composé mécaniquement de la mémoire de
  travail : ni langue, ni intention de dire quelque chose de particulier.
  Les textes ne sont transmis que s'ils sont dans la mémoire de travail.
- **Pas de réponse spécifique.** Une question reçue est traitée comme une
  question à résoudre, et la réponse n'est envoyée que si un besoin de
  transmettre apparaît ensuite.
- **Perte possible des émissions** de la dernière période en cas d'arrêt
  brutal (§7).
- **Temps réel** : la cohabitation par fils a été vérifiée sur deux
  nouveau-nés pendant 20 s réelles : CPU quasi nul pendant l'attente,
  sauvegardes correctes, horloge commune à 20 s. Il ne s'est rien passé en
  20 s, ce qui n'est pas une validation de l'échange en temps réel.
- **Témoignages humains** : ils restent des observations (§4). La question
  est ouverte.
- **Perceptibilité uniforme.** Elle ne dépend ni de l'état ni de l'activité de
  l'observé.
