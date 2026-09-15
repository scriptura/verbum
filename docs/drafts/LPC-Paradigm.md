# Document fondateur conceptuel — LPC (pipeline AOT) — v14

> **Statut** : brouillon / schéma directeur — à auditer avant finalisation.
> **Portée** : ce document couvre le **pipeline AOT** (transformation des PNG LPC en artefacts consommables). Le runtime n'est mentionné que pour fixer les **invariants de frontière**.
> **Exhaustivité** : ce document **n'est pas exhaustif** sur les animations. Il pose le **principe**, des **exemples illustratifs** et des **invariants**. La liste complète des animations relève des **YAML**.

---

## 1. Portée et principes

Ce document établit le **paradigme** qui gouverne la transformation des spritesheets LPC (PNG) en données consommables par un runtime ECS/DOD.

### Principes fondateurs

1. **LPC est une banque de pixels, pas un format d'animation.**
2. **Toute complexité est absorbée AOT.**
3. **Le runtime ne manipule que des identifiants.**
4. **Le système d'animation est un service moteur global.**
5. **Le pipeline est un traducteur, pas un perroquet.**

### Convention d'indexation

**Toutes les rows et colonnes sont indexées en 0-based**, par cohérence avec :
- la référence de mire utilisée pour la vérification visuelle des PNG,
- les conventions du générateur LPC officiel.

### Convention de vocabulaire — le mot « row »

**Dans ce document, « row » désigne une ligne d'animation LPC** — c'est-à-dire **une direction**, exactement comme dans la convention LPC standard, Godot ou le générateur officiel.

- Une animation à 4 directions occupe **4 rows**.
- Une animation à 1 direction (`hurt`, `climb`) occupe **1 row**.
- La taille d'une frame (`frame_size`) est une propriété **indépendante** des rows : un PNG peut contenir des frames 64×64, 128×128 ou 192×192, chacune occupant une row logique distincte.

La structure physique d'un PNG (comment une row logique se traduit en bandes de pixels) est un **détail d'implémentation du pipeline AOT**. Elle relève de la Phase 5. Le présent document traite `rows` comme une **unité logique** stable.

### Convention de vocabulaire — pas de préfixes

**Les `MotorAction` ne portent pas de préfixe de catégorie.**

- On n'écrit pas `move_walk`, on écrit `walk`.
- On n'écrit pas `use_watering`, on écrit `watering`.

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

### Étage 2 — StrokeProfile et CompositionProfile

Deux notions **distinctes** :

- **StrokeProfile** — profil d'**extraction**. Décrit ce que les PNG fournissent : `rows`, `frame_count`, `directions`.
- **CompositionProfile** — profil de **composition**. Décrit comment les frames extraites sont ordonnées en séquence. Peut référencer un ou plusieurs StrokeProfiles sources.

**Exemples** :

| Nom | Type | Source | Remarque |
|---|---|---|---|
| `slash`, `thrust`, `shoot`, `spellcast`, `walk`, `run`, `jump`, `climb`, `idle`, `sit`, `hurt`, `emote`… | StrokeProfile | — | Extraction directe |
| `watering` | CompositionProfile | `thrust` | Séquence `[0,1,4,4,4,4,5]` |
| `tool_whip` | CompositionProfile | `slash` | Séquence `[0,1,2,3,4,5]` |
| `tool_axe` | CompositionProfile | `slash` | Séquence `[5,5,4,4,3,1,0,0,0,0]` |
| `swim` | CompositionProfile | `spellcast` | Faute de mieux pour l'instant |

**Note** : la liste des `MotorAction` (§1) et la liste des profils (Stroke / Composition) sont **deux espaces distincts**.

### Étage 3 — FrameSequence

Séquence linéaire, immuable, indexée, produite exclusivement AOT.

- Ne contient **aucun pixel**.
- Référence des frames stockées dans des structures globales.
- Le runtime n'itère que sur des **indices et offsets**.

---

## 3. Rôle du bucket (paramètre orthogonal)

| Bucket | Résolution |
|---|---|
| Small | 64×64 |
| Medium | 128×128 |
| Large | 192×192 |

