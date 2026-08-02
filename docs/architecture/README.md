# Verbum — Index architectural

Ce document ne pose aucune nouvelle règle. Il sert de carte de navigation pour quiconque — développeur, futur collaborateur, ou IA — aborde le corpus documentaire de Verbum. Commencer la lecture ici.

## Vision
Verbum est un moteur de jeu 2D top-down pixel art (RPG), écrit en Rust, selon des principes DOD, avec un pipeline de production de données Ahead-Of-Time (AOT) et un modèle ECS au runtime. Le nom reflète le soin porté à la construction d'un vocabulaire architectural rigoureux — le corpus s'est progressivement révélé être moins une collection de décisions qu'un système axiomatique : un petit nombre de primitives, et une méthode pour vérifier que tout nouveau domaine s'y réduit sans l'étendre.

## Méthode : réfutation plutôt que conception
À partir du domaine Assets, chaque nouvelle ADR de domaine n'a plus cherché à *concevoir* une solution, mais à *tester* si les invariants déjà posés y suffisaient. Le raisonnement implicite était systématiquement le même : voici un domaine réputé difficile dans un moteur ECS — est-il capable de casser les invariants existants ?

> Les ADR de domaine ne cherchent pas à introduire de nouveaux concepts. Elles confrontent les invariants existants à un domaine réputé difficile afin de déterminer si le langage architectural est expressif, ou s'il doit être étendu.

C'est cette discipline — plus proche d'une tentative de réfutation que d'une conception incrémentale — qui explique pourquoi la majorité des ADR de ce corpus concluent par une instanciation plutôt que par une invention. Voir la section « Comment évaluer une nouvelle idée », en fin de document, pour la méthode formalisée.

### Chronologie des découvertes
Cette table rend visible que le corpus n'a pas été écrit d'un bloc : il a été mis à l'épreuve, étape par étape, chaque domaine testant la suffisance de ce qui avait déjà été posé.

| Étape | Résultat |
|---|---|
| Runtime ECS (`01-runtime-ecs.md`, ADR-001 à 006) | Définition du langage de base. |
| Renderer (ADR-R01, ADR-R02) | Découverte de la Structure interne. |
| Shell (ADR-S01) | Découverte de la Plateforme. |
| Assets (ADR-A01) | Première preuve de fermeture. |
| Sauvegarde (ADR-SV01) | Deuxième preuve de fermeture. |
| Réseau (ADR-N01) | Troisième preuve de fermeture. |
| Audio (ADR-AU01) | Quatrième preuve de fermeture. |
| Scripts (ADR-Sc01) | Cinquième preuve de fermeture. |

