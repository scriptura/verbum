# Document fondateur conceptuel — LPC (pipeline AOT) — v11

> **Statut** : brouillon / schéma directeur — à auditer avant finalisation.
> **Portée** : ce document couvre le **pipeline AOT** (transformation des PNG LPC en artefacts consommables). Le runtime n'est mentionné que pour fixer les **invariants de frontière**. Une spécification runtime concrète relève des phases ultérieures (8/9).
> **Exhaustivité** : ce document **n'est pas exhaustif** sur les animations. Il pose le **principe**, des **exemples illustratifs** et des **invariants**. La liste complète des animations relève des **YAML** d'extraction.

---

## 1. Portée et principes

Ce document établit le **paradigme** qui gouverne la transformation des spritesheets LPC (PNG) en données consommables par un runtime ECS/DOD.

Il ne décrit : ni un langage, ni une implémentation, ni un format binaire. Il décrit **ce qui doit rester vrai** quel que soit le langage, le format ou l'outillage.

### Principes fondateurs

1. **LPC est une banque de pixels, pas un format d'animation.**
2. **Toute complexité est absorbée AOT.**
3. **Le runtime ne manipule que des identifiants.**
4. **Le système d'animation est un service moteur global.**
5. **Le pipeline est un traducteur, pas un perroquet.**

### Convention d'indexation

**Toutes les bandes et colonnes sont indexées en 0-based**, par cohérence avec :
- la référence de mire utilisée pour la vérification visuelle des PNG,
- les conventions du générateur LPC officiel.

### Convention de vocabulaire — « row » vs « bande »

**Le champ YAML `rows` désigne des bandes verticales de 64 px**, pas des lignes d'animation LPC.

- Une **bande** = une tranche verticale de 64 px de haut.
- Une **ligne d'animation LPC** (une direction) = `frame_size / cellule_de_base` bandes consécutives.
- Pour une frame 64×64, 1 ligne d'animation = 1 bande.
- Pour une frame 128×128, 1 ligne d'animation = 2 bandes.
- Pour une frame 192×192, 1 ligne d'animation = 3 bandes.

Cette convention lève l'ambiguïté qui rendait incohérentes les mesures de dimensions PNG. La hauteur totale d'un PNG est `nombre_total_de_bandes × 64`.

### Convention de vocabulaire — pas de préfixes

**Les `MotorAction` ne portent pas de préfixe de catégorie.** Un préfixe est redondant, ajoute du bruit et crée une hiérarchie implicite discutable.

- On n'écrit pas `move_walk`, on écrit `walk`.
- On n'écrit pas `use_watering`, on écrit `watering`.
- On n'écrit pas `social_sit`, on écrit `sit`.

---

## 2. Architecture à trois étages

```
[ÉTAGE 1]   MotorAction          ← ce que le runtime connaît
                │
                │  résolution AOT (arme, bucket, contexte)
                ▼
[ÉTAGE 2]   StrokeProfile        ← ce que les PNG fournissent (extraction)
            CompositionProfile   ← comment les frames sont composées
                │
                │  extraction + composition AOT
                ▼
[ÉTAGE 3]   FrameSequence        ← ce que le runtime consomme
```

### Étage 1 — MotorAction

C'est le langage du gameplay. C'est ce que les systèmes écrivent, et la **seule** chose que le runtime connaît.

Exemples (liste **illustrative**, non exhaustive, **sans préfixes**) :
`attack`, `cast`, `walk`, `run`, `swim`, `idle`, `die`, `stagger`, `watering`, `hoe`, `shovel`, `sit`, `emote`…

- Indépendant de LPC, de l'arme, de la taille des sprites.
- Ne dit **rien** sur la cinématique ni sur les pixels.
- **Pas de préfixes de catégorie** : `walk` et non `move_walk` ; `watering` et non `use_watering`.

### Étage 2 — StrokeProfile et CompositionProfile

Deux notions **distinctes** :

- **StrokeProfile** — profil d'**extraction**. Décrit ce que les PNG fournissent : `rows` (bandes), `frame_count`, `directions`, `frame_size`.
- **CompositionProfile** — profil de **composition**. Décrit comment les frames extraites sont ordonnées en séquence. Peut référencer un StrokeProfile source (`source_stroke`).

**Exemples** :

| Nom | Type | Source | Remarque |
|---|---|---|---|
| `slash`, `thrust`, `shoot`, `spellcast`, `walk`, `run`, `jump`, `climb`, `idle`, `sit`, `hurt`, `emote`… | StrokeProfile | — | Extraction directe |
| `watering` | CompositionProfile | `thrust` | Séquence `[0,1,4,4,4,4,5]` |
| `tool_whip` | CompositionProfile | `slash` | Séquence `[0,1,2,3,4,5]`, 192×192 |
| `tool_axe` | CompositionProfile | `slash` | Séquence `[5,5,4,4,3,1,0,0,0,0]`, 128×128 |
| `swim` | CompositionProfile | `spellcast` | Faute de mieux pour l'instant |

- Les StrokeProfiles déterminent bandes, nombre de frames, directions.
- **Ni les StrokeProfiles ni les CompositionProfiles ne sont visibles du runtime.**

**Note** : la liste des `MotorAction` (§1) et la liste des profils (Stroke / Composition) sont **deux espaces distincts**. Une même action (`watering`) peut exister dans les deux, mais elles ne jouent pas le même rôle.

### Étage 3 — FrameSequence

Séquence linéaire, immuable, indexée, produite exclusivement AOT.

