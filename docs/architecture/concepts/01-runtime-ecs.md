# 01 — Runtime ECS

## Statut
**Normatif — v1.0.** Date de rédaction : 31 juillet 2026. Ce document ne décrit pas un état du projet ; il définit une norme architecturale. Les cinq sections de ce document forment un seul modèle d'exécution cohérent, élaboré et convergé collectivement (développeur, Claude, GPT, Gemini). Elles ne doivent pas être lues indépendamment : le Modèle d'exécution présuppose les Flux de données, qui présupposent le Cycle de vie du World, qui présuppose l'Identité des entités.

Toute décision ultérieure (renderer, audio, IA, navigation, sauvegardes, réseau, UI...) doit être compatible avec ce vocabulaire et ces invariants, sans les redéfinir.

---

## 1. Identité des entités

- Une entité est **un identifiant opaque sans sémantique métier**. Toute la sémantique du jeu est portée exclusivement par les composants. `EntityId` ne connaît ni catégorie métier (`Player`, `Enemy`, `Chest`...), ni comportement.
- `EntityId` est un type opaque (`pub struct EntityId(u64)`, accesseurs privés). Le découpage interne (index/génération, largeur des champs) est une décision d'implémentation, non architecturale — réversible sans impact sur l'API.
- La validité d'un handle repose sur la comparaison `(index, génération)` : `is_alive(id) == (generation_table[id.index] == id.generation)`, en O(1).
- La politique de réutilisation des index libérés (FIFO, LIFO, ou autre) est une décision d'implémentation, mesurable et arbitrable par micro-benchmark, sans impact sur l'architecture.
- La composabilité par **capacité** (`InputControlled`, `CameraTarget`, `LocalPlayer`) est préférée à la composabilité par **catégorie** (`PlayerTag`). Quand une notion est mutuellement exclusive par nature (faction, équipe, biome...), un composant de valeur (`enum Faction`) est préférable à une multiplication de tags.

## 2. Cycle de vie du World

- **Seul le `World` crée et détruit les entités.** Aucune autre structure ne fabrique d'`EntityId`. Les systèmes ne créent jamais d'identifiants eux-mêmes ; ils émettent des commandes (`Spawn(...)`, `Destroy(entity)`, etc.) interprétées par le `World`.
- **Toute modification structurelle** (spawn, despawn, ajout de composant, retrait de composant) passe par un mécanisme de commandes différées — jamais par mutation directe pendant l'itération d'un système.
- **Les systèmes ne voient qu'un monde structurellement stable pendant leur exécution.** Les commandes structurelles sont appliquées à des points de synchronisation déterministes (cf. section 5 — la phase est l'unité de validation).
- **Une entité référencée peut avoir disparu.** La validité d'une référence n'est jamais supposée ; elle se vérifie systématiquement via `(index, génération)`.

## 3. Relations entre entités

- Une relation entre entités est toujours représentée par un `EntityId` porté comme donnée (ex: `source: EntityId`, `target: EntityId`), jamais comme une relation de propriété implicite du framework.
- **Une relation n'implique jamais une propriété de cycle de vie par défaut.** Une référence (`source`, `target`, `observes`...) n'entraîne aucune cascade automatique.
- Trois familles de relations sont reconnues, mais **ne sont pas traitées de manière universelle** — chacune peut ou non impliquer une politique de cycle de vie, et cette politique est toujours explicite, jamais déduite de la nature de la relation :
  - Relation forte (ex: `AttachedTo`) — peut impliquer une cascade, si une politique le décide explicitement.
  - Relation faible (ex: `Source`, `Target`, `Observes`) — n'implique jamais de cascade.
  - Composition spatiale (hiérarchie de transformation) — **hors périmètre de cette section**, traitée par le Modèle d'exécution (section 5) car elle impose une contrainte d'ordre de traversée, pas seulement une relation de données.
