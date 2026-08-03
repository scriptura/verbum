# 02 — Concepts

## Statut
**Normatif.** Date de rédaction : 31 juillet 2026. Ce document fixe le vocabulaire officiel du moteur — son *ubiquitous language*. Il ne redéfinit aucun invariant déjà posé dans `01-runtime-ecs.md` ; il nomme et rassemble en un seul endroit les briques conceptuelles qui y sont disséminées, pour que les ADR techniques puissent s'y référer sans les reformuler.

Toute ADR technique doit utiliser ces termes tels que définis ici. Si un terme semble manquer de précision pour un usage donné, c'est ce document qui doit être amendé — pas l'ADR qui doit inventer une variante locale du vocabulaire.

---

## Entité (`Entity`, `EntityId`)
Identifiant opaque sans sémantique métier. Ne porte ni catégorie de gameplay, ni comportement. Sert uniquement de clé stable pour retrouver des composants.
→ Défini en détail en `01-runtime-ecs.md`, section 1.

## Composant (`Component`)
Structure de donnée pure (`Copy`/`Clone`), sans méthode ni logique. Rattachée à une entité. Porte toute la sémantique que l'`EntityId` n'a pas — c'est la seule source de sens du moteur.
→ Principe posé en `00-principes.md` (séparation données/logique) ; usage détaillé en `01-runtime-ecs.md`, sections 1 et 4.

## Système (`System`)
Fonction de transformation déterministe : elle lit un ensemble de données et en écrit un autre. Un système ne connaît ni catégorie métier ni type d'entité — seulement des formes de données. Nommé d'après la transformation qu'il applique, jamais d'après une mécanique de gameplay.
→ Règle de nommage en `01-runtime-ecs.md`, section 5.

## Monde (`World`)
Infrastructure unique, ignorante de toute sémantique métier. Seule autorité habilitée à créer et détruire des entités et à appliquer des mutations structurelles. Ne connaît que : des entités, des composants, des commandes.
→ Défini en `01-runtime-ecs.md`, section 2.

## Ressource (`Resource`)
Donnée nommée dont le cardinal est de 1 **par instance de `World`** — par opposition à un composant, adressé par `EntityId` et dont le cardinal varie de 0 à N. Une ressource est identifiée par une clé unique (aujourd'hui, le type Rust concerné ; potentiellement, demain, une clé générée par la Forge ou tout autre mécanisme d'identification — le choix concret reste une décision d'implémentation). Une ressource peut être persistante ou transitoire, selon l'axe « durée de vie » du modèle général ci-dessous ; seuls le cardinal et le mode d'adressage la distinguent structurellement d'un composant.

## Modèle général de donnée
Toute donnée manipulée par le moteur — composant, ressource, ou catégorie à venir — se caractérise par quatre axes orthogonaux, plutôt que par une sémantique propre à chaque catégorie :

| Axe | Composant | Ressource |
|---|---|---|
| **Cardinalité** | 0..N par `World` (une valeur par entité porteuse) | 1 par `World` |
| **Mode d'adressage** | par `EntityId` | par clé unique (aujourd'hui : type Rust) |
| **Durée de vie** | persistante ou transitoire (cf. entrées dédiées) | persistante ou transitoire |
| **Politique de consommation** | lue/écrite tant que l'entité existe | lue/écrite tant que la ressource existe |

Un composant et une ressource ne diffèrent donc pas par leur sémantique, mais uniquement par leur cardinalité et leur mode d'adressage. Les deux participent identiquement aux contrats de système (ADR-003) et aux dépendances production → consommation (ADR-004) — aucune règle de dépendance ou de contrat distincte n'existe pour l'une ou pour l'autre.

*Note sur `Event` :* une catégorie à cardinalité 0..N mais durée de vie éphémère (contrairement à un composant, qui persiste avec son entité) occuperait une case distincte de cette grille. Ceci n'introduit pas la notion d'Événement dans le moteur — son statut reste **non introduit**, cf. entrée dédiée plus bas. Elle n'est mentionnée ici qu'à titre d'illustration de l'espace des axes.