- Bucket **orthogonal** au StrokeProfile.
- Toutes les frames d'un bucket ont même taille, même **centre géométrique d'alignement AOT**.
- Cellules hétérogènes **ajoutées en fin de PNG**, par ordre croissant.
- **Armes et outils uniquement** : les PNG > 64×64 sont réservés aux **Oversized Weapons** et aux **outils** (pioche, hache, fouet…).
- Corps, cheveux, vêtements, armures, bouclier : **toujours** 64×64.
- Les layers non-arme/non-outil sont **reprojetés** vers le bucket cible imposé par l'arme ou l'outil.

### Trois familles de layers

| Famille | Exemples | Buckets possibles | Rôle |
|---|---|---|---|
| **Corporel** | corps, cheveux, vêtements, armures, bouclier | Small uniquement | Reprojeté vers le bucket cible |
| **Arme** | épée, arc, bâton… | Small / Medium / Large | Détermine le bucket cible |
| **Outil** | pioche, hache, fouet… | Small / Medium / Large | Détermine le bucket cible, comme une arme |

### Promotion automatique de bucket

**Le bucket cible est déterminé par le maximum des buckets disponibles pour l'arme (ou l'outil) équipé.**

- Sans arme/outil équipé : **Small par défaut**.
- Si l'AssetSet est en Small par défaut mais que l'arme n'a qu'une variante Large : **promotion automatique vers Large**.
- **La promotion s'applique identiquement aux armes et aux outils.**

### Alignement des frames hétérogènes

**Toute frame source, quelle que soit sa taille (64×64, 128×128, 192×192…), est centrée sur un point d'ancrage commun avant reprojection.**

- Le **centre géométrique** de la frame source doit coïncider avec le **centre géométrique** de la frame cible dans le bucket.
- Cette convention est **invariante**, non négociable, non configurable.
- Elle est **indépendante** du pivot d'ancrage gameplay (qui est hors-scope AOT).
- C'est une **décision de design**. Sa validation visuelle a été effectuée via le générateur officiel.

### Blocs multi-tailles dans un PNG

**Un PNG étendu peut contenir plusieurs blocs de tailles différentes empilés verticalement.**

Exemple concret (trident observé) :

```
Bloc 64×64      (rows 0–53)
Bloc 128×128    (rows 54–61, walk_128)
Bloc 192×192    (rows 62–73, thrust_192)
```

**Structure monotone des PNG étendus :**

- Le bloc étendu (rows 21–53) est **strictement identique d'un PNG à l'autre**.
- Les rows sont **monotones de 0 à 53**.
- Tous les blocs étendus (128, 192) **commencent au row 54**.
- Les variantes `*_128` et `*_192` **partagent les mêmes rows** (pas de padding).

**Les blocs sont contigus, sans padding.** Le YAML est la seule source qui identifie où commence et où finit chaque bloc.

**Note** : la correspondance entre rows logiques (une direction) et structure physique des PNG (bandes de 64 px) relève du pipeline AOT et sera précisée en Phase 5. Le présent document traite `rows` comme unité logique.

### Extensibilité à d'autres gabarits

Le paradigme est **extensible** aux autres gabarits (monstres, créatures, boss) par configuration :

- **Buckets** configurables.
- **Cellule de base** configurable par corpus (`64` pour LPC humanoïde).
- **Frame sizes** : tout multiple entier de la cellule de base.
- **StrokeProfiles** extensibles (`fly`, `breathe_fire`, `swipe_tail`…).
- **Aucune convention implicite** : chaque YAML déclare explicitement ses valeurs.

**Cas particulier — créatures à animation unique** : 1 direction, 1 frame, sans séquence custom.

---

## 4. Extraction, Composition, et propriétés AOT

### Extraction

**Ce qu'on lit** dans le PNG :

- quelles **rows** LPC (champ `rows` du YAML),
- quel **nombre de frames source** (`frame_count`),
- quel **bucket cible**.

### Composition

**Ce qu'on joue** à partir de ce pool :

