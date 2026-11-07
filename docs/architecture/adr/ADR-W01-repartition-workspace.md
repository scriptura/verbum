# ADR-W01 — Répartition des responsabilités entre Engine, Extensions et Games

## Statut
Adopté. Date : 7 novembre 2026. Issu de la note préparatoire « Répartition des responsabilités entre Engine, Extensions et Games » (r2).

Cette ADR ne modifie la portée d'aucune décision adoptée.

## Contexte
Le workspace est organisé en trois répertoires, `engine/`, `extensions/` et `games/`. Cette organisation traduit une répartition de responsabilités que le corpus n'énonçait pas. Aucune ADR ne précise les critères d'appartenance aux trois niveaux de composition ni leur organisation concrète. ADR-000, point 3, établit toutefois déjà le principe général de frontière de connaissance et de dépendance que cette ADR décline à l'organisation du workspace. Le cas Scripts (ADR-Sc01) fait apparaître une capacité dont les responsabilités traversent les trois répertoires.

**Collision de vocabulaire.** `02-concepts.md` emploie « strate » pour la grille Plateforme / Modèle architectural / Structure interne. Les trois répertoires du workspace ne sont pas cette grille. Pour éviter qu'« une frontière de strate » désigne deux choses, cette ADR appelle les trois répertoires des **niveaux de composition** (niveau Engine, niveau Extension, niveau Application). Le mot « strate » reste réservé à `02-concepts.md`. Le répertoire `games/` est appelé niveau Application, parce que son rôle est la composition d'une application, et que `games/` désigne seulement son emplacement actuel.

## Test de réduction (méthode « Comment évaluer une nouvelle idée »)
1. *Nouveau type d'état ?* Non. L'ADR ne définit aucune donnée.
2. *Nouveau modèle d'exécution ou acteur échappant à ADR-004 ?* Non. Un niveau ne change pas l'ordonnancement.
3. *Nouvelle frontière ?* Non. « Engine ne peut pas nommer une Extension » est un interdit de la forme « X ne peut pas nommer Y ». C'est une instanciation de la frontière de connaissance (`00-principes.md`), déjà encodée dans le graphe de dépendances des crates par ADR-000, point 3.
4. *Exprimable comme instanciation ?* Oui, pour la direction des dépendances (frontière de connaissance, ADR-000) comme pour la présence du support d'un format dans Engine. Pour un asset, ADR-A01 point 2 impose que seules la Forge et le chargement runtime parsent et lisent. Le support d'un format spécialisé dans Forge et Assets est donc une conséquence d'ADR-A01, pas un choix du workspace.
5. *Concept nouveau nécessaire ?* Non. Le « niveau de composition » est une relation d'ordre partiel entre crates, pas une catégorie architecturale du moteur.

**Conclusion.** Aucune primitive, ni du modèle d'exécution, ni du modèle de donnée. Cette ADR n'est pas non plus une preuve de fermeture au sens du README, qui porte sur un domaine du moteur : elle ne s'ajoute ni à la table de synthèse par domaine ni à la chronologie des découvertes. Elle applique la frontière de connaissance et la séparation des responsabilités à l'organisation du workspace. Elle formalise des règles qui n'étaient pas énoncées jusqu'ici : critères d'appartenance, direction des dépendances de production et de build par niveau, dépendances entre Extensions, distinction de deux compositions, portée de l'optionalité.

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
| Production du format, validation du programme dans son ensemble (analyses globales sur le contenu produit) | Engine (Forge, build-time) | Validité globale du programme au regard des règles du langage et de ses contraintes de contrôle d'exécution |
| Prise en charge du format binaire à la lecture, intégration au pipeline de chargement | Engine (Assets) | Conformité du contenu lu : bien-formé au sens du type décodé |
| Sémantique d'exécution : état d'instance, transitions, accumulation | Extension | Application du contenu à l'exécution |
| Raccordement au Scheduler, déclaration du contrat de données du système, intégration au modèle ECS | Application | Composition |

Le contrôle de conformité à la lecture porte sur ce que le décodeur doit garantir pour que la valeur décodée respecte les invariants de son propre type : le contenu décodé constitue une structure bien formée. Il ne s'appuie ni sur un autre asset, ni sur un état runtime, ni sur l'intention de design, et ne constitue pas une validation de cohérence métier. Il se distingue de la validation globale de la Forge, qui reste son seul propriétaire, et de la sémantique d'exécution, qui reste à l'Extension.

