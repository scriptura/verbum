# verbum-schedule

Ordonnanceur du moteur [Verbum](https://github.com/scriptura/verbum) : modèle
d'exécution construit à partir des contrats de systèmes, `no_std`, sans allocation.

## Rôle

L'ordre d'exécution est construit une fois à partir des contrats déclarés, jamais
décidé par les systèmes eux-mêmes. Le parallélisme se situe à l'intérieur d'un
système, jamais entre systèmes ou phases.

## Statut

Crate du workspace [Verbum](https://github.com/scriptura/verbum). Voir
[`docs/architecture/01-runtime-ecs.md`](https://github.com/scriptura/verbum/blob/main/docs/architecture/01-runtime-ecs.md)
et [`docs/architecture/adr/ADR-004-execution-model.md`](https://github.com/scriptura/verbum/blob/main/docs/architecture/adr/ADR-004-execution-model.md).

## Licence

MIT OR Apache-2.0
