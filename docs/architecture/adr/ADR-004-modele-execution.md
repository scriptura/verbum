# ADR-004 — Construction du modèle d'exécution

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
ADR-003 a défini le contrat d'un système : une interface déclarative (lectures, écritures, productions) entièrement inspectable, sans que le système ne décide jamais quand il s'exécute. `01-runtime-ecs.md` (section 5) a par ailleurs déjà fixé l'ordre *entre* phases : une frame est une composition fixe de types de phase, connue à la compilation ou au démarrage. Cette ADR répond à la question qui reste ouverte : comment un ensemble de contrats de systèmes, **à l'intérieur d'une instance de phase**, devient-il un ordre d'exécution déterministe ?

```
Contrat A
Contrat B
Contrat C
Contrat D
      │
      ▼
Construction du modèle d'exécution
      │
      ▼
Programme d'exécution déterministe
```

## Décision

### 1. L'entrée est un ensemble de contrats, jamais une liste de fonctions
Le matériau de construction n'est pas le code des systèmes, mais l'ensemble de leurs contrats tels que définis en ADR-003 (interface déclarative : lectures, écritures, productions). La construction ne connaît ni l'identité d'un système, ni son implémentation — seulement ce qu'il expose.

### 2. L'invariant est l'existence d'un ordre, pas sa représentation
> Il existe un ordre d'exécution déterministe satisfaisant l'ensemble des contraintes exprimées par les contrats.

Que cet ordre soit matérialisé par un graphe de dépendances (DAG), une liste topologique, un pipeline figé écrit à la main, ou du code généré, est une décision d'implémentation. Le terme *graphe* est délibérément absent de cet invariant pour ne pas figer une représentation particulière.

### 3. Les dépendances sont déduites, jamais déclarées
Un système ne déclare jamais explicitement une relation d'ordre (« après Physics », « avant AI »). Il déclare uniquement ce qu'il lit, ce qu'il écrit, ce qu'il produit (ADR-003). Toute contrainte d'ordre entre deux systèmes d'une même phase est **déduite** de ces déclarations — jamais exprimée par le système lui-même. C'est un prolongement direct de l'invariant ADR-003 selon lequel un système ne décide jamais quand il s'exécute.

> Toute dépendance est déduite d'une relation de production → consommation déclarée.

Cette formulation couvre indifféremment les trois catégories de données déjà établies en `01-runtime-ecs.md` : écriture persistante → lecture persistante, production transitoire → consommation transitoire, et toute autre relation de production/consommation qui apparaîtra avec les catégories de données restant à formaliser (ressources, commandes — ADR-005, ADR-006). Elle n'a donc pas besoin d'être amendée lorsque ces documents seront rédigés.

Grâce au principe « propriétaire unique » déjà posé en `02-concepts.md`, deux systèmes d'une même phase ne peuvent jamais entrer en conflit d'écriture sur une même donnée persistante (un seul propriétaire en écriture existe par construction) — ce qui exclut toute relation de dépendance de type écriture-écriture au niveau des données persistantes. La relation production → consommation reste néanmoins la seule source de contrainte possible, quelle que soit la catégorie de donnée concernée.

Cette déduction ne s'applique qu'à l'intérieur d'une instance de phase. L'ordre entre phases reste celui déjà fixé par `01-runtime-ecs.md` : un système d'une phase N+1 lisant une donnée validée en phase N n'introduit aucune contrainte de dépendance supplémentaire à déduire — la frontière de phase la garantit déjà.

### 4. Absence d'ordre valide = erreur de construction, jamais une erreur runtime
> L'absence d'ordre déterministe constitue une erreur de construction du modèle d'exécution.

Si les contrats déclarés d'un ensemble de systèmes ne permettent la déduction d'aucun ordre valide (par exemple deux systèmes qui, mutuellement, lisent une donnée que l'autre écrit — une dépendance cyclique), le modèle d'exécution ne se construit pas. Ce n'est jamais un comportement dégradé accepté en release, ni une erreur découverte pendant la simulation : la construction échoue avant que la première frame ne s'exécute, cohérent avec l'esprit AOT du moteur (`00-principes.md`) — on valide la cohérence avant le runtime, pas pendant.

### 5. Le modèle construit est immuable
Une fois construit, le modèle d'exécution n'est plus modifié, réoptimisé, ni recalculé pendant la simulation. Il devient lui-même une donnée, produite une seule fois (à la compilation ou au démarrage), puis strictement exécutée frame après frame :

```
Contrats → Construction (une fois) → Programme d'exécution → Frame, Frame, Frame, ...
```

Ceci est cohérent avec l'invariant déjà posé en `01-runtime-ecs.md` : l'ordre des systèmes et des phases est fixe et identique à chaque frame — jamais un scheduling dynamique dont l'issue varierait d'une frame à l'autre.

> Un système n'a pas d'ordre intrinsèque.

Sa position dans l'exécution est entièrement relative à la phase dans laquelle il est placé et aux contrats des autres systèmes présents dans cette même phase. Déplacer un système d'une phase à une autre ne modifie pas le système lui-même — cela modifie exclusivement le modèle d'exécution construit à partir de sa nouvelle position. Ceci prévient toute dérive où un système finirait, dans les faits, par être conçu comme un « système de la phase X » plutôt que comme une transformation indépendante (cf. règle de nommage, `01-runtime-ecs.md`, section 5).

## Ce qui n'est pas tranché ici
Le mécanisme concret de construction (algorithme de tri topologique, structure de graphe, étape de build vs étape de démarrage, outillage de diagnostic en cas d'échec) est une décision d'implémentation, à documenter séparément.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| L'entrée de la construction est un ensemble de contrats, jamais des fonctions ou identités de système | Invariant |
| Existence d'un ordre déterministe satisfaisant les contraintes des contrats | Invariant |
| Les dépendances sont déduites des contrats, jamais déclarées par un système | Invariant |
| Toute dépendance est une relation production → consommation (couvre données persistantes, transitoires, et catégories futures) | Invariant |
| Aucun conflit écriture-écriture possible sur une donnée persistante | Invariant (conséquence du principe « propriétaire unique ») |
| Absence d'ordre valide = erreur de construction, jamais une erreur runtime | Invariant |
| Le modèle construit est immuable pendant la simulation | Invariant |
| Un système n'a pas d'ordre intrinsèque (sa position est relative à la phase et aux autres contrats) | Invariant (conséquence) |
| Représentation concrète du modèle (graphe, liste, code généré...) | Hors périmètre — implémentation |
| Algorithme de construction (tri topologique ou autre) | Hors périmètre — implémentation |

## Suite prévue
Avec ADR-001 à ADR-004, la colonne vertébrale du runtime ECS couvre : identité, stockage, requête, contrat de système, et construction de l'ordre d'exécution. ADR-005 (Ressources) et ADR-006 (Command Buffer) restent à rédiger, mais constituent des spécialisations de ce modèle désormais établi plutôt que des documents fondateurs supplémentaires.