- une **séquence ordonnée** d'indices,
- qui peut **répéter**, **sauter**, **réordonner**, ou toute **combinaison**.

### Deux types de YAML

Le pipeline distingue **deux types de YAML** aux rôles complémentaires :

**1. YAML canonique d'animations** — un seul, liste **toutes** les animations du corpus.

Chaque entrée porte :
- `name` — nom de l'animation,
- `type` — `StrokeProfile` (défaut) ou `CompositionProfile`,
- `frame_count` — nombre de frames source,
- `directions` — liste des directions,
- `sequence` — ordre de jeu (optionnel, défaut = `0..frame_count-1`),
- `source_stroke` — pour les `CompositionProfile` uniquement.

**2. YAML d'équipement** — lie une arme ou un outil à ses variantes et à leurs rows dans les PNG.

Chaque entrée porte :
- `equipment_id` — identifiant de l'arme/outil,
- `bucket` — bucket disponible,
- `rows` — rows LPC du bloc concerné,
- `animation` — référence à une entrée du YAML canonique.

**Répartition des responsabilités :**

| Champ | YAML canonique | YAML d'équipement |
|---|---|---|
| `name` | ✔ | — |
| `type` | ✔ | — |
| `frame_count` | ✔ | — |
| `directions` | ✔ | — |
| `sequence` | ✔ | — |
| `source_stroke` | ✔ (CompositionProfiles) | — |
| `rows` | — | ✔ |
| `bucket` | — | ✔ |
| `equipment_id` | — | ✔ |

### Contrat YAML d'équipement — schéma révisé

Exemple concret (basé sur les YAML legacy, révisés) :

```yaml
contract_id: "LPC_Base_64"

applies_to:
  path_prefix: "Textures/Characters/"

target_bucket: "Small"
align: "center"

stroke_profiles:
  spellcast:
    reference: spellcast
    rows: [0, 1, 2, 3]

  thrust:
    reference: thrust
    rows: [4, 5, 6, 7]

  walk:
    reference: walk
    rows: [8, 9, 10, 11]

  slash:
    reference: slash
    rows: [12, 13, 14, 15]

  shoot:
    reference: shoot
    rows: [16, 17, 18, 19]

  hurt:
    reference: hurt
    rows: [20]
```

**Notes** :
- `frame_count`, `directions`, `sequence` **ne sont plus déclarés ici** — ils vivent dans le YAML canonique.
- `reference` lie l'entrée du contrat à son animation canonique.
- `rows` reste local au contrat (dépend du PNG).
- `pivot` disparaît (hors-scope AOT).
- `align: "center"` remplace `pivot: "center"` pour désambiguïser.
- Indexation 0-based.

### Indices locaux vs offsets globaux

**Les indices de `sequence` sont locaux** au pool extrait pour un StrokeProfile donné.

Le pipeline AOT les remappe en **offsets globaux** dans le buffer final. L'**index global `0`** est réservé à la **frame transparente canonique**.

### Cohérence `rows` ↔ `directions`

Le pipeline AOT **vérifie** la cohérence :

```
len(rows) == len(directions)
```

Si le YAML déclare `rows: [4,5,6,7]` (4 rows) mais `directions: [top, left, down]` (3 dir) → **erreur de build**.

### Cas notable : partage de rows entre profils

Un CompositionProfile peut **dériver d'un StrokeProfile** :

- `watering` dérive de `thrust`.
- `tool_whip` dérive de `slash`.
- `tool_axe` dérive de `slash`.

Les variantes oversize (`slash_192`, `slash_reverse_192`, `thrust_192`) **partagent les mêmes rows**.

---

## 5. Vocabulaire LPC vs vocabulaire interne

| Vocabulaire | Rôle | Homogénéité |
|---|---|---|
| **LPC** (PNG) | Entrée du pipeline | Hétérogène, subi |
| **Interne** (artefacts AOT) | Sortie du pipeline | Uniforme, choisi |

### Décisions

- **Nomenclature interne uniformisée** : suffixes de taille en `_128`, `_192`.
  - Anciens `*_oversize` → `*_192`.
