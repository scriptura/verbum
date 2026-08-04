# Filiation méthodologique — De Marius à Verbum

## Objet

Ce document ne décrit ni une architecture, ni une décision technique. Il décrit la continuité du **procédé de conception** ayant conduit de Marius à Verbum. Les deux projets appartiennent à des domaines très différents :

* **Marius** est un moteur de projection HTML AOT piloté par PostgreSQL.
* **Verbum** est un moteur de jeu ECS/DOD destiné à la simulation.

Ils ne partagent pratiquement aucun code, aucune API, aucune infrastructure, ni même le même problème métier. Pourtant, le chemin ayant conduit à leur architecture présente une continuité qu'il nous faut souligner ici.

## Une même manière de rechercher

Dans les deux projets, la démarche n'a jamais consisté à accumuler des fonctionnalités. Le travail a consisté à chercher les invariants les plus simples capables d'expliquer l'ensemble du système. Lorsqu'une nouvelle difficulté apparaissait, la première question n'était jamais :

> « Quelle nouvelle couche faut-il ajouter ? »

mais plutôt :

> « Cette difficulté est-elle réellement nouvelle, ou peut-elle être exprimée avec les concepts déjà présents ? »

Cette discipline conduit naturellement à une architecture où les concepts deviennent rares, mais extrêmement généraux.

## Compression structurelle

Marius a progressivement fait émerger une idée devenue centrale :

> une architecture progresse souvent davantage en supprimant des catégories qu'en en ajoutant.

Chaque fois qu'une nouvelle responsabilité semblait apparaître, le travail consistait à déterminer si elle constituait réellement une catégorie nouvelle, ou simplement une instanciation d'un concept déjà existant. Cette recherche de compression structurelle ne visait pas seulement l'élégance, elle réduisait simultanément :

* les coûts cognitifs ;
* les coûts d'exécution ;
* les interfaces ;
* les points de couplage ;
* les possibilités de divergence.

Cette manière de raisonner réapparaît presque intacte dans Verbum.

## La Forge comme discipline

Marius (et avant lui notre calendrier liturgique, mais de manière plus simple avec un binaire comme frontière) a également introduit une séparation devenue structurante :

* tout ce qui demande de comprendre un format humain appartient à la Forge ;
* le runtime ne fait qu'exécuter des structures déjà préparées.

Cette séparation était d'abord une optimisation. Avec le temps, elle est devenue une discipline intellectuelle :

> déplacer la complexité vers un moment où elle peut être analysée, vérifiée et figée.

Verbum applique exactement la même logique. La Forge n'y est pas une fonctionnalité. Elle est une frontière architecturale.

## Le runtime comme conséquence

Le runtime de Marius était volontairement réduit :

* structures plates ;
* données déjà préparées ;
* parcours déterministes ;
* absence d'interprétation.

Verbum poursuit exactement cette idée. Le runtime n'est pas intelligent. Il est spécialisé. Son rôle n'est pas de découvrir quoi faire. Son rôle est d'exécuter le plus fidèlement possible une organisation décidée ailleurs. Cette idée traverse les deux projets malgré des domaines totalement différents.

## Ce qui a réellement été transmis

Verbum n'est pas un successeur de Marius. Il n'en réutilise ni les composants, ni les abstractions. Ce qui s'est transmis est plus abstrait. Il s'agit d'une manière de construire une architecture :

* rechercher les invariants avant les mécanismes ;
* déplacer toute connaissance possible hors du runtime ;
* préférer la réduction du nombre de concepts à leur multiplication ;
* considérer toute nouvelle idée comme une tentative de réfutation des concepts existants avant de l'accepter comme primitive.

Autrement dit, ce qui passe de Marius à Verbum n'est pas une solution. C'est une méthode de découverte.

## Une observation rétrospective

Le projet de calendrier liturgique développé auparavant constitue rétrospectivement une étape intermédiaire intéressante. Sans chercher à reproduire Marius, il a spontanément retrouvé plusieurs des mêmes choix :

* Forge AOT ;
* format binaire orienté données ;
* runtime minimal ;
* accès déterministes ;
* absence d'interprétation au runtime.

À l'époque, ces ressemblances paraissaient relever de choix techniques indépendants. Avec Verbum, elles apparaissent plutôt comme les manifestations répétées d'une même manière de raisonner. Le point commun n'est donc ni le domaine, ni le langage, ni les bibliothèques. C'est le processus intellectuel qui conduit progressivement vers les mêmes formes architecturales lorsque l'on cherche systématiquement à réduire les concepts, externaliser la connaissance et préserver un runtime aussi simple que possible.

## Conclusion

Le calendrier liturgique, Marius et Verbum ne forment pas une lignée de logiciels. Ils constituent plutôt trois observations successives d'un même procédé de conception appliqué à des problèmes différents. Le véritable invariant qui traverse ces projets n'est donc pas technique, c'est une méthode : chercher le plus petit langage architectural capable d'absorber un domaine entier sans devoir s'étendre.

---

_le 3 août 2026_
