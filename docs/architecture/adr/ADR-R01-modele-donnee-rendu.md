# ADR-R01 — Modèle de donnée de rendu

## Statut
Adopté, avec une question ouverte assumée (cf. section « Test de résistance du modèle général »). Date : 31 juillet 2026.

## Contexte
Conformément au critère de validité posé en `02-concepts.md`, cette ADR ne parle ni de GPU, ni d'API graphique, ni de backend. Elle ne répond qu'à une question : quelle est la donnée produite par le Renderer, et comment s'insère-t-elle dans le modèle général (cardinalité, mode d'adressage, durée de vie, politique de consommation) ?

## Décision

### 1. La contribution de rendu n'est pas un composant
Une `RenderContribution` (position à l'écran, référence de sprite, calque, teinte, transform...) n'est :
- ni persistante : elle ne survit pas au-delà de la phase d'Extraction (`01-runtime-ecs.md`, section 5) qui l'a produite ;
- ni adressable individuellement : rien dans le moteur ne la retrouve par `EntityId` ou par clé après sa production ;
- ni requêtable : aucune Query (ADR-002) ne porte sur des `RenderContribution` déjà produites.

Elle est produite une fois, consommée une fois, puis n'existe plus. Ce constat pose une question qui dépasse le seul renderer — développée en section 6.

### 2. Cardinalité et durée de vie
Cardinalité : 0..N par instance de la phase d'Extraction — un nombre variable de contributions produites par frame, sans lien avec le nombre d'entités porteuses d'un composant donné (une entité peut produire zéro, une, ou plusieurs contributions de rendu selon son état visuel).

Durée de vie : transitoire, mais dans sa forme la plus stricte déjà envisagée par le modèle général (`02-concepts.md`) — elle ne survit même pas jusqu'à la phase suivante. Elle naît et meurt à l'intérieur d'une seule instance de phase.

### 3. Test de résistance du modèle général — mode d'accès
C'est le seul point où cette ADR n'apporte pas de réponse définitive, par choix méthodologique : plutôt que d'anticiper une généralisation du modèle général, on observe ici ce que le cas concret exige réellement.

Constat empirique pour la contribution de rendu : à aucun moment de son cycle de vie elle n'a besoin d'être *identifiée*. Elle n'est jamais recherchée par clé, jamais relue après production, jamais distinguée individuellement d'une autre contribution similaire — elle est immédiatement capturée par la composition (section 4) qui suit sa production, dans l'ordre où elle a été émise. Contrairement à un composant (identifié par `EntityId`) ou une Ressource (identifiée par clé unique — ADR-005), la contribution de rendu ne possède donc aucun mode d'adressage au sens où `02-concepts.md` l'entend aujourd'hui.

Deux lectures restent en balance, non tranchées ici :
- ce cas constitue une instanciation valide d'un axe « mode d'adressage » généralisé en « mode d'accès » (adressé vs séquentiel) ;
- ou ce cas révèle que l'axe « adressage » (identification d'une donnée) et l'axe « parcours » (manière dont on itère cette donnée) sont en réalité deux axes indépendants, à distinguer dans le modèle général.

Cette ADR ne préjuge d'aucune des deux réponses. Elle documente le comportement observé (production puis consommation strictement séquentielle, sans identification individuelle) et laisse la décision de faire évoluer `02-concepts.md` à un futur constat, une fois qu'un second cas concret (ADR-R02 ou un domaine ultérieur) aura confirmé ou infirmé le besoin de généraliser.

> **Addendum — résolution (cf. ADR-R02).** La question se dissout une fois les deux plans distingués : au niveau du modèle ECS, la Ressource qui accumule les contributions est parfaitement adressée (par sa clé/type), exactement comme toute autre Ressource. L'absence d'identification individuelle ne concerne que les éléments internes de son contenu — hors périmètre du modèle général, qui ne décrit jamais la représentation interne d'une donnée (`02-concepts.md`). Aucune généralisation de l'axe « mode d'adressage » n'est nécessaire.