- **Noms d'animations en lowercase** : `spellcast`, `thrust`, `walk`, `slash`, `shoot`, `hurt`.

### Hors-scope — terminologie obsolète

- `1h_slash`, `1h_backslash`, `backslash`, `halfslash` — rows 46–53. Obsolètes.
- `backslash_128`, `halfslash_128` — **hors-scope par héritage**.

### Pivot d'ancrage — hors-scope AOT

**Le pivot d'ancrage gameplay** est **hors-scope** du pipeline AOT.

- Il concerne le **runtime**.
- Il n'apparaît **pas** dans les YAML.
- Il ne conditionne **pas** la reprojection AOT (alignée par **centre géométrique**).

**Note** : le champ `align` (valeur `"center"`) dans les YAML désigne **l'alignement AOT** (centre géométrique). Il ne doit pas être confondu avec le pivot d'ancrage gameplay.

---

## 6. Invariant de timing

- **Toutes les frames ont la même durée.**
- Tenir une pose = **répéter la frame** dans la séquence.

### Politique de boucle — résolue AOT

**Mécanisme retenu — graphe de succession.**

L'AOT construit un **graphe de succession** entre séquences :

- `séquence A → séquence A` pour un **loop**.
- `séquence A → séquence B` pour une **transition**.

Chaque `FrameSequence` porte un champ `next_sequence_id`.

**Conséquence runtime** : zéro allocation, topologie dictée par AOT, aucune connaissance sémantique de la boucle.

### Limite reconnue

L'invariant est calibré pour le gameplay (séquences ≤ 13 frames). Cinématiques longues : hors-scope.

---

## 7. Table canonique des profils

> **Note d'exhaustivité** : cette table est **illustrative**. La liste complète relève du YAML canonique.

### Directions

| Contrainte | Valeur |
|---|---|
| Direction canonique | `[top, left, down, right]` |
| Exception `hurt` | `[down]` uniquement |
| Exception `climb` | `[top]` uniquement |
| Variantes oversize (armes, outils) | **toujours 4 directions** |

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

**Note sur `walk`** : `FrameCount = 9`. La frame 0 existe mais est ignorée. Séquence à partir de `1`.

### CompositionProfiles (exemples illustratifs)

| CompositionProfile | Source | Sequence | Statut |
|---|---|---|---|
| `watering` | `thrust` | `[0,1,4,4,4,4,5]` | ✔ |
| `tool_whip` | `slash` | `[0,1,2,3,4,5]` | ✔ |
| `tool_axe` | `slash` | `[5,5,4,4,3,1,0,0,0,0]` | ✔ |
| `swim` | `spellcast` | à préciser | ✔ |

### Variantes Oversized Weapons (uniformisées)

| StrokeProfile de base | Variante Small | Variante Medium | Variante Large |
|---|---|---|---|
| `walk` | `walk` | `walk_128` | `walk_192` |
| `thrust` | `thrust` | — | `thrust_192` |
| `slash` | `slash` | `slash_128` | `slash_192` |
| `slash_reverse` | — | — | `slash_reverse_192` |

### Rows LPC canoniques (0-based) — bloc 64

| StrokeProfile | Rows (0-based) | Nb rows |
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
| `1h_slash` / `1h_backslash` | 46–53 | hors-scope |

**Blocs oversize** (exemple trident) :

| StrokeProfile | Rows (0-based) | FrameSize |
|---|---|---|
| `walk_128` | 54–61 | 128×128 |
| `thrust_192` | 62–73 | 192×192 |

**Note** : le bloc étendu (rows 21–53) est **strictement identique d'un PNG à l'autre**. Les blocs étendus commencent toujours au row 54.

**Note sur la correspondance physique** : la conversion rows logiques → bandes physiques de 64 px relève du pipeline AOT (Phase 5).

### Profils écartés ou hors-scope

