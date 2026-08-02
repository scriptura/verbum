# ADR-006 — Command Buffer

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
`02-concepts.md` pose un critère de validité : toute nouvelle catégorie de donnée doit s'exprimer comme une instanciation du modèle général (cardinalité, mode d'adressage, durée de vie, politique de consommation), sans invariant de contrat ou de dépendance nouveau. Cette ADR vérifie ce critère pour le Command Buffer, qui formalise le mécanisme de mutations structurelles différées déjà posé en `01-runtime-ecs.md` (sections 2 et 4).

## Décision

### 1. Cardinalité et adressage
Le Command Buffer est une donnée de cardinalité 1 par `World`, identifiée par une clé unique — en tout point identique à une Ressource sur ces deux axes (ADR-005). Rien ne le distingue structurellement d'une Ressource sur la cardinalité ou l'adressage.

### 2. Durée de vie : transitoire
Le Command Buffer instancie l'axe « durée de vie » du modèle général dans sa variante transitoire, exactement selon les règles déjà posées en `01-runtime-ecs.md` (section 4) : durée de vie bornée à une phase, état initial déterministe (vide) avant toute écriture. Le mécanisme concret de remise à l'état initial (vidage après application, double buffering...) reste, comme pour toute donnée transitoire, une décision d'implémentation.

### 3. Politique de consommation : le seul axe qui le distingue d'une Ressource transitoire ordinaire
Le Command Buffer partage les trois premiers axes avec une Ressource transitoire quelconque. Sa spécificité tient entièrement à sa politique de consommation :
- **Accumulation libre en écriture** : tout système peut y produire des commandes (`Spawn`, `Destroy`, `AddComponent`, `RemoveComponent`) sans être le propriétaire exclusif de cette écriture — cohérent avec le principe déjà posé « produire est libre ».
- **Résolution unique au point de validation** : un seul point d'application. Le `World`, seule autorité habilitée à appliquer des mutations structurelles (`01-runtime-ecs.md`, section 2), consomme l'ensemble des commandes accumulées et les applique de façon atomique au point de validation de la phase (`01-runtime-ecs.md`, section 5). Le `World` n'est pas pour autant propriétaire du Command Buffer lui-même — il reste, comme déjà établi, propriétaire des entités et des mutations structurelles ; le buffer n'est que le support transitoire par lequel ces mutations lui parviennent.

Cette politique n'est pas un mécanisme nouveau : c'est l'instanciation, pour cette donnée précise, du principe déjà généralisé en `01-runtime-ecs.md` (« produire est libre ; résoudre est unique »). L'autorité de résolution est ici le `World`, cohérent avec son rôle déjà défini de seule autorité sur les mutations structurelles — sans que cela n'étende son périmètre de propriété au buffer en tant que donnée.

### 4. Participation aux contrats et aux dépendances
Un système qui écrit dans le Command Buffer déclare cette écriture dans son interface déclarative, exactement comme pour toute donnée (ADR-003). La dépendance entre un système producteur de commandes et le `World` qui les applique n'est pas une dépendance entre systèmes au sens d'ADR-004 : le point de validation de phase constitue déjà, par construction, une barrière garantissant que toutes les commandes émises pendant la phase sont accumulées avant application — aucune déduction d'ordre supplémentaire n'est nécessaire entre producteurs de commandes.

### 5. Aucune sémantique métier portée par le Command Buffer
Conformément à l'invariant déjà posé en `01-runtime-ecs.md` (section 3) — le `World` reste une infrastructure ignorante de toute sémantique métier — le Command Buffer n'interprète jamais le contenu des commandes qu'il accumule. Il ne fait qu'accumuler puis transmettre au `World` ; toute politique (cascade, condition, etc.) reste portée par les systèmes qui émettent les commandes, jamais par le buffer lui-même.

## Constat — vérification du critère de validité

| Axe | Instanciation pour le Command Buffer |
|---|---|
| Cardinalité | 1 par `World` |
| Mode d'adressage | Clé unique |
| Durée de vie | Transitoire, bornée à la phase |
| Politique de consommation | Accumulation libre en écriture, résolution unique par le `World` au point de validation |

Le seul axe qui distingue réellement le Command Buffer d'une Ressource transitoire ordinaire (ADR-005) est la politique de consommation — et celle-ci n'introduit aucun invariant nouveau : elle applique littéralement le principe « produire est libre ; résoudre est unique » déjà posé en `01-runtime-ecs.md`. Le critère de validité de `02-concepts.md` est satisfait.

## Ce qui n'est pas tranché ici
La représentation concrète du Command Buffer (une structure unique, une par thread puis fusionnée, une par type de commande...) est une décision d'implémentation. L'ordre d'application entre commandes de nature différente (ex: un `Spawn` et un `Destroy` visant potentiellement la même entité) reste également hors périmètre — à documenter séparément si le besoin se présente, sans qu'il s'agisse d'un invariant architectural.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Cardinalité 1, adressage par clé unique | Invariant (instanciation d'ADR-005) |
| Durée de vie transitoire, bornée à la phase | Invariant (instanciation de `01-runtime-ecs.md`, section 4) |
| Accumulation libre en écriture, résolution unique par le `World` | Invariant (instanciation de « produire est libre, résoudre est unique ») |
| Aucune sémantique métier portée par le buffer | Invariant (instanciation de `01-runtime-ecs.md`, section 3) |
| Représentation concrète (structure unique, par thread, par type de commande...) | Hors périmètre — implémentation |
| Ordre d'application entre commandes de nature différente | Hors périmètre — à documenter si besoin |

## Bilan de la colonne vertébrale
Avec ADR-001 à ADR-006, le runtime ECS est fermé d'un point de vue architectural :

```
Principes → Concepts (modèle général de donnée)
                │
Sparse Set (ADR-001) → Query (ADR-002) → Contrat de système (ADR-003)
                │
        Modèle d'exécution (ADR-004)
                │
        Ressources (ADR-005) → Command Buffer (ADR-006)
```

Les domaines périphériques (Renderer, Assets, Audio, Navigation, Sauvegarde, IA...) deviennent désormais des consommateurs de ce socle, chacun objet de sa propre ADR, sans vocation à remettre en cause les invariants ci-dessus — sauf à démontrer, comme l'exige `02-concepts.md`, qu'une catégorie de donnée ne peut être exprimée par le modèle général actuel.
