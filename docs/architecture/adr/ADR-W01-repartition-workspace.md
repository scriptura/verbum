# ADR-W01 — Répartition des responsabilités entre Engine, Extensions et Games

## Statut
**Proposé** — non adopté. Date de rédaction : 7 novembre 2026. Issu de la note préparatoire « Répartition des responsabilités entre Engine, Extensions et Games » (r2). La numérotation `W01` (Workspace) est provisoire, à confirmer.

Cette ADR ne modifie aucun document adopté. Là où une tension avec un texte existant a été relevée, elle est signalée dans « Points à arbitrer » et non corrigée.

## Contexte
Le workspace est organisé en trois répertoires, `engine/`, `extensions/` et `games/`. Cette organisation traduit une répartition de responsabilités que le corpus n'énonce nulle part. Aucune ADR ne dit selon quels critères une crate appartient à l'un ou à l'autre, ni dans quel sens les dépendances peuvent aller. Le cas Scripts (ADR-Sc01 et étape 6) fait apparaître une capacité dont les responsabilités traversent les trois répertoires, et l'étape 6 a laissé en suspens la place des tests d'intégration moteur ↔ extension.

**Collision de vocabulaire.** `02-concepts.md` emploie « strate » pour la grille Plateforme / Modèle architectural / Structure interne. Les trois répertoires du workspace ne sont pas cette grille. Pour éviter qu'« une frontière de strate » désigne deux choses, cette ADR appelle les trois répertoires des **niveaux de composition** (niveau Engine, niveau Extension, niveau Application). Le mot « strate » reste réservé à `02-concepts.md`. Le répertoire `games/` est appelé niveau Application, parce que son rôle est la composition d'une application, et que `games/` désigne seulement son emplacement actuel.

## Test de réduction (méthode « Comment évaluer une nouvelle idée »)
1. *Nouveau type d'état ?* Non. L'ADR ne définit aucune donnée.
2. *Nouveau modèle d'exécution ou acteur échappant à ADR-004 ?* Non. Un niveau ne change pas l'ordonnancement.
3. *Nouvelle frontière ?* Non. « Engine ne peut pas nommer une Extension » est un interdit de la forme « X ne peut pas nommer Y ». C'est une instanciation de la frontière de connaissance (`00-principes.md`), déjà encodée dans le graphe de dépendances des crates par ADR-000, point 3.
4. *Exprimable comme instanciation ?* Oui, pour la direction des dépendances (frontière de connaissance, ADR-000) comme pour la présence du support d'un format dans Engine. Pour un asset, ADR-A01 point 2 impose que seules la Forge et le chargement runtime parsent et lisent. Le support d'un format spécialisé dans Forge et Assets est donc une conséquence d'ADR-A01, pas un choix du workspace.
5. *Concept nouveau nécessaire ?* Non. Le « niveau de composition » est une relation d'ordre partiel entre crates, pas une catégorie architecturale du moteur.

**Conclusion.** Aucune primitive, ni du modèle d'exécution, ni du modèle de donnée. Cette ADR n'est pas non plus une preuve de fermeture au sens du README, qui porte sur un domaine du moteur : elle ne s'ajoute ni à la table de synthèse par domaine ni à la chronologie des découvertes. Elle applique la frontière de connaissance et la séparation des responsabilités à l'organisation du workspace. Elle formalise des règles qui n'étaient pas énoncées jusqu'ici : critères d'appartenance, direction des dépendances de production et de build par niveau, dépendances entre Extensions, portée de l'optionalité. La politique des tests et des `dev-dependencies` n'en fait pas partie : c'est une convention d'implémentation (point 6).

## Décision

### 1. Quel est le contrat du domaine ?
Un niveau de composition désigne une **responsabilité dans la construction d'une application Verbum**. Ce n'est ni un environnement d'exécution, ni une fréquence d'usage, ni un degré de réutilisabilité.

- **Engine** possède les infrastructures, les contrats communs et le support de pipeline : ce sur quoi une application et les capacités reposent.
- **Extension** possède une capacité fonctionnelle spécialisée qui exploite les mécanismes d'Engine : ses données, ses transformations et sa sémantique d'exécution. Engine peut connaître les formats et contrats nécessaires à ses propres pipelines de production et de lecture ; il ne dépend ni de l'implémentation des Extensions ni de leur sémantique d'exécution.
- **Application** possède les choix de composition et les comportements propres à une application concrète.