| Profil | Statut | Raison |
|---|---|---|
| `combat_idle` | Écarté | Support très limité. |
| `1h_slash` | Hors-scope | Terminologie obsolète. |
| `1h_backslash` | Hors-scope | Terminologie obsolète. |
| `1h_halfslash` | Hors-scope | Terminologie obsolète. |
| `backslash` | Hors-scope | Nommage plugin. |
| `halfslash` | Hors-scope | Nommage plugin. |
| `backslash_128` | Hors-scope par héritage | Dérivé de `backslash`. |
| `halfslash_128` | Hors-scope par héritage | Dérivé de `halfslash`. |

**Invariant de sélection** : un StrokeProfile n'est canonique que si tous les layers essentiels le supportent (whitelist empirique).

**Invariant d'héritage** : un CompositionProfile dérivé d'un StrokeProfile hors-scope est hors-scope par construction.

---

## 8. Table de résolution `MotorAction → StrokeProfile`

### Structure conceptuelle

```
MotorAction
├── default
├── by_weapon
└── fallback
```

### Exemples illustratifs

```
attack → slash (par défaut), thrust (spear/crossbow), shoot (bow)
walk → walk (Small), walk_128 (Medium), walk_192 (Large)
run → run
swim → swim (CompositionProfile, source spellcast)
cast → spellcast
idle → idle
die → hurt
stagger → hurt
watering → watering (CompositionProfile, source thrust)
sit → sit
emote → emote
```

### Plusieurs intentions, une cinématique

| MotorAction | StrokeProfile |
|---|---|
| `die` | `hurt` |
| `stagger` | `hurt` |

### Matérialisation runtime

```
[LayerId][BucketId][MotorAction][Direction][WeaponId] → SequenceId
```

- Aucun parcours d'arbre au runtime.
- `StrokeProfile` et `CompositionProfile` disparaissent du binaire final.
- `WeaponId` est un **index**, pas un prédicat.

### Note terminologique

| Notion | Niveau | Nature |
|---|---|---|
| **Convergence** | Étage 1 → 2 | Plusieurs `MotorAction` vers un même profil. |
| **Composition de séquence** | Étage 2 → 3 | Réordonnancement, répétition. |
| **Interning** | Étage 3 (AOT) | Déduplication mécanique. |
| **Assemblage multi-couches** | Runtime | Superposition spatiale. |

---

## 9. Reprojection, composition multi-couches, interning

### Reprojection

Pour chaque combinaison `(LayerId, MotorAction, Direction, contexte)` :

1. Résoudre le profil cible.
2. Extraire les rows du PNG source.
3. Reprojeter dans le bucket cible :
   - Toile vide `BucketSize × BucketSize`.
   - Offset centré : `offset = (BucketSize - source_size) / 2`.
   - Blitter la source.
   - Padding transparent autour.
4. Si le layer n'a **pas** de variante : **frame transparente canonique** (index global `0`).
5. Composer la séquence finale.
6. Empaqueter.

### Invariant de validation : `source_size ≤ BucketSize`

Si `source_size > BucketSize` : **erreur de build**.

### Politique de bucket cible

- Cible = maximum des buckets disponibles pour l'arme/outil équipé.
- Sans équipement : **Small**.
- Promotion automatique sinon.

### Fallback transparent vs erreur de build

| Situation | Traitement |
|---|---|
| Layer ne peut pas fournir une animation **requise** | **Erreur de build** |
| Layer n'a pas une animation **non requise** | **Frame transparente** injectée, longueur alignée |
| CompositionProfile dont la source manque | **Erreur de build** |

### Traitement des overlays et effets visuels

Un effet visuel peut être traité de **deux manières** :

- **Layer séparé** si l'asset LPC est superposable (aura, effet temporaire).
- **Variante d'un layer existant** si l'asset remplace intégralement un layer (`body_wounded` remplace `body`).

**Règle de classification** : déterminée au build par inspection de l'asset.

### Interning — périmètre

**Au sein d'un même layer** (ou variante de layer).

### Anti-explosion combinatoire

**Une séquence par (layer, bucket)**, jamais par combinaison de layers.

**Chiffrage indicatif** : ~70 000 séquences.

**Corollaire DOD** : composition runtime branchless, O(layers).

