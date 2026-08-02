# ADR-001 — Stockage des composants : Sparse Set

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
`01-runtime-ecs.md` et `02-concepts.md` établissent qu'un composant est une donnée pure rattachée à une entité, et que le `World` est seul propriétaire de l'identité des entités. Cette ADR décide du mécanisme de stockage concret des composants, compatible avec l'invariant « zéro allocation sur le hot path » (`00-principes.md`) et le principe « propriétaire unique » (`02-concepts.md`).

## Décision

### 1. Un stockage indépendant par type de composant
Chaque type de composant possède un stockage indépendant. Dans cette ADR, ce stockage est défini comme étant un **Sparse Set** : un tableau *sparse* (indexé par l'index d'entité, résolvant vers une position dans le tableau dense ou une sentinelle « absent »), un tableau dense d'`EntityId` parallèle, et un tableau dense de données `[T]` contigu, castable en octets via `bytemuck`.

### 2. Séparation stricte entre identité et stockage
> **Le Sparse Set est un index de présence, jamais un mécanisme de validation d'identité.**

Le Sparse Set ne revalide pas la génération d'une `EntityId` — ce serait une seconde source de vérité pour une responsabilité déjà portée par le `World` (cf. `01-runtime-ecs.md`, section 1). Il fait confiance à l'appelant, qui a déjà validé la vivacité de l'entité en amont. Le Sparse Set ne stocke et ne vérifie que la correspondance `index → position dense`.

Trois propriétaires distincts, cohérents avec la chaîne déjà établie :
- le `World` est propriétaire de l'identité des entités ;
- le Sparse Set est propriétaire du stockage d'un type de composant ;
- les systèmes sont propriétaires des transformations.

### 3. Isolation entre stockages
**Un Sparse Set ne connaît que son propre composant.** Un stockage de `Position` ignore totalement celui de `Velocity` ou de `Health`. Toute coordination entre plusieurs stockages (requêtes multi-composants, cohérence entre types) est réalisée par les requêtes et consommée par les systèmes — jamais par les Sparse Sets eux-mêmes, et jamais par le `World`.

> **Le `World` expose des stockages ; les requêtes construisent des vues ; les systèmes consomment ces vues.**

Le `World` ne connaît que des entités, des composants et des commandes (`01-runtime-ecs.md`, section 2) — il n'a donc aucune connaissance des requêtes multi-composants. Une requête (`Query`) est une vue de lecture construite au-dessus des stockages exposés par le `World`, jamais une responsabilité du `World` lui-même. Cette séparation exclut par construction toute méthode `world.query()`, tout cache de requête interne au `World`, ou toute optimisation couplant `World` et `Query` — le Query Model constitue un acteur architectural à part entière, objet d'une ADR dédiée.

### 4. Stratégie de suppression : swap-remove
Au retrait d'un composant, l'élément en fin de tableau dense est déplacé à la position libérée, puis le tableau est tronqué. Garantit une contiguïté parfaite, sans tombstone ni trou, en O(1).

Conséquence directe : **aucune itération ne doit supposer un ordre stable du tableau dense entre deux frames.** Aucun système ne doit conserver un index dense en cache d'une frame à l'autre.

### 5. Capacité fixée avant simulation
> Chaque stockage de composant possède une capacité maximale fixée avant le début de la simulation.

Aucune croissance dynamique de tableau sur le hot path. Toute tentative de dépassement est un bug de configuration, détecté au build ou par assertion en debug — jamais un cas géré silencieusement en release.

La Forge constitue le mécanisme privilégié pour déterminer ces capacités dans le pipeline AOT nominal (cf. `00-principes.md`). L'architecture reste toutefois généralisée sur ce point : rien n'exclut qu'un éditeur de niveaux, un serveur configurable, un benchmark, ou un mode debug fournisse ces capacités par un autre mécanisme, tant que la contrainte « fixé avant simulation » est respectée.

### 6. Pas de grouping multi-composants en v1
Une requête choisit un **stockage pilote** parmi les composants demandés, puis résout les autres par consultation de leur index de présence. Le critère de choix du stockage pilote (le plus petit tableau, le plus filtrant statistiquement, un choix spécialisé par la Forge...) est une décision d'implémentation, tant que le principe — un seul stockage piloté par itération, les autres résolus par consultation — est respecté. Aucun mécanisme de *grouping*/tri corrélé entre plusieurs Sparse Sets n'est implémenté en v1.

*Note de cohérence terminologique (ADR-002) : le « stockage pilote » défini ici est le cas particulier, pour une requête portant uniquement sur des Sparse Sets, du concept plus général de* **source d'itération** *— une requête pouvant demain être pilotée par une structure spatiale ou tout autre résultat déterministe, sans remettre en cause cette ADR.*

Justification architecturale, au-delà de la seule prudence de complexité : le grouping créerait une dépendance entre stockages, ce qui contredirait directement l'invariant d'isolation (point 3). S'il devient nécessaire (mesuré par profiling, jamais présupposé), il fera l'objet d'une ADR dédiée plutôt que d'une extension informelle de celle-ci.

## Ce qui n'est pas tranché ici
L'objectif de taille de composant (ordre de grandeur d'une ligne de cache CPU) est une heuristique d'optimisation, pas un invariant architectural — elle dépend de la plateforme, du composant, et de mesures réelles. Elle relève d'un futur guide d'optimisation, pas de cette ADR normative.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Un stockage indépendant par type de composant | Invariant |
| Le Sparse Set ne valide jamais l'identité (index de présence uniquement) | Invariant |
| Isolation stricte entre stockages | Invariant |
| `World` expose des stockages ; les requêtes construisent des vues ; les systèmes consomment ces vues | Invariant |
| Swap-remove (implique : ordre dense non stable inter-frame) | Invariant |
| Capacité fixée avant simulation | Invariant — le mécanisme de détermination (Forge ou autre) est implémentation |
| Requête : un stockage pilote + résolution des autres par consultation | Invariant — le critère de choix du pilote est implémentation |
| Pas de grouping multi-composants en v1 | Décision révisable, réévaluée par mesure |
| Taille cible par composant (ligne de cache) | Hors périmètre — heuristique d'optimisation |
