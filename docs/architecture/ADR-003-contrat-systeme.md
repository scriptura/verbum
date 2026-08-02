# ADR-003 — Contrat d'un système

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
`01-runtime-ecs.md` définit un système comme une fonction de transformation déterministe, nommée d'après sa transformation. ADR-001 et ADR-002 ont figé la chaîne `World → Sparse Set → Query`. Il manque la pièce qui relie ces briques de calcul au Modèle d'exécution (`01-runtime-ecs.md`, section 5) : le contrat architectural d'un système — ce qu'il *est*, indépendamment de toute implémentation Rust (fonction, trait, macro).

## Décision

### 1. Un système est une fonction de transformation déclarative
> Un système est une fonction de transformation déclarative. Son contrat est entièrement déductible de son interface déclarative.

L'interface déclarative d'un système est aujourd'hui constituée des Queries et Ressources qu'il consomme (cf. ADR-002), et des données transitoires ou commandes qu'il produit. Sa représentation concrète (signature de fonction Rust, macro, métadonnées générées...) reste une décision d'implémentation — le terme *interface déclarative* est délibérément indépendant du langage, pour ne pas figer implicitement le contrat dans la syntaxe Rust actuelle.

En observant uniquement cette interface déclarative, on doit pouvoir répondre sans lire le corps du système :
- Que lit-il ?
- Que peut-il écrire ?
- Que peut-il créer ou détruire ?
- Produit-il des données transitoires ?
- Produit-il des commandes structurelles ?

Si l'une de ces réponses nécessite d'ouvrir l'implémentation, une partie du contrat est implicite — ce qui est proscrit. Ceci n'introduit aucune règle nouvelle : c'est une conséquence directe de l'invariant déjà posé en ADR-002 selon lequel une Query et une Ressource déclarent explicitement, dans leur type, leur mode d'accès.

### 2. Absence d'état persistant caché
Un système ne possède aucun état interne durable. Toute persistance transite exclusivement par des composants (via le `World`) ou des ressources (cf. future ADR-005) — jamais par une variable conservée d'un appel à l'autre au sein du système lui-même. Ceci découle du principe « propriétaire unique » déjà posé en `02-concepts.md` : un état caché dans un système serait une donnée sans propriétaire identifiable dans la chaîne d'ownership déjà établie.

### 3. Un système ne décide jamais quand il s'exécute
Le système expose uniquement son contrat (ce qu'il lit, écrit, produit). L'ordre, les dépendances, le parallélisme, l'appartenance à une phase ou à un type de phase (`01-runtime-ecs.md`, section 5) appartiennent exclusivement au Modèle d'exécution.

Conséquences directes :
- un système n'appelle jamais un autre système ;
- un système ne s'enregistre pas lui-même dans un ordonnancement ;
- un système ne demande jamais explicitement à s'exécuter avant ou après un autre.

Cette séparation stricte est ce qui permettra, dans la future ADR sur le Modèle d'exécution, de construire un ordre de traitement à partir des seuls contrats déclarés — sans jamais modifier les systèmes eux-mêmes.

### 4. Déterminisme
À contrat identique et données d'entrée identiques, un système produit toujours la même sortie déclarée (données transitoires écrites, commandes émises). Ceci découle de l'invariant de déterminisme déjà posé en `00-principes.md`.

### 5. Isolation de l'unité de calcul et principe de localité
> Chaque élément traité par une itération constitue une unité indépendante de calcul.

Le traitement d'un élément par un système ne dépend jamais de l'ordre ni du résultat du traitement d'un autre élément de la même itération. Ce n'est pas une propriété recherchée en vue du parallélisme : le parallélisme intra-système (`rayon::par_iter`, déjà acté en `01-runtime-ecs.md`) en est une simple conséquence, jamais l'inverse. Un système qui viole cette isolation (par exemple en accumulant un état entre deux éléments traités successivement) viole en réalité le point 2 (absence d'état caché).

> Un système ne raisonne que sur les données explicitement fournies par son interface déclarative.

Aucune recherche globale, aucun singleton caché, aucun registre implicite, aucun accès direct au `World` en dehors de ce qui est déclaré. Ce principe de localité ferme la porte à toute réintroduction déguisée d'une logique orientée objet — un système ne « va pas chercher » une donnée par un autre chemin que celui explicitement injecté par son interface déclarative.

### 6. Interchangeabilité par contrat
> Deux systèmes ayant le même contrat sont interchangeables.

Le Modèle d'exécution (ADR-004) ne connaît jamais l'identité d'un système, seulement son contrat. Un `MovementSystemV2` peut ainsi remplacer `MovementSystem` sans qu'aucune autre partie du runtime n'ait besoin d'être modifiée. Cette propriété découle directement de la séparation déjà établie entre contrat, exécution et implémentation.

## Ce qui n'est pas tranché ici
La syntaxe Rust concrète d'un système (fonction libre avec paramètres génériques, trait dédié, macro de déclaration) est une décision d'implémentation, à documenter lors de l'écriture du code — tant qu'elle permet la déduction statique du contrat exigée au point 1.

## Rôle du système dans l'architecture
> Le système est l'unique lieu où réside la logique métier. Toutes les autres briques du runtime (`World`, stockages, Queries, Modèle d'exécution, Command Buffer) ne sont que des infrastructures permettant d'exécuter cette logique de manière déterministe.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Le contrat d'un système est entièrement déductible de son interface déclarative | Invariant |
| Aucun état persistant caché | Invariant |
| Un système ne décide jamais quand il s'exécute (pas d'appel, d'auto-enregistrement, d'ordre demandé) | Invariant |
| Déterminisme (mêmes entrées déclarées → mêmes sorties déclarées) | Invariant |
| Chaque élément traité constitue une unité indépendante de calcul | Invariant |
| Principe de localité (aucune donnée hors de l'interface déclarative) | Invariant |
| Interchangeabilité par contrat (identité du système ignorée par l'exécution) | Invariant (conséquence) |
| Représentation concrète de l'interface déclarative (signature Rust, macro...) | Hors périmètre — implémentation |

## Suite prévue
Cette ADR clôt le niveau « contrat ». Les ADR suivantes s'appuient dessus sans le rouvrir :
- ADR-004 — Modèle d'exécution (construction de l'ordre à partir des contrats, dépendances, phases, validation)
- ADR-005 — Ressources (`Resource<T>`, singleton, durée de vie, accès)
- ADR-006 — Command Buffer (mutations structurelles, validation)

Une fois ces documents rédigés, la colonne vertébrale du runtime ECS sera close : les domaines périphériques (Renderer, Assets, Audio, Navigation, Sauvegarde...) deviendront des consommateurs de ce socle, sans le modifier.
