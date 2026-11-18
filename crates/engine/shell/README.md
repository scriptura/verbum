# verbum-shell

Shell du moteur [Verbum](https://github.com/scriptura/verbum) : boucle provisoire
à pas fixe et acquisition d'événements, pilotant l'ordonnanceur par closures
uniquement.

## Rôle

Le Shell possède le cycle de vie du processus, le temps réel, les périphériques
et les infrastructures de plateforme. Il **traduit** les événements bruts en
entrées pour le modèle d'exécution, sans jamais les **interpréter** au sens métier.

- Boucle à pas fixe (ticks déterministes).
- Acquisition d'événements (clavier, souris, réseau, script, replay…).
- Assemblage et orchestration des infrastructures de la strate Plateforme.
- Aucune connaissance des entités, composants, ressources ou systèmes.

## Invariants

- Le Shell traduit, il n'interprète jamais.
- Aucune infrastructure ne peut modifier une itération logique en cours.
- Le Shell n'est jamais informé en retour pendant une itération.

## Statut

Crate du workspace [Verbum](https://github.com/scriptura/verbum). Voir
[`docs/architecture/adr/ADR-S01-shell-responsibilities.md`](https://github.com/scriptura/verbum/blob/main/docs/architecture/adr/ADR-S01-shell-responsibilities.md).

## Licence

MIT OR Apache-2.0