- **Le moteur (`World`) ne définit aucune sémantique implicite des relations entre entités.** Toute politique de cycle de vie entre entités (cascade, survie, conditionnalité) est exprimée explicitement par des systèmes métier utilisant les primitives du `World` (`spawn`, `despawn`, `add_component`, `remove_component`).

  > **Le World reste une infrastructure ignorante de toute sémantique métier.**
  >
  > Cette phrase résume à elle seule la raison pour laquelle les relations n'ont pas de sémantique intrinsèque, pourquoi le `World` ne connaît aucune catégorie de gameplay, et pourquoi toute politique (cascade, faction, capacité...) est portée par les systèmes et jamais par le cœur du moteur.
- Une politique de cycle de vie n'est pas nécessairement une fermeture transitive universelle : **chaque système qui implémente une politique de cycle de vie est responsable de produire un ensemble cohérent de commandes avant le point de validation** de la phase qui le concerne. La cascade est un cas particulier de cette responsabilité, pas un mécanisme central du moteur.
- Garde anti-cycle : la détection de graphe cyclique dans une politique de relation est un mécanisme (pas une borne de profondeur arbitraire). En debug, un cycle détecté déclenche un arrêt immédiat (panic) ; en release, il est journalisé et la propagation concernée est arrêtée sans affecter le reste du moteur.

## 4. Flux de données

Trois catégories de données sont distinguées ; leur traitement diffère strictement :

1. **Données persistantes** — les composants attachés durablement à une entité (`Position`, `Health`, `Velocity`...). Elles vivent aussi longtemps que l'entité.
2. **Données transitoires** — données intermédiaires d'une ou plusieurs phases (accumulateurs, intentions, requêtes, contraintes...), qui n'existent que pour organiser un calcul. Une donnée transitoire possède une **durée de vie bornée et un état initial déterministe avant toute lecture** ; le mécanisme concret de remise à l'état initial (memset, double buffering, génération monotone, bitset de validité...) est une décision d'implémentation. Le stockage concret (composant, ressource, buffer SoA, arena...) est également une décision d'implémentation, à trancher selon les volumes réels.
3. **Mutations structurelles** — spawn, despawn, ajout/retrait de composant. Ce sont les seules opérations qui nécessitent impérativement une validation transactionnelle (cf. section 5).

Principe générateur, dont découle tout ce qui précède et suit :

> **Une donnée possède un propriétaire unique.**
> Selon la nature de la donnée, ce principe se décline en : un propriétaire d'écriture (pour une donnée persistante), un propriétaire de durée de vie (pour une donnée transitoire), un propriétaire de validation (pour une mutation structurelle), un propriétaire de résolution (pour un ensemble de contributions transitoires transformées en état persistant).

Ce principe ne restreint pas le nombre de *producteurs* d'une donnée transitoire — un accumulateur peut légitimement recevoir des contributions de plusieurs systèmes. Ce qui reste unique, c'est la responsabilité de résolution : un seul système lit l'ensemble des contributions et les transforme en écriture sur une donnée persistante. Ceci se formule comme suit :

> **Produire est libre ; résoudre est unique.**
> Une donnée persistante possède un unique propriétaire en écriture au sein d'une phase. Tout autre système souhaitant influencer cette donnée produit une donnée transitoire (un accumulateur, une intention, une contrainte), qui sera consommée et résolue par ce propriétaire unique.

Exemple générique : `ConstraintProducerA`, `ConstraintProducerB` et `ConstraintProducerC` écrivent chacun dans un même accumulateur transitoire (ex: `ForceAccumulator`) ; seul un unique système de résolution (ex: `MotionResolutionSystem`) lit cet accumulateur et écrit la donnée persistante correspondante (ex: `Position` ou `Velocity`). Aucune donnée persistante n'a jamais deux écrivains concurrents au sein d'une même phase.

## 5. Modèle d'exécution — pipeline de simulation