- Ne contient **aucun pixel**.
- Référence des frames stockées dans des structures globales (atlases, arrays).
- Le runtime n'itère que sur des **indices et offsets**.

---

## 3. Rôle du bucket (paramètre orthogonal)

| Bucket | Résolution |
|---|---|
| Small | 64×64 |
| Medium | 128×128 |
| Large | 192×192 |

- Bucket **orthogonal** au StrokeProfile.
- Toutes les frames d'un bucket ont même taille, même **centre géométrique d'alignement AOT** (à ne pas confondre avec le pivot d'ancrage gameplay — voir §5).
- Cellules hétérogènes **ajoutées en fin de PNG**, par ordre croissant.
- **Armes et outils uniquement** : les PNG > 64×64 sont réservés aux **Oversized Weapons** (armes surdimensionnées) **et aux outils** (pioche, hache, fouet…), qui se comportent comme une arme pour l'animation.
- Corps, cheveux, vêtements, armures, bouclier : **toujours** 64×64.
- Les layers non-arme/non-outil sont **reprojetés** vers le bucket cible imposé par l'arme ou l'outil.

### Trois familles de layers

| Famille | Exemples | Buckets possibles | Rôle |
|---|---|---|---|
| **Corporel** | corps, cheveux, vêtements, armures, bouclier | Small uniquement | Reprojeté vers le bucket cible |
| **Arme** | épée, arc, bâton… | Small / Medium / Large selon variantes | Détermine le bucket cible |
| **Outil** | pioche, hache, fouet… | Small / Medium / Large selon variantes | Détermine le bucket cible, comme une arme |

### Promotion automatique de bucket

**Le bucket cible est déterminé par le maximum des buckets disponibles pour l'arme (ou l'outil) équipé.**

- Sans arme/outil équipé : **Small par défaut**.
- Si l'AssetSet est en Small par défaut mais que l'arme n'a qu'une variante Large (ex. `slash_192` sans `slash` Small — cas du fouet), alors **promotion automatique vers Large**.
- Un outil qui n'a qu'une variante Medium sans variante Small : promotion vers Medium.

**Conséquence** : la cible de bucket n'est **pas** une propriété fixe d'AssetSet, mais une conséquence de l'équipement.

### Alignement des frames hétérogènes

**Toute frame source, quelle que soit sa taille (64×64, 128×128, 192×192…), est centrée sur un point d'ancrage commun avant reprojection.**

- Le **centre géométrique** de la frame source doit coïncider avec le **centre géométrique** de la frame cible dans le bucket.
- Cette convention est **invariante**, non négociable, non configurable.
- Elle est **indépendante** du pivot d'ancrage gameplay (qui peut être aux pieds, au centre, etc.).
- C'est une **décision de design**, pas une hypothèse empirique. Sa validation visuelle sur assets réels relève de la Phase 5 (implémentation). Si un décalage systématique est observé, la convention reste et un **offset par AssetSet** peut être ajouté pour compenser.

### Blocs multi-tailles dans un PNG

**Un PNG étendu peut contenir plusieurs blocs de tailles différentes empilés verticalement.**

Exemple concret (trident observé) :

```
Bloc 64×64      (bandes 0–53)
Bloc 128×128    (bandes 54–61, walk_128)
Bloc 192×192    (bandes 62–73, thrust_192)
```

**Règle de dimensionnement d'un bloc** :

```
bandes_occupées = len(directions) × (frame_size / cellule_de_base)
```

Où :
- `cellule_de_base` est une **propriété du corpus d'assets** (par défaut **64** pour le corpus humanoïde LPC).
- `frame_size / cellule_de_base` est un **multiplicateur entier** pour ce corpus : 64 → 1, 128 → 2, 192 → 3.

**Vérifications** :
- `spellcast` (64, 4 dir) : 4 × 1 = 4 bandes
- `walk_128` (128, 4 dir) : 4 × 2 = 8 bandes
- `slash_192` (192, 4 dir) : 4 × 3 = 12 bandes
- `hurt` (64, 1 dir) : 1 × 1 = 1 bande
- `climb` (64, 1 dir) : 1 × 1 = 1 bande

**Les blocs sont contigus, sans padding.** Le YAML est la seule source qui identifie où commence et où finit chaque bloc.

### Structure monotone des PNG étendus

**Le bloc étendu (bandes 21–53) est strictement identique d'un PNG à l'autre.** Les bandes sont **monotones de 0 à 53**.

- Aucun padding entre blocs.
- Tous les blocs étendus (128, 192) commencent **à partir de la bande 54**.
- Les variantes `*_128` et `*_192` **partagent les mêmes bandes** (pas de padding, pas de décalage). Une même zone de bandes peut porter une variante 128 ou 192 selon le PNG.

### Extensibilité à d'autres gabarits

Le paradigme n'est **pas spécifique aux humanoïdes LPC 64/128/192**. Il est **extensible** aux autres gabarits (monstres, créatures, boss) par configuration, sans réécriture :

