# ADR-R03 — Backend graphique

## Statut
Adopté. Date : 31 juillet 2026.

**Addendum proposé le 26 novembre 2026** (point 3 : réception et présentation), **en attente d'adoption explicite**. Il ne se prononce ni sur la production de la représentation de frame, ni sur la relation entre ADR-S01 et ADR-R02, ni sur la forme de l'interface de franchissement d'ADR-000, point 3. Tant qu'il n'est pas adopté, le texte adopté ci-dessous fait seul foi ; le statut « Adopté » n'est pas modifié.

## Contexte
ADR-R01 et ADR-R02 ont établi que la représentation de frame est produite entièrement à l'intérieur du modèle ECS, par composition d'une Ressource transitoire, sans jamais nommer de backend, de GPU, ou d'API graphique. Cette ADR traite ce qui se passe une fois la représentation composée : comment elle traverse la frontière vers le backend, et ce qui, de l'autre côté de cette frontière, cesse d'exister.

## Décision

### 1. La frontière est une réduction de modèle, pas une simple délégation
Le backend graphique ne reçoit ni entités, ni composants, ni ressources, ni systèmes, ni phases, ni contrats. Il reçoit exclusivement la représentation de frame produite par la composition (ADR-R02) — une donnée déjà entière, déjà ordonnée, déjà dépourvue de toute référence au vocabulaire ECS. Ce n'est pas une couche supplémentaire du même modèle : c'est la sortie du modèle vers un domaine qui l'ignore totalement.

```
ECS (Entité, Composant, Ressource, Query, Système, Phase, Contrat)
        │
        ▼
Représentation de frame (composée, immuable — ADR-R02)
        │
        ▼
Backend graphique (aucune connaissance de ce qui précède)
        │
        ▼
GPU
```

### 2. Le backend est une infrastructure passive, symétrique au `World`
Le backend applique la représentation qu'il reçoit ; il ne la produit pas, ne l'interprète pas au sens métier, ne la questionne pas. Cette symétrie complète une paire déjà établie :
- le `World` applique les mutations structurelles accumulées dans le Command Buffer, sans connaître leur provenance métier (ADR-006) ;
- le backend applique la représentation de frame accumulée par composition, sans connaître sa provenance ECS.

Dans les deux cas, l'infrastructure terminale n'invente aucune logique : elle exécute une donnée déjà entièrement construite en amont. Aucune décision de gameplay, de layout mémoire, ou d'ordre de traitement n'est prise par le backend — tout ce qu'il applique a déjà été décidé par le modèle d'exécution et les systèmes qui le précèdent.

### 3. Le backend ne participe à aucun contrat, aucune dépendance, aucune phase
Contrairement à un système (ADR-003), le backend n'a pas d'interface déclarative inspectable par le modèle d'exécution (ADR-004) : il n'est jamais un nœud du graphe de dépendances construit à partir des contrats. Son point d'invocation est fixé par construction à la fin de la phase d'Extraction (`01-runtime-ecs.md`, section 5) — un point d'entrée unique, hors du système de contrats, pas un système de plus parmi d'autres.

> **Addendum proposé (26 novembre 2026) — réception et présentation.**
> Deux opérations sont distinguées, que l'expression « invocation » du présent point ne séparait pas :
> - la **réception** est la remise au Backend d'une nouvelle représentation de frame, qui actualise l'état qu'il retient. C'est le point d'entrée unique du présent point ; il demeure hors du système de contrats ;
> - la **présentation** est l'exécution graphique de l'état retenu.
>
> **Localisation de la réception.** L'expression « à la fin de la phase d'Extraction » du présent point est lue ainsi : la réception intervient après l'achèvement de la progression complète (ADR-S02, point 4), c'est-à-dire après la phase d'Extraction de la dernière itération exécutée, et hors de toute itération. Elle ne désigne pas un instant interne à une itération, et le présent point ne comporte pas d'autre localisation. Lorsqu'une progression n'exécute aucune itération, aucune réception n'a lieu. Cette lecture est celle du présent addendum ; elle n'est établie ni par ADR-S01 ni par ADR-S02.
>
> **Cardinalité.** La cardinalité établie par ADR-S01, point 5 (0..N itérations de `World::execute()` pour 0..1 représentation de frame transmise au Backend) compte les réceptions, non les présentations : la réception a lieu au plus une fois par progression.
>
> **Présentation.** Le Backend peut présenter plusieurs fois un état déjà reçu, sans nouvelle réception ; il s'agit d'une possibilité, ni d'une obligation ni d'une politique de cadence. Ni la réception ni la présentation ne renvoient d'information au `World` ni à l'ECS (point 5). L'état retenu relève de la structure interne du Backend, hors du modèle architectural.
>
> Le présent addendum ne traite que le côté Backend de la frontière. Il ne se prononce ni sur la production de la représentation de frame, ni sur la relation entre ADR-S01, point 3, et ADR-R02, point 3, ni sur la forme de l'interface de franchissement (ADR-000, point 3, qui reste ouvert). Il ne fixe ni l'état du Backend avant sa première réception, ni la règle de publication lorsqu'une progression se termine par une erreur ou un arrêt, ni le mécanisme qui déclenche une présentation.