### 4. Politique de consommation : composition, pas résolution
Plusieurs systèmes produisent des `RenderContribution` indépendamment les uns des autres (extraction de sprites, de texte, d'UI, de particules...), sans connaissance mutuelle — cohérent avec « produire est libre ». Un unique système en aval consomme l'ensemble de ces productions.

Ce système n'arbitre entre aucune contribution concurrente : il **compose** — toutes les contributions produites sont conservées et agrégées dans une représentation unique de la frame, sans qu'aucune ne soit écartée au profit d'une autre. Ceci diffère d'une **résolution** au sens strict (ex: `HealthResolutionSystem`, ADR-004), où plusieurs contributions concurrentes sur une même donnée sont arbitrées pour produire un résultat unique.

Cette distinction affine, sans le contredire, le principe déjà posé « produire est libre ; résoudre est unique » (`01-runtime-ecs.md`) : la composition et la résolution en sont deux instanciations, selon que la transformation arbitre ou agrège. *Observation, non tranchée dans cette ADR :* le Command Buffer (ADR-006) relève probablement lui aussi d'une composition plutôt que d'une résolution stricte (le `World` applique toutes les commandes accumulées, sans en arbitrer aucune) — un futur addendum terminologique à ADR-006 pourrait le préciser, sans changement de fond.

### 5. Chaîne résultante
```
Sprite extraction
Text extraction
UI extraction
Particle extraction
        │
        ▼
RenderCompositionSystem
        │
        ▼
Représentation unique de la frame (nom et forme concrète : ADR-R02)
```
Cette chaîne reste entièrement interne au modèle ECS — elle n'implique aucune notion de backend, de GPU, ou d'API graphique.

### 6. Hypothèse ouverte — une quatrième vocation de production ?
Cette section documente une hypothèse plus profonde que celle de la section 3, sans la trancher, pour la même raison méthodologique : elle mérite d'être confirmée par un second cas concret avant toute modification de `02-concepts.md`.

Jusqu'ici, tout ce qui est produit par un système rejoint l'une de trois destinations déjà établies : une donnée persistante (composant), une donnée singleton (Ressource), ou une mutation structurelle (Command Buffer, ADR-006) — chacune *directement consommée* selon une politique propre, mais consommée en tant que telle.

La contribution de rendu semble suivre un chemin différent : elle n'est stockée nulle part, elle n'est mutée nulle part, elle **participe à une composition** dont le résultat est une représentation distincte de chacune de ses contributions individuelles. Si cette différence se confirme, elle suggérerait une quatrième vocation de production, aux côtés de persistance, singleton, et mutation structurelle — la *contribution à une composition*.

Cette ADR ne l'affirme pas comme un invariant. Elle observe que le cas s'y prête, et laisse à ADR-R02 le rôle de second test : si son écriture s'appuie naturellement sur le vocabulaire actuel (`02-concepts.md` + cette ADR), l'hypothèse reste locale au Renderer. Si, en revanche, elle exige de réinventer les notions de contribution et de composition pour un cas indépendant du rendu, ce sera le signal qu'une révision de `02-concepts.md` est justifiée — pas avant.

> **Addendum — résolution (cf. ADR-R02).** Cette hypothèse ne s'est pas confirmée. ADR-R02 a montré que la difficulté provenait d'une confusion entre deux plans d'abstraction distincts : la catégorie de donnée du modèle ECS (une Ressource transitoire unique, accumulant les contributions) et la représentation interne de cette Ressource (la collection de contributions qu'elle contient). Le modèle général ne décrit que le premier plan ; il n'a jamais eu vocation à classifier le contenu interne d'une donnée. Aucune quatrième vocation de production n'est nécessaire — cf. `02-concepts.md`, section « Frontière entre modèle architectural et structure interne ».

## Ce qui n'est pas tranché ici
- La forme concrète de la représentation produite par la composition (nom, layout mémoire, tri) : objet d'ADR-R02.
- La question ouverte de la section 3 (généraliser « mode d'adressage » en « mode d'accès », ou scinder adressage/parcours en deux axes) : décision différée, dans l'attente d'un second cas concret.
- Tout ce qui concerne le backend consommateur de cette représentation : objet d'ADR-R03.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| La contribution de rendu n'est ni persistante, ni adressable, ni requêtable | Invariant |
| Cardinalité 0..N par instance de phase d'Extraction, durée de vie bornée à cette seule instance | Invariant (instanciation stricte de l'axe « durée de vie ») |
| Absence d'identification individuelle (production puis consommation strictement séquentielle) | Résolu (ADR-R02) — concerne la représentation interne d'une Ressource, hors périmètre du modèle général |
| Politique de consommation par composition (agrégation sans arbitrage), distincte de la résolution | Invariant — affine « produire est libre, résoudre est unique » |
| Quatrième vocation de production (« contribution à une composition ») distincte de persistance/singleton/mutation | Résolu (ADR-R02) — confusion entre catégorie ECS et représentation interne, aucune nouvelle catégorie nécessaire |
| Forme concrète de la représentation de frame | Hors périmètre — ADR-R02 |

## Suite prévue
ADR-R02 — Pipeline de production : quels systèmes produisent des `RenderContribution`, à quelle phase, selon quelles dépendances (instanciation directe d'ADR-003 et ADR-004, sans invariant nouveau attendu). La forme mémoire concrète de la représentation de frame (SoA, clés de tri par matériau/pipeline...) y sera également abordée comme le prolongement naturel de cette ADR.