- Les **buckets** sont configurables : `Small / Medium / Large` par défaut, extensibles à `Huge`, `Gigantic`, etc., **quand le besoin réel se présente**.
- La **cellule de base** est configurable par corpus (`64` pour LPC humanoïde, `32`, `128`, etc. pour d'autres).
- Les **frame sizes** sont tout multiple entier de la cellule de base.
- La **règle de dimensionnement** `bandes = directions × (frame_size / cellule_de_base)` reste valable.
- Les **StrokeProfiles** sont extensibles : la table canonique (LPC) peut être complétée par des profils propres à d'autres corpus (`fly`, `breathe_fire`, `swipe_tail`, etc.).
- **Aucune convention implicite** : chaque YAML déclare explicitement ses valeurs.

**Cas particulier — créatures à animation unique** : certaines créatures (slime, plante) peuvent n'avoir qu'**une seule animation** (repos) et **une seule direction**. Le paradigme supporte ce cas : un StrokeProfile à 1 direction, 1 frame, sans séquence custom.

---

## 4. Extraction, Composition, et propriétés AOT

### Extraction

**Ce qu'on lit** dans le PNG :

- quelles **bandes** LPC (champ `rows` du YAML),
- quel **nombre de frames source** (`frame_count`),
- quel **bucket cible**.

L'extraction produit un **pool de frames disponibles** pour un StrokeProfile donné.

### Composition

**Ce qu'on joue** à partir de ce pool :

- une **séquence ordonnée** d'indices,
- qui peut **répéter** une frame (`4-4-4-4`),
- **sauter** des frames (`0-1-4-5`),
- **réordonner** (`5-4-3-2-1-0`),
- ou toute **combinaison** (`0-1-4-4-4-4-5`).

La composition produit la **FrameSequence finale**, de longueur potentiellement différente du `frame_count` source.

### Contrat YAML — cinq propriétés

Un contrat d'extraction expose **cinq** propriétés :

1. `rows` — les **bandes** LPC à lire (bandes de 64 px)
2. `frame_count` — le nombre de frames source disponibles
3. `sequence` — l'ordre de jeu (optionnel, défaut = `0..frame_count-1`)
4. `frame_size` — la taille d'une frame (64, 128, 192…)
5. `directions` — la liste des directions couvertes par le StrokeProfile

Les trois premières décrivent l'**extraction et la composition** ; les deux dernières décrivent la **géométrie**.

### Indices locaux vs offsets globaux

**Les indices de `sequence` sont locaux** au pool extrait pour un StrokeProfile donné (0 à `frame_count-1`).

Le pipeline AOT les traduit en **offsets globaux** dans le buffer final :

1. `sequence` utilise des **indices locaux** au pool extrait.
2. L'AOT les remappe en **offsets globaux** dans le buffer final.
3. L'**index global `0`** est réservé à la **frame transparente canonique**.
4. Comme les indices locaux sont remappés, aucune collision n'est possible : la frame 0 d'un pool local devient un offset global ≥ 1.

### `frame_size` par bloc, pas par fichier

**Un même PNG peut contenir plusieurs blocs de tailles différentes.** Le `frame_size` n'est donc **pas** une propriété globale du fichier, mais une **propriété du bloc**.

**Décision (v11)** : **un YAML par bloc**, plus un **manifest par PNG/AssetSet** qui lie les blocs. Chaque YAML correspond à un bloc de taille uniforme dans un ou plusieurs PNG.

### Cohérence `bandes_occupées` ↔ `directions` ↔ `frame_size`

Le pipeline AOT **vérifie** la cohérence :

```
bandes_occupées attendues = len(directions) × (frame_size / cellule_de_base)
```

Si le YAML déclare `rows: [54..61]` (8 bandes) mais `directions: 4` et `frame_size: 128`, alors `4 × 2 = 8` ✓.

**Si incohérent → erreur de build.**

**Cas des CompositionProfiles** : un CompositionProfile dérivé d'un StrokeProfile source **hérite** de la validation de cohérence de son StrokeProfile source. Il n'a pas de validation propre tant qu'il ne définit pas ses propres bandes.

### Cas notable : partage de bandes entre profils

Un CompositionProfile peut **dériver d'un StrokeProfile** :

- `watering` dérive de `thrust` (`[0,1,4,4,4,4,5]`).
- `tool_whip` dérive de `slash` (`[0,1,2,3,4,5]`).
- `tool_axe` dérive de `slash` (`[5,5,4,4,3,1,0,0,0,0]`).

Les variantes oversize (`slash_192`, `slash_reverse_192`, `thrust_192`) **partagent les mêmes bandes** dans le générateur officiel, produisant différentes séquences des mêmes frames source.

Cela **valide** la séparation Extraction / Composition : la séquence est une propriété de premier ordre, distincte de l'extraction.

### Contiguïté mémoire

Le pipeline AOT garantit que **chaque FrameSequence est stockée de manière adjacente** dans un buffer global. Le runtime accède à une séquence par un **offset** dans ce buffer, sans double indirection.

Cette garantie est une **propriété du paradigme**, indépendante du format binaire concret (Phase 8).

---

## 5. Vocabulaire LPC vs vocabulaire interne

| Vocabulaire | Rôle | Homogénéité |
|---|---|---|
| **LPC** (PNG) | Entrée du pipeline | Hétérogène, subi |
| **Interne** (artefacts AOT) | Sortie du pipeline | Uniforme, choisi |

### Hétérogénéités observées

- Séparateur : `_` vs `-` selon la source (`slash_oversize` vs `slash-oversize`).
- Préfixes : `1h_*` (générateur) vs `backslash` / `halfslash` (plugin Godot).
- FrameCounts variables selon la source.
- **Composition de blocs multi-tailles dans un même PNG**.

### Décisions

- **Nomenclature interne uniformisée** : suffixes de taille en `_128`, `_192`.
  - Anciens `*_oversize` → `*_192` (`slash_192`, `slash_reverse_192`, `thrust_192`).
  - `walk_128`, `slash_128`, `walk_192` conservés tels quels.
  - Penser à `reverse`.
- **Terminologie communautaire** : « Oversized Weapons » / « armes surdimensionnées » désignent les animations dont la frame source dépasse 64×64. Les outils en relèvent également.
- **Le pipeline absorbe** ces hétérogénéités ; elles ne se propagent jamais au runtime.

### Hors-scope — terminologie obsolète

Les StrokeProfiles suivants sont placés **hors-scope** : ils existent physiquement dans certains PNG (bandes 46–53) mais la plupart des assets ne les supportent pas.

- `1h_slash`, `1h_backslash` — bandes 46–49 et 50–53. Correspondent à `handed_*` / `1_handed_*`. Terminologie obsolète.
- `backslash`, `halfslash` — nommages plugin, mêmes bandes. Obsolètes.
- `backslash_128`, `halfslash_128` — CompositionProfiles dérivés de profils hors-scope. **Hors-scope par héritage** (voir §11).

Ils sont conservés dans ce document **pour traçabilité**.

### Pivot d'ancrage — hors-scope AOT

**Le pivot d'ancrage gameplay** (typiquement `feet`, c'est-à-dire le centre-bas du sprite) est **hors-scope** du pipeline AOT.

