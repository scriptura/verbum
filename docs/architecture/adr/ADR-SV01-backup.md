# ADR-SV01 — Domaine Sauvegarde

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
Cette ADR applique la grammaire des ADR (`00-principes.md`) au domaine Sauvegarde. Conformément au test méthodologique déjà établi, elle a été précédée d'une page d'axes (sans décision de mécanisme), soumise à Gemini et GPT, qui ont conclu qu'aucun concept architectural nouveau n'était nécessaire : le domaine s'exprime intégralement par instanciation du vocabulaire déjà posé (extraction/composition — ADR-R01/R02 ; principe de circulation — `02-concepts.md` ; propriété exclusive du `World` sur la création d'entités — `01-runtime-ecs.md` ; persistance déclarative, par analogie avec l'absence de parsing implicite — ADR-A01).

## Décision

### 1. Quel est le contrat du domaine ?
Une sauvegarde est une représentation persistante de l'**état logique de la simulation** — pas seulement du `World`. Cet état est constitué des données persistantes portées par le `World` (composants, Ressources) et de tout autre élément que le modèle architectural désigne explicitement comme faisant partie de l'état déterministe, quel que soit l'endroit où il vit concrètement. Reprendre une sauvegarde doit produire une évolution de simulation indiscernable de la continuation sans interruption, pour tout ce qui a été persisté.

Ce contrat est un contrat de **sémantique**, jamais de représentation : ce qui compte est que l'état logique soit reconstructible à l'identique, pas que la structure mémoire du fichier corresponde à la structure mémoire du runtime.

### 2. Qui possède quoi ?
- **Le `World`** reste l'unique créateur d'`EntityId`, y compris pendant une reconstruction : aucun mécanisme de sauvegarde ne réattribue directement un `EntityId` sans passer par le `World`.
- **Un ou plusieurs systèmes d'extraction** composent l'état logique persistant en une représentation unique — même motif que la composition du Renderer (ADR-R01/R02), appliqué à l'état logique plutôt qu'à une frame visuelle.
- **Un système de reconstruction** interprète la représentation lue et émet les commandes structurelles nécessaires (`Spawn`, `AddComponent`...) via le Command Buffer (ADR-006).
- **Le Shell** possède l'unique accès au médium de persistance (disque, ou tout autre support), en tant que dépendance de plateforme (ADR-S01, point 4), toujours de façon asynchrone lorsque ce médium l'exige. Le médium concret (disque, mémoire pour un usage éphémère...) et la fréquence de l'extraction/composition (rare et durable, ou continue et éphémère) sont des décisions d'implémentation — l'invariant est le motif d'extraction/composition et de reconstruction, jamais un médium particulier.
- **La Forge** n'intervient jamais dans ce domaine : elle ne connaît que les artefacts statiques produits avant l'exécution, jamais l'état dynamique d'une partie en cours.
- **Chaque type de composant ou de Ressource** déclare explicitement son appartenance (ou non) à l'état persistant ; le mécanisme de cette déclaration relève de l'implémentation — cf. point 4.

