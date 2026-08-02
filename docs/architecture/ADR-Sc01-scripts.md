# ADR-Sc01 — Domaine Scripts

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
La page d'axes a isolé la question centrale de ce domaine : l'état d'exécution d'un script est-il intégralement externalisé dans le `World`, ou existe-t-il un état privé qui survivrait entre deux itérations ? Cette ADR tranche cette question sans équivoque, et statue également sur la variation apparente de l'interface déclarative d'un système interpréteur selon le script chargé — deux points identifiés comme les seuls véritables risques de collision avec ADR-003 et ADR-004.

## Décision

### 1. Quel est le contrat du domaine ?
Un script est un Asset (ADR-A01) — une donnée produite par la Forge, jamais du code connu à la compilation du moteur — interprété par un ou plusieurs systèmes ordinaires qui satisfont intégralement le contrat de système déjà posé (ADR-003). Un script ne possède **jamais** d'état d'exécution privé qui survivrait à une itération : toute information nécessaire à la reprise de son exécution (position dans le programme, variables, délais d'attente) est portée par un composant ou une Ressource, exactement comme n'importe quelle autre donnée persistante ou transitoire du moteur.

### 2. Qui possède quoi ?
- **La Forge** (ou un compilateur de DSL dédié) produit l'artefact exécutable du script — instanciation directe de son rôle déjà défini (`00-principes.md`, ADR-A01).
- **Le ou les systèmes interpréteurs** possèdent l'exécution effective, jamais le script lui-même. Leur interface déclarative est **fixe**, décidée à la conception du moteur, et ne varie jamais selon le script chargé — cf. point 4.
- **Le composant ou la Ressource portant l'état d'exécution d'un script en cours** (position dans le programme, variables, minuterie) a un unique propriétaire en écriture : le système interpréteur qui le fait progresser, exactement comme pour toute donnée persistante ou transitoire (`02-concepts.md`).

### 3. Quelles frontières s'appliquent ?
Les frontières déjà nommées suffisent, sans exception :
- **Frontière de connaissance** : un système interpréteur exécute des instructions déterministes, il n'interprète jamais le sens métier du contenu qu'il joue — un `AudioDiffResolutionSystem` ne sait pas ce qu'est une explosion, un interpréteur de dialogue ne sait pas ce qu'est une intrigue. C'est une transformation de données comme une autre.
- **Frontière de compilation** : précisée par la page d'axes — *build-time* signifie « avant consommation runtime », pas « avant compilation du moteur ». Aucune nouvelle frontière n'a été nécessaire.

### 4. Quelles garanties sont offertes ?

**Réponse à la première question — l'état d'exécution suspendu est interdit sans équivoque.**
> Un script n'a jamais le droit de porter une pile d'exécution suspendue hors du `World`.

Aucune primitive de suspension du langage hôte du moteur (`async`/`await` Rust, thread bloquant, fibre, coroutine de langage) ne peut être utilisée pour représenter une attente ou une pause de script. Toute construction de type « attends 3 secondes », « boucle », « séquence de répliques » s'exprime par un **programme-compteur explicite**, stocké comme un champ ordinaire d'un composant ou d'une Ressource, mis à jour par le système interpréteur à chaque itération où il progresse. Un « yield » devient ainsi littéralement : le système interpréteur cesse d'exécuter des instructions pour cette itération, écrit la position atteinte dans l'état persisté, et reprendra exactement à cette position à l'itération suivante où les conditions de reprise sont remplies (ex: une minuterie logique écoulée). Ce n'est pas une nouvelle mécanique : c'est une application stricte de l'invariant déjà posé — un système n'existe jamais entre deux itérations (ADR-003, point 5) — combinée à l'absence d'état persistant caché (ADR-003, point 2).

**Réponse à la seconde question — l'interface déclarative ne varie jamais selon le script chargé.**
> L'interface déclarative d'un système interpréteur est fixe, décidée à la conception du moteur, et couvre un ensemble borné de composants et de Ressources désignés une fois pour toutes comme « accessibles aux scripts » — jamais un ensemble qui s'élargirait dynamiquement selon le contenu d'un script particulier.

Concrètement : le système interpréteur déclare, comme tout système (ADR-003), un accès fixe à cet ensemble borné — que tel script utilise l'intégralité de cet ensemble ou seulement une fraction ne change rien à l'interface déclarée, exactement comme un système classique peut ignorer une partie des données auxquelles il a accès. La Forge, au moment de compiler chaque script individuel, valide qu'il ne référence jamais une donnée hors de cet ensemble fixe — toute violation est une erreur de compilation du script, détectée avant que le runtime ne le consomme, jamais une découverte pendant l'exécution. Ceci préserve intégralement l'invariant déjà posé en ADR-004 : le modèle d'exécution est construit une fois, à partir de contrats immuables, sans jamais varier selon les données chargées au runtime.

Cette contrainte s'applique à tout mécanisme générique (Modèles 1/3 de la page d'axes). Le Modèle 4 (compilation directe en code Rust généré, sans interpréteur au runtime) y échappe naturellement — chaque script y devient son propre système, avec sa propre interface figée à la compilation du moteur — mais au prix de l'arbitrage de production déjà identifié (recompilation du moteur à chaque modification de contenu).