- Il concerne le **runtime** : positionnement de l'entité dans le monde, hitbox de collision, gestion Y/Z.
- Il n'apparaît **pas** dans les YAML d'extraction.
- Il ne conditionne **pas** la reprojection AOT (qui est toujours alignée par **centre géométrique**).
- Il est porté par un **composant runtime dédié**, distinct de l'animation.

**Conséquence pour les YAML** : les YAML ne portent **aucun** champ `pivot`. Ils se limitent à `rows`, `frame_count`, `sequence`, `frame_size`, `directions`, plus leurs métadonnées de contrat (`contract_id`, `applies_to`, etc.).

---

## 6. Invariant de timing

- **Toutes les frames ont la même durée.**
- Pas de timing par frame.
- Tenir une pose = **répéter la frame** dans la séquence.

Justifications : résolution monotone, DOD-friendly, déterminisme strict.

### Politique de boucle — résolue AOT

**L'AOT résout intégralement la politique de boucle.** Le runtime n'a **aucune conscience sémantique** du concept de « boucle ».

- L'AOT **ne transmet pas** de flag `is_loop: bool`.
- Un booléen impliquerait une évaluation conditionnelle au runtime (`if flag`), ce qui contredirait l'invariant « pas de logique d'animation au runtime ».
- L'AOT **abaisse la sémantique de bouclage en une topologie de succession figée** : la séquence transmise est une suite d'offsets à parcourir linéairement.

**Conséquence** : une séquence qui « boucle » est une séquence dont la topologie de succession a été figée par l'AOT pour produire l'effet visuel attendu. Le runtime se contente d'incrémenter un curseur.

**Mécanisme concret à préciser** en Phase 5 (voir §13).

### Limite reconnue

L'invariant est calibré pour le **gameplay** (séquences LPC ≤ 13 frames). Il n'est pas prévu pour des animations de très longue durée ou des cinématiques procédurales, où un schéma de type RLE (`[frame_id, duration]`) pourrait devenir préférable.

Ces cas sont **hors-scope** et ne remettent pas en cause l'invariant pour le jeu.

---

## 7. Table canonique des profils

> **Note d'exhaustivité** : cette table est **illustrative**, non exhaustive. Elle présente les profils couramment rencontrés et les invariants qui les gouvernent. La liste complète relève des **YAML** d'extraction. De nouveaux CompositionProfiles (`hoe`, `shovel`, autres outils…) peuvent être ajoutés sans modifier ce document.

### Directions

| Contrainte | Valeur |
|---|---|
| Direction canonique | `[top, left, down, right]` |
| Exception `hurt` | `[down]` uniquement |
| Exception `climb` | `[top]` uniquement |
| Variantes oversize (armes, outils) | **toujours 4 directions**, sans exception |

### StrokeProfiles canoniques

| StrokeProfile | Directions | FrameCount | Sequence | Statut |
|---|---|---|---|---|
| `spellcast` | 4 | 7 | défaut | ✔ |
| `thrust` | 4 | 8 | défaut | ✔ |
| `walk` | 4 | 9 | `[1,2,3,4,5,6,7,8]` | ✔ |
| `slash` | 4 | 6 | défaut | ✔ |
| `shoot` | 4 | 13 | défaut | ✔ |
| `hurt` | 1 (`down`) | 6 | défaut | ✔ |
| `climb` | 1 (`top`) | 6 | défaut | ✔ |
| `run` | 4 | 8 | défaut | ✔ |
| `jump` | 4 | 5 | `[0,1,2,3,4,1]` | ✔ |
| `sit` | 4 | 3 | `[0×5,1×5,2×5]` | ✔ |
| `emote` | 4 | 3 | `[0×5,1×5,2×5]` | ✔ |
| `idle` | 4 | 2 | `[0,0,1]` | ✔ |

**Note sur `walk`** : `FrameCount = 9`. La frame 0 existe physiquement mais **tous les systèmes LPC l'ignorent**. La séquence commence donc à `1`.

### CompositionProfiles (exemples illustratifs)

| CompositionProfile | Source | Sequence | Bucket | FrameSize | Statut |
|---|---|---|---|---|---|
| `watering` | `thrust` | `[0,1,4,4,4,4,5]` | Small | 64×64 | ✔ |
| `tool_whip` | `slash` | `[0,1,2,3,4,5]` | Large | 192×192 | ✔ |
| `tool_axe` | `slash` | `[5,5,4,4,3,1,0,0,0,0]` | Medium | 128×128 | ✔ |
| `swim` | `spellcast` | à préciser | Small | 64×64 | ✔ |