**Deux responsabilités de composition distinctes.**
- Le **Shell** demeure l'unique point de composition et de pilotage des infrastructures de la Plateforme (ADR-S01, point 7).
- L'**Application** compose les systèmes, leurs contrats de données et les capacités fonctionnelles du jeu. Le modèle d'exécution ordonne ensuite ces systèmes une fois pour toutes (ADR-004).

Une infrastructure constitutive de la Plateforme est, par le critère du point 1, de nature Engine : une Extension n'en devient jamais propriétaire. Une Extension peut en revanche fournir un adaptateur spécialisé qui exploite un contrat de Plateforme défini par Engine. L'Application choisit cet adaptateur et le fournit au Shell à travers ce contrat ; le Shell reste responsable de son raccordement dans la composition de la Plateforme. Le Shell ne dépend jamais nominativement d'une Extension : les dépendances restent orientées selon le point 3.

**Une capacité peut traverser plusieurs niveaux.** La règle « toute la fonctionnalité dans une seule strate » est rejetée (cf. Modèles examinés).

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
- **Données contractuelles venant d'une Extension** : quand une crate d'Engine doit se conformer à un contrat défini en aval (ex. la Forge valide un script contre l'ensemble de données accessible à l'interpréteur, ADR-Sc01 point 4), ce contrat lui parvient comme donnée d'entrée de la chaîne de production, jamais par dépendance de crate.

### 4. Quelles garanties sont offertes au reste du moteur ?
- Engine est complet sans Extension : ses crates se construisent, en production et en build, sans elle.
- **Le niveau ne qualifie pas le régime d'exécution.** Le noyau d'ADR-000 est défini par le graphe d'appel de `world.execute()`, pas par un niveau. Une Extension atteignable depuis `world.execute()` appartient au noyau, et la garantie sans allocation s'applique à elle comme à tout code du noyau. Réciproquement, Engine contient du code hors noyau (Forge, chargement, Shell).
- **Fermeture de dépendances du noyau.** La fermeture des dépendances Cargo actives des crates atteignables depuis `world.execute()` respecte les contraintes du noyau définies par ADR-000. L'audit porte sur les dépendances réellement présentes dans le graphe de compilation concerné, et pas seulement sur les fonctions directement appelées. Une séparation par modules ne retire pas une dépendance de cette fermeture ; une séparation par crate ou par feature peut être employée lorsqu'elle est nécessaire pour exclure du graphe actif du noyau des dépendances incompatibles avec `no_std` ou l'absence d'allocation. Il s'agit de l'application d'ADR-000, non d'une exigence propre à cette ADR, et aucune séparation n'est imposée sans incompatibilité constatée.
- **Optionalité.** Quatre sens sont distingués : fonctionnelle (l'application adopte ou non la capacité), de dépendance (aucune dépendance de production sur la crate), de compilation (le code peut être exclu d'une configuration), de présence dans le binaire (le support peut être absent du binaire final). Cette ADR ne garantit que les deux premiers. Le support d'un format peut rester présent dans Engine même lorsque la capacité fonctionnelle correspondante n'est pas adoptée, car ADR-A01 l'y place.
- **Aucune règle de dépendance Cargo ne dépend du chemin physique.** Déplacer une crate d'un répertoire à un autre ne change pas sa responsabilité ; seul un changement de responsabilité change son niveau.

### 5. Qu'est-ce qui est explicitement exclu du périmètre ?
- Engine défini comme « le code partagé par plusieurs jeux » ou comme « tout ce qui participe au tick ».
- Extension entendue comme plugin dynamique, enregistrement par registre ou découverte à l'exécution. L'Extension est une crate compilée et composée statiquement, ce qui est cohérent avec l'ordre d'exécution construit une fois (ADR-004), l'absence de scheduling dynamique (`00-principes.md`) et l'absence de découverte dynamique (ADR-A01).
- Games réduit à des données seules : l'Application contient du code d'intégration.
- Interdiction générale des dépendances entre Extensions.
- Toute règle imposant qu'une capacité tienne dans un seul niveau.
- Toute règle imposant l'absence de support d'un format dans Engine quand l'Extension correspondante n'est pas adoptée.
- Toute garantie de modularité de compilation ou d'absence du code d'une capacité dans le binaire final.
- La politique des dépendances de développement, des tests et des exemples.
- Toute cartographie des crates du workspace.

### 6. Quelles questions sont volontairement laissées à l'implémentation ?
- La liste des crates, leurs noms Cargo et leurs chemins physiques.
- L'outillage qui vérifie le graphe (script CI, `cargo metadata`, `cargo deny`, autre).
- Les features Cargo et la compilation conditionnelle.
- Le choix entre séparation par crate et par feature, lorsqu'une séparation est nécessaire à la fermeture de dépendances du noyau (point 4).
- La forme des données contractuelles transmises à Forge (point 3).

