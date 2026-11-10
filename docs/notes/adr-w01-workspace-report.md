# ADR-W01 — Note de travail : arbitrages, vérifications et suites

**Statut :** note de travail, non normative. Date : 7 novembre 2026.
**Objet :** conserver, hors de l'ADR, ce qui relève du compte rendu de la mise au point d'ADR-W01 (adoptée le 7 novembre 2026). L'ADR reste découplée de l'état du dépôt ; cette note en garde la trace.

## 1. Arbitrages du 7 novembre 2026

| Point | Décision |
|---|---|
| Optionalité de compilation et de présence binaire | Seules l'optionalité fonctionnelle et de dépendance sont garanties. Le support d'un format peut rester présent dans Engine sans que la capacité soit adoptée. |
| Tests U4 et `dev-dependency` de `verbum-shell` vers `verbum-script` | Sujet sorti du périmètre d'ADR-W01. Rien n'est déplacé : les tests restent dans `verbum-shell`, la `dev-dependency` est conservée, aucune crate `games/demo` n'est créée. À réexaminer lors de la réalisation de l'application de démonstration. |
| Contrôles locaux de `Script::read` | Relèvent de la conformité du format et des invariants du type décodé (opcodes, registres, cibles de saut, durée de `Wait` non nulle). Les analyses globales de Forge (cycles sans `yield`, budget) relèvent d'une autre portée. ADR-000 n'est pas modifiée. |
| ADR-Sc01, point 4 : « décidée à la conception du moteur » | Lu comme « déterminé statiquement avant l'exécution ». Correction rédactionnelle minimale appliquée (voir § 4). |
| Composition de la Plateforme (ADR-S01) | Shell, unique point de composition des infrastructures de la Plateforme ; Application, composition des systèmes et capacités. Une Extension peut fournir un adaptateur exploitant un contrat de Plateforme défini par Engine ; elle ne devient pas propriétaire d'une infrastructure constitutive. ADR-S01 n'est pas modifiée. |
| Fermeture de dépendances du noyau | Application d'ADR-000 : l'audit porte sur le graphe de compilation actif. Une séparation par module ne suffit pas ; crate ou feature si nécessaire. Aucune séparation de `verbum-assets` imposée sans incompatibilité constatée. |
| Sections « Points à arbitrer » et « Limites de cet examen » | Supprimées de l'ADR. Leur contenu utile est ici. |

## 2. Vérifications effectuées le 7 novembre 2026

Sources : `Cargo.toml` racine, manifeste de `crates/engine/shell`, `tree.md` du 7 novembre 2026, résultats de `cargo tree` exécutés par le mainteneur.

- `crates/extensions/script` est le seul membre du workspace sous `extensions/`. `collisions` et `lpc` ne contiennent que des README et des brouillons ; `games/demo` ne contient qu'un README.
- `verbum-shell` : section `[dependencies]` vide. Ses `dev-dependencies` sont `verbum-assets`, `verbum-ecs`, `verbum-forge`, `verbum-render`, `verbum-schedule` et `verbum-script`.
- `cargo tree -i verbum-script -e normal,build` : aucune crate dépendante.
- `cargo tree -p verbum-script -e normal,build --depth 1` : `verbum-assets` est l'unique dépendance directe.

Conclusion : la direction Application → Extension → Engine est respectée sur les graphes de production et de build. Aucune violation, aucun changement de code.

## 3. Non examiné

- Le texte d'ADR-S01 (références prises dans les autres documents).
- Le classement individuel des crates Engine.
- La séparation entre lecture de format et chargement disque dans `verbum-assets`.
- Les dépendances de développement autres que celles de `verbum-shell`.

## 4. Modifications documentaires associées

- `ADR-W01-repartition-workspace.md` : statut Adopté, 7 novembre 2026.
- `README.md` (architecture) : ADR-000 et ADR-W01 ajoutées à la liste des ADR adoptées et au parcours de lecture (points 8 et 9).
- `ADR-Sc01-scripts.md` : « décidée à la conception du moteur » remplacé par « décidée statiquement avant toute exécution » (deux occurrences, § 2 et § 4). Portée de la décision inchangée.

## 5. Sujets documentaires restants, indépendants d'ADR-W01

- **ADR-S03** (`ADR-S03-acquisition-evenements.md`) : présente dans `docs/architecture/adr/`, absente du README. À ajouter une fois son intitulé, son statut et sa décision majeure connus.
- **`02-concepts.md`** : décider s'il faut y poser une note de désambiguïsation entre « strate » et « niveau de composition » (ADR-W01 la traite dans son Contexte).
- **`ROADMAP.md` et plan d'étape 6** : cohérence avec le chemin `crates/extensions/script/` non vérifiée.
- **Tests U4 / `games/demo`** : à réexaminer à la création de l'application de démonstration.

## 6. Emplacement de cette note

Proposition : `docs/implementation/adr-w01-workspace-report.md`. Ce dossier contient déjà un rapport (`06-verbum-scripts-report-u3.md`) ; cette note est de même nature (compte rendu de travail, non normatif). `docs/reflexivity/` est réservé à la méthode du corpus, `docs/handoffs/` aux passations d'implémentation.