Les niveaux sont ordonnés par dépendance : Application → Extension → Engine, avec les raccourcis Application → Engine. Les dépendances Extension → Extension sont autorisées sous les conditions du point 3.

**Critère d'appartenance.** Il combine trois questions, à poser dans cet ordre :

1. *Nature de la responsabilité.* La crate implémente-t-elle une infrastructure (Plateforme, `World` en tant qu'exécutant, stockages, Forge, support d'un format d'artefact), ou fournit-elle des données et des transformations du modèle architectural ? Dans le premier cas, c'est Engine. Dans le second cas, c'est une Extension si la capacité est adoptable indépendamment d'un jeu, et l'Application si elle exprime un choix propre à un jeu.
2. *Placement d'un contrat partagé.* Un type, un format ou un contrat nommé par plusieurs crates appartient au niveau le plus bas parmi les crates qui doivent le nommer pour remplir leur propre responsabilité, c'est-à-dire au niveau auquel toutes ces crates peuvent accéder sans inverser la direction des dépendances. « Doit » s'entend comme une obligation, pas comme une commodité : la réutilisation d'un type par plusieurs crates ne le promeut pas à elle seule. Par exemple, le format `VSCR` est nommé par la Forge (production) et par Assets (lecture) : il est donc en Engine. L'état d'exécution d'un script n'est nommé que par l'interpréteur et l'adaptateur applicatif : il est donc en Extension.
3. *Test d'indépendance.* Les crates d'Engine se construisent, en production et en build, sans aucune Extension. Les crates d'une Extension se construisent, en production et en build, sans aucune Application.

Ni la réutilisabilité, ni le caractère `no_std`, ni la fréquence d'exécution, ni la distinction build-time / runtime ne sont des critères d'appartenance.

### 2. Qui possède quoi ?
**Support d'un format ≠ possession de la capacité.** Pour une capacité reposant sur un asset, trois responsabilités distinctes se répartissent, avec trois portées de validation :

| Responsabilité | Niveau | Portée |
|---|---|---|
| Production du format, validation du programme dans son ensemble (analyses globales sur le contenu produit) | Engine (Forge, build-time) | Cohérence de l'artefact au regard des règles de son format |
| Prise en charge du format binaire à la lecture, intégration au pipeline de chargement | Engine (Assets) | Conformité du contenu lu : bien-formé au sens du type décodé |
| Sémantique d'exécution : état d'instance, transitions, accumulation | Extension | Application du contenu à l'exécution |
| Raccordement au Scheduler, déclaration du contrat de données du système, intégration au modèle ECS | Application | Composition |

Le contrôle de conformité à la lecture porte sur ce que le décodeur doit garantir pour que la valeur décodée respecte les invariants de son propre type. Il ne s'appuie ni sur un autre asset, ni sur un état runtime, ni sur l'intention de design. Il se distingue de la validation globale de la Forge, qui reste son seul propriétaire, et de la sémantique d'exécution, qui reste à l'Extension.

**Deux compositions distinctes, jamais confondues.** Le Shell est l'unique point de composition des infrastructures de la Plateforme (ADR-S01, point 7). L'Application compose les systèmes et leurs contrats, que le modèle d'exécution ordonne ensuite une fois pour toutes (ADR-004). Une infrastructure de la Plateforme est, par le critère du point 1, de nature Engine. Le cas d'un adaptateur spécialisé que le Shell assemblerait mais qu'une Extension fournirait n'est pas tranché ici (cf. Points à arbitrer).

**Une capacité peut traverser plusieurs niveaux.** La règle « toute la fonctionnalité dans une seule strate » est rejetée (cf. Modèles écartés).

### 3. Quelles frontières s'appliquent ?
Frontière de connaissance, instanciée par le graphe de dépendances des crates (ADR-000, point 3) :

| Source | Peut dépendre de | Ne peut pas nommer |
|---|---|---|
| Engine | Engine | Extension, Application |
| Extension | Engine, autre Extension | Application |
| Application | Engine, Extension | — |

- **« Nommer »** désigne la dépendance à une crate et à son API. Ce n'est pas la connaissance d'un format de données : Forge et Assets connaissent `VSCR` sans nommer `verbum-script`.
- **Entre Extensions**, la dépendance est autorisée à condition d'être acyclique et de ne pas renverser la direction générale. Elle rend l'optionalité transitive : adopter l'Extension A impose d'adopter B si A dépend de B.
- **Graphe de production et graphe de build** (`dependencies`, `build-dependencies`) : la table ci-dessus s'applique strictement.
- **Graphe de développement** (`dev-dependencies`) : hors invariant. Une dépendance de développement ne vaut jamais preuve d'une dépendance fonctionnelle de production, et la table ci-dessus ne lui est pas appliquée. Elle n'est pas ignorée pour autant : sa politique est une convention d'implémentation, formulée au point 6.
- **Données contractuelles venant d'une Extension** : quand une crate d'Engine doit se conformer à un contrat défini en aval (ex. la Forge valide un script contre l'ensemble de données accessible à l'interpréteur, ADR-Sc01 point 4), ce contrat lui parvient comme donnée d'entrée de la chaîne de production, jamais par dépendance de crate.

### 4. Quelles garanties sont offertes au reste du moteur ?
- Engine est complet sans Extension : ses crates se construisent, en production et en build, sans elle.
- **Le niveau ne qualifie pas le régime d'exécution.** Le noyau d'ADR-000 est défini par le graphe d'appel de `world.execute()`, pas par un niveau. Une Extension atteignable depuis `world.execute()` appartient au noyau, et la garantie sans allocation et son audit de dépendances s'appliquent à elle comme à tout code du noyau. Réciproquement, Engine contient du code hors noyau (Forge, chargement, Shell). Le graphe d'appel détermine l'appartenance au noyau ; le graphe Cargo détermine ce que l'audit doit couvrir. L'audit d'ADR-000 porte sur l'arbre de dépendances de crates : une crate atteinte par le noyau (ex. Assets, depuis un interpréteur) doit être elle-même sans `std` ni `alloc` telle qu'elle est compilée pour le noyau, ou voir sa partie lecture isolée par feature ou par crate. Une séparation par modules ne suffit pas tant que la crate compilée contient du code `std` ou `alloc`.
- **Optionalité.** Quatre sens sont distingués : fonctionnelle (l'application adopte ou non la capacité), de dépendance (aucune dépendance de production sur la crate), de compilation (le code peut être exclu d'une configuration), de présence dans le binaire (le support peut être absent du binaire final). Cette ADR ne garantit que les deux premiers. Le support d'un format reste présent dans Engine, car ADR-A01 l'y place. Les deux autres sens ne sont ni garantis ni exclus (cf. Points à arbitrer).
- **Aucune règle de dépendance Cargo ne dépend du chemin physique.** Déplacer une crate d'un répertoire à un autre ne change pas sa responsabilité ; seul un changement de responsabilité change son niveau.

### 5. Qu'est-ce qui est explicitement exclu du périmètre ?
- Engine défini comme « le code partagé par plusieurs jeux » ou comme « tout ce qui participe au tick ».
- Extension entendue comme plugin dynamique, enregistrement par registre ou découverte à l'exécution. L'Extension est une crate compilée et composée statiquement, ce qui est cohérent avec l'ordre d'exécution construit une fois (ADR-004), l'absence de scheduling dynamique (`00-principes.md`) et l'absence de découverte dynamique (ADR-A01).
- Games réduit à des données seules : l'Application contient du code d'intégration.
- Interdiction générale des dépendances entre Extensions.
- Toute règle imposant qu'une capacité tienne dans un seul niveau.
- Toute règle imposant l'absence de support d'un format dans Engine quand l'Extension correspondante n'est pas adoptée.

### 6. Quelles questions sont volontairement laissées à l'implémentation ?
- La liste des crates, leurs noms Cargo et leurs chemins physiques : ils relèvent d'un document descriptif du workspace, distinct de cette ADR.
- L'outillage qui vérifie le graphe (script CI, `cargo metadata`, `cargo deny`, autre).
- Les features Cargo et la compilation conditionnelle.
- La politique des `dev-dependencies` et le placement des tests et exemples. Convention recommandée, non normative : une crate Engine n'a pas de `dev-dependency` vers une Extension, afin que ses tests se construisent sans Extension ; une Extension peut en avoir vers Engine ; un test qui compose Engine et une Extension se place au niveau qui possède la composition (l'Application quand elle assemble le scénario) ; les tests du contrat propre d'une crate restent dans cette crate. Cette convention peut être révisée sans modifier cette ADR.
- La forme des données contractuelles transmises à Forge (point 3).

