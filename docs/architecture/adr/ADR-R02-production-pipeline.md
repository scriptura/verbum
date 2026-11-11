# ADR-R02 — Pipeline de production de la représentation de frame

## Statut
Adopté, avec un bilan a posteriori sur les deux questions laissées ouvertes par ADR-R01. Date : 31 juillet 2026.

## Contexte
ADR-R01 a laissé deux hypothèses explicitement non tranchées : la nécessité de distinguer « adressage » et « parcours » dans le modèle général, et l'existence d'une possible quatrième vocation de production (« contribution à une composition »). Cette ADR ne cherche pas à les confirmer. Elle décrit le pipeline de production de la représentation de frame à partir des seuls besoins concrets du problème, sans référence aux hypothèses — le bilan sur leur nécessité vient en dernière section, une fois la description terminée.

## Décision — description du pipeline

### 1. Chaque système producteur ignore les autres
Un système qui extrait une contribution de rendu (à partir d'un sprite, d'un texte, d'un élément d'UI, d'une particule...) ne connaît ni l'existence, ni le contenu, ni l'ordre d'exécution des autres systèmes producteurs. C'est une instanciation directe d'ADR-003 : le contrat d'un système est indépendant des autres contrats du même ensemble.

### 2. La production est append-only
Pendant une instance de la phase d'Extraction, chaque contribution produite s'ajoute à l'ensemble accumulé, sans jamais modifier ni relire une contribution déjà produite par un autre système (ou par soi-même) durant cette même instance. Aucune contribution n'est jamais consultée avant le point de composition.

### 3. La composition est unique
Un seul système consomme l'ensemble des contributions accumulées et produit une représentation unique de la frame. Aucun autre système ne participe à cette transformation — cohérent avec le principe déjà posé « produire est libre ; résoudre est unique » (ici, sous sa forme composition plutôt que résolution, cf. ADR-R01 section 4).

La dépendance entre les systèmes producteurs et le système de composition est déduite, non déclarée (ADR-004) : chaque producteur écrit dans la même destination que celle que le système de composition consomme, ce qui suffit à la construction du modèle d'exécution pour ordonner tous les producteurs avant la composition, sans qu'aucun système n'exprime explicitement cette relation.

### 4. La représentation devient immuable
Une fois produite par la composition, la représentation de frame n'est plus modifiée. Elle est transmise telle quelle en aval (hors périmètre de cette ADR — cf. ADR-R03). Ceci est cohérent avec l'invariant déjà posé en ADR-004 : une fois construit, un modèle ou une donnée qui en résulte n'est plus recalculé pendant son usage.

### 5. Aucun système producteur ne connaît la représentation finale
Un système producteur de contribution ne connaît que sa propre contribution à décrire — jamais le tri, le regroupement, le format mémoire de la représentation finale, ni a fortiori le GPU, le batching, ou le backend. Il décrit une intention visuelle, rien de plus. Ceci découle du principe de localité déjà posé en ADR-003 : un système ne raisonne que sur les données explicitement fournies par son interface déclarative.

## Bilan a posteriori

### Question 1 — fallait-il distinguer « adressage » et « parcours » ?
Non — et l'écriture de cette ADR permet de préciser pourquoi, au-delà du simple constat empirique. Le pipeline décrit ci-dessus ne manipule, au niveau du modèle ECS, qu'une seule donnée : la Ressource transitoire qui accumule les contributions durant une instance de phase. Cette Ressource est parfaitement adressée par sa clé/type, exactement comme toute autre Ressource (ADR-005) — aucun problème d'adressage ne se pose à ce niveau. L'absence d'identification individuelle ne concerne que le contenu interne de cette Ressource (la collection de contributions qu'elle porte), qui relève de la représentation interne — un niveau que le modèle général n'a jamais eu vocation à décrire. La question ne se tranche donc pas en faveur de l'une des deux lectures proposées en ADR-R01 : elle se dissout, une fois les deux plans distingués.

### Question 2 — la contribution de rendu est-elle une nouvelle vocation de production ?
Non — et la raison est plus précise qu'un simple constat de parcimonie. ADR-R01 mélangeait implicitement deux plans d'abstraction distincts, sans le nommer explicitement :

- **le plan du modèle ECS** : une Ressource transitoire unique, de cardinalité 1, qui accumule les contributions durant une instance de phase ;
- **le plan de la représentation interne** : le contenu de cette Ressource — une collection de contributions individuelles, produites par plusieurs systèmes, jamais adressées individuellement.

Le modèle général de donnée (`02-concepts.md`) ne décrit que le premier plan. Il n'a jamais eu vocation à classifier la structure interne du contenu d'une Ressource — exactement comme il ne classifie pas les `Contact` d'une collection de contacts physiques, ou les `InventoryItem` d'un inventaire. Sous cette lecture :
- la cardinalité « 0..N » notée en ADR-R01 décrit le contenu de la Ressource, pas un nouvel axe de cardinalité du modèle général (dont la cardinalité reste 1, celle de la Ressource elle-même) ;
- l'absence d'adressage individuel des contributions n'est pas une lacune du modèle général — celui-ci ne s'étend jamais au contenu interne d'une donnée ;
- la composition n'est pas une nouvelle nature de transformation : c'est la politique de résolution qu'applique le propriétaire unique de cette Ressource, qui choisit d'agréger plutôt que d'arbitrer.

L'hypothèse d'ADR-R01 n'était donc pas fausse en tant que telle — elle reposait sur une confusion entre une catégorie de donnée du modèle ECS et le contenu interne de cette donnée. Une fois cette distinction posée explicitement (cf. `02-concepts.md`, section « Frontière entre modèle architectural et structure interne »), la nécessité d'une quatrième vocation de production disparaît : il n'y en a probablement jamais eu qu'une lecture à deux niveaux d'une seule et même Ressource transitoire, déjà couverte par ADR-005.

Ce résultat, contrairement au bilan initial de cette ADR, ne se limite plus à un simple constat de parcimonie : il s'appuie sur une clarification de périmètre désormais actée dans `02-concepts.md`, confirmée par deux cas concrets convergents (Command Buffer, pipeline de rendu).

## Ce qui n'est pas tranché ici
- La forme mémoire concrète de la représentation de frame (SoA, clés de tri par matériau/pipeline...) reste hors périmètre — à documenter séparément, si un besoin concret l'exige, plutôt que par anticipation.
- Tout ce qui concerne le backend consommateur de la représentation : ADR-R03.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Chaque système producteur ignore les autres | Invariant (instanciation d'ADR-003) |
| Production append-only, pas de relecture pendant la même instance de phase | Invariant |
| Composition unique, dépendance déduite non déclarée | Invariant (instanciation d'ADR-004) |
| Représentation immuable une fois composée | Invariant |
| Aucun système producteur ne connaît la représentation finale, le tri, le batching, le backend | Invariant (instanciation du principe de localité, ADR-003) |
| Nécessité de distinguer adressage/parcours | Résolu — question dissoute par la distinction modèle ECS / représentation interne (`02-concepts.md`) |
| Quatrième vocation de production | Résolu — confusion entre catégorie ECS et représentation interne ; la contribution de rendu est le contenu interne d'une Ressource transitoire (ADR-005), aucune nouvelle catégorie |

## Suite prévue
ADR-R03 — Backend : le backend graphique consomme la représentation de frame produite ici, en pur consommateur passif, symétriquement au `World` qui consomme le Command Buffer (ADR-006).
