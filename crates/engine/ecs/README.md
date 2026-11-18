# verbum-ecs

Cœur du moteur [Verbum](https://github.com/scriptura/verbum) : un ECS strict,
`no_std`, sans allocation sur le chemin chaud.

## Rôle

`verbum-ecs` fournit les primitives fondamentales du runtime :

- `EntityId` — identifiant opaque, sans sémantique métier, validé par
  `(index, génération)` en O(1).
- `World` — seule autorité de création/destruction d'entités et d'application
  des mutations structurelles.
- Composants et Ressources — données pures (`Copy`/`Clone`), sans logique.
- Requêtes — vues passives, spécialisées statiquement, qui ne produisent jamais
  de donnée.
- Command Buffer — mutations structurelles accumulées librement, résolues en un
  point unique par le `World`.

## Invariants

- Une entité est un handle opaque ; seul le `World` en crée.
- Produire est libre ; résoudre est unique.
- Une phase est l'unité atomique de cohérence du `World`.
- Zéro allocation sur le chemin chaud.

## Statut

Crate du workspace [Verbum](https://github.com/scriptura/verbum). Voir
[`docs/architecture/01-runtime-ecs.md`](https://github.com/scriptura/verbum/blob/main/docs/architecture/01-runtime-ecs.md)
pour le modèle normatif.

## Licence

MIT OR Apache-2.0