- **L'unité atomique de cohérence du `World` est la phase, pas la frame.** Chaque phase se termine par un point de validation atomique : les mutations structurelles qu'elle produit (y compris par plusieurs systèmes de cette phase) ne sont visibles qu'à partir de ce point de validation, donc par les phases suivantes.
- Il n'existe **aucune restriction sur la lecture de données persistantes ou transitoires mises à jour plus tôt dans la même phase** (ex: `Velocity` mis à jour par un système peut être lu par le système suivant de la même phase) — seules les mutations *structurelles* (spawn/despawn/add/remove) sont différées jusqu'au point de validation de la phase. Ceci découle directement du principe « produire est libre, résoudre est unique » : deux systèmes peuvent légitimement former un pipeline séquentiel de calcul au sein d'une phase, tant qu'ils n'ont jamais plus d'un propriétaire en écriture sur une même donnée persistante.
- L'ordre d'exécution des systèmes et des phases est **fixe, connu à la compilation (ou figé au démarrage via un graphe de dépendances construit une fois puis exécuté comme pipeline figé)** — jamais un scheduling dynamique dont l'issue varierait d'une frame à l'autre.
- Le parallélisme se situe **à l'intérieur** d'un système (ex: `rayon::par_iter()` sur une tranche de données contiguë), jamais **entre** systèmes ou phases. L'ordre d'exécution séquentiel des phases et des systèmes n'est jamais remis en cause par le parallélisme intra-système.
- Le pipeline d'exécution est composé à partir de **cinq types de phase** génériques, indépendants de tout domaine métier (RPG, RTS, city builder, ou autre simulation) :

  1. **Acquisition** — le monde reçoit des informations extérieures (clavier, souris, réseau, script, replay, IA externe...).
  2. **Décision** — les systèmes produisent des *intentions* (données transitoires), jamais des mutations directes.
  3. **Résolution** — les intentions et accumulateurs sont transformés en mutations (structurelles ou en écriture de données persistantes par leur propriétaire unique).
  4. **Validation** — application atomique des mutations structurelles accumulées durant la phase.
  5. **Extraction** — préparation des données destinées à l'extérieur du monde de simulation (GPU, audio, logs, télémétrie, réseau...).

  Une frame (ou un tick de simulation) est une **composition concrète** de ces types de phase, fixée à la compilation ou au démarrage — elle n'est pas nécessairement une séquence linéaire à une seule occurrence de chaque type. Un pipeline peut par exemple enchaîner plusieurs cycles Décision → Résolution → Validation avant une unique phase d'Extraction, si le domaine le justifie. Ce qui reste invariant, quelle que soit la composition retenue : chaque instance de phase se termine par un point de validation atomique (cf. plus haut), et l'ordre de composition est fixe et identique à chaque frame — jamais recalculé dynamiquement. Chaque domaine métier concret (combat, IA, buffs, animation...) s'instancie *à l'intérieur* d'une instance de phase ; les types de phase eux-mêmes ne sont jamais nommés d'après un domaine de gameplay.

Le **Scheduler**, au sens implémentation, est le mécanisme concret qui exécute ce modèle (construction du graphe de dépendances, répartition du travail intra-système sur les threads, etc.) — c'est une décision d'implémentation substituable, distincte du modèle d'exécution lui-même qui est normatif.

## Règle de nommage des systèmes

> Un système est nommé selon la transformation déterministe qu'il applique aux données, jamais selon les catégories d'entités qu'il manipule ni selon une mécanique de gameplay spécifique.

Test de validité : *si un nouveau type d'entité est introduit demain, ce système devra-t-il être renommé ?* Si oui, son nom est trop spécifique au métier.

Exemples : `MovementSystem`, `LifetimeSystem`, `HealthResolutionSystem`, `SpatialQuerySystem` plutôt que `EnemySystem`, `PlayerSystem`, `CombatSystem`, `PhysicsSystem`. À l'inverse, un nom trop générique (`ProcessingSystem`, `UpdateSystem`) ne décrit plus aucune transformation identifiable et doit être évité symétriquement.

---

## Ce qui reste hors périmètre de ce document

- Le choix concret des bibliothèques (implémentation du Sparse Set, moteur de rendu, sérialisation) fait l'objet d'ADR indépendantes.
- La composition spatiale (hiérarchies de transformation) sera traitée dans une ADR dédiée une fois ce socle stabilisé en pratique.
- La liste exhaustive des systèmes concrets par phase (combat, IA, navigation, buffs...) sera développée progressivement, en s'appuyant strictement sur le vocabulaire et les invariants ci-dessus.