### 4. Aucune indépendance de plateforme n'est requise au niveau architectural
Le choix d'une bibliothèque de rendu concrète (wgpu, une autre API graphique, un backend headless pour les tests, etc.) ne concerne que ce qui se passe après la frontière — donc jamais le modèle ECS. Le remplacement d'un backend par un autre ne modifie ni la représentation de frame, ni sa composition, ni aucun invariant déjà posé en ADR-R01/R02 : c'est une décision d'implémentation entièrement contenue de l'autre côté de la frontière.

### 5. La frontière ne s'inverse jamais
Aucune information ne remonte du backend vers l'ECS pendant l'exécution d'une frame. Le backend ne modifie jamais la représentation qu'il reçoit, ne produit aucune donnée consommée par un système, et n'a aucun moyen d'influencer le modèle d'exécution. Toute information destinée à revenir vers la simulation (résultat d'un raycast GPU, retour de picking, mesure de performance...) constituerait une entrée nouvelle acquise à la phase d'Acquisition d'une frame ultérieure (`01-runtime-ecs.md`, section 5) — jamais un canal direct depuis le backend vers un système en cours d'exécution.

## Ce qui n'est pas tranché ici
- Le choix concret de la bibliothèque de rendu (wgpu ou autre) : ADR dédiée, hors du périmètre architectural.
- Les optimisations de composition ou de transmission (batching, atlas, instancing, tri par état GPU...) : politiques d'implémentation, à documenter séparément, sans jamais remonter contaminer ADR-R01/R02.
- Le mécanisme concret par lequel une donnée de retour (picking, mesure de performance) est réinjectée à l'Acquisition d'une frame future.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| La frontière ECS → backend est une réduction de modèle, non une délégation entre pairs | Invariant |
| Le backend est une infrastructure passive, symétrique au `World` vis-à-vis du Command Buffer | Invariant |
| Le backend ne participe à aucun contrat, dépendance, ou phase du modèle d'exécution | Invariant |
| Le choix de la bibliothèque de rendu est hors périmètre architectural | Décision d'implémentation |
| Aucune information ne remonte du backend vers l'ECS pendant l'exécution d'une frame | Invariant |
| Optimisations de composition/transmission (batching, atlas, instancing, tri GPU) | Hors périmètre — politiques d'implémentation |
| Réception (remise d'une nouvelle représentation ; « 0..1 » compte les réceptions ; localisée après la progression complète, hors de toute itération) distincte de la présentation (exécution de l'état retenu) | Clarification de portée (instanciation d'ADR-S01, point 5) — addendum proposé le 26 novembre 2026, en attente d'adoption |
| Présentation répétée d'un état déjà reçu, sans nouvelle réception | Possibilité, non obligation ; aucune politique de cadence — addendum proposé le 26 novembre 2026, en attente d'adoption |

## Bilan de la trilogie Renderer
Avec ADR-R01 à ADR-R03, le Renderer est architecturalement clos :
- ADR-R01 a établi la nature de la donnée produite (une Ressource transitoire accumulant des contributions, sans nouvelle catégorie architecturale) ;
- ADR-R02 a établi le pipeline de production et de composition (instanciation directe d'ADR-003/004), et résolu les deux hypothèses ouvertes par ADR-R01 en clarifiant la frontière modèle architectural / structure interne, désormais actée en `02-concepts.md` ;
- ADR-R03 établit la frontière de sortie vers le backend comme une réduction de modèle, symétrique à celle déjà posée pour le `World` et le Command Buffer.

Le Renderer confirme, sans exception, que le socle établi en ADR-001 à ADR-006 absorbe un domaine périphérique entier sans qu'aucun invariant n'ait dû être ajouté au-delà d'une clarification de périmètre déjà prévue par la méthode.
