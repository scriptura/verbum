# verbum-forge

Compilateur AOT du moteur [Verbum](https://github.com/scriptura/verbum) : convertit
les formats humains (YAML, JSON…) en artefacts binaires consommés par le runtime.

## Rôle

Seul composant autorisé à lire et parser des formats humains. Valide la cohérence
des données de design, convertit les unités humaines en unités de simulation
déterministes, et produit les artefacts statiques consommés par le runtime.

## Statut

Crate du workspace [Verbum](https://github.com/scriptura/verbum). Voir
[`docs/architecture/adr/ADR-R02-production-pipeline.md`](https://github.com/scriptura/verbum/blob/main/docs/architecture/adr/ADR-R02-production-pipeline.md)
et [`docs/architecture/adr/ADR-A01-assets.md`](https://github.com/scriptura/verbum/blob/main/docs/architecture/adr/ADR-A01-assets.md).

## Licence

MIT OR Apache-2.0