### Critère de validité d'une nouvelle catégorie de donnée
> Toute nouvelle catégorie de donnée introduite dans le moteur doit pouvoir être exprimée comme une instanciation du modèle général ci-dessus — en renseignant ses quatre axes (cardinalité, mode d'adressage, durée de vie, politique de consommation) — sans qu'aucun nouvel invariant architectural ne soit nécessaire pour ses contrats ou ses dépendances (ADR-003, ADR-004). Si l'introduction d'une catégorie exige un invariant qui ne découle pas de ces quatre axes, c'est le modèle général qui doit être révisé en premier — jamais une ADR locale qui doit inventer une exception.

Ce critère gouverne toute ADR future introduisant une catégorie de donnée (Command Buffer, Événement, ou autre) : elle doit démontrer, point par point, que chaque axe est renseigné et qu'aucune règle nouvelle n'est requise au-delà de ce qui est déjà posé.

### Frontière entre modèle architectural et structure interne
> Le modèle architectural décrit les catégories de données et leurs relations. Il ne décrit pas la structure interne de ces données, qui relève de leur implémentation.

Une Ressource ou un composant peut porter, en interne, n'importe quelle structure composée — une collection, un `Vec<T>`, une disposition SoA, un sparse set, un B-Tree, un octree, un BVH, un graphe compressé, un arena, un slab, une zone mmap, ou toute autre représentation à venir. Le modèle architectural s'arrête à la Ressource ou au composant lui-même — sa cardinalité, son mode d'adressage, sa durée de vie, sa politique de consommation. Il ne classifie jamais la structure interne de son contenu : celle-ci relève de l'implémentation, hors périmètre du vocabulaire architectural, au même titre qu'un `Contact` dans une collection de contacts physiques, un `InventoryItem` dans un inventaire, ou une commande accumulée dans un Command Buffer. Le Data Layout (SoA, alignement mémoire, etc.) n'est qu'un cas particulier de cette structure interne — le principe reste valable quelle que soit la technique de représentation retenue demain.

Cette frontière, initialement implicite, a été rendue explicite par ADR-R01/ADR-R02 (pipeline de rendu) : une confusion entre les deux plans avait fait apparaître une hypothèse de catégorie de donnée supplémentaire, qui s'est résorbée une fois la distinction posée — sans qu'aucun axe du modèle général n'ait eu besoin d'être modifié.

### Conséquence — le runtime ne connaît jamais le contenu individuel
> Le runtime (`World`, Query, modèle d'exécution) ne connaît jamais un élément de donnée pris individuellement — il ne connaît que la catégorie qui le porte (`Resource<T>` ou l'ensemble des `T` d'un composant), jamais `T` lui-même en tant qu'instance isolée hors de cette catégorie.

C'est cette conséquence qui explique, rétroactivement, pourquoi une Query raisonne sur des catégories de données (ADR-002), pourquoi le modèle d'exécution déduit ses dépendances au niveau des catégories (ADR-004), et pourquoi le principe de propriétaire unique s'attache à une catégorie plutôt qu'à un contenu (`02-concepts.md`, plus haut). Le runtime n'a, par construction, aucun moyen d'accéder à un élément interne autrement qu'à travers la catégorie qui le porte.

## Grille de classification des domaines
Trois strates de responsabilité, qui ne se mélangent jamais, structurent désormais l'ensemble du moteur :

| Strate | Contenu | Question de classement |
|---|---|---|
| **Plateforme** | Shell, Backend, `World` en tant qu'exécutant, OS, threads, fenêtrage, horloge réelle, réseau, disque | Interagit-il avec l'OS, le matériel, ou le temps réel ? |
| **Modèle architectural** | Entités, composants, ressources, systèmes, contrats, phases | Définit-il une donnée métier, un traitement logique, ou un contrat d'exécution ? |
| **Structure interne** | Sparse Set, SoA, arena, `FrameRepresentation`, collections internes d'une Ressource | Concerne-t-il le layout mémoire, l'optimisation cache, ou l'adressage interne d'une catégorie ? |

Cette grille est une conséquence a posteriori, confirmée par des cas concrets déjà traités : ADR-R01/R02 ont montré qu'une collection interne à une Ressource relève de la structure interne, jamais du modèle architectural ; ADR-R03 a montré que le Backend relève de la plateforme, hors du modèle architectural ; le Shell (ADR-S01) confirme la même appartenance à la plateforme. Elle sert de test de classement pour tout futur domaine (Assets, Audio, Navigation, Réseau, Sauvegarde...) : la question à poser en priorité n'est plus « comment ce domaine fonctionne-t-il », mais à laquelle des trois strates il appartient — avant même d'ouvrir la moindre ADR.

*Note sur la strate Plateforme :* le Shell n'y occupe pas une position de simple pair vis-à-vis du Backend ou d'une future infrastructure Audio/Assets. Il en est le point de composition (ADR-S01) — celui qui crée, assemble et orchestre toutes les autres infrastructures de cette strate, ainsi que le `World` lui-même en tant qu'exécutant.

*Note sur la Forge :* la grille ci-dessus répond à la question « où se situe ce domaine pendant l'exécution du moteur ? ». La Forge n'y répond pas, non pas parce qu'elle échapperait à une catégorisation, mais parce qu'elle est extérieure au système qu'elle produit — exactement comme `rustc`, Cargo, ou tout autre outil de build. Elle produit des artefacts AOT (données binaires, code généré) consommés ensuite par le runtime, séparée de lui par une frontière de compilation :

```
Forge (build-time)
        │
        ▼
Artefacts AOT
        │
   ─── frontière de compilation ───
        │
        ▼
Runtime du moteur
        │
   Plateforme → Modèle architectural → Structure interne
```

La Forge n'est donc pas un quatrième cas de la grille : elle est hors périmètre de la question que la grille pose.

### Principe de circulation entre strates
> Les frontières entre strates ne constituent jamais un dialogue synchrone. Chaque flux est unidirectionnel ; lorsqu'une communication existe dans les deux sens entre deux domaines, elle résulte de deux flux indépendants, séparés dans le temps, chacun régi par ses propres invariants.

Ce principe ne décrit aucune catégorie de donnée — c'est un principe transversal sur la manière dont les flux traversent les frontières entre strates, à distinguer des concepts définis ailleurs dans ce document. Il se vérifie déjà dans les cas traités : le Shell n'alimente l'ECS que dans un sens (événements → Acquisition, ADR-S01) ; le Backend ne consomme la sortie du Renderer que dans l'autre (`FrameRepresentation` → présentation, ADR-R03). Un domaine qui semble nécessiter les deux directions à la fois (ex: une infrastructure qui consomme une intention du modèle architectural et produit, plus tard, un fait acquis en retour) ne devient pas pour autant un dialogue synchrone : ce sont deux flux indépendants, chacun soumis à ses propres règles déjà posées ailleurs — une intention est une donnée transitoire produite librement (ADR-004), un fait acquis est une nouvelle entrée de la phase d'Acquisition d'une itération future, jamais une modification d'une itération en cours (ADR-S01). Aucune promesse, aucun callback, aucune future, aucune corrélation garantie entre l'émission d'une intention et l'arrivée d'un fait acquis ne relie les deux flux.

Ce principe sert de base commune à tout domaine combinant les deux directions (Assets, Audio, Réseau, Sauvegarde) — chacun l'instanciera plutôt que de le redécouvrir séparément.

### Trois notions de frontière, à ne pas confondre
Ce corpus emploie désormais trois notions de frontière, qui ne séparent pas le même axe et ne doivent jamais être confondues dans une future ADR :

| Frontière | Sépare |
|---|---|
| **De compilation** | Build-time (Forge) ↔ Runtime du moteur |
| **De strate** | Plateforme ↔ Modèle architectural ↔ Structure interne |
| **De connaissance** | Ce qu'un domaine a le droit de connaître ↔ ce qu'il lui est interdit de connaître (`00-principes.md`) |

Une ADR qui invoque « une frontière » sans préciser laquelle des trois est ambiguë par construction — chaque référence future doit nommer explicitement le type de frontière concerné.

## Phase
Unité atomique de cohérence du `World`. Une phase se termine par un point de validation atomique : les mutations structurelles qu'elle produit ne sont visibles qu'à partir des phases suivantes. Une phase appartient à l'un des cinq types génériques (Acquisition, Décision, Résolution, Validation, Extraction).
→ Défini en `01-runtime-ecs.md`, section 5.

## Frame / Tick
Composition concrète, fixée à la compilation ou au démarrage, d'une séquence d'instances de phases. N'est pas nécessairement une occurrence unique de chaque type de phase — une frame peut enchaîner plusieurs cycles Décision → Résolution → Validation.
→ Défini en `01-runtime-ecs.md`, section 5.

## Modèle d'exécution
L'ensemble des règles normatives régissant l'ordre des phases, la visibilité des données, et les garanties de déterminisme. Distinct du **Scheduler**, qui est le mécanisme d'implémentation concret exécutant ce modèle (construction du graphe de dépendances, répartition sur les threads). Une fois construit (ADR-004), ce modèle devient une donnée immuable, parfois désignée « programme d'exécution » — il ne s'agit pas d'une seconde catégorie, mais de la forme instanciée du même modèle.
→ `01-runtime-ecs.md`, section 5 ; ADR-004.

## Composition
Politique de consommation d'une donnée transitoire à producteurs multiples qui agrège l'ensemble des contributions produites, sans en écarter aucune ni arbitrer entre elles — par opposition à une résolution, qui arbitre des contributions concurrentes pour produire un résultat unique. Instanciation du principe « produire est libre » (cf. Propriétaire unique), appliquée à l'agrégation plutôt qu'à l'arbitrage.
→ Introduit en ADR-R01, point 4 ; réemployé en ADR-SV01 (extraction/composition de l'état logique) et ADR-N01.

## Infrastructure
Composant de la strate Plateforme (ou le `World` en tant qu'exécutant) qui applique une donnée déjà entièrement produite par le modèle architectural, sans jamais décider de son contenu ni l'interpréter au sens métier — symétrie déjà posée entre le `World` appliquant le Command Buffer et le Backend appliquant la représentation de frame. Assemblée exclusivement par le Shell, seul point de composition de la strate.
→ ADR-R03, point 2 ; ADR-S01, point 7.

## Commande (`Command`)
Émission différée d'une intention de mutation structurelle (`Spawn`, `Destroy`, `AddComponent`, `RemoveComponent`). Émise par un système, jamais exécutée directement par lui ; appliquée par le `World` au point de validation de la phase.
→ `01-runtime-ecs.md`, section 2.

## Donnée persistante
Donnée (composant ou ressource) dont la durée de vie est liée à celle de l'entité qui la porte (ou, pour une ressource, à la durée de vie du `World` — non bornée à une phase, jusqu'à libération explicite le cas échéant ; cf. le déchargement d'un asset, ADR-A01, point 5). Possède, au sein d'une phase, un unique propriétaire en écriture.
→ `01-runtime-ecs.md`, section 4.

## Donnée transitoire
Donnée intermédiaire (accumulateur, intention, requête, contrainte) qui n'existe que pour organiser un calcul au sein d'une ou plusieurs phases. Possède une durée de vie bornée et un état initial déterministe avant toute lecture. Peut avoir plusieurs producteurs ; sa résolution en donnée persistante a toujours un propriétaire unique.
→ `01-runtime-ecs.md`, section 4.

## Mutation structurelle
Spawn, despawn, ajout ou retrait de composant. Seule catégorie d'opération nécessitant un passage par le mécanisme de commandes et une application au point de validation de phase — jamais appliquée directement pendant l'itération d'un système.
→ `01-runtime-ecs.md`, sections 2 et 4.

## Relation
Référence d'une entité vers une autre, portée comme donnée (`EntityId`) et non comme mécanisme de propriété implicite du framework. N'implique par défaut aucune politique de cycle de vie.
→ `01-runtime-ecs.md`, section 3.

## Politique de cycle de vie
Règle métier explicite, portée par un système, qui décide si et comment la disparition d'une entité entraîne des conséquences sur d'autres entités (cascade, survie, conditionnalité). Jamais portée par le `World` lui-même.
→ `01-runtime-ecs.md`, section 3.

## Propriétaire unique (principe d'*ownership*)
Principe générateur : toute donnée ou responsabilité a un propriétaire unique — propriétaire d'écriture (donnée persistante), propriétaire de durée de vie (donnée transitoire), propriétaire de validation (mutation structurelle), propriétaire de résolution (contributions transitoires → état persistant). Un accumulateur transitoire peut avoir plusieurs producteurs ; sa résolution reste toujours unique.
→ `01-runtime-ecs.md`, section 4.

## Événement (`Event`)
**Non introduit à ce stade.** Aucun mécanisme d'événement n'est actuellement normatif dans le moteur ; les communications inter-systèmes passent par des données transitoires (accumulateurs, requêtes) ou par des commandes. Si un besoin de notification asynchrone découplée émerge (ex: télémétrie, hooks de modding), ce terme sera défini dans une ADR dédiée et ce document sera amendé en conséquence.

---

## Ce qui reste hors périmètre de ce document

- Les choix concrets de représentation mémoire (Sparse Set, layout binaire, bibliothèques) sont définis dans les ADR techniques, qui doivent utiliser ce vocabulaire sans le redéfinir.
- Ce document décrit le moteur, jamais le jeu. Aucun concept de Game Design (quête, PNJ, faction, objet...) n'y figure — ils seront définis dans le Game Design Document, une fois le noyau architectural stabilisé par les ADR techniques.
