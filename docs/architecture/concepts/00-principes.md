# 00 — Principes fondateurs — Verbum

## Statut
**Normatif.** Date de rédaction : 31 juillet 2026. Ce document ne décrit pas un état du projet ; il définit une norme. Il fixe la philosophie générale du moteur. Toute ADR ultérieure doit être compatible avec ces principes ; en cas de conflit apparent, c'est la nouvelle décision qui doit être reformulée, pas ce document qui doit être amendé sans discussion explicite.

## Contexte
**Verbum** est un moteur de jeu 2D, écrit en Rust, conçu selon des principes DOD (Data-Oriented Design), avec un pipeline de production de données Ahead-Of-Time (AOT) et un modèle ECS (Entity-Component-System) au runtime. Le nom reflète le soin porté à la construction d'un vocabulaire architectural rigoureux tout au long de ce corpus (`02-concepts.md`).

## Invariants de philosophie générale

1. **AOT comme principe fondateur.** Un outil externe (la Forge) est seul autorisé à lire et parser des formats de configuration humains (YAML, JSON, etc.). Il valide la cohérence des données de design, convertit les unités humaines (secondes, etc.) en unités de simulation déterministes (ticks), et produit des données binaires ou du code généré. Le runtime ne fait que consommer ces données précompilées — aucun parsing, aucune validation de cohérence métier au runtime.

2. **DOD strict — séparation données / logique.** Les composants sont des structures pures (`Copy`/`Clone`), sans méthode ni comportement. Toute logique réside dans des systèmes qui transforment des données en données.

3. **Déterminisme de la simulation.** Pour une même séquence d'entrées, le moteur doit produire exactement la même séquence d'états. Ceci exclut notamment : le `deltaTime` flottant pour les mécaniques de jeu (remplacé par des ticks fixes), tout ordre d'exécution non spécifié explicitement, et toute forme de scheduling dynamique dont l'issue dépendrait de facteurs externes (timing du système d'exploitation, etc.).

4. **Contrôle explicite des allocations.** L'objectif structurant est **zéro allocation sur le hot path** (la boucle de simulation par frame), et non zéro allocation au runtime au sens absolu. Les allocations sont acceptables hors de cette boucle : chargement de carte, génération procédurale, sauvegarde, streaming d'assets.

5. **Runtime réduit à de la consommation de données précompilées.** Le moteur au runtime ne fait que des accès directs (indices entiers, structures `#[repr(C)]` castées depuis des buffers) — jamais de résolution de références symboliques, de recherche par nom, ou de logique de validation de configuration.

## Règle méta-architecturale

> **Une ADR ne doit figer que des invariants structurels.** Les mécanismes qui permettent de satisfaire ces invariants restent des décisions d'implémentation tant qu'ils demeurent substituables sans modifier les autres documents d'architecture.

Cette règle gouverne la manière dont chaque document doit être écrit et relu. Exemples déjà tranchés selon cette grille :

| Sujet | Statut |
|---|---|
| Layout binaire interne de `EntityId` (u32/u32, 24/40...) | Implémentation |
| Politique FIFO/LIFO de réutilisation d'index | Implémentation |
| Mécanisme de reset de l'état transitoire (memset, double buffering, génération monotone...) | Implémentation |
| Stockage d'une donnée transitoire (composant, ressource, buffer SoA...) | Implémentation |
| Mécanisme concret de détection de cycle dans un graphe de relations | Implémentation |
| Un système ne connaît que des données et pas de catégories métier | Invariant |
| Le World ne connaît que des entités, des composants et des commandes | Invariant |
| Une phase est l'unité atomique de cohérence du World | Invariant |

## Principe des frontières de connaissance

> **Chaque domaine du moteur est défini autant par ce qu'il connaît que par ce qu'il lui est interdit de connaître.**

Ce principe n'est pas un invariant technique supplémentaire à appliquer localement : c'est une lecture unificatrice de la plupart des invariants déjà posés dans ce corpus, qui explique pourquoi ils tiennent ensemble plutôt que d'apparaître comme une collection de règles indépendantes. Chaque frontière du moteur interdit une catégorie de connaissance précise à l'un des deux domaines qu'elle sépare :

| Frontière | Ce qui est interdit de connaître |
|---|---|
| Système ↔ autres systèmes | Un système ignore l'existence, l'ordre, et le contenu des autres systèmes (`01-runtime-ecs.md` ; ADR-003) |
| `World` ↔ sémantique métier | Le `World` ne connaît aucune politique de cycle de vie, aucune catégorie de gameplay (`01-runtime-ecs.md`) |
| Modèle architectural ↔ structure interne | Le modèle ne classifie jamais le contenu interne d'une donnée (`02-concepts.md`) |
| Backend ↔ modèle architectural | Le Backend ignore entités, composants, ressources, systèmes, phases, contrats (ADR-R03) |
| Shell ↔ sémantique métier | Le Shell traduit des événements bruts sans jamais les interpréter au sens métier (ADR-S01) |
| Toute infrastructure ↔ itération en cours | Aucune infrastructure ne peut modifier une itération logique déjà en cours (ADR-S01) |

Ce principe n'ajoute aucune règle nouvelle : il rend explicite ce que chaque ADR de ce corpus a déjà découvert indépendamment. Il sert de test de lecture pour toute ADR future : décrire un domaine ne consiste pas seulement à énumérer ce qu'il fait, mais aussi, et avec la même rigueur, ce qu'il lui est structurellement interdit de savoir.

### Test méthodologique pour toute nouvelle ADR
Ce principe se traduit en un réflexe de rédaction, à appliquer systématiquement lors de l'ouverture d'un nouveau domaine :

1. Que possède ce domaine ?
2. Que produit-il ?
3. Qui est autorisé à consommer cette production ?
4. Que lui est-il interdit de connaître ?
5. Quelle frontière protège cette ignorance ?

Les deux premières questions décrivent des responsabilités positives ; les trois dernières décrivent une séparation de connaissance et de consommation — et c'est cette dernière triade qui, dans ce corpus, s'est révélée à chaque fois porteuse de l'invariant le plus durable (la passivité du Backend, l'ignorance du `World` vis-à-vis de la plateforme, l'ignorance du Shell vis-à-vis du métier, l'indépendance mutuelle des systèmes). Une ADR qui ne répond qu'aux deux premières questions décrit une architecture par responsabilités ; une ADR qui répond aux cinq décrit une architecture par frontières de connaissance — plus difficile à faire dériver avec le temps.

