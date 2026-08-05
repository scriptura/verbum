# ADR-005 — Ressources

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
`02-concepts.md` établit qu'une donnée du moteur — composant ou ressource — se caractérise par quatre axes orthogonaux : cardinalité, mode d'adressage, durée de vie, politique de consommation. Un composant est une donnée de cardinalité 0..N adressée par `EntityId`. Cette ADR instancie ce même modèle pour le cas de cardinalité 1 : la Ressource.

Conformément à la lecture établie collectivement, cette ADR ne cherche pas à créer un mécanisme ECS distinct. Elle vérifie, point par point, qu'aucune règle nouvelle n'est nécessaire au-delà de ce qui est déjà posé en ADR-003 et ADR-004.

## Décision

### 1. Une Ressource est une donnée de cardinalité 1, identifiée par une clé unique
Une Ressource n'est pas un « singleton global » au sens objet du terme — elle n'implique ni identité propre, ni cycle de vie particulier, ni mécanisme d'accès dédié. C'est une donnée dont le cardinal est de 1 par instance de `World`, identifiée par une clé unique plutôt que par `EntityId`. Le choix concret de cette clé (type Rust, `TypeId`, identifiant généré par la Forge...) est une décision d'implémentation. Rien d'autre ne la distingue structurellement d'un composant.

### 2. Une Ressource participe aux mêmes contrats qu'un composant
Une Ressource peut être lue ou écrite par un système exactement comme un composant : elle apparaît dans l'interface déclarative du système (ADR-003), avec le même mode d'accès explicite (lecture/écriture) qu'exigé pour toute donnée consultée. Aucune Ressource n'est accessible par un système sans figurer dans son interface déclarative — le principe de localité (ADR-003) s'applique identiquement.

### 3. Une Ressource participe aux mêmes dépendances qu'un composant
Une relation de production → consommation impliquant une Ressource (un système écrit une Ressource, un autre la lit) est déduite exactement comme pour un composant persistant ou transitoire (ADR-004). Aucun mécanisme de dépendance distinct, aucune priorité particulière, aucun traitement spécial dans la construction du modèle d'exécution.

### 4. La durée de vie d'une Ressource instancie l'axe correspondant du modèle général
La durée de vie d'une Ressource (persistante ou transitoire) n'est pas une propriété qui lui serait propre : c'est une instanciation directe de l'axe « durée de vie » défini dans `02-concepts.md`, applicable identiquement à toute catégorie de donnée. Une Ressource persistante (ex: une configuration de jeu chargée par la Forge au démarrage) vit aussi longtemps que le `World`. Une Ressource transitoire (ex: un graphe de recherche de chemin partagé le temps d'une phase) suit les mêmes règles de durée de vie bornée et d'état initial déterministe déjà posées pour toute donnée transitoire.

### 5. Le propriétaire unique s'applique identiquement
Le principe « propriétaire unique » (`02-concepts.md`) s'applique à une Ressource comme à un composant persistant : au sein d'une phase, une Ressource persistante a un unique propriétaire en écriture. Tout système souhaitant l'influencer sans en être le propriétaire produit une donnée transitoire consommée par ce propriétaire, exactement selon le schéma déjà établi (`Produire est libre ; résoudre est unique`).

## Ce qui n'est pas tranché ici
Le mécanisme concret d'accès à une Ressource en Rust (paramètre de fonction typé, trait dédié, registre par `TypeId`...) est une décision d'implémentation. Le mécanisme de déclaration de la capacité ou de l'existence d'une Ressource par la Forge, s'il y a lieu, suit le même statut que pour les composants (cf. ADR-001, point 5) : décision d'implémentation, la Forge restant le mécanisme privilégié sans être obligatoire.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Une Ressource est une donnée de cardinalité 1 par `World`, identifiée par une clé unique | Invariant |
| Une Ressource participe aux contrats de système exactement comme un composant | Invariant (instanciation d'ADR-003) |
| Une Ressource participe aux dépendances production → consommation exactement comme un composant | Invariant (instanciation d'ADR-004) |
| Une Ressource peut être persistante ou transitoire selon les mêmes règles qu'un composant | Invariant (instanciation de `01-runtime-ecs.md`, section 4) |
| Le principe de propriétaire unique s'applique identiquement | Invariant (instanciation de `02-concepts.md`) |
| Mécanisme concret d'accès en Rust | Hors périmètre — implémentation |

## Constat — vérification du critère de validité
`02-concepts.md` pose désormais un critère de validité pour toute nouvelle catégorie de donnée : elle doit s'exprimer comme une instanciation des quatre axes du modèle général, sans nécessiter de nouvel invariant de contrat ou de dépendance. Cette ADR vérifie ce critère pour la Ressource :

| Axe | Instanciation pour la Ressource |
|---|---|
| Cardinalité | 1 par `World` |
| Mode d'adressage | Clé unique (implémentation : type Rust) |
| Durée de vie | Persistante ou transitoire, selon l'axe déjà défini |
| Politique de consommation | Lue/écrite tant que la ressource existe, propriétaire unique en écriture |

Les quatre axes sont renseignés sans qu'aucune section de cette ADR n'ait eu besoin de formuler un invariant nouveau au-delà de ceux déjà posés en ADR-003, ADR-004, `01-runtime-ecs.md` et `02-concepts.md`. Le critère de validité est donc satisfait.

## Suite prévue
ADR-006 — Command Buffer instanciera le même modèle général pour le cas d'une donnée de cardinalité 1, transitoire, et consommée selon une politique spécifique (accumulation puis vidage au point de validation de phase).