## Modèles écartés

| Modèle | Raison du rejet |
|---|---|
| **A — Encapsulation verticale stricte** (toute la capacité dans Extension) | Pour une capacité à base d'asset, ADR-A01 impose que Forge et chargement runtime, qui sont Engine, produisent et lisent le format. Le modèle A dupliquerait ces mécanismes ou ferait dépendre Engine de l'Extension, en contradiction avec la direction du graphe. |
| **C — Engine conteneur de toutes les capacités d'exécution** | Il assimile le niveau au régime d'exécution. ADR-000 définit le noyau par le graphe d'appel, pas par un niveau, ce qui rend le modèle C inutile à la garantie d'allocation. Il ferait aussi absorber par Engine toute capacité utilisée au tick. |
| **B — Répartition par responsabilité** | **Retenu.** Il instancie ADR-A01, ADR-000 et la frontière de connaissance. |

## Application au cas Scripts (illustrative, non normative)
- `verbum-forge` et `verbum-assets` (Engine) : production, validation globale et lecture conforme du format `VSCR`. Cette répartition est imposée par ADR-A01 et ADR-Sc01 point 4.
- `verbum-script` (Extension) : état d'exécution, émissions, table, `step`. Il ne dépend que d'Assets, sens autorisé, et ne nomme ni `World` ni Scheduler.
- Adaptateur Scheduler (Application) : contrat de données et raccordement.
- Tests U4 : ce sont des tests de composition (Shell, Scheduler, ECS, interpréteur), pas du contrat propre de Shell. Selon la convention du point 6, ils relèvent de l'Application (`games/demo`), qui possède l'adaptateur Scheduler. Les tests propres à Shell, notamment avec closures, restent dans Shell. La dépendance de développement `verbum-shell` → `verbum-script`, confirmée par le manifeste de `verbum-shell`, ne viole aucun invariant de cette ADR : elle écarte seulement la convention recommandée.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Les niveaux désignent des responsabilités de construction, pas des régimes d'exécution | Invariant (instanciation de la séparation des responsabilités) |
| Direction Application → Extension → Engine sur le graphe de production et de build | Invariant (instanciation de la frontière de connaissance et d'ADR-000, point 3) |
| Placement d'un contrat partagé au niveau le plus bas qui doit le nommer | Invariant (conséquence de la direction des dépendances) |
| Support d'un format d'asset spécialisé en Engine, sémantique d'exécution en Extension | Invariant (instanciation d'ADR-A01, point 2, et d'ADR-Sc01) |
| Une capacité peut traverser plusieurs niveaux | Invariant |
| Extension = crate composée statiquement, jamais plugin dynamique | Invariant (instanciation de `00-principes.md`, ADR-004, ADR-A01) |
| Les niveaux sont indépendants du noyau d'ADR-000 (défini par graphe d'appel) | Invariant (précision de vocabulaire) |
| Dépendances Extension → Extension autorisées, acycliques | Décision révisable |
| Une infrastructure de la Plateforme est de nature Engine | Invariant (conséquence du critère du point 1) |
| Politique des `dev-dependencies` et placement des tests et exemples (convention recommandée, point 6) | Hors périmètre — implémentation |
| Optionalité de compilation et de présence binaire | Hors périmètre — non tranché |
| Liste des crates, chemins, outillage, features | Hors périmètre — implémentation |