## Modèles examinés

| Modèle | Statut et raison |
|---|---|
| **A — Encapsulation verticale stricte** (toute la capacité dans Extension) | **Rejeté.** Pour une capacité à base d'asset, ADR-A01 impose que Forge et chargement runtime, qui sont Engine, produisent et lisent le format. Le modèle A dupliquerait ces mécanismes ou ferait dépendre Engine de l'Extension, en contradiction avec la direction du graphe. |
| **C — Engine conteneur de toutes les capacités d'exécution** | **Rejeté.** Il assimile le niveau au régime d'exécution. ADR-000 définit le noyau par le graphe d'appel, pas par un niveau, ce qui rend le modèle C inutile à la garantie d'allocation. Il ferait aussi absorber par Engine toute capacité utilisée au tick. |
| **B — Répartition par responsabilité** | **Retenu.** Il instancie ADR-A01, ADR-000 et la frontière de connaissance. |

## Application au cas Scripts (illustrative, non normative)
- `verbum-forge` et `verbum-assets` (Engine) : production, validation globale et lecture conforme du format `VSCR`. Cette répartition est imposée par ADR-A01 et ADR-Sc01 point 4.
- Les contrôles locaux de `Script::read` (opcodes reconnus, registres valides, cibles de saut valides, durée de `Wait` non nulle) relèvent de la conformité du format et des invariants du type décodé. Les analyses globales de la Forge (cycles sans `yield`, budget d'exécution) relèvent d'une autre portée.
- `verbum-script` (Extension) : état d'exécution, émissions, table, `step`. Il dépend d'Assets, sens autorisé, et ne nomme ni `World` ni Scheduler.
- Adaptateur Scheduler (Application) : contrat de données et raccordement.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Les niveaux désignent des responsabilités de construction, pas des régimes d'exécution | Invariant (instanciation de la séparation des responsabilités) |
| Direction Application → Extension → Engine sur le graphe de production et de build | Invariant (instanciation de la frontière de connaissance et d'ADR-000, point 3) |
| Placement d'un contrat partagé au niveau le plus bas qui doit le nommer | Invariant (conséquence de la direction des dépendances) |
| Support d'un format d'asset spécialisé en Engine, sémantique d'exécution en Extension | Invariant (instanciation d'ADR-A01, point 2, et d'ADR-Sc01) |
| Le contrôle de conformité à la lecture ne constitue pas une validation de cohérence métier | Invariant (instanciation d'ADR-000, point 3) |
| Une capacité peut traverser plusieurs niveaux | Invariant |
| Shell, unique point de composition de la Plateforme ; Application, composition des systèmes et des capacités | Invariant (instanciation d'ADR-S01, point 7) |
| Une infrastructure constitutive de la Plateforme est de nature Engine ; une Extension peut fournir un adaptateur exploitant un contrat de Plateforme défini par Engine | Invariant (conséquence du critère du point 1) |
| Extension = crate composée statiquement, jamais plugin dynamique | Invariant (instanciation de `00-principes.md`, ADR-004, ADR-A01) |
| Les niveaux sont indépendants du noyau d'ADR-000 (défini par graphe d'appel) | Invariant (précision de vocabulaire) |
| Fermeture de dépendances Cargo du noyau soumise aux contraintes d'ADR-000 | Invariant (instanciation d'ADR-000) |
| Dépendances Extension → Extension autorisées, acycliques | Décision révisable |
| Optionalité fonctionnelle et de dépendance garantie ; optionalité de compilation et de présence binaire non garantie | Décision |
| Dépendances de développement, tests, exemples | Hors périmètre |
| Liste des crates, chemins, outillage, features, choix crate/feature | Hors périmètre — implémentation |

## Bilan — test à cinq questions

| Question | Réponse |
|---|---|
| Que possède ce domaine ? | Aucun état ni acteur d'exécution propre. Cette ADR formalise les frontières de responsabilité et de dépendance qui structurent la construction des applications. |
| Que produit-il ? | Une règle d'appartenance et une direction de dépendance vérifiables sur le graphe de crates. |
| Qui consomme cette production ? | Toute crate nouvelle ou déplacée, et l'outillage de vérification du graphe. |
| Que lui est-il interdit de connaître ? | La sémantique métier d'une capacité et le contenu des crates : il ne définit aucune donnée ni aucun système. |
| Quelle frontière protège cette ignorance ? | La frontière de connaissance, encodée dans le graphe de dépendances (ADR-000, point 3). |
