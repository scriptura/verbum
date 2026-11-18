# verbum-script

Runtime de scripts du moteur [Verbum](https://github.com/scriptura/verbum) : état
d'exécution et étape d'interprétation pure, `no_std`, sans allocation.

## Rôle

Un script n'a jamais de pile d'exécution suspendue hors du `World` ; l'interface de
son interpréteur est fixe, jamais variable selon le script chargé. Les scripts ne
constituent pas un second modèle d'exécution : ils s'inscrivent dans le modèle
unique du runtime.

## Statut

Crate du workspace [Verbum](https://github.com/scriptura/verbum). Voir
[`docs/architecture/adr/ADR-Sc01-scripts.md`](https://github.com/scriptura/verbum/blob/main/docs/architecture/adr/ADR-Sc01-scripts.md).

## Licence

MIT OR Apache-2.0