## Bilan — test à cinq questions

| Question | Réponse |
|---|---|
| Que possède ce domaine ? | Rien au runtime : il organise la construction du code. |
| Que produit-il ? | Une règle d'appartenance et une direction de dépendance vérifiables sur le graphe de crates. |
| Qui consomme cette production ? | Toute crate nouvelle ou déplacée, et l'outillage de vérification du graphe. |
| Que lui est-il interdit de connaître ? | La sémantique métier d'une capacité et le contenu des crates : il ne définit aucune donnée ni aucun système. |
| Quelle frontière protège cette ignorance ? | La frontière de connaissance, encodée dans le graphe de dépendances (ADR-000, point 3). |

## Documents à mettre en cohérence
- `02-concepts.md` : note de désambiguïsation entre « strate » et « niveau de composition » (à arbitrer).
- `README.md` : ADR-W01 y figure comme **proposée**, dans une rubrique distincte de la liste des ADR adoptées, et dans le parcours de lecture. Rien dans « Primitives vs preuves de fermeture », la table de synthèse par domaine, les invariants majeurs ni la chronologie des découvertes.
- ADR-000 : aucune modification. Voir le point 3 ci-dessous.
- Document descriptif du workspace (éventuel, hors de cette ADR) : cartographie des crates et leur niveau.
- `ROADMAP.md` et plan d'étape 6 : chemin `crates/extensions/script/` (vérification de cohérence uniquement).