---

## 10. Frontière AOT / runtime

### Connaît
- `LayerId`, `BucketId`, `MotorAction`, `Direction`
- Référence à l'arme/outil équipé
- Taille de frame résolue

### Ignore
- LPC, rows, PNG, StrokeProfiles, CompositionProfiles
- Buckets comme entités sémantiques
- Pivot d'ancrage gameplay

### Fait
- Écrit `MotorAction` + `Direction`
- **Indexe par l'arme/outil** via `WeaponId`, **sans branche conditionnelle**
- Lit un identifiant de séquence et un index de frame
- **Applique la topologie de succession** (`next_sequence_id`)
- Superpose les layers dans l'ordre canonique

### Ne fait jamais
- Déduire, parser, inférer
- Brancher conditionnellement sur l'arme
- Évaluer une politique de boucle

### Frontière temporelle

L'AOT fige la topologie spatiale et ordinale. Le runtime est maître du domaine temporel (tick rate, transitions).

---

## 11. Invariants LPC préservés

### Invariants généraux

- **« Row » = ligne d'animation LPC (une direction).**
- Directions canoniques `[top, left, down, right]`, sauf `hurt` = `[down]`, `climb` = `[top]`.
- Indexation 0-based.
- FrameCounts figés par profil source.
- **Un PNG peut contenir plusieurs blocs de tailles différentes (64, 128, 192).**
- **Les blocs sont contigus, sans padding.**
- **Le bloc étendu (rows 21–53) est strictement identique d'un PNG à l'autre.**
- **Les blocs étendus (128, 192) commencent au row 54.**
- **Toute frame est centrée sur le centre géométrique de sa toile cible.**
- **Une frame source ne peut jamais être plus grande que son bucket cible.**
- **Le pivot d'ancrage gameplay est hors-scope AOT.**
- **Les frames commencent toujours à x = 0.**
- **Les variantes oversize ont toujours 4 directions.**
- « L'arme pilote la cinématique du corps, pas l'inverse ». Seule l'arme d'attaque (ou l'outil) détermine la cinématique. Bouclier surajouté, sans impact. **Jamais de dual wield.**
- **Trois familles de layers** : corporel / arme / outil.
- **Promotion automatique de bucket** (armes et outils).
- Un StrokeProfile n'est canonique que si tous les layers essentiels le supportent.
- **Un CompositionProfile dérivé d'un StrokeProfile hors-scope est hors-scope par construction.**
- **Nomenclature** : `_128`, `_192`, lowercase.
- **`1h_*`, `backslash`, `halfslash` : hors-scope.**
- **Politique de boucle résolue AOT** par graphe de succession.
- **Interning par layer.**
- **Séquence pré-calculée par (layer, bucket).**
- **Fallback transparent vs erreur de build.**
- **Deux YAML distincts** : canonique + équipement.
- **Traitement des overlays** : layer ou variante selon asset.
- **Le document n'est pas exhaustif** : les YAML le sont.

### Ordre d'empilement canonique

```
 1. background
 2. body
 3. body_overlays
 4. hair_behind
 5. cape_behind
 6. torso
 7. legs
 8. feet
 9. arms
10. hair_front
11. head
12. facial
13. neck
14. arms_armor
15. weapon_behind
16. weapon
17. shield (position variable selon direction)
18. cape_front
19. weapon_front
20. overlays
```

**Exception bouclier** (validé via générateur officiel) :

| Direction | Position |
|---|---|
| `down` | Devant |
| `top` | **Derrière** |
| `left` | Devant |
| `right` | Devant |

---

## 12. Sources de vérité

| Source | Rôle | Fiabilité |
|---|---|---|
| **YAML canonique** | Vérité sémantique des animations | Autoritaire |
| **YAML d'équipement** | Liaison PNG ↔ équipement | Autoritaire |
| **Liste des assets au build** | Vérité d'existence | Autoritaire |
| **Disque** | Vérité physique | Autoritaire |
| **PNG eux-mêmes** | Vérité ultime | **Autoritaire** |
| **Plugin Godot / Générateur web** | Sources croisées | Fiables, non autoritaires |

Règle : `Asset utilisé = déclaré au build ∩ présent sur disque ∩ déclaré en YAML`. Toute divergence = **erreur de build**.

---

## 13. Points ouverts

### Confirmés

- ✔ Rows LPC canoniques (0-based).
- ✔ **« Row » = ligne d'animation LPC (une direction)** — retour à la convention LPC standard.
- ✔ **Indexation 0-based.**
- ✔ **Champ `rows` conservé**, sémantique = direction.
- ✔ Le bloc étendu est strictement identique d'un PNG à l'autre.
- ✔ Les blocs étendus commencent au row 54.
- ✔ Les variantes `*_128` et `*_192` partagent les mêmes rows.
- ✔ `walk_128` = 9 frames.
- ✔ Alignement par **centre géométrique** (validé visuellement).
- ✔ Pivot d'ancrage gameplay **hors-scope AOT**.
- ✔ Champ `pivot` supprimé, remplacé par `align: "center"`.
- ✔ Validation `source_size ≤ BucketSize`.
- ✔ Nomenclature uniformisée `_128` / `_192`, lowercase.
- ✔ Hors-scope `1h_*`, `backslash`, `halfslash`, + héritage.
- ✔ Trois familles de layers.
- ✔ Promotion automatique de bucket.
- ✔ Politique de boucle par graphe de succession.
- ✔ Interning par layer.
- ✔ Séquence par (layer, bucket).
- ✔ Fallback transparent vs erreur de build.
- ✔ MotorActions sans préfixes.
- ✔ CompositionProfiles : `watering`, `tool_whip`, `tool_axe`, `swim`.
- ✔ Document non exhaustif.
- ✔ Convention de directions `[top, left, down, right]`.
- ✔ Ordre d'empilement canonique figé.
- ✔ CompositionProfiles à source unique. Manque → erreur de build.
- ✔ Deux YAML distincts : canonique + équipement.
- ✔ Traitement des overlays : layer ou variante.
- ✔ **Référence croisée** : les contrats d'équipement référencent les animations canoniques par `reference`. Les champs `frame_count`, `directions`, `sequence` **ne sont plus dupliqués** dans les contrats.
- ✔ **Divergences avec les YAML legacy résolues** : 0-based, lowercase, `hurt` (pas `death`), `align` (pas `pivot`), `bucket` (pas `target_bucket`).

### À vérifier sur PNG (Phase 5)

- **A3** — Les nouveaux outils (`tool_whip`, `tool_axe`) partagent-ils les rows de `slash` ?
- **A4 (nouveau)** — Conversion rows logiques ↔ bandes physiques de 64 px (comment une row logique 192×192 se traduit dans la structure PNG).

### À trancher (conceptuel, futur)

- **B11** — CompositionProfiles multi-source : si introduit plus tard, comment la validation s'applique-t-elle ?

---

## 14. Ce que ce document écarte

- Dépendance à un langage ou framework.
- Signatures, types concrets, formats binaires figés.
- Décisions d'outillage.
- Implémentations antérieures obsolètes (le « cadavre C# »).
- **Pivot d'ancrage gameplay** — hors-scope.
- **Spécification runtime concrète.**
- **Sémantique runtime du bouclage.**
- **Terminologie obsolète** `1h_*`, `backslash`, `halfslash`.
- **Préfixes de catégorie sur les MotorActions.**
- **Exhaustivité sur les animations.**
- **Terme « bande »** — remplacé partout par « row ».
- **Champ `pivot` dans les YAML** — remplacé par `align`.

---

## 15. Synthèse en une phrase

> **Le pipeline fournit au moteur un contrat strict : les PNG LPC et leurs conventions sont une source absorbée AOT ; le runtime ne manipule que des actions, des directions, des identifiants de layer, de bucket et de séquence, et applique une topologie de succession figée (`next_sequence_id`), sans jamais connaître ni LPC, ni les StrokeProfiles, ni les pixels, ni la sémantique de bouclage.**