La troisième question (« qui consomme ? ») n'introduit aucun principe nouveau — c'est une conséquence déjà posée par ADR-004, ADR-005, ADR-006 et la trilogie Renderer : toute production possède un propriétaire de consommation unique (le `World` pour le Command Buffer, le système de résolution pour un accumulateur, le système de composition pour les contributions de rendu, le Backend pour la `FrameRepresentation`). Elle reste néanmoins une question de relecture précieuse : si plusieurs domaines semblent devoir consommer directement une même production, c'est le signe qu'il manque soit une frontière, soit une étape de composition ou de résolution — un signal d'alerte à vérifier avant de considérer une nouvelle ADR comme close.

### Heuristique de détection d'un domaine manquant
> Si deux domaines doivent se connaître mutuellement pour fonctionner, c'est souvent le signe qu'une frontière est mal placée — ou qu'il manque un troisième domaine qui les relie sans qu'ils aient à se connaître.

Cette heuristique ne crée aucun invariant supplémentaire : c'est un outil de découverte, pas une règle à appliquer localement. Elle a produit son résultat le plus net lors de la question initialement mal posée « Renderer ou ECS, lequel pilote l'autre ? » — la réponse n'a été ni l'un ni l'autre, mais l'introduction du Shell, précisément pour que les deux n'aient jamais à se connaître (ADR-S01). Face à une future tentation de faire connaître deux domaines l'un à l'autre pour résoudre un problème de coordination, cette heuristique invite à chercher d'abord le domaine tiers manquant plutôt qu'à créer un couplage direct.

## Grammaire des ADR
Les ADR de domaine (Renderer, Shell, Assets, Sauvegarde, et celles à venir) répondent, de fait, aux mêmes six questions, indépendamment les unes des autres — ce constat a posteriori est ici formalisé comme structure attendue pour toute future ADR de domaine :

1. Quel est le contrat du domaine ?
2. Qui possède quoi ?
3. Quelles frontières s'appliquent (compilation, strate, connaissance) ?
4. Quelles garanties sont offertes au reste du moteur ?
5. Qu'est-ce qui est explicitement exclu du périmètre du domaine ?
6. Quelles questions sont volontairement laissées à l'implémentation ?

Cette grammaire n'est pas un simple gabarit de mise en forme : c'est un test de maturité. Si l'une des six questions ne peut pas être renseignée lors de la rédaction d'une ADR, c'est le signe que le domaine n'est pas encore assez compris pour être figé — mieux vaut alors produire une page d'axes (sans décision de mécanisme) et la faire relire avant de rédiger l'ADR elle-même, comme cela a été fait pour Assets et pour la Sauvegarde.

## Modèles écartés et justifications

| Concept écarté | Écosystème d'origine | Justification du rejet |
|---|---|---|
| Garbage Collector | .NET / C# | Pauses imprévisibles sur le hot path, anti-déterministe. |
| Object Pools explicites | MonoGame / C# | Indirection inutile ; un Sparse Set agit nativement comme un pool par simple écrasement de slot. |
| Interfaces / vtable (polymorphisme dynamique) | C# orienté objet | Bloque la prédiction de branchement et l'inlining ; remplacé par des tagged unions (`enum` Rust) résolues au compile-time. |
| Allocations dynamiques dans les composants (`Vec`, `String` en champ direct) | Prototypes usuels | Casse la contiguïté mémoire (cache miss systématique). Remplacé par des index externes ou tableaux inline de taille fixe. |
| Temps en `deltaTime` flottant pour les mécaniques de jeu | Approche naïve | Dérives d'arrondi, non-déterminisme. Remplacé par des ticks fixes générés à la compilation AOT. |

## Ce que ce document ne couvre pas

Le modèle d'exécution concret (identité des entités, cycle de vie, relations, flux de données, scheduler) est spécifié dans `01-runtime-ecs.md`. Les choix de crates et bibliothèques concrètes font l'objet d'ADR indépendantes (`ADR-00X`), qui doivent être compatibles avec les invariants posés ici et dans `01-runtime-ecs.md`, mais qui restent, par nature, des décisions d'implémentation substituables.