## Points à arbitrer
1. **Optionalité de compilation et de présence binaire.** Recommandation : ne garantir que l'optionalité fonctionnelle et de dépendance, et traiter la modularité de compilation (feature gating du support de format dans Assets et Forge) comme un besoin à justifier le jour où il apparaît.
2. **Tests U4 et dev-dependency de `verbum-shell` vers `verbum-script` (convention d'implémentation, hors invariant).** Recommandation : placer ces tests dans `games/demo`, qui possède la composition, et retirer la dépendance de développement de `verbum-shell`. Option alternative : la conserver, au prix de la subordination de la construction des tests de Shell à `verbum-script`. Décision d'implémentation, sans effet sur l'ADR. Aucun déplacement n'est effectué ici.
3. **Tension apparente avec ADR-000, point 3.** ADR-000 qualifie le rejet au chargement de « vérification de conformité de format, non validation de cohérence métier ». Les contrôles locaux de `Script::read` (cibles de saut, registres, durée de `Wait` non nulle) relèvent de la conformité au sens de cette ADR (bien-formé du type décodé). La durée de `Wait` est le cas le moins net. Faut-il préciser le critère dans ADR-000, ou laisser le critère de cette ADR faire foi ?
4. **ADR-Sc01 point 4 : « fixe, décidée à la conception du moteur ».** Si le système interpréteur est composé par l'Application, l'ensemble des données accessibles aux scripts est défini à la composition. Faut-il lire « à la conception du moteur » comme « avant l'exécution, jamais selon le script chargé » ? La lecture proposée ne change rien à l'invariant d'ADR-Sc01 ; seul le texte est ambigu.
5. **Tension apparente avec ADR-S01.** « Le Shell est le seul point de composition de la Plateforme » ne doit pas être lu comme excluant la composition applicative des systèmes. La distinction du point 2 est-elle suffisante ? Par ailleurs, une Extension peut-elle fournir un adaptateur spécialisé que le Shell assemble ? Par le critère du point 1, une infrastructure de Plateforme est Engine ; le cas de l'adaptateur n'est pas tranché, et le texte d'ADR-S01 point 7 doit être relu avant toute décision.
6. **Noyau et dépendances d'une Extension.** Le noyau d'ADR-000 inclut les Extensions atteignables depuis `world.execute()`, donc leur arbre de dépendances est audité (`no_std`, sans `alloc`). L'audit portant sur l'arbre de crates, la séparation entre lecture de format et chargement disque doit exister au niveau d'une crate ou d'une feature de `verbum-assets`. À vérifier dans le code réel, non examiné ici.

## Limites de cet examen
Cet examen s'appuie sur les documents fournis (`00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, `scribe-conservateur.md`, `README.md`, ADR-000, ADR-A01, ADR-Sc01) et sur la note r2. Le dépôt n'est pas accessible dans cette session ; la vérification du graphe de dépendances s'appuie sur les fichiers fournis. ADR-S01 n'a pas été fourni : ses références (points 3, 4 et 7) viennent des autres documents.

**Vérifié** le 7 novembre 2026, d'après le `Cargo.toml` racine, le manifeste de `verbum-shell` et `tree.md` :
- `crates/extensions/script` est le seul membre du workspace sous `extensions/` ; `collisions` et `lpc` ne contiennent que des README et des brouillons, et `games/demo` qu'un README ;
- `verbum-shell` a une section `[dependencies]` vide : son graphe de production et de build ne contient aucune Extension, la direction du point 3 est respectée ;
- sa seule dépendance vers une Extension est une `dev-dependency` vers `verbum-script`, ce qui relève de la convention du point 6 et non d'un invariant.

**Non vérifié** (manifestes non fournis) :
- `verbum-script` ne dépend que de `verbum-assets` ;
- aucune autre crate Engine (`ecs`, `schedule`, `render`, `assets`, `forge`, `verbum`) n'a de dépendance normale ou de build vers `verbum-script`. La déclaration de `verbum-script` dans `[workspace.dependencies]` ne renseigne pas sur ses utilisateurs.

Cette ADR ne cartographie pas les crates et ne démontre pas que chacune appartient à Engine. Avant adoption, une vérification suffit : le graphe des dépendances normales et de build respecte la direction du point 3.