### 3. Quelles frontières s'appliquent ?
- **Frontière de connaissance** : ce n'est pas le runtime qui doit ignorer sa structure interne — il peut parfaitement la connaître. C'est le **format de sauvegarde**, en tant qu'artefact externe et durable, qui ne doit jamais en dépendre. Un format qui encoderait le layout physique d'un Sparse Set (ordre du tableau dense, position après swap-remove, capacité préallouée par la Forge) ferait franchir cette frontière au contrat externe — la faute serait dans le format, jamais dans le moteur.
- **Frontière de strate** : la composition de l'état logique est Modèle architectural ; l'écriture/lecture physique sur disque est Plateforme (Shell) ; le layout binaire concret du fichier (dump, table d'offsets, format compact) est Structure interne, opaque aux systèmes métier.
- **Frontière de compilation** : sans objet ici — la Forge n'intervient pas dans ce domaine (cf. point 2).
- **Identité** : un `EntityId` n'a aucune validité au-delà de la session qui l'a créé. Toute Relation référencée dans une sauvegarde est portée par un identifiant stable *local au fichier*, remappé vers de nouveaux `EntityId` à la reconstruction — une conséquence directe de la propriété exclusive du `World` sur la création d'identité (`01-runtime-ecs.md`), pas une exception.

### 4. Quelles garanties sont offertes ?
- Reprendre une sauvegarde produit un état logique équivalent à la continuation sans interruption, pour tout ce qui a été persisté.
- Le format ne dépend jamais de la structure interne du moteur — seulement de catégories de données stables.
- Aucune opération de sauvegarde ne bloque la boucle déterministe (écriture et lecture disque confiées au Shell, asynchrones, cohérentes avec le principe de circulation — `02-concepts.md`).
- **La persistance est déclarative, jamais implicite.** Un composant ou une Ressource n'est pas persistant par défaut : son appartenance à l'état sauvegardé est déclarée explicitement, jamais découverte dynamiquement par inspection de la mémoire du runtime. C'est le même registre d'exigence que l'absence de parsing implicite déjà posée pour les Assets (ADR-A01) — la sauvegarde suit un contrat déclaré à l'avance, elle ne devine jamais quoi écrire.

### 5. Qu'est-ce qui est explicitement exclu ?
Une sauvegarde ne contient jamais :
- les Assets et Ressources issues de la Forge (reproductibles depuis les artefacts déjà présents localement) ;
- les caches et données dérivées (tout ce qui est recalculable depuis l'état persisté et les règles du jeu ne doit jamais être persisté séparément, sous peine de créer deux sources de vérité) ;
- les données transitoires (accumulateurs, intentions, contributions) — elles n'ont de sens qu'à l'intérieur d'une seule instance de phase ;
- le contenu du Command Buffer — toujours vide à tout point de stabilité, appliqué à chaque validation de phase ;
- les données d'Acquisition déjà consommées ;
- la structure interne du `World` (layout des Sparse Sets, état de la free-list, compteurs de génération, représentation physique des `EntityId`) ;
- l'état propre du Shell ou de la plateforme (taille de fenêtre, timers, threads...).

### 6. Quelles questions sont volontairement laissées à l'implémentation ?
- Le mécanisme concret de déclaration de persistance (trait Rust, génération Forge, table statique, annotation).
- L'identifiant stable local au fichier représentant une Relation, et le mécanisme précis de remapping au chargement.
- La compatibilité de version d'un composant entre deux versions du jeu (question ouverte depuis la toute première session — non tranchée ici).
- Le format binaire concret (dump structuré, table d'offsets, format compact, compression).
- Le statut d'une sauvegarde partielle (autosave rapide) — variante à traiter séparément, ou instanciation du même contrat avec un sous-ensemble de composants déclarés.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Contrat de sémantique (état logique équivalent), jamais de représentation | Invariant |
| Le `World` reste seul créateur d'`EntityId`, y compris à la reconstruction | Invariant (instanciation de `01-runtime-ecs.md`) |
| Extraction/composition par des systèmes, écriture/lecture disque par le Shell | Invariant (instanciation d'ADR-R01/R02 et ADR-S01) |
| Persistance déclarative, jamais implicite | Invariant (instanciation d'ADR-A01) |
| Format externe jamais dépendant de la structure interne du moteur | Invariant (instanciation de la frontière de connaissance, `00-principes.md`) |
| Liste des exclusions (Assets, caches, données transitoires, Command Buffer, Acquisition consommée, structure interne, état plateforme) | Invariant |
| Mécanisme de déclaration, identifiant de remapping, compatibilité de version, format binaire, autosave partielle | Hors périmètre — implémentation |

## Bilan
Conformément au verdict de Gemini (validé par GPT), cette ADR n'introduit aucun concept architectural nouveau. Chaque section instancie un invariant déjà posé — `01-runtime-ecs.md`, `02-concepts.md`, ADR-R01/R02, ADR-S01, ADR-A01 — confirmant que le langage architectural du corpus reste suffisamment expressif pour absorber ce domaine sans inflation conceptuelle.
