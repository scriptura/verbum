# Handoff — Implémentation de l'étape 0 (`verbum-ecs`)

## Contexte
Le corpus architectural de Verbum est figé et a traversé un audit contradictoire complet (cohérence, complétude, implémentabilité, fermeture — voir historique Git, commits jusqu'à `6b628f9`). Une roadmap d'implémentation (`ROADMAP.md`) et un plan détaillé de sa première étape (`PLAN-ETAPE-0-ECS.md`) ont ensuite été produits et audités à leur tour (commits jusqu'à `c5008a6`).

**Rôle attendu de Claude dans cette nouvelle session : implémenteur, pas architecte.** Le travail de conception est terminé. Cette session écrit le code Rust de `verbum-ecs`, phase par phase, en suivant `PLAN-ETAPE-0-ECS.md` à la lettre.

## Fichiers à transmettre à la nouvelle session
L'intégralité du corpus (`README.md`, `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, tous les ADR), plus `ROADMAP.md` et `PLAN-ETAPE-0-ECS.md`. L'historique Git du répertoire de travail, s'il est disponible, sert de journal de référence pour les décisions déjà prises — inutile de le reparcourir en détail, mais utile en cas de doute sur une formulation.

## Discipline générale
- **Une phase à la fois**, dans l'ordre de `PLAN-ETAPE-0-ECS.md`. Ne pas anticiper la phase suivante, même si l'implémentation semble triviale.
- **Le test précède le code.** Écrire le test qui exprime l'invariant de la phase avant d'écrire l'API qui le fait passer.
- **Un commit par invariant démontré**, pas un commit par numéro de phase — si une phase en révèle deux (cf. Phase 4a/4b dans le plan), deux commits.
- **`cargo test` vert à chaque commit**, même si la phase globale n'est pas terminée.
- **Signal d'alarme** : toute pensée du type « tant que j'y suis » ou « ça servira plus tard » signifie qu'on code hors du périmètre de la phase en cours. La réponse est de s'arrêter, jamais de continuer « juste un peu ».
- **Rien sans invariant.** `EntityBuilder`, `Bundle`, `Archetype`, `Scene`, `Prefab`, `Registry`, `Plugin`, `Service`, `Factory` restent absents tant qu'aucun invariant du corpus ne les réclame explicitement.

## Le principe directeur de cette session
> Pendant l'implémentation, traiter les ADR comme un compilateur traite une spécification de langage : si le code paraît difficile à écrire, le premier réflexe n'est jamais de modifier la spécification. Il faut d'abord essayer plusieurs implémentations différentes. Une ADR n'est remise en question que si plusieurs implémentations indépendantes échouent pour la **même** raison structurelle.

Concrètement, si une phase résiste :
1. Essayer une deuxième approche d'implémentation, indépendante de la première.
2. Si les deux échouent pour des raisons différentes, ce n'est probablement pas l'architecture — continuer à chercher côté code.
3. Si les deux échouent pour la **même** raison structurelle, alors seulement envisager qu'un ADR soit en cause.
4. Si l'ADR est réellement en cause, rouvrir le corpus avec la même discipline que `HANDOFF-AUDIT.md` : citer le texte exact, proposer la correction minimale, documenter le bilan — jamais réécrire un document adopté par confort d'implémentation.

## Ce que cette session ne doit pas faire
- Ne pas rouvrir de débat de conception déjà tranché par le corpus ou par `PLAN-ETAPE-0-ECS.md`.
- Ne pas introduire de mécanisme générique (registre, `TypeId`, dispatch) avant que son besoin soit démontré par un test concret — cf. Phase 3/4b du plan, qui documentent explicitement pourquoi ce moment est différé.
- Ne pas traiter les documents de planification (`ROADMAP.md`, `PLAN-ETAPE-0-ECS.md`) comme figés au même titre qu'une ADR : ce sont des documents de travail, révisables sans procédure, qui peuvent devenir le journal de construction de l'étape (micro-phases, critères validés, décisions prises en cours de route, difficultés rencontrées, références de commits) au fur et à mesure de l'avancement.

## Sortie attendue
À l'issue de l'étape 0 : un crate `verbum-ecs` où `cargo test` fait passer l'acceptance test déjà posé dans `ROADMAP.md` (« spawn → ajout composant → query → command buffer → resolve → query »), construit phase par phase selon `PLAN-ETAPE-0-ECS.md`, avec un historique de commits qui documente l'apparition de chaque invariant.