## Parcours de lecture conseillé
1. `00-principes.md` — philosophie, règle méta-architecturale, frontières de connaissance, grammaire des ADR.
2. `01-runtime-ecs.md` — modèle d'exécution normatif (identité, cycle de vie, relations, flux de données, phases).
3. `02-concepts.md` — vocabulaire officiel (Ubiquitous Language), modèle général de donnée, grille des trois strates.
4. `adr/ADR-001` à `ADR-006` — noyau ECS (Sparse Set, Query, Contrat de système, Modèle d'exécution, Ressources, Command Buffer).
5. `adr/ADR-R01` à `ADR-R03` — Renderer (découverte de la structure interne et de la frontière Plateforme).
6. `adr/ADR-S01` — Shell (découverte du troisième domaine, orchestrateur des infrastructures).
7. `adr/ADR-A01`, `ADR-SV01`, `ADR-N01`, `ADR-AU01`, `ADR-Sc01` — domaines périphériques (Assets, Sauvegarde, Réseau, Audio, Scripts), lisibles dans n'importe quel ordre entre eux, tous postérieurs au noyau et au Shell.

## Concepts fondamentaux
Résumés ici ; définition complète dans `02-concepts.md`.

| Concept | Rôle en une phrase |
|---|---|
| `EntityId` | Identifiant opaque sans sémantique métier. |
| `World` | Seule autorité sur la création/destruction d'entités et l'application des mutations structurelles. |
| Composant | Donnée de cardinalité 0..N, adressée par `EntityId`. |
| Ressource | Donnée de cardinalité 1, identifiée par une clé unique. |
| Système | Fonction de transformation déclarative ; ne décide jamais quand il s'exécute. |
| Phase | Unité atomique de cohérence du `World` ; se termine par un point de validation. |
| Modèle d'exécution | Ordre construit une fois, à partir des contrats déclarés, immuable pendant la simulation. |
| Command Buffer | Mutations structurelles accumulées librement, résolues en un point unique par le `World`. |
| Forge | Outil AOT ; seule autorisée à parser des formats humains et à produire des artefacts statiques. |
| Shell | Possède le cycle de vie du processus, le temps réel, les dépendances de plateforme ; assemble et orchestre les infrastructures. |
| Structure interne | Ce que le modèle architectural ne décrit jamais — le contenu interne d'une donnée. |

## Invariants majeurs (à ne jamais violer sans révision explicite du document source)
- Zéro allocation sur le hot path ; ticks fixes ; déterminisme (`00-principes.md`).
- Une entité est un handle opaque ; seul le `World` en crée (`01-runtime-ecs.md`).
- Produire est libre ; résoudre est unique (`01-runtime-ecs.md`).
- Chaque domaine est défini autant par ce qu'il connaît que par ce qu'il lui est interdit de connaître (`00-principes.md`).
- Un système ne décide jamais quand il s'exécute ; son contrat est déductible de son interface déclarative (ADR-003).
- Le Shell traduit, il n'interprète jamais (ADR-S01).
- Aucune infrastructure ne peut modifier une itération logique en cours (ADR-S01).

## Primitives vs preuves de fermeture
Distinction structurante du corpus : une **primitive** enrichit réellement le pouvoir d'expression du langage architectural ; une **preuve de fermeture** démontre que ce langage suffit déjà à absorber un nouveau domaine, sans l'étendre.

| Catégorie | Documents | Ce qu'ils apportent |
|---|---|---|
| **Primitives** | `01-runtime-ecs.md`, `02-concepts.md`, ADR-001 à 006 | Le noyau ECS lui-même : identité, stockage, requête, contrat de système, modèle d'exécution, ressources, command buffer. |
| **Primitive** | ADR-R01, ADR-R02 | Découverte de la frontière modèle architectural / structure interne, et du principe de circulation. |
| **Primitive** | ADR-S01 | Découverte du Shell comme troisième domaine (Plateforme), orchestrateur sans connaissance métier. |
| **Preuve de fermeture** | ADR-R03, ADR-A01, ADR-SV01, ADR-N01, ADR-AU01, ADR-Sc01 | Chacune démontre qu'un domaine entier s'exprime par instanciation des primitives ci-dessus, sans vocabulaire nouveau. |

### Table de synthèse par domaine

| Domaine | Nouveau concept ? | Concept mobilisé |
|---|---|---|
| Renderer | ✅ | Structure interne / principe de circulation |
| Shell | ✅ | Shell (Plateforme) |
| Assets | ❌ | Forge + Ressources |
| Sauvegarde | ❌ | Ressources + principe de circulation (extraction/reconstruction) |
| Réseau | ❌ | Shell + principe de circulation |
| Audio | ❌ | Shell + données transitoires + « produire est libre, résoudre est unique » |
| Scripts | ❌ | Assets + Contrat de système + Forge (validation) |

Cette table raconte, en une ligne par domaine, l'histoire de la fermeture conceptuelle de Verbum : deux primitives découvertes après le noyau (Structure interne, Shell), cinq domaines refermés sans extension.

## Liste des ADR (one-liner de la décision majeure)

| ADR | Décision majeure |
|---|---|
| ADR-001 — Sparse Set | Chaque composant a un stockage indépendant, isolé, sans connaissance mutuelle entre stockages. |
| ADR-002 — Query Model | Une Query est une vue passive, spécialisée statiquement, qui ne produit jamais de donnée. |
| ADR-003 — Contrat de système | Un système est une fonction déclarative dont le contrat est déductible de son interface ; il ne décide jamais quand il s'exécute. |
| ADR-004 — Modèle d'exécution | L'ordre d'exécution est construit une fois à partir des contrats déclarés, jamais décidé par les systèmes eux-mêmes. |
| ADR-005 — Ressources | Une Ressource est une donnée de cardinalité 1, instanciant exactement les mêmes contrats qu'un composant. |
| ADR-006 — Command Buffer | Les mutations structurelles s'accumulent librement et sont résolues en un point unique par le `World`. |
| ADR-R01 — Modèle de donnée de rendu | Une contribution de rendu est le contenu interne d'une Ressource transitoire, jamais une nouvelle catégorie de donnée. |
| ADR-R02 — Pipeline de production | Les systèmes composent librement une représentation de frame unique, sans jamais se connaître entre eux. |
| ADR-R03 — Backend | Le backend graphique est un pur consommateur passif, symétrique au `World` vis-à-vis du Command Buffer. |
| ADR-S01 — Shell | Le Shell possède le cycle de vie du processus et le temps réel ; il traduit sans jamais interpréter, et n'est jamais informé en retour pendant une itération. |
| ADR-A01 — Assets | Un asset est une Ressource produite par la Forge, adressée par identifiant opaque, jamais découverte dynamiquement. |
| ADR-SV01 — Sauvegarde | Une sauvegarde persiste la sémantique de l'état, jamais sa représentation interne. |
| ADR-N01 — Réseau | Chaque instance de `World` reste locale et déterministe ; le réseau ne fait que garantir la bonne séquence d'Acquisition. |
| ADR-AU01 — Audio | Le Shell ne décide jamais une transition audio — seulement son application, une fois décidée en amont. |
| ADR-Sc01 — Scripts | Un script n'a jamais de pile d'exécution suspendue hors du `World` ; l'interface de son interpréteur est fixe, jamais variable selon le script chargé. |

## Comment évaluer une nouvelle idée
Cette méthode a été appliquée, de facto, tout au long de la construction de ce corpus. La voici formalisée, pour qu'elle continue à s'appliquer à mesure que Verbum grandit :

1. La proposition introduit-elle un nouveau type d'état, en dehors des catégories déjà posées (donnée persistante, transitoire, mutation structurelle) ?
2. Introduit-elle un nouveau modèle d'exécution, ou un acteur qui échapperait au modèle d'exécution déjà construit (ADR-004) ?
3. Introduit-elle une nouvelle frontière, en dehors des trois déjà nommées (compilation, strate, connaissance) ?
4. Peut-elle être exprimée comme une instanciation d'un invariant existant plutôt que comme une règle nouvelle ?
5. Si elle exige réellement un nouveau concept, celui-ci est-il orthogonal aux concepts existants — ou une reformulation déguisée de l'un d'entre eux ?

Si les réponses à 1, 2 et 3 sont négatives, et que 4 est positive : la proposition est une preuve de fermeture de plus, à documenter comme telle. Si une réponse positive à 1, 2 ou 3 résiste à la tentative de réduction (question 4), c'est le signal légitime d'une nouvelle primitive — à traiter avec la même rigueur que la découverte de la structure interne ou du Shell : une page d'axes d'abord, sans décision de mécanisme, confrontée à plusieurs modèles concurrents, avant toute rédaction d'ADR.
