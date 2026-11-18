# verbum-render

Modèle de données de rendu du moteur [Verbum](https://github.com/scriptura/verbum) :
contributions de rendu et représentation de frame, `no_std`, sans allocation.

## Rôle

Une contribution de rendu est le contenu interne d'une Ressource transitoire,
jamais une nouvelle catégorie de donnée. Les systèmes composent librement une
représentation de frame unique, sans se connaître entre eux. Le backend graphique
est un pur consommateur passif.

## Statut

Crate du workspace [Verbum](https://github.com/scriptura/verbum). Voir
[`docs/architecture/adr/ADR-R01-rendering-data-model.md`](https://github.com/scriptura/verbum/blob/main/docs/architecture/adr/ADR-R01-rendering-data-model.md)
à [`ADR-R03`](https://github.com/scriptura/verbum/blob/main/docs/architecture/adr/ADR-R03-backend.md).

## Licence

MIT OR Apache-2.0