**Note** : `hoe`, `shovel` et autres outils suivent le même principe (CompositionProfile dérivé d'un StrokeProfile existant). Leur définition appartient aux YAML.

### Variantes Oversized Weapons (uniformisées)

| StrokeProfile de base | Variante Small | Variante Medium | Variante Large |
|---|---|---|---|
| `walk` | `walk` | `walk_128` | `walk_192` |
| `thrust` | `thrust` | — | `thrust_192` |
| `slash` | `slash` | `slash_128` | `slash_192` |
| `slash_reverse` | — | — | `slash_reverse_192` |

Détails :

| Variante | FrameCount | Sequence | Bucket | FrameSize |
|---|---|---|---|---|
| `walk_128` | 9 | `[1,2,3,4,5,6,7,8]` | Medium | 128×128 |
| `walk_192` | 9 | `[1,2,3,4,5,6,7,8]` | Large | 192×192 |
| `slash_128` | 6 | défaut | Medium | 128×128 |
| `slash_192` | 6 | défaut | Large | 192×192 |
| `slash_reverse_192` | 6 | `[5,4,3,2,1,0]` | Large | 192×192 |
| `thrust_192` | 8 | défaut | Large | 192×192 |

### Bandes LPC canoniques (0-based)

Source : **observation directe sur PNG** (confirmée).

**Bloc canonique 64×64** (1 bande = 1 ligne d'animation) :

| StrokeProfile | Bandes (0-based) | Nb bandes |
|---|---|---|
| `spellcast` | 0–3 | 4 |
| `thrust` | 4–7 | 4 |
| `walk` | 8–11 | 4 |
| `slash` | 12–15 | 4 |
| `shoot` | 16–19 | 4 |
| `hurt` | 20 | 1 |
| `climb` | 21 | 1 |
| `idle` | 22–25 | 4 |
| `jump` | 26–29 | 4 |
| `sit` | 30–33 | 4 |
| `emote` | 34–37 | 4 |
| `run` | 38–41 | 4 |
| `combat_idle` | 42–45 | 4 (écarté) |
| `1h_slash` / `1h_backslash` (bloc 1) | 46–49 | 4 (hors-scope) |
| `1h_slash` / `1h_backslash` (bloc 2) | 50–53 | 4 (hors-scope) |

**Blocs oversize** (commencent à la bande 54, exemple trident) :

| StrokeProfile | Bandes (0-based) | FrameSize | Nb bandes |
|---|---|---|---|
| `walk_128` | 54–61 | 128×128 | 8 |
| `thrust_192` | 62–73 | 192×192 | 12 |

**Note critique** : le **bloc étendu (bandes 21–53) est strictement identique d'un PNG à l'autre**. Les bandes sont **monotones de 0 à 53**. Les blocs étendus (128, 192) commencent toujours à la bande 54.

### Alias et nommage primaire

- **`hurt`** = `death` (alias historique).
  Nom primaire : **`hurt`**.
  Justification : la cinématique représente une **chute au sol**, sans présumer la mort (évanouissement, choc, mort).

**Note** : `1h_slash`, `1h_backslash`, `backslash`, `halfslash` **ne sont pas des alias de `slash`**. Ils sont **hors-scope**.

### Profils écartés ou hors-scope

Conservés dans ce document **pour traçabilité**, avec justification.

| Profil | Statut | Raison |
|---|---|---|
| `combat_idle` | Écarté | Support **très limité** sur les layers essentiels. Inexploitable en composition multi-couches canonique. |
| `1h_slash` | Hors-scope | Bandes 46–49. Terminologie obsolète (`handed_*`). |
| `1h_backslash` | Hors-scope | Bandes 46–49 et 50–53. Terminologie obsolète. |
| `1h_halfslash` | Hors-scope | Terminologie obsolète. |
| `backslash` | Hors-scope | Nommage plugin, mêmes bandes. Obsolète. |
| `halfslash` | Hors-scope | Nommage plugin, mêmes bandes. Obsolète. |
| `backslash_128` | Hors-scope par héritage | CompositionProfile dérivé de `backslash` (hors-scope). |
| `halfslash_128` | Hors-scope par héritage | CompositionProfile dérivé de `halfslash` (hors-scope). |

**Invariant de sélection** :
> Un StrokeProfile n'est canonique que si **tous les layers essentiels le supportent**. La détermination des layers essentiels est **empirique** : le développeur intègre une liste blanche fondée sur les assets effectivement présents et testés. Elle n'est pas tenue explicite dans ce document.

**Invariant d'héritage** :
> Un CompositionProfile dérivé d'un StrokeProfile **hors-scope** est **hors-scope par construction**.

---

## 8. Table de résolution `MotorAction → StrokeProfile`

**Artefact central du paradigme.** Il matérialise l'idée fondatrice :

> **Une action moteur peut correspondre à plusieurs cinématiques, selon le contexte.**

### Structure conceptuelle

```
MotorAction
├── default                  → StrokeProfile ou CompositionProfile par défaut
├── by_weapon                → selon l'arme (ou outil) équipé(e)
└── fallback                 → politique si aucune règle ne matche
```

**Note (v11)** : `by_speed` et `by_context` **disparaissent** au profit de l'éclatement des `MotorAction`. `walk` et `run` sont deux MotorActions distincts ; `watering` et `hoe` sont deux MotorActions distincts. Aucun préfixe de catégorie.

### Exemples illustratifs

```
attack
├── default:      slash
├── by_weapon:
│   ├── spear:        thrust
│   ├── crossbow:     thrust
│   ├── bow:          shoot
│   ├── slingshot:    shoot
│   └── longsword:    slash

walk
├── default:      walk
└── by_bucket:
    ├── medium:   walk_128
    └── large:    walk_192

run
└── default:      run

swim
└── default:      swim   (CompositionProfile dérivé de spellcast)

cast
└── default:      spellcast

idle
└── default:      idle

die
└── default:      hurt

stagger
└── default:      hurt

watering
└── default:      watering   (CompositionProfile, source thrust)

sit
└── default:      sit

emote
└── default:      emote
```

### Plusieurs intentions, une cinématique

Une même cinématique peut servir **plusieurs intentions gameplay** :

| MotorAction | StrokeProfile | Traitement runtime différencié (hors-scope AOT) |
|---|---|---|
| `die` | `hurt` | Chute au sol, disparition de l'entité |
| `stagger` | `hurt` | Chute au sol, clignotement shader (si le moteur le gère) |

Le StrokeProfile `hurt` reste unique — c'est **la même animation** LPC.

### Matérialisation runtime

La structure arborescente décrite ci-dessus est un **concept AOT**. Sa matérialisation runtime est une **LUT plate et multidimensionnelle** :

```
[LayerId][BucketId][MotorAction][Direction][WeaponId] → SequenceId
```

- **Aucun parcours d'arbre, aucun hashing dynamique au runtime.**
- Le `StrokeProfile` et le `CompositionProfile` restent des **concepts AOT internes** : ils servent au build, au nommage, au debug. Ils **disparaissent du binaire final**.
- Le runtime résout directement `SequenceId` depuis la signature gameplay, en une seule indirection.
- `BucketId` est **imposé par l'arme ou l'outil équipé(e)** (ou Small par défaut, avec promotion automatique).
- Le `WeaponId` est un **index**, pas un prédicat.

**Note** : les dimensions exactes, les types d'index et l'organisation mémoire relèvent de la Phase 8.

### Note terminologique

Quatre notions **distinctes** :

| Notion | Niveau | Nature |
|---|---|---|
| **Convergence** | Étage 1 → 2 | Plusieurs `MotorAction` se résolvent vers un même profil. Décision sémantique de design. |
| **Composition de séquence** | Étage 2 → 3 | Réordonnancement, répétition, saut de frames pour produire une `FrameSequence`. |
| **Interning** | Étage 3 (AOT) | Déduplication **mécanique** : deux séquences identiques partagent le même stockage. |
| **Assemblage multi-couches** | Runtime | Superposition spatiale de layers pour un même AssetSet. |

---

## 9. Reprojection, composition multi-couches, interning

### Reprojection

Pour chaque combinaison `(LayerId, MotorAction, Direction, contexte)` :

1. Résoudre le profil cible (StrokeProfile ou CompositionProfile) via la table (§8).
2. Extraire les bandes du PNG source.
3. Reprojeter dans le bucket cible :
   - **Créer une toile vide `BucketSize × BucketSize`.**
   - **Calculer l'offset centré** : `offset = (BucketSize - source_size) / 2`.
   - **Blitter la source** à cet offset.
   - **Padding transparent** autour.
4. Si le layer n'a **pas** de variante pour ce profil ou ce bucket : référencer la **frame transparente canonique** (index global réservé `0`).
5. Composer la séquence finale selon la règle de composition (§4).
6. Empaqueter en séquence unifiée.

**Aucune condition runtime. Aucune duplication de frame.**

### Invariant de validation : `source_size ≤ BucketSize`

**Une frame source ne peut jamais être plus grande que son bucket cible.**

Si `source_size > BucketSize` : **erreur de build**.

**Cette validation est effectuée au build, jamais au runtime.**

### Politique de bucket cible

- **Cible déterminée par le maximum des buckets disponibles pour l'arme/outil équipé(e).**
- Sans équipement : **Small**.
- Promotion automatique si l'arme/outil n'a que Medium ou Large.

### Fallback transparent vs erreur de build

Deux cas **distincts** :

**Cas 1 — animation requise manquante → erreur de build.**
Un AssetSet déclare la liste des animations **requises** (whitelist empirique). Pour chaque animation requise, chaque layer de l'AssetSet doit fournir la variante correspondante. Sinon → **erreur de build explicite**. Le runtime ne teste jamais l'existence.

**Cas 2 — animation non requise absente pour un layer → frame transparente.**
Un layer peut ne pas avoir de variante pour une animation non requise, tout en devant coexister avec les autres. Une **frame transparente** (index global `0`) est injectée par l'AOT. La séquence transparente est de **même longueur** que la séquence de base pour ce `MotorAction`, afin que le runtime puisse itérer en lockstep sur tous les layers.

| Situation | Traitement |
|---|---|
| Layer ne peut pas fournir une animation **requise** | **Erreur de build** |
| Layer n'a pas une animation **non requise** | **Frame transparente** injectée, longueur alignée |

### Interning — périmètre

**L'interning opère au sein d'un même layer (ou d'une même variante de layer).**

- Deux layers distincts ne peuvent pas partager de séquences : leurs offsets globaux pointent vers des **pixels différents**.
- L'interning capture :
  - les **convergences** (`die`/`stagger` → `hurt` pour un même layer),
  - les **duplications internes** à un même layer.

**Propriété du pipeline AOT**, pas une décision du runtime.

### Anti-explosion combinatoire

**Décision structurante** : l'AOT pré-calcule **une séquence par (layer, bucket)**, jamais par combinaison de layers.

- La **LUT runtime est indexée par layer atomique** (et par bucket).
- Le runtime **résout une séquence par layer** indépendamment, puis **superpose** les layers à l'écran.
- Le coût mémoire AOT est proportionnel au **nombre de variantes de layer × nombre de buckets pertinents**.
- Chaque séquence est **partagée** entre toutes les entités qui portent le même layer et le même bucket.

**Chiffrage indicatif** (~70 000 séquences) :

| Type de layer | Buckets possibles | Séquences estimées |
|---|---|---|
| Corps | Small | ~10 000 |
| Cheveux | Small | ~8 000 |
| Vêtements | Small | ~16 000 |
| Arme | Small + Medium + Large | ~30 000 |
| Outil | Small + Medium + Large | (inclus dans arme) |
| Overlays | Small | ~5 000 |
| **Total** | | **~70 000 séquences** |

**Corollaire DOD** : la composition runtime est **branchless** et **O(layers)** par entité.

---

## 10. Frontière AOT / runtime

Cette section fixe **les invariants de frontière** — ce que le runtime doit connaître et ignorer pour que le contrat AOT tienne.

### Connaît
- `LayerId` (un layer atomique)
- `BucketId` (imposé par l'arme/outil équipé(e), ou Small par défaut avec promotion)
- `MotorAction`
- `Direction`
- Référence à l'arme ou l'outil équipé(e) (composant dédié)
- La **taille de frame résolue** (via métadonnée de séquence/atlas)

### Ignore
- LPC, bandes, PNG, StrokeProfiles, CompositionProfiles.
- Les buckets **comme entités sémantiques** (mais connaît `BucketId` comme index).
- Le pivot d'ancrage gameplay (relève d'un composant dédié).

### Fait
- Écrit `MotorAction` + `Direction`.
- **Indexe par l'arme/outil équipé(e)** (via `WeaponId`), **sans branche conditionnelle**. L'arme est un **index**, pas un **prédicat**.
- Lit un identifiant de séquence et un index de frame.
- Superpose les layers à l'écran dans l'ordre canonique d'empilement.

### Ne fait jamais
- Déduire, parser, inférer.
- **Brancher conditionnellement sur l'arme** (`if weapon == spear { … }`).
- Brancher sur taille, couche.
- Spécialiser Player / NPC / MOB / Boss.
- Évaluer une politique de boucle (résolue AOT, §6).

### Frontière temporelle

L'AOT fige la **topologie spatiale** (VRAM) et **ordinale** (séquence). Le runtime reste **maître du domaine temporel** :

- **Tick rate** : décidé au runtime.
- **Transitions** : gérées au runtime.
- **Politique de boucle** : **résolue AOT**, abaissée en topologie de succession.

Le runtime ne fait que **cadencher** la lecture, il ne modifie jamais l'ordre des frames.

---

## 11. Invariants LPC préservés

- Directions canoniques, sauf exceptions (`hurt`, `climb`).
- Bandes LPC 0-based, canoniques.
- FrameCounts figés par profil source.
- Séquences explicites si non triviales, uniformément jouées.
- Overlays non normalisables globalement, traitement au cas par cas (Phase 5).
- **Un PNG peut contenir plusieurs blocs de tailles différentes (64, 128, 192).**
- **Un bloc de taille N × cellule_de_base occupe N × len(directions) bandes.**
- **Les blocs sont contigus, sans padding.**
- **Le bloc étendu (bandes 21–53) est strictement identique d'un PNG à l'autre. Bandes monotones de 0 à 53.**
- **Les blocs étendus (128, 192) commencent à la bande 54.**
- **Les variantes `*_128` et `*_192` partagent les mêmes bandes.**
- **Les bandes de départ des blocs étendus sont stables ; le YAML déclare les bandes réelles.**
- **Toute frame est centrée sur le centre géométrique de sa toile cible.**
- **Une frame source ne peut jamais être plus grande que son bucket cible** (erreur de build sinon).
- **Le pivot d'ancrage gameplay (feet) est hors-scope du pipeline AOT.**
- **Les frames commencent toujours à x = 0.**
- **Les variantes oversize ont toujours 4 directions.**
- « L'arme pilote la cinématique du corps, pas l'inverse ». Seule l'**arme d'attaque** (ou l'outil) détermine la cinématique. Bouclier surajouté, sans impact. **Jamais de dual wield.**
- **Trois familles de layers** : corporel (Small), arme (Small/Medium/Large), outil (Small/Medium/Large).
- **Promotion automatique de bucket** : cible = maximum des buckets disponibles pour l'arme/outil équipé(e).
- Un StrokeProfile n'est canonique que si **tous** les layers essentiels le supportent (whitelist empirique).
- **Un CompositionProfile dérivé d'un StrokeProfile hors-scope est hors-scope par construction.**
- Terminologie « Oversized Weapons » / « armes surdimensionnées » pour les animations source > 64×64 (armes et outils).
- **Nomenclature interne uniformisée** : `_128`, `_192`.
- **`1h_*`, `backslash`, `halfslash` : hors-scope.**
- **Politique de boucle résolue AOT** : pas de flag `is_loop`, abaissement en topologie de succession.
- **Interning par layer** (ou variante de layer).
- **Séquence pré-calculée par (layer, bucket).**
- **Fallback transparent vs erreur de build** : distinction animation requise / non requise.
- **Le document n'est pas exhaustif sur les animations** : les YAML le sont.

---

## 12. Sources de vérité

| Source | Rôle | Fiabilité |
|---|---|---|
| **YAML** | Vérité sémantique de l'extraction | Autoritaire |
| **Liste des assets déclarés au build** | Vérité d'existence | Autoritaire |
| **Disque** | Vérité physique des fichiers | Autoritaire |
| **PNG eux-mêmes** | Vérité ultime des pixels, bandes, frames | **Autoritaire** |
| **Plugin Godot LPCAnimatedSprite2D** | Source croisée, riche | **Fiable, non autoritaire** |
| **Générateur web** | Source croisée | **Fiable, non autoritaire** |

Règle stricte :

```
Asset utilisé = déclaré au build ∩ présent sur disque ∩ déclaré en YAML
```

Toute divergence = **erreur de build**.

### Note sur la fiabilité des sources croisées

Les données issues du plugin Godot ou du générateur web sont **fiables mais non autoritaires**. **La vérification sur PNG prime toujours.**

---

## 13. Points ouverts

### Confirmés (retirés des points ouverts)

- ✔ Bandes LPC canoniques (0-based).
- ✔ **Les « rows » du YAML sont des bandes de 64 px.**
- ✔ **Le bloc étendu (bandes 21–53) est strictement identique d'un PNG à l'autre.**
- ✔ **Les blocs étendus (128, 192) commencent à la bande 54.**
- ✔ **Les variantes `*_128` et `*_192` partagent les mêmes bandes.**
- ✔ `walk_128` = 8 bandes (4 dir × 2), **FrameCount = 9**.
- ✔ `slash_192` = 12 bandes (4 dir × 3).
- ✔ `thrust_192` = 12 bandes (4 dir × 3).
- ✔ Absence de padding entre blocs.
- ✔ Alignement par **centre géométrique** (décision de design).
- ✔ Pivot d'ancrage (feet) hors-scope AOT.
- ✔ Validation `source_size ≤ BucketSize` → erreur de build.
- ✔ **Dimensions PNG** : trident = 4736 px = 74 bandes × 64 ; rapière = 4224 px = 66 bandes × 64.
- ✔ **Nomenclature uniformisée** `_128` / `_192`.
- ✔ **`1h_*`, `backslash`, `halfslash` : hors-scope.**
- ✔ **Héritage hors-scope** : invariant acté.
- ✔ **Trois familles de layers** : corporel / arme / outil.
- ✔ **Promotion automatique de bucket** actée.
- ✔ **Politique de boucle résolue AOT**.
- ✔ **Interning par layer.**
- ✔ **Séquence pré-calculée par (layer, bucket).**
- ✔ **Fallback transparent vs erreur de build** : distinction actée.
- ✔ **MotorActions sans préfixes.**
- ✔ **`watering`, `tool_whip`, `tool_axe`, `swim` : CompositionProfiles.**
- ✔ **Document non exhaustif** sur les animations.

### À vérifier sur PNG (Phase 5)

- **A3 — Les nouveaux outils (`tool_whip`, `tool_axe`) partagent-ils les bandes de `slash` ?**

### À trancher (conceptuel)

- **B6 — Ordre d'empilement canonique** : liste proposée en v10, à valider visuellement ou figer.
- **B7 — Nature conceptuelle des overlays** : layer à part entière, ou variante d'un layer existant ?
- **B8 — Mapping bandes → directions** : convention `[top, left, down, right]`, à figer ou renvoyer Phase 8.
- **B9 — Mécanisme concret de boucle AOT** : comment l'AOT produit la topologie de succession (séquence allongée, cyclique, autre) ?
- **B10 — Schéma YAML exact** : `type` (StrokeProfile / CompositionProfile) déclaré ? `source_stroke` champ de premier ordre ? `bucket` déclaré ou déduit ?
- **B11 — Validation des CompositionProfiles** : acté que le CompositionProfile hérite de la validation de sa source. Cas des CompositionProfiles sans source unique à préciser.
- **C2 — Nom du champ YAML** : `rows` (compatibilité LPC) ou `bands` (cohérence sémantique) ?
- **C9 — Bouclier** : famille « arme » ou « corporel » ? (probablement corporel)
- **C10 — Promotion de bucket pour outils** : même politique que pour les armes ?

---

## 14. Ce que ce document écarte

- Dépendance à un langage ou framework.
- Signatures, types concrets, formats binaires figés.
- Décisions d'outillage.
- Implémentations antérieures obsolètes (le « cadavre C# »).
- **Pivot d'ancrage gameplay** — hors-scope, runtime only.
- **Spécification runtime concrète** — seuls les invariants de frontière sont posés (§10).
- **Sémantique runtime du bouclage** — résolue AOT.
- **Terminologie obsolète** `1h_*`, `backslash`, `halfslash`.
- **Préfixes de catégorie sur les MotorActions.**
- **Exhaustivité sur les animations** — le document pose le principe, les YAML listent.

---

## 15. Synthèse en une phrase

> **Le pipeline fournit au moteur un contrat strict : les PNG LPC et leurs conventions sont une source absorbée AOT ; le runtime ne manipule que des actions, des directions, des identifiants de layer, de bucket et de séquence, sans jamais connaître ni LPC, ni les StrokeProfiles, ni les pixels, ni la sémantique de bouclage.**