### 5. Qu'est-ce qui est explicitement exclu ?
- Toute primitive de suspension d'exécution portée par le langage hôte du moteur pour représenter une attente de script ;
- tout accès d'un script à une donnée hors de l'ensemble fixe déclaré par le système interpréteur — détecté et rejeté à la compilation par la Forge, jamais au runtime ;
- toute variation de l'interface déclarative d'un système interpréteur en fonction du script chargé ;
- toute exécution de script en dehors du modèle d'exécution construit (ADR-004) — un interpréteur reste un système parmi d'autres, jamais un second ordonnanceur.

### 6. Quelles questions sont volontairement laissées à l'implémentation ?
- Format concret du bytecode ou du DSL ;
- ensemble concret des composants et Ressources désignés comme « accessibles aux scripts » ;
- mécanisme précis de validation par la Forge ;
- choix entre Modèles 1/3 (interpréteur générique) et Modèle 4 (codegen direct) — arbitrage de production, pas architectural, laissé au développeur ;
- langage source du DSL (dialogue, quête, comportement...).

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Un script est un Asset interprété par un système ordinaire, jamais une nouvelle catégorie architecturale | Invariant (instanciation d'ADR-A01 et ADR-003) |
| Aucune pile d'exécution suspendue hors du `World` — tout état est externalisé dans un composant/Ressource | Invariant (instanciation d'ADR-003, points 2 et 5) |
| Un « yield » s'exprime par un programme-compteur explicite stocké comme champ ordinaire, jamais par une primitive de suspension du langage hôte | Invariant (conséquence directe du point précédent) |
| L'interface déclarative du système interpréteur est fixe, jamais variable selon le script chargé | Invariant (instanciation d'ADR-004) |
| La Forge valide qu'un script ne dépasse jamais l'ensemble de données fixé, à la compilation du script | Invariant (instanciation du rôle de la Forge, `00-principes.md`, ADR-A01) |
| Format du bytecode/DSL, ensemble concret de données scriptables, choix Modèle 1/3/4 | Hors périmètre — implémentation |

## Bilan
Les deux questions posées par Gemini sont tranchées sans ambiguïté, et aucune des deux réponses n'a nécessité l'introduction d'un concept architectural nouveau. L'interdiction de l'état d'exécution suspendu est une application stricte de deux invariants déjà posés en ADR-003. La fixité de l'interface déclarative est une application stricte d'ADR-004, combinée à l'extension du rôle de validation déjà confié à la Forge (ADR-A01) — appliquée ici à la vérification qu'un script respecte les bornes d'accès fixées par son interpréteur, plutôt qu'à la seule cohérence des données de configuration.

Avec ADR-Sc01, les huit domaines de la feuille de route initiale sont clos : noyau ECS, Renderer, Shell, Assets, Sauvegarde, Réseau, Audio, Scripts. Aucun n'a exigé l'introduction d'un concept architectural fondamental au-delà de la découverte de la structure interne (Renderer) et de la nécessité du Shell — les deux seules véritables extensions du langage architectural depuis `01-runtime-ecs.md`.
