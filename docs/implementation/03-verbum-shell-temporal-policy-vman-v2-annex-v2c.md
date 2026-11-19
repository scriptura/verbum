# Verbum — Annexe V2-C : note de conception du paquet `VMAN` v2 — r5 (version finale soumise à validation)

## 0. Statut, portée, marqueurs

**Non normative. Note de conception, révision 5, version finale soumise à validation (19 novembre 2026), annexe à statut distinct du plan `03-verbum-shell-temporal-policy-vman-v2.md` (r1, § 10.2, ARB-19). Elle remplace les révisions 1 à 4.** Elle ne modifie pas le corps du plan. Elle n'autorise aucune implémentation : V2-L et V2-P restent bloquées jusqu'à **P-CONC**, validation explicite de l'utilisateur, **non franchie**.

Aucun code, test, `Cargo.toml`, ADR ou commit n'est créé. Les extraits Rust sont des **esquisses non normatives**. Aucune commande n'a été exécutée par la session : tout énoncé « démontré » désigne un raisonnement et des tests prévus, non des résultats.

Changements de la révision 5 (arbitrages, quatrième série) :
- **A-1 clos : option X** (§ 4.5.4). L'enveloppe délimite la section de manifeste par le premier offset ; elle ne recalcule ni ne lit la longueur du manifeste ; l'écart de longueur relève du lecteur. Séparation explicite des responsabilités (§ 4.5.1). Options Y refusée, Z exclue.
- **A-2 confirmé, avec ordre de priorité** des contrôles d'extraction (§ 4.2, § 4.5.4).
- **Clos** : preuve du § 4.2 acceptée ; P-3, P-4 ; nomenclature Forge (P-5) ; P-7 ; `Version` (doc) ; O-1 option `unreachable!` écartée.
- **F-14** : résultat négatif déclaré par l'utilisateur pour `VPKG` sur le dépôt réel ; commande, périmètre et état du dépôt **non encore consignés**. **P-1 reste une proposition, non adoptée**, en attente d'un dossier de preuve vérifiable (§ 3.5).
- **P-2 adopté** (19 novembre 2026), y compris `manifest_v2::ENTRY_LEN` conservée publique ; les valeurs de `package::SIGNATURE` et `package::VERSION` restent soumises à P-1.
- **Reste à traiter** : P-1 seule. **Ouvert hors P-CONC** : O-1 (bloque V2-F seulement). Aucun cas limite de classification ne demeure.
- Mentions « sous réserve de A-1 / A-2 » supprimées ; RK-4, RK-13, § 13 mis à jour.

Changements de la révision 4 (arbitrages du 18 novembre 2026, troisième série) :
- **Classification de la section de manifeste** (§ 4.5) : parcours réel du lecteur confronté aux invariants L3b ; cas de la section de longueur nulle classé explicitement ; **deux cas limites exposés pour arbitrage, A-1 et A-2** (§ 4.5.4).
- **Preuve structurelle d'inatteignabilité** du `unreachable!` de `artifact_bytes` (§ 4.2) : préconditions, exclusion de chaque cause d'échec, examen des données externes.
- **Décisions closes** : documentation de `Version` en français ; P-3 et P-4 ; O-1 option fondée sur `unreachable!` définitivement écartée.
- **Propositions maintenues** : `VPKG` / version `1`, noms des constantes, noms de l'API Forge (présentés pour validation, § 7.1), fichiers par unité (D-N6, § 8.2).

Changements de la révision 3 (seconde série) : notation `n'` (enveloppe) / `n` (manifeste) ; lecteur v2 champ par champ ; espaces de noms `package` et `manifest_v2` ; erreur interne du lecteur ; O-1 sans `unreachable!` ; correspondance VQ ↔ tests ; limites de provenance.

| Marqueur | Sens |
|---|---|
| **[Contrat]** | Posé par une ADR adoptée ou par la spécification r3 |
| **[Fait]** | Constaté dans un fichier fourni ou par une commande exécutée par l'utilisateur |
| **[Arbitré]** | Décision de l'utilisateur (17, 18 et 19 novembre 2026) |
| **[Proposition]** | Choix de cette note, en attente de validation ; ne vaut pas décision |
| **[Ouvert]** | Non déterminé ; énoncé sans choix |

Sources lues : les cinq documents de référence, le plan V2, la spécification r3, la roadmap r1.1, ADR-000, ADR-A01, ADR-S04 ; `assets/src/{lib,manifest,header,format,load,sprite,tileset,map,font,script}.rs`, `forge/src/{lib,batch,id,sprite,script}.rs`, `forge/tests/pipeline.rs`, `script/src/{lib,table}.rs`, `shell/Cargo.toml`, `shell/tests/borrowed_resource.rs`, `Cargo.toml` et `Cargo.lock` racine, `tree.md` du 17 novembre 2026.

---

## 1. Décisions adoptées et propositions

### 1.1 Adoptées [Arbitré]

| Réf. | Décision |
|---|---|
| **L3b** | Ordre : en-tête d'enveloppe de 10 octets, table de `n'` débuts d'artefacts, manifeste v2, artefacts. Offsets `u64` absolus depuis le début du paquet, little-endian ; compteur `u32` LE ; version d'enveloppe `u16` LE ; signature distincte de `VMAN` ; aucun alignement ni bourrage ; borne finale implicite = fin du paquet. Pour `n' = 0`, aucun offset ; le manifeste occupe le reste du paquet. |
| **Notation** | `n'` : compteur de l'enveloppe ; `n` : compteur du manifeste. |
| **Invariants** | Monotonie stricte, bornes, contiguïté, sections non vides. `n'` est confronté à `n` par `check_bindings` (niveau 2). Le premier offset délimite la section de manifeste ; sa longueur attendue n'est pas vérifiée par l'enveloppe (**A-1, option X**, ci-dessous). |
| **Arithmétique** | Conversions et calculs de taille tous vérifiés. Aucun `as`, aucune saturation ni troncature, aucun `unwrap_or(MAX)`. |
| **`unreachable!`** | Proscrit lorsqu'il masquerait un échec possible provenant de données externes, d'un calcul arithmétique ou d'une conversion susceptible d'échouer. Acceptable pour un invariant interne dont l'inatteignabilité est **établie** par les validations précédentes (preuve au § 4.2). Le `unreachable!` de `artifact_bytes` est conservé. |
| **Classification** | Table invalide ou section d'artefact vide : `Offsets`. Section d'artefact correctement délimitée mais trop courte : niveau 3, `ReadError::Truncated` avec index. Section de manifeste présente mais insuffisante : troncature du lecteur. Détail et cas limites : § 4.5. |
| **N1** | Noyau interne unique pour les niveaux 2 à 4 ; fermeture générique `Fn(usize) -> Option<&'a [u8]>` ; dispatch statique ; aucun trait, `dyn` ni allocation supplémentaire ; aucun ordre, contrôle ni comportement d'erreur dupliqué ; trois `unreachable!` conservés dans leur rôle logique ; N2 rejetée. |
| **M1 + P-b** | `Manifest<'a>` réutilisé ; retour interne `(NonZeroU64, Manifest<'a>)` ; helper partagé pour la partie réellement commune de la validation des entrées ; `Manifest::parse` strictement v1. |
| **A-1** | **Option X.** Le rôle de l'enveloppe est de délimiter la section de manifeste au moyen du premier offset ; la longueur attendue du manifeste relève du lecteur et de son contrôle de cohérence interne. L'extraction ne lit aucun octet du manifeste et ne recalcule pas sa longueur. Option Y refusée (duplique la règle de longueur, rend `EntryCountMismatch` inatteignable) ; option Z exclue. Section trop courte : `Load(Manifest(Truncated))` ; trop longue : `Load(Manifest(TrailingBytes))`. |
| **A-2** | Section de manifeste de longueur nulle : `len ≤ T` → `Truncated` (absence physique, `n' = 0` compris) ; `len > T`, `n' > 0` et `s_0 ≤ T` → `Offsets`. **Priorité** : le contrôle physique précède le contrôle de la table (§ 4.2). Pour `n' = 0` et `len > T`, le manifeste occupe le reste du paquet, sa validité relève du lecteur. |
| **Ordre du lecteur v2** | Disponibilité contrôlée champ par champ : six octets, signature, version, huit octets de q, lecture de q et rejet de zéro, quatre octets de `n`, lecture de `n`, longueur attendue vérifiée, longueur exacte, doublons. Pour un manifeste de 14 à 17 octets : `q = 0` → `ZeroQuantum` ; `q ≠ 0` → `Truncated`. |
| **E3** | `PackageError` public, `#[non_exhaustive]`, variantes `Truncated`, `Signature`, `Version`, `Offsets`, `ZeroQuantum`, `Load(LoadError)`. Aucune variante ajoutée à `ManifestError`, `LoadError`, `ForgeError`. Version de manifeste incompatible dans une enveloppe valide : `Load(LoadError::Manifest(ManifestError::Version))`. Manifeste v1 nu donné comme paquet : `Signature`. |
| **API de chargement** | Nom `load_package` ; retour `Result<NonZeroU64, PackageError>` ; assets par le mécanisme de dépôt existant ; q rendu seulement si le chargement complet réussit ; aucun accès public autonome à q. |
| **Modules et réexport (P-3)** | Modules publics `package` et `manifest_v2` ; aucune constante générique à la racine ; constantes v1 inchangées. `load_package` et `PackageError` réexportés à la racine, par continuité avec `load`. |
| **Lecteur et erreur (P-4)** | Le lecteur v2 de V2-L rend un type d'erreur interne ; `PackageError` n'est introduit que par V2-P. |
| **`Version` (doc)** | Formulation française : « Version du manifeste non prise en charge par ce lecteur. » Commentaire seul, en V2-L ; aucune assertion modifiée. |
| **P-2** | Constantes des modules `package` et `manifest_v2` : noms, types, visibilités, consommateurs et justifications du § 6.1 adoptés. `manifest_v2::ENTRY_LEN` conservée comme constante publique : elle expose à la Forge la largeur déclarée par le format v2 ; `Manifest::from_region` utilise la largeur commune `MANIFEST_ENTRY_LEN` ; l'assertion de compilation garantit leur cohérence. La duplication de déclaration entre v1 et v2 est intentionnelle (deux contrats de format, un invariant commun), sans coût d'exécution ni dépendance entre lecteurs. **Les valeurs de `package::SIGNATURE` et `package::VERSION` restent soumises à P-1** : leur présence au tableau ne vaut pas adoption de `VPKG` / `1`. |
| **Forge (P-5)** | Nomenclature acceptée : `compile_package`, `CompiledPackage`, `as_bytes`. Contrat de signature et de comportement explicité (§ 7.1). V-F1 ; `compile_assets` strictement inchangée ; q validé avant la compilation (`ForgeError::ZeroValue("q")`) ; V-F2 non autorisée ; type résultat à champs privés exposant le paquet complet sans accesseurs séparés ; `TooManyAssets` jamais détourné. |
| **O-1** | Ouvert, bloque **V2-F seulement** ; l'option fondée sur une assertion de compilation et un `unreachable!` est définitivement écartée. |
| **Digest** | Écarté définitivement. |
| **Unités et tests (P-7)** | Fichiers par unité : prévisions acceptées, confirmées à l'ouverture de chaque unité. V2-L et V2-P séparées ; **D-N6** : une unité par session par défaut, aucun regroupement implicite ; `expect(dead_code)` temporaires, localisés ; T1 ; tests historiques inchangés ; noms de tests laissés à cette note, avec correspondance aux VQ. |
| **Provenance** | La garantie protège la chaîne normale de production et de chargement, non la fabrication volontaire d'un paquet structurellement valide (§ 11.1). |

### 1.2 Proposition restante [Proposition]

| # | Proposition | Section |
|---|---|---|
| **P-1** | Signature d'enveloppe `VPKG`, version d'enveloppe `1`. Résultat négatif **déclaré** sur le dépôt réel (F-14), dossier de preuve à consigner (§ 3.5). **Non adoptée** ; en attente de la vérification. | § 3.5 |

### 1.3 Ouvert

| # | Point | Effet |
|---|---|---|
| **O-1** | Représentation de l'erreur Forge pour une taille ou un offset non représentable, et pour les invariants de la sortie de `compile_assets` | bloque **V2-F** seulement ; n'empêche pas V2-L ni V2-P ; hors P-CONC |

Aucun cas limite de classification ne reste à arbitrer (A-1 et A-2 clos, § 4.5.4).

---

## 2. Faits de référence [Fait]

| # | Fait | Source |
|---|---|---|
| F-1 | `Manifest<'a>` ne porte que la région des entrées (`entries: &'a [u8]`, champ privé). `entries()`, `find()`, `len()`, `check_bindings()` ne lisent jamais l'en-tête. Le seul constructeur est `parse`. | `manifest.rs` |
| F-2 | Ordre de `parse` : moins de 10 octets → `Truncated` ; signature ; version (**avant** la longueur) ; `n` × 16 avec `checked_mul` et conversion vérifiée ; octets restants inférieurs (`Truncated`) ou supérieurs (`TrailingBytes`) ; doublons (`DuplicateId`). | `manifest.rs` |
| F-3 | Un manifeste v1 vide fait 10 octets, moins que l'en-tête v2 de 18. | `manifest.rs`, D-1d |
| F-4 | `load` : niveau 1 (`Manifest::parse`), niveau 2 (`check_bindings(artifacts.len(), slot_formats.len())` puis boucle `SlotFormat`), niveau 3, niveau 4. `artifacts` n'est lu que par `.len()` et `artifact_bytes`. Trois `unreachable!` : seconde lecture (niveau 4), `expected_format`, `artifact_bytes`. Le premier appel à `artifact_bytes` suit `check_bindings(...)?`. | `load.rs` |
| F-5 | `Asset<'a>` n'emprunte que des octets d'artefact. `ScriptTable::deposit(SlotIndex, Script<'a>)` a le même `'a`. | `format.rs`, `table.rs` |
| F-6 | Les lecteurs d'artefacts décodent par `from_le_bytes` sans exigence d'alignement. Un artefact de moins de 10 octets rend `Truncated` ; son identité est alors inconnue (`Format::of_identity` rend `None`). | lecteurs, `header.rs`, `format.rs` |
| F-7 | Identités existantes : `VMAN`, `VSPR`, `VTLS`, `VMAP`, `VFNT`, `VSCR`. Noms exportés à la racine d'Assets : `HEADER_LEN`, `FORMAT_ID`, `FORMAT_VERSION`, `PAYLOAD_LEN`, `ARTIFACT_LEN`, `MANIFEST_*`, `SCRIPT_*`, `TILESET_*`, `MAP_*`, `FONT_*`. `lib.rs` n'expose aucun module public : modules privés et `pub use`. | `lib.rs` |
| F-8 | `CompiledSet` : champs privés, `manifest()` et `artifacts()` séparés. `compile_assets` : signature épinglée par doctest ; refuse déjà plus de `u32::MAX` entrées (`TooManyAssets`). | `batch.rs` |
| F-9 | `ForgeError::ZeroValue(&'static str)` existe ; `ForgeError` n'est pas `Copy` ni `non_exhaustive`. `TooManyAssets` (nombre d'entrées) et `PayloadTooLarge` (cartes) décrivent d'autres causes. | `sprite.rs` |
| F-10 | Aucun contenu d'artefact ne dépend de q. | `script.rs`, plan V2 § 3.4 |
| F-11 | `[dependencies]` d'Assets vide, dev-dépendances comprises ; `rust-version = "1.85"`. | `Cargo.toml` |
| F-12 | `TemporalPolicy::from_nanos(u64, u32, u32)` refuse déjà q = 0. | plan V2 § 4.9 |
| F-13 | Commande exécutée par l'utilisateur (18 novembre 2026) : `grep -rnI -E 'VPKG\|0x56[[:space:]]*,[[:space:]]*0x50[[:space:]]*,[[:space:]]*0x4[Bb][[:space:]]*,[[:space:]]*0x47\|\\x56\\x50\\x4[Bb]\\x47' crates docs` : **aucun résultat**. Périmètre : `crates` et `docs`. Les fichiers de cette annexe ne figuraient pas encore dans `docs`. La recherche sur le dépôt complet est traitée en F-14. | utilisateur |
| F-14 | Recherche de `VPKG` sur le dépôt réel dans son état actuel (déclaration de l'utilisateur, 19 novembre 2026) : **aucun résultat**. La commande exacte et le périmètre précis ne sont pas consignés ici ; ils ne sont pas vérifiés par la session. | utilisateur |

---

## 3. Layout du paquet

### 3.1 Schéma

```
offset 0                                                                           len(paquet)
│ En-tête (10)   │ Table (8·n')      │ Manifeste v2 (18+16·n)      │ Artefact 0 │ … │ Artefact n'−1 │
│ sig  ver   n'  │ s_0 … s_{n'−1}    │                             │            │   │               │
   4    2    4      8 octets chacun   ▲                             ▲
                                      │ T = 10 + 8·n'               └─ s_0
```

Frontières : `B_0 = T`, `B_1 = s_0`, …, `B_{n'} = s_{n'−1}`, `B_{n'+1} = len`. Manifeste = `[B_0, B_1)` ; artefact `i` = `[B_{i+1}, B_{i+2})`. Si `n' = 0`, manifeste = `[T, len)` avec `T = 10`.

### 3.2 Champs, offsets symboliques

| Zone | Offset | Taille | Contenu |
|---|---:|---:|---|
| Signature d'enveloppe | 0 | 4 | proposition `VPKG` (P-1) |
| Version d'enveloppe | 4 | 2 | `u16` LE, proposition `1` (P-1) |
| Nombre d'artefacts `n'` | 6 | 4 | `u32` LE, lu avant la table : il détermine sa taille |
| Table : `s_i` | 10 + 8·i | 8 | début de l'artefact `i`, `u64` LE, absolu |
| Manifeste : signature | T | 4 | `VMAN` |
| Manifeste : version | T + 4 | 2 | `u16` LE = 2 |
| Manifeste : `q_ns` | T + 6 | 8 | `u64` LE |
| Manifeste : `n` | T + 14 | 4 | `u32` LE, nombre d'entrées |
| Manifeste : entrée `k` | T + 18 + 16·k | 16 | `AssetId` (8), `artifact_index` (4), `slot_index` (4), identique à v1 |
| Artefact `i` | `s_i` | `s_{i+1} − s_i` (dernier : `len − s_{n'−1}`) | octets de l'artefact |

`T = 10 + 8·n'`. Pas d'alignement, pas de bourrage, pas de champ réservé. Aucune chaîne ni chemin (ADR-A01 pt 3).

### 3.3 Invariants et lieu de vérification

| Invariant | Mécanisme | Erreur |
|---|---|---|
| Manifeste immédiatement après l'en-tête et la table | `T` calculé, non lu | — |
| Offsets strictement croissants, tranches contiguës | chaîne `T < s_0 < … < s_{n'−1} < len` vérifiée à l'extraction, **après** le contrôle physique `len ≤ T` (A-2) | `Offsets` |
| Section d'artefact non vide | conséquence de la stricte croissance (`s_i < s_{i+1}`, `s_{n'−1} < len`) | `Offsets` |
| Section de manifeste non vide | physiquement : `len > T` ; déclarée : `s_0 > T` | `Truncated` / `Offsets` (**A-2, clos** : § 4.5.3, § 4.5.4) |
| Fin du dernier artefact = fin du paquet | borne implicite ; aucun champ à vérifier | — |
| `n' = 0` : manifeste = reste du paquet, aucun offset ; pas de `s_0` | `len > T = 10` ; sa validité relève du lecteur | `Truncated` si `len ≤ 10` |
| Première borne et fin du manifeste | section manifeste = `[T, s_0)` ; sa longueur est confrontée à `18 + 16·n` par le lecteur | **A-1, option X** (§ 4.5.4) : écart constaté par le lecteur, `Load(Manifest(Truncated \| TrailingBytes))` ; l'extraction ne recalcule pas la longueur |
| `n'` = `n` | niveau 2 existant : `check_bindings(n', …)` | `Load(Binding(EntryCountMismatch))` |
| Conversions et tailles vérifiées | § 4.1 | erreur typée |

### 3.4 Exemple chiffré (non normatif, à recalculer à l'exécution)

Lot de référence de l'étape 2 : « a » (sprite 32×16, slot 1), « foobar » (sprite 96×48, slot 0) ; q = 10 ns (valeur de test, spéc. § 8). `n' = n = 2`, `T = 26`, manifeste de 50 octets, `s_0 = 76`, `s_1 = 90`, `len = 104`.

```
octets          contenu
[0..4)          56 50 4B 47                         signature d'enveloppe VPKG (proposition)
[4..6)          01 00                               version d'enveloppe 1
[6..10)         02 00 00 00                         n' = 2
[10..18)        4C 00 00 00 00 00 00 00             s_0 = 76
[18..26)        5A 00 00 00 00 00 00 00             s_1 = 90
[26..30)        56 4D 41 4E                         VMAN
[30..32)        02 00                               version du manifeste = 2
[32..40)        0A 00 00 00 00 00 00 00             q_ns = 10
[40..44)        02 00 00 00                         n = 2
[44..60)        8C EC 01 86 4C DC 63 AF | 00 00 00 00 | 01 00 00 00     « a » : artefact 0, slot 1
[60..76)        E8 67 39 F7 71 41 94 85 | 01 00 00 00 | 00 00 00 00     « foobar » : artefact 1, slot 0
[76..90)        56 53 50 52 01 00 04 00 00 00 20 00 10 00               artefact 0 : sprite 32×16
[90..104)       56 53 50 52 01 00 04 00 00 00 60 00 30 00               artefact 1 : sprite 96×48
```

Paquet vide (`n' = n = 0`, 28 octets) : `56 50 4B 47 01 00 00 00 00 00` puis `56 4D 41 4E 02 00`, q (8 octets), `00 00 00 00`.

### 3.5 Identité d'enveloppe : statut [Proposition P-1]

| Élément | Valeur | Statut |
|---|---|---|
| Signature | `VPKG` (`56 50 4B 47`) | **Proposition P-1, non adoptée.** Absente des fichiers lus (F-7) ; recherche `crates` et `docs` négative (F-13) ; recherche sur le dépôt réel négative **déclarée** (F-14), non encore consignée de manière vérifiable. |
| Version initiale | `1` (`u16` LE) | **Proposition P-1**, même statut. Indépendante de la version du manifeste (2) : l'enveloppe v1 contient un manifeste v2. |

Les occurrences purement documentaires du nouveau format dans les documents V2 (plan, roadmap, cette annexe) sont attendues et sans effet sur la collision. **Dossier de preuve de P-1 (à consigner par l'utilisateur ; non exécuté par la session) :**

| Élément | Valeur |
|---|---|
| Commande exacte | *à fournir* |
| Périmètre (répertoires, exclusions) | *à fournir* |
| État du dépôt (référence de révision ou date) | *à fournir* |
| Résultat | *à fournir* (déclaré : aucun résultat) |

**Limite explicite.** La recherche textuelle vérifie les occurrences dans les sources et documents. Elle **ne démontre pas** l'absence de la séquence d'octets `56 50 4B 47` dans les fichiers binaires et n'est pas présentée comme une vérification binaire. Une vérification binaire exigerait une recherche d'octets distincte, non réalisée ; la séquence étant l'ASCII `VPKG`, une recherche traitant les binaires comme du texte (option `-a` de `grep`) serait un moyen possible, **proposé, non exécuté, non requis par cette note**.

Un manifeste v1 nu donné comme paquet échoue sur la signature d'enveloppe, la signature `VPKG` différant de `VMAN`.

### 3.6 Canonicité

Un contenu donné (q, entrées, artefacts ordonnés) a un seul encodage valide : pas de bourrage, pas de champ redondant, offsets déterminés par les longueurs, section vide interdite. La Forge ne produit que cet encodage ; le lecteur rejette tout autre.

---

## 4. Algorithmes, dans leur ordre

### 4.1 Arithmétique vérifiée [Arbitré]

Aucune opération n'utilise `as`, `saturating_*`, `wrapping_*` ni `unwrap_or(MAX)`. Toute défaillance rend une erreur typée.

| Opération | Contrôle | Défaillance |
|---|---|---|
| `n'` : `u32` → `usize` | `usize::try_from` | `Truncated` |
| `8·n'` | `checked_mul` en `usize` | `Truncated` |
| `T = 10 + 8·n'` | `checked_add` en `usize` | `Truncated` |
| Offset `s` : `u64` → `usize` | `usize::try_from` | `Offsets` (un offset non représentable ne peut pas désigner un octet du paquet) |
| Comparaisons de la chaîne | en `usize`, aucune soustraction | — |
| Conversion d'un bloc de 8 octets | `<[u8; 8]>::try_from` | `Offsets` (branche typée, non masquée par `unreachable!`) |
| Accès à l'artefact `i` | `checked_add`, `get(start..end)` | `None` (preuve au § 4.2) |
| Longueur du manifeste, `n × 16` | `checked_mul`, conversion vérifiée (existant) | `ManifestError::Truncated` |

### 4.2 Extraction de l'enveloppe (préparation avant le niveau 1) [Contrat ARB-1]

Fonction interne `pub(crate)`, sans allocation. Elle n'est pas un niveau : elle ne valide que ce qu'il faut pour découper des tranches sûres, sans règle sémantique, et ne lit aucun octet du manifeste.

1. `package.len() ≥ 10`, sinon `Truncated`.
2. Octets `0..4` égaux à `package::SIGNATURE`, sinon `Signature`.
3. `u16` LE des octets `4..6` égal à `package::VERSION`, sinon `Version`.
4. **`n'`** : `u32` LE des octets `6..10`, lu avant la table puisqu'il en détermine la taille ; converti en `usize`.
5. `T = 10 + 8·n'` (§ 4.1). Si `T ≥ len` : `Truncated` (le paquet ne contient pas l'en-tête, la table et au moins un octet de manifeste). Rejet en O(1) d'un `n'` hostile, avant toute boucle.
6. `table = package.get(10..T)`.
7. `prev = T`. Pour chaque bloc de 8 octets de `table` : `s` = `u64` LE converti en `usize` ; si `s ≤ prev` : `Offsets` ; `prev = s`. Après la boucle : si `prev ≥ len` : `Offsets`.
8. `manifest = package.get(T..first)`, avec `first = s_0` si `n' > 0`, sinon `len`.
9. Résultat : `Envelope { package, table, manifest, count: n' }`, `Copy`.

Une seule boucle, O(n'), bornée par `len`, sans allocation. La cohérence de `n'` avec `n` n'est pas vérifiée ici : elle relève du niveau 2 (§ 4.5).

**Ordre de priorité des contrôles d'extraction (A-2).** Les contrôles s'exécutent dans l'ordre des étapes ci-dessus ; la priorité utile à la classification est :
1. `len < 10` → `Truncated` (étape 1) ; signature, puis version d'enveloppe (étapes 2, 3).
2. `T` non calculable, ou **`len ≤ T`** → `Truncated` (étape 5). Aucun octet n'existe après l'en-tête et la table : la section de manifeste est absente physiquement. Cela inclut `n' = 0` avec `len ≤ 10` et une table complète sans aucun octet de manifeste.
3. `len > T` et `n' > 0` : chaîne d'offsets ; **`s_0 ≤ T` → `Offsets`** (étape 7) : la table déclare un manifeste vide ou pointe dans la table.
4. `n' = 0` : il n'existe pas de `s_0`. Lorsque `len > T`, le manifeste occupe le reste du paquet et sa validité relève du lecteur.
Le contrôle physique (2) précède donc toujours le contrôle de la table (3) : un paquet physiquement trop court est `Truncated`, jamais `Offsets`.

**Accès à l'artefact `i`** : `Envelope::artifact(&self, i: usize) -> Option<&'a [u8]>`. `None` si `i ≥ count`. `start` = offset `i` (lecture `u64` LE, conversion vérifiée). `end` = offset `i + 1` si `i + 1 < count`, sinon `package.len()`. `package.get(start..end)`. **La référence `package: &'a [u8]` est copiée avant `get`** : le résultat est de durée `'a`, non liée à `&self`.

#### Preuve structurelle : le `None` de l'accès est inatteignable dans le chemin de chargement

**Site concerné.** `artifact_bytes(index)` convertit `index.get()` (`u32`) en `usize`, appelle la fermeture `|i| envelope.artifact(i)`, et traite `None` par le `unreachable!` existant (F-4). Aucun site n'est ajouté ; la conversion de bloc de 8 octets du § 4.1 est, elle, une erreur typée.

**Causes possibles d'un `None`.**
(a) échec de `usize::try_from(index.get())` ; (b) `i ≥ count` ; (c) lecture hors de `table` ; (d) échec de la conversion d'un offset en `usize` ; (e) débordement de `i + 1` ; (f) échec de `package.get(start..end)`.

**Préconditions établies avant tout appel.**

| # | Précondition | Établie par |
|---|---|---|
| **P1** | `table.len() = 8·n'` exactement ; `T = 10 + 8·n'` calculé sans débordement ; `T < len` | extraction, étapes 4 à 6 : `checked_mul`, `checked_add`, test `T ≥ len`, `get(10..T)` réussi |
| **P2** | `T < s_0 < … < s_{n'−1} < len`, chaque `s_i` converti en `usize` avec succès | extraction, étape 7 |
| **P3** | `count = n'` et chaque `artifact_index` d'entrée vérifie `usize::try_from(artifact_index) = Ok(i)` avec `i < n'` | niveau 2 : `check_bindings(n', …)` (`within` impose la conversion réussie et `index < limit` ; un écart `n ≠ n'` est rejeté par `EntryCountMismatch`) |
| **P4** | tout appel à `artifact_bytes` suit `check_bindings(...)?` | structure de `load_core` : premier appel dans la boucle de format de slot, après le `?` ; les niveaux 3 et 4 suivent |
| **P5** | les octets du paquet, de la table et du manifeste ne changent pas entre les contrôles et les accès | `package: &'a [u8]` est une référence partagée sur des octets sans mutabilité intérieure ; la fermeture est `Fn` et ne capture que `&Envelope` ; `deposit` reçoit des `Asset<'a>` qui empruntent le paquet en lecture seule |

**Exclusion de chaque cause.**

| Cause | Exclue par | Raison |
|---|---|---|
| (a) | P3 | l'indice a déjà été converti avec succès par `check_bindings` |
| (b) | P3 | `i < n' = count` |
| (c) | P1, P3 | `8·i + 8 ≤ 8·n' = table.len()` pour `i < n'` ; `8·i` ne déborde pas car `8·n'` a été calculé sans débordement |
| (d) | P2, P5 | les offsets lus sont ceux de l'étape 7, déjà convertis avec succès |
| (e) | P1, P3 | `i + 1 ≤ n'` et `n'` se convertit en `usize` : pas de débordement |
| (f) | P1, P2 | `start = s_i`, `end = s_{i+1}` ou `len` ; la chaîne donne `start < end ≤ len` ; un intervalle ordonné et inclus dans `[0, len]` est accepté par `get` |

**Données externes et contournement.** Les entrées externes sont : les octets du paquet (`n'`, table, section de manifeste), `slot_formats`, la fermeture `deposit`.
- Les octets du paquet : toutes les valeurs lues par l'accès sont celles que P1 à P3 ont validées (P5). Le manifeste ne peut pas introduire un autre compte : un `n ≠ n'` échoue au niveau 2 avant tout accès, un `artifact_index ≥ n'` aussi.
- `slot_formats` : n'intervient dans aucun calcul d'accès.
- `deposit` : ne peut ni modifier ni remplacer le paquet ou l'`Envelope` (références partagées) ; il peut paniquer, ce qui est hors de la garantie, comme en v1.
- `n'` n'est lu qu'une fois (champ `count` de l'`Envelope`) ; le noyau reçoit ce même `count`.

**Conclusion et résidu.** Sous P1 à P5, aucun `None` n'est atteignable : le `unreachable!` exprime un invariant interne **établi**, non un échec masqué. Le résidu est la prémisse d'immuabilité (P5), identique à celle de la seconde lecture du niveau 4 en v1 (R14) : valable pour des tampons ordinaires, à reconsidérer si un accès projeté en mémoire, modifiable par un tiers, était ajouté. Le chemin v1 est inchangé (`artifacts.get(i)` avec `i < artifacts.len()` établi par `check_bindings`).

**Contrôles associés** : `pk13` balaye tous les indices de plusieurs enveloppes valides (`n' = 0, 1, 2`) et les indices `n'` et `usize::MAX` ; ce balayage est un contrôle, non la preuve.

### 4.3 Lecteur v2 du manifeste (niveau 1) [Arbitré]

Entrée : la section manifeste. Sortie interne : `(NonZeroU64, Manifest<'a>)`. Ici `n` est le nombre d'entrées **du manifeste**, lu après q ; `n'` n'est pas lu.

1. Au moins 6 octets disponibles, sinon `Truncated`.
2. Signature égale à `manifest_v2::SIGNATURE`, sinon `Signature`.
3. Version égale à `manifest_v2::VERSION` (2), sinon `Version`. **Ce contrôle précède toute exigence sur la suite** : un manifeste v1, vide compris (10 octets), rend `Version`.
4. Au moins 8 octets disponibles pour `q_ns`, sinon `Truncated`.
5. `q_ns` : `u64` LE. Nul : `ZeroQuantum`.
6. Au moins 4 octets disponibles pour `n`, sinon `Truncated`.
7. `n` : `u32` LE. Région des entrées : les octets restants (à partir de l'offset 18).
8. **Helper commun** — longueur attendue `n × 16` en arithmétique vérifiée (`checked_mul`, conversion vérifiée), sinon `Truncated`.
9. Helper — longueur exacte : `Truncated` si la région est plus courte, `TrailingBytes` si elle est plus longue.
10. Helper — doublons d'identifiants : `DuplicateId`.

Conséquence : un manifeste v2 de 14 à 17 octets avec `q_ns = 0` rend `ZeroQuantum` ; avec `q_ns ≠ 0`, `Truncated`.

Les erreurs des étapes 1 à 4, 6 et 8 à 10 sont des `ManifestError` ; l'étape 5 est `ZeroQuantum`. Le lecteur les rend dans un type interne (§ 6.3) ; `load_package` les convertit en `PackageError`.

### 4.4 Helper commun (P-b) et noyau (N1)

**Helper.** Extrait de la fin de `Manifest::parse`, comportement identique : `Manifest::from_region(count: u32, rest: &'a [u8])` (nom provisoire, `pub(crate)`), qui exécute les étapes 8 à 10. `parse` garde son en-tête de 10 octets, sa signature, sa version **avant** la longueur, puis appelle le helper. Témoins : `m1` à `m8`, inchangés.

**Noyau** (`pub(crate)`, dans `load.rs`), esquisse non normative :

```rust
fn load_core<'a, F>(
    manifest: &Manifest<'_>,
    artifact_count: usize,
    artifact: F,
    slot_formats: &[Format],
    deposit: impl FnMut(SlotIndex, Asset<'a>),
) -> Result<(), LoadError>
where F: Fn(usize) -> Option<&'a [u8]>;
```

Contenu : le corps actuel de `load` à partir de `check_bindings`, sans changement d'ordre : (2) `check_bindings` puis boucle `SlotFormat` ; (3) `materialize` de chaque artefact, résultat abandonné ; (4) `materialize` puis `deposit`. `artifact_bytes` convertit `ArtifactIndex` en `usize`, appelle la fermeture ; `None` reste `unreachable!` (preuve au § 4.2). `expected_format` et la seconde lecture gardent leur `unreachable!`.

**Partage des validations :**

| Validation | Écrite une seule fois dans | v1 | v2 |
|---|---|---|---|
| Signature, version, longueur d'en-tête | `parse` (v1) / lecteur v2 | `parse` | lecteur v2 (en-têtes différents, rien à factoriser) |
| `n × 16`, longueur exacte, doublons | `Manifest::from_region` | oui | oui |
| Compte, capacité, bornes, doublons de liaison | `check_bindings` | noyau | noyau |
| Format de slot, décodage, références, dépôt | noyau | noyau | noyau |
| Structure de l'enveloppe | extraction | — | oui (propre à v2, non sémantique) |

### 4.5 `load_package` : ordre et classification des erreurs

Ordre : (1) extraction ; (2) lecteur v2 ; (3) `load_core(&manifest, n', |i| envelope.artifact(i), slot_formats, deposit)` ; (4) `Ok(q)`.

**Priorité globale** : enveloppe → manifeste v2 (signature, version, q, longueur, doublons) → liaisons (compte `n'` contre `n`, capacité, bornes, doublons d'artefact puis de slot) → format de slot → artefacts → références → dépôt. Aucun dépôt avant la fin du niveau 3 ; q n'est rendu qu'après les dépôts.

#### 4.5.1 Trois familles d'erreurs

| Famille | Variantes de `PackageError` | Origine |
|---|---|---|
| **Structure de l'enveloppe** (avant le niveau 1) | `Truncated`, `Signature`, `Version`, `Offsets` | extraction (§ 4.2) |
| **Lecture du manifeste dans le paquet** (niveau 1) | `ZeroQuantum` (propre à v2) ; `Load(LoadError::Manifest(ManifestError::…))` | lecteur v2 (§ 4.3) |
| **Niveaux de chargement existants** (2 à 4) | `Load(Binding(…))`, `Load(SlotFormat { .. })`, `Load(Artifact { .. })`, `Load(Reference { .. })` | noyau |

**Séparation des responsabilités (A-1, option X).**

| Responsable | Répond de | Ne fait pas |
|---|---|---|
| **Enveloppe** (extraction) | table d'offsets : chaîne, bornes, sections vides déclarées, absence physique ; **délimitation** de la section de manifeste par le premier offset | ne lit aucun octet du manifeste ; ne calcule ni ne vérifie la longueur attendue du manifeste |
| **Lecteur v2** | contenu de la section : en-tête, version, q, longueur exacte `18 + 16·n`, doublons ; décide `Truncated` ou `TrailingBytes` | ne connaît pas l'enveloppe |
| **Noyau** | liaisons (dont `n'` contre `n`), formats de slot, artefacts, références, dépôt | — |

#### 4.5.2 Parcours du lecteur sur la section de manifeste `S = [T, s_0)` (longueur `L ≥ 1`)

Le lecteur lit `S` sans connaître l'enveloppe. Pour une section dont les premiers octets sont ceux d'un manifeste v2 valide :

| Longueur `L` de `S` | Étape qui tranche | Résultat |
|---|---|---|
| 1 à 5 | étape 1 | `Truncated` |
| 6 à 13 | étape 4 | `Truncated` (si signature et version valides) |
| 14 à 17 | étape 5 puis 6 | `ZeroQuantum` si `q = 0`, sinon `Truncated` |
| `L ≥ 18`, `L − 18 < 16·n` | étape 9 | `Truncated` |
| `L ≥ 18`, `L − 18 > 16·n` | étape 9 | `TrailingBytes` |
| `L = 18 + 16·n` | — | section exacte ; contrôle de doublons ; puis niveau 2 compare `n` à `n'` |

#### 4.5.3 Classification retenue

| Situation | Étape qui tranche | Erreur |
|---|---|---|
| `len < 10` | extraction 1 | `Truncated` |
| Signature ou version d'enveloppe inattendue (manifeste v1 nu compris) | extraction 2, 3 | `Signature`, `Version` |
| `T` non calculable, ou **`len ≤ T`** : aucun octet de manifeste (**section de manifeste de longueur nulle par absence physique**, `n' = 0` compris) ; table incomplète | extraction 5 | `Truncated` |
| Offset non strictement croissant, borne hors du paquet, offset non représentable, dernier offset `≥ len` : **table incohérente, borne invalide** | extraction 7 | `Offsets` |
| **`s_0 ≤ T` avec `len > T`** : la table déclare un manifeste vide ou pointe dans la table (**section de manifeste de longueur nulle déclarée**) | extraction 7 | `Offsets` |
| `s_i = s_{i+1}` ou `s_{n'−1} = len` : **artefact vide** | extraction 7 | `Offsets` |
| Section de manifeste non vide mais insuffisante pour le lecteur | lecteur (§ 4.5.2) | `Load(Manifest(Truncated))` |
| Section de manifeste plus longue que `18 + 16·n` | lecteur, étape 9 | `Load(Manifest(TrailingBytes))` (**A-1, option X**) |
| Signature de manifeste inattendue | lecteur, étape 2 | `Load(Manifest(Signature))` |
| **Version de manifeste incompatible dans une enveloppe valide** (v1 compris) | lecteur, étape 3 | `Load(Manifest(Version))` |
| `q_ns = 0` | lecteur, étape 5 | `ZeroQuantum` |
| Doublon d'identifiant | lecteur, étape 10 | `Load(Manifest(DuplicateId))` |
| `n' ≠ n` (section de longueur exacte pour `n`) | niveau 2 | `Load(Binding(EntryCountMismatch))` |
| Section d'artefact non vide, correctement délimitée, trop courte ou invalide | niveau 3 | `Load(Artifact { index, error: Truncated \| … })` |
| Format de slot, référence | niveaux 2 et 3 | `Load(SlotFormat)`, `Load(Reference)` |

**Cas limites de la section de manifeste, sur l'exemple du § 3.4** (`T = 26`, `s_1 = 90`, `len = 104`, `n' = n = 2`, manifeste de 50 octets) :

| # | Situation | Étape | Erreur |
|---:|---|---|---|
| 1 | `len ≤ 26` (aucun octet de manifeste) | extraction 5 | `Truncated` |
| 2 | `s_0 = 26` (table déclare un manifeste vide) | extraction 7 | `Offsets` |
| 3 | `s_0 = 20` (pointe dans la table) | extraction 7 | `Offsets` |
| 4 | `s_0 = 90 = s_1` (artefact 0 vide) | extraction 7 | `Offsets` |
| 5 | `s_0 ∈ 27..=31` (`L` = 1 à 5) | lecteur 1 | `Load(Manifest(Truncated))` |
| 6 | `s_0 ∈ 32..=39` (`L` = 6 à 13) | lecteur 4 | `Load(Manifest(Truncated))` |
| 7 | `s_0 ∈ 40..=43` (`L` = 14 à 17) | lecteur 5, 6 | `ZeroQuantum` si `q = 0` ; sinon `Truncated` |
| 8 | `s_0 ∈ 44..=75` (`L` = 18 à 49) | lecteur 9 | `Load(Manifest(Truncated))` |
| 9 | `s_0 = 76` | — | section exacte |
| 10 | `s_0 ∈ 77..=89` (`L` = 51 à 63) | lecteur 9 | `Load(Manifest(TrailingBytes))` (A-1, option X) |
| 11 | `n' = 0`, manifeste valide suivi d'octets (`len > 28`) | lecteur 9 | `Load(Manifest(TrailingBytes))` |
| 12 | `n' = 0`, `len ≤ 10` | extraction 5 | `Truncated` |

#### 4.5.4 A-1 et A-2 : arbitrages rendus [Arbitré]

**A-1 — option X adoptée.** Le rôle de l'enveloppe est de délimiter la section de manifeste au moyen du premier offset ; la longueur attendue du manifeste relève du lecteur et de son contrôle de cohérence interne.

| Option | Statut | Motif |
|---|---|---|
| **X** | **adoptée** | l'écart est détecté par le lecteur (trop court → `Truncated`, trop long → `TrailingBytes`) ; `Offsets` se limite à la structure de la table et aux sections vides ; aucune duplication de la règle de longueur ; l'extraction ne lit aucun octet du manifeste ; `EntryCountMismatch` reste atteignable pour `n' ≠ n` |
| **Y** | refusée | l'extraction exigerait `s_0 = T + 18 + 16·n'` : la formule existerait deux fois ; `n ≠ n'` se manifesterait par une erreur du lecteur et `EntryCountMismatch` deviendrait inatteignable dans le chemin paquet |
| **Z** | exclue | l'extraction lirait `n` à `T + 14`, donc le manifeste avant le niveau 1 et avant le contrôle de version v2 (ARB-1, RK-2) |

Conséquence sur les formulations antérieures : la formulation « première borne ≠ fin du manifeste → `Offsets` » ne s'applique pas à la longueur du manifeste ; `Offsets` désigne la table (chaîne, bornes, sections vides déclarées).

**A-2 — confirmé, avec priorité** (ordre au § 4.2).
- **Absence physique** : `len ≤ T` → `Truncated`, `n' = 0` compris, table complète sans octet de manifeste comprise (cas 1 et 12).
- **Déclarée par la table** : `len > T`, `n' > 0`, `s_0 ≤ T` → `Offsets` (cas 2 et 3).
- **`n' = 0`** : pas de `s_0` ; pour `len > T`, le manifeste occupe le reste du paquet et le lecteur décide.

Justification : `Truncated` désigne un paquet physiquement trop court pour porter l'en-tête, la table et un octet de manifeste ; `Offsets` désigne un contenu de table invalide.

### 4.6 Graphe de chargement

```
load(manifest, artifacts, slot_formats, deposit)            [v1, public, inchangé en surface]
  └─ Manifest::parse ─► load_core(&m, artifacts.len(), |i| artifacts.get(i).copied(), …)

load_package(package, slot_formats, deposit)                [v2, public, additif]
  ├─ Envelope::extract(package)                             [pub(crate), package.rs]
  ├─ read_manifest_v2(envelope.manifest) ─► (q, Manifest)   [pub(crate), manifest_v2.rs] ─► Manifest::from_region
  └─ load_core(&m, n', |i| envelope.artifact(i), …) ─► Ok(q)

Dépendances de modules : package.rs ─► {manifest_v2.rs, manifest.rs, load.rs, format.rs} ;
                         manifest_v2.rs ─► manifest.rs ; load.rs ─► {manifest.rs, format.rs, map.rs}.
Aucune arête load.rs ─► package.rs, aucune arête manifest_v2.rs ─► package.rs.
```

### 4.7 Coût

| Opération | CPU | Mémoire |
|---|---|---|
| Extraction | 1 boucle O(n'), 1 lecture `u64` par entrée | pile : un `Envelope` (5 mots) |
| Accès artefact `i` | O(1) : 2 lectures `u64`, conversions, 1 `get` | aucune |
| Niveaux 1 à 4 | inchangés (O(n²) pour doublons et liaisons ; n accès et décodages pour les niveaux 3 et 4) | aucune |
| Code | deux instances du noyau (v1, v2) | second exemplaire du noyau |

Aucune allocation, aucune copie d'artefact.

---

## 5. Conditions de la refactorisation de `load.rs`

| Condition posée | Satisfaction |
|---|---|
| Signature et comportement de `load` | inchangés ; `load` devient une enveloppe mince |
| Ordre des niveaux, priorité des erreurs | corps du noyau = corps actuel ; niveau 1 avant le noyau dans chaque point d'entrée |
| Tout-ou-rien, aucune allocation | aucun dépôt avant la fin du niveau 3 ; `[dependencies]` vide |
| Dispatch statique, aucun trait ni `dyn` | fermeture générique ; deux monomorphisations |
| Trois `unreachable!` | mêmes sites logiques (§ 4.4) ; preuve d'inatteignabilité pour le site de la fermeture (§ 4.2) |
| Tests historiques | `l1`–`l5`, `tb1`, `ld1`, `sf1`–`sf6`, `rf1`–`rf6`, `scr_l*`, `lt5` et consommateurs du recensement du plan V2 inchangés |

`load.rs` est classé **S** (plan V2 § 0.2). Tout test historique qui devrait changer est un arrêt G-CM avec rapport.

---

## 6. API publique, constantes, visibilité, erreurs

### 6.1 Constantes et espaces de noms [Arbitré : modules ; P-2 adopté, sauf les valeurs soumises à P-1]

Le format v1 conserve **toutes** ses constantes et leurs valeurs, à la racine : `MANIFEST_SIGNATURE`, `MANIFEST_VERSION`, `MANIFEST_HEADER_LEN`, `MANIFEST_ENTRY_LEN`. Les nouvelles constantes sont publiques mais sous deux modules publics, sans nom générique à la racine. Le chemin qualifié distingue enveloppe, manifeste v1 et manifeste v2.

| Nom exact | Module | Type | Valeur | Visibilité | Consommateur | Justification |
|---|---|---|---|---|---|---|
| `SIGNATURE` | `package` | `[u8; 4]` | `*b"VPKG"` (**P-1**, non adoptée) | publique | extraction (V2-P) ; Forge (V2-F) | publique : la Forge, autre crate, écrit ces octets (précédent : constantes v1 à la racine) ; identité de l'enveloppe, distincte de `VMAN` |
| `VERSION` | `package` | `u16` | `1` (**P-1**, non adoptée) | publique | extraction ; Forge | publique : écrite par la Forge ; version de l'enveloppe, indépendante de celle du manifeste |
| `HEADER_LEN` | `package` | `usize` | `10` | publique | extraction ; Forge (calcul de `T`) | publique : la Forge calcule `T` ; en-tête d'enveloppe seul, distinct de `MANIFEST_HEADER_LEN` (10, v1) et de `manifest_v2::HEADER_LEN` |
| `OFFSET_WIDTH` | `package` | `usize` | `8` | publique | extraction ; Forge (calcul de `T`) | publique : largeur d'un offset de table, nécessaire à la Forge pour `T = 10 + 8·n'` |
| `SIGNATURE` | `manifest_v2` | `[u8; 4]` | `*b"VMAN"` | publique | lecteur v2 (V2-L) ; Forge | publique : écrite par la Forge ; égale à la signature v1 (D-1d), liée par assertion de compilation |
| `VERSION` | `manifest_v2` | `u16` | `2` | publique | lecteur v2 ; Forge | publique : écrite par la Forge ; contrôlée par le lecteur avant toute exigence de longueur |
| `HEADER_LEN` | `manifest_v2` | `usize` | `18` | publique | lecteur v2 ; Forge (calcul de `s_0`) | publique : la Forge calcule `s_0 = T + 18 + 16·n` ; `4 + 2 + 8 + 4` par assertion |
| `ENTRY_LEN` | `manifest_v2` | `usize` | `16` | publique | Forge (calcul de `s_0`) | publique : même besoin ; égale à `MANIFEST_ENTRY_LEN` (D-1d), liée par assertion |
| `MANIFEST_SIGNATURE`, `MANIFEST_VERSION`, `MANIFEST_HEADER_LEN`, `MANIFEST_ENTRY_LEN` | racine (v1) | inchangés | inchangés | publiques, inchangées | `Manifest::parse`, Forge (`MANIFEST_HEADER_LEN` pour reprendre les entrées v1) | **réutilisées, non redéfinies** : sens v1 conservé ; les constantes v2 ne les remplacent pas |

**Points de vérification des responsabilités (sans modification des noms ni des valeurs).**
- **Séparation `package::` / `manifest_v2::`** : `package::` ne décrit que l'enveloppe (signature, version, en-tête, largeur d'offset) ; `manifest_v2::` ne décrit que la section de manifeste. Aucune constante de `package::` n'est dérivée de `manifest_v2::` ni l'inverse ; l'extraction ne consomme aucune constante de `manifest_v2::` (A-1, option X : elle ne connaît pas la longueur du manifeste). Seule la Forge les combine (`s_0`).
- **Lien avec le v1** : uniquement par les assertions de compilation de `manifest_v2` vers les constantes racine (sens unique, à la compilation, sans dépendance d'exécution). Aucun lien entre `package::` et le lecteur v1.
- **Point tranché (P-2)** : `Manifest::from_region` (§ 4.4), partagé par `parse` (v1) et le lecteur v2, applique la largeur d'entrée de 16 octets. `manifest_v2::ENTRY_LEN` n'a donc pas de consommateur dans le lecteur v2 : l'égalité avec `MANIFEST_ENTRY_LEN` est garantie par assertion, et `ENTRY_LEN` sert la Forge et la lisibilité du format. **Décision : `manifest_v2::ENTRY_LEN` est conservée publique** ; la duplication de déclaration v1 / v2 est intentionnelle, sans coût d'exécution ni dépendance entre lecteurs.
- **Constantes sans consommateur interne** : en V2-L, les constantes `manifest_v2` sont publiques et ne déclenchent pas `dead_code` ; seuls les éléments `pub(crate)` portent le `expect(dead_code)` (§ 8.2).

- Les constantes du manifeste v2 sont **autonomes** : elles ne réutilisent aucune constante `MANIFEST_*` avec un autre sens. Des assertions de compilation (`const _`) établissent leur cohérence interne (`HEADER_LEN = 4 + 2 + 8 + 4`) et leur égalité avec les valeurs v1 là où D-1d l'impose : `SIGNATURE` et `ENTRY_LEN`. `PartialEq` n'étant pas `const`, la signature se compare par `u32::from_le_bytes` des deux tableaux.
- Les constantes `HEADER_LEN` des modules `package` (10) et `manifest_v2` (18), `MANIFEST_HEADER_LEN` (10) et `HEADER_LEN` du sprite (10, exportée à la racine) sont distinctes. Les tests écrivent leurs littéraux à la main ; les usages se qualifient, jamais par import global.
- La Forge lit les octets du manifeste v1 à partir de `MANIFEST_HEADER_LEN` (sens inchangé : longueur de l'en-tête v1).
- Conséquence : `lib.rs` déclare ses **premiers modules publics** (`pub mod manifest_v2;` en V2-L, `pub mod package;` en V2-P). Tout élément `pub(crate)` reste invisible de l'extérieur.

### 6.2 Chargement et durées de vie [Arbitré]

```rust
// esquisse non normative
pub fn load_package<'a>(
    package: &'a [u8],
    slot_formats: &[Format],
    deposit: impl FnMut(SlotIndex, Asset<'a>),
) -> Result<NonZeroU64, PackageError>;
```

- Les assets sortent par `deposit` ; q sort par la valeur de retour. `Ok(q)` n'existe que si les niveaux 1 à 4 ont réussi **et** que tous les dépôts ont eu lieu : q ne peut être ni obtenu sans chargement, ni dissocié des artefacts. Aucun `read_quantum()`.
- Aucun type de succès nouveau : `NonZeroU64` suffit.
- **Durées de vie** : `Envelope<'a>` est `Copy` ; `artifact` rend `&'a [u8]` indépendant de `&self` ; la fermeture `|i| envelope.artifact(i)` a le type `Fn(usize) -> Option<&'a [u8]>` ; `Manifest<'a>` n'est utilisé que le temps de l'appel ; `Asset<'a>` et `Script<'a>` empruntent des sous-tranches du paquet, comme ils empruntent aujourd'hui les tranches d'artefacts. Un appelant déclare le tampon avant le `World` (motif de `lt5`).
- **Réexport (P-3, adopté)** : `pub use package::{PackageError, load_package}` à la racine.

**Contrat de `load_package` [Arbitré].**

| Aspect | Contrat |
|---|---|
| Entrées | `package` : octets du paquet complet ; `slot_formats` : table des formats de slot (comme `load`) ; `deposit` : `FnMut(SlotIndex, Asset<'a>)` |
| Succès | `Ok(q)` avec `q: NonZeroU64`, **uniquement** après les niveaux 1 à 4 et tous les dépôts |
| Échec | `Err(PackageError)` ; **aucun dépôt** pour tout échec des niveaux 1 à 3 ; ordre et priorité du § 4.5 |
| Allocation | aucune |
| Durées de vie | `Asset<'a>` emprunte le paquet ; indépendant de `&self` de l'enveloppe |
| Accès à q | aucun autre que la valeur de retour |

**Surface publique ajoutée à Assets** : modules `manifest_v2` (constantes) et `package` (constantes, `PackageError`, `load_package`), et le réexport. Rien d'autre.

| Élément | Visibilité |
|---|---|
| Extraction, `Envelope`, `Envelope::artifact` | `pub(crate)` |
| Lecteur v2 du manifeste, son type d'erreur, `Manifest::from_region` | `pub(crate)` |
| Noyau `load_core` | `pub(crate)` |
| `load` (v1), `Manifest::parse`, constantes v1 | inchangés |

### 6.3 `PackageError` et erreur interne du lecteur [Arbitré]

```rust
// esquisse non normative
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PackageError {
    Truncated,        // enveloppe, table ou premier octet de manifeste physiquement absents
    Signature,        // signature d'enveloppe
    Version,          // version d'enveloppe
    Offsets,          // table incohérente, borne invalide, section d'artefact vide, section de manifeste déclarée vide
    ZeroQuantum,      // q_ns = 0
    Load(LoadError),  // validations existantes : manifeste (niveau 1), niveaux 2 à 4
}
```

Couverture : `ZeroQuantum` (B-1) ; `Truncated`, `Signature`, `Version`, `Offsets` (B-4) ; B-2 : `LoadError::Manifest(ManifestError::Version)` du chargement historique ; B-3 : `Signature` (v1 nu) ou `Load(Manifest(Version))` (v1 dans une enveloppe). Aucune variante n'est ajoutée à `ManifestError`, `LoadError`, `ForgeError`. `#[non_exhaustive]` impose un bras par défaut aux `match` extérieurs au crate (aucun dans le dépôt lu). La documentation de `Offsets` ne couvre pas les écarts de longueur de la section de manifeste, qui relèvent du lecteur (A-1, option X) ; elle couvre la section de manifeste déclarée vide par la table (A-2).

**Erreur interne du lecteur (P-4, adopté).** Le lecteur de `manifest_v2.rs` rend un type `pub(crate)` :

```rust
// esquisse non normative
enum ManifestV2Error { Manifest(ManifestError), ZeroQuantum }
```

`load_package` le convertit : `Manifest(e)` → `PackageError::Load(LoadError::Manifest(e))` ; `ZeroQuantum` → `PackageError::ZeroQuantum`. Raisons : graphe de modules acyclique ; aucun type public inutilisé en V2-L ; `PackageError` apparaît avec son seul producteur public, en V2-P.

### 6.4 Documentation de `ManifestError::Version` [Arbitré]

Texte adopté (commentaire seul, V2-L ; aucune assertion modifiée) : « Version du manifeste non prise en charge par ce lecteur. » Il reste exact pour `Manifest::parse` (version 1) et pour le lecteur v2 (version 2).

---

## 7. Forge

### 7.1 API [P-5 : nomenclature acceptée]

| Nom | Nature | Visibilité | Consommateurs |
|---|---|---|---|
| `compile_package` | fonction | publique | tests Forge ; application (F3b) |
| `CompiledPackage` | type résultat, champ privé `Vec<u8>` | publique, sans constructeur public | tests Forge, application |
| `CompiledPackage::as_bytes` | méthode `&self -> &[u8]` | publique | idem |

```rust
// esquisse non normative
pub fn compile_package(
    sources: &[AssetSource<'_>],
    slot_count: usize,
    quantum_ns: u64,
) -> Result<CompiledPackage, ForgeError>;   // type d'erreur : O-1

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledPackage { /* champ privé */ }
impl CompiledPackage { pub fn as_bytes(&self) -> &[u8]; }
```

**Contrat.** `compile_package(sources, slot_count, quantum_ns)` : `quantum_ns == 0` → `ForgeError::ZeroValue("q")` avant toute compilation ; sinon mêmes erreurs et même ordre que `compile_assets` ; en cas de succès, un `CompiledPackage` dont `as_bytes()` est le paquet complet (enveloppe, manifeste v2, artefacts) et rien d'autre. `CompiledPackage` : champ privé, aucun constructeur public, aucun accesseur de q, du manifeste ou des artefacts. Le type d'erreur pour les tailles non représentables reste **O-1**.

**API minimale** : `as_bytes` seule. Aucun accesseur séparé du manifeste ou des artefacts (contrairement à `CompiledSet`) ; aucun constructeur public ; aucune méthode qui rende q. Nomenclature **acceptée** (P-5) ; motifs : le préfixe `compile_` suit `compile_assets` ; `package` reprend le vocabulaire de `package::` et de `load_package`. Alternative non retenue : un suffixe de version (`compile_assets_v2`), qui désignerait le format plutôt que le produit.

`compile_assets`, `compile_assets_with`, `CompiledSet`, `batch.rs`, `sprite.rs`, `script.rs` ne changent pas. `CompiledPackage` est le seul producteur public de paquet.

### 7.2 Algorithme de production (V-F1)

1. `quantum_ns == 0` → `ForgeError::ZeroValue("q")`, **avant** toute compilation : une erreur de source ne masque pas q nul.
2. `let set = compile_assets(sources, slot_count)?` : mêmes erreurs, même ordre, mêmes octets que la voie v1.
3. Composants repris du manifeste v1, sans conversion : le compteur (octets `6..10` de `set.manifest()`) et les entrées (octets à partir de `MANIFEST_HEADER_LEN`). Un compteur identique est écrit dans l'enveloppe (`n'`) et dans le manifeste (`n`).
4. Manifeste v2 : `manifest_v2::SIGNATURE`, `manifest_v2::VERSION` LE, `quantum_ns` LE, compteur, entrées.
5. Offsets : `T = package::HEADER_LEN + package::OFFSET_WIDTH · n'` ; `s_0 = T + manifest_v2::HEADER_LEN + manifest_v2::ENTRY_LEN · n` ; `s_{i+1} = s_i + longueur(artefact i)`. **Toute opération est vérifiée** (`checked_*`, `try_from`) ; offsets écrits en `u64` LE.
6. Assemblage : en-tête, table, manifeste v2, artefacts, dans cet ordre ; `debug_assert` sur la longueur totale.
7. Retour de `CompiledPackage`.

Allocations et copies acceptées (build). La Forge n'ajoute aucune dépendance ; `Cargo.lock` inchangé.

### 7.3 O-1 : erreurs de représentation côté Forge [Ouvert]

**Règle imposée** : toute arithmétique de taille et d'offset est vérifiée. Aucun `unwrap_or(MAX)`, aucune saturation, aucun `as`, aucun `unreachable!` qui masquerait un échec possible, aucun détournement de `TooManyAssets` ni de `PayloadTooLarge`. Aucune variante n'est ajoutée sans arbitrage.

Faits :
- les étapes 3 et 5 contiennent des conversions, additions et découpages dont l'échec est improbable (la sortie de `compile_assets` respecte son layout documenté, épinglé par `f2`, `f8`, `vm_1` ; `compile_assets` refuse plus de `u32::MAX` entrées), mais non établi par les types ;
- elles doivent rendre une erreur typée ;
- aucune variante existante de `ForgeError` ne convient sans détournement.

Options (l'option fondée sur une assertion de compilation et un `unreachable!` est **définitivement écartée**) :

| Option | Effet |
|---|---|
| B | nouvelle variante de `ForgeError` pour une taille ou un offset de paquet non représentable (arbitrage requis) |
| C | type d'erreur distinct pour `compile_package`, qui encapsule `ForgeError` (API différente de `compile_assets`) |

Le choix attend V2-F. **Aucune implémentation de V2-F ne démarre avant sa résolution.** V2-L et V2-P n'en dépendent pas.

---

## 8. Responsabilités, fichiers, lint temporaire

### 8.1 Responsabilités

| Responsabilité | Assets | Forge |
|---|---|---|
| Layout et constantes | définit et lit | écrit, avec les constantes publiques des modules `package` et `manifest_v2` |
| Extraction, niveaux 1 à 4, q | oui | non |
| Production du paquet | non | oui, unique |
| Cohérence manifeste ↔ artefacts | structurelle (niveaux 1 à 4) | par construction (même appel) |
| Validation de q > 0 | octets (`ZeroQuantum`) | paramètre (`ZeroValue("q")`) |

### 8.2 Fichiers par unité et sessions [P-7 clos ; D-N6]

**D-N6** : une unité par session par défaut. V2-L, V2-P, V2-F et V2-Z sont quatre sessions distinctes ; **aucun regroupement implicite** ; tout regroupement exige un arbitrage explicite. V2-L et V2-P peuvent démarrer après P-CONC sans attendre O-1.

| Unité | Assets | Forge |
|---|---|---|
| **V2-L** | `manifest.rs` (helper `from_region` ; commentaire de `Version`) ; nouveau `manifest_v2.rs` (constantes, lecteur v2, type d'erreur interne, tests) ; `lib.rs` (`pub mod manifest_v2`) | aucun |
| **V2-P** | `load.rs` (`load_core`, `load` mince) ; nouveau `package.rs` (constantes, `PackageError`, extraction, `load_package`, tests) ; `lib.rs` (`pub mod package`, réexport) | aucun |
| **V2-F** (après O-1) | aucun | nouveau `package.rs` ; `lib.rs` (exports) ; nouveau `tests/package.rs` |
| **V2-Z** | consignations additives | consignations additives |

Les listes sont des prévisions, confirmées à l'ouverture de chaque unité. `batch.rs`, `sprite.rs`, `script.rs`, `tests/pipeline.rs` ne sont pas modifiés. V2-L ne livre aucun chargement de paquet : seulement le lecteur de manifeste v2.

**Lint temporaire (V2-L).** Le lecteur v2 et son type d'erreur sont `pub(crate)` et inutilisés hors tests entre V2-L et V2-P. Forme adoptée :

```rust
#[cfg_attr(not(test), expect(dead_code, reason = "utilisé par le chargement de paquet, V2-P"))]
```

- placé sur **chaque élément** signalé par le compilateur (le lecteur, son type d'erreur, tout helper privé qu'il est seul à appeler : le lint se propage) ; leur nombre est minimisé ;
- `expect` (stable depuis Rust 1.81, compatible MSRV 1.85) : dès que l'élément est utilisé, l'attente non remplie devient un avertissement (`unfulfilled_lint_expectations`) et fait échouer `clippy -D warnings` : **le retrait en V2-P est forcé** ;
- aucun `allow` général, aucun attribut au niveau du module ou du crate.

---

## 9. Garanties 1 à 9 : mécanisme et statut

| # | Garantie | Mécanisme | Preuve prévue | Statut |
|---|---|---|---|---|
| 1 | Un v1 ne fournit jamais un q | version contrôlée avant toute exigence de longueur ; `Version` pour tout v1, vide compris | `mv5`, `pk7` | par conception ; à exécuter |
| 2 | Le chargement historique refuse v2 | `Manifest::parse` : `Version` ; aucune modification | `mv8`, `pk10` (VQ-9b) | `parse` déjà établi (`m4`) ; assertion à la frontière de `load` à ajouter |
| 3 | Aucune voie publique v2 à deux entrées | extraction, lecteur et noyau `pub(crate)` ; seul `load_package` public | `compile_fail` (VQ-9a), revue de l'API | par conception ; à exécuter |
| 4 | Pas de recomposition A/B par la chaîne normale | `CompiledPackage` sans accesseur séparé ; aucun assembleur public dans Assets | `compile_fail` (VQ-9c), `pf8` | **limite** : § 11.1 |
| 5 | Paquet vérifiable sans allocation | extraction O(n') sans tampon ; accès O(1) | `[dependencies]` vide, `no_std`, `cargo tree` | par conception |
| 6 | Niveaux 1 à 4 conservés sans duplication | noyau unique | tests historiques inchangés, `pk8` différentiel | par conception ; à exécuter |
| 7 | Contrats d'emprunt `Asset<'a>`, `Script<'a>` | accès rend `&'a [u8]` indépendant de `&self` | doctests `compile_fail` et jumeau valide sur `load_package` | par conception ; à exécuter |
| 8 | Tailles, offsets, bornes vérifiables | § 4.1 ; preuve d'inatteignabilité (§ 4.2) | `pk5`, `pk11`, `pk13`, `pk14` | par conception ; à exécuter |
| 9 | Chemin v1 inchangé | `parse` et `load` inchangés en comportement | suites historiques inchangées (VQ-7) | à exécuter |

---

## 10. Tests, mutations, critères de sortie

Noms laissés à cette note ; ils peuvent être ajustés au document d'exécution sans nouvel arbitrage, la correspondance aux VQ restant. Octets littéraux **écrits à la main**, indépendants des constantes. Valeurs de q de test asymétriques (ex. `0x0102030405060708`).

### 10.1 Correspondance VQ ↔ tests

| VQ | Énoncé abrégé | Tests | Unité |
|---|---|---|---|
| VQ-1 | manifeste v2 littéral : q attendu | `mv1` | V2-L |
| VQ-2 | q = 0 refusé : octets, Forge, `TemporalPolicy` | `mv2` (octets), `pk7` (dans une enveloppe), `pf5` (Forge) ; `TemporalPolicy` déjà couvert (F-12) | V2-L, V2-P, V2-F |
| VQ-3 | longueur exacte `18 + 16·n`, troncature, surplus, ordre d'octets | `mv3`, `mv4`, `mv6`, `pk3`, `pk5`, `pk14` | V2-L, V2-P |
| VQ-4 | un v1 ne produit jamais q | `mv5`, `pk7` | V2-L, V2-P |
| VQ-5 | aller-retour Forge → paquet → lecture ; v1 identique | `pf1`–`pf4`, `pf7` | V2-F |
| VQ-6 | niveaux, tout-ou-rien, absence d'allocation, `cargo tree` | `pk3`, `pk8`, `pk9`, tests historiques, contrôles `cargo tree` | V2-P, V2-Z |
| VQ-7 | suites des étapes 2, 5, 6 inchangées | toutes les suites historiques | toutes |
| VQ-8 | bout en bout jusqu'à `TemporalPolicy` | **hors V2-C : dépendance de F3b** | F3b |
| VQ-9a | aucun chemin public v2 à deux entrées | doctests `compile_fail`, revue de l'API | V2-P |
| VQ-9b | le chargement à deux entrées refuse v2, sans dépôt | `mv8`, `pk10` | V2-L, V2-P |
| VQ-9c | A/B non recomposable par la chaîne normale | `pf8`, revue | V2-F |
| VQ-9d | un v1 n'est pas chargeable comme jeu conforme S04 | `pk4`, `pk7` | V2-P |

### 10.2 V2-L (`manifest_v2.rs`, tests Assets)

| Test | Énoncé |
|---|---|
| `mv1` | manifeste v2 littéral à deux entrées (mêmes entrées que le littéral v1) : q, entrées, `find` attendus |
| `mv2` | `q_ns = 0` : `ZeroQuantum` ; **priorité champ par champ** : q nul avec 14 octets disponibles, ou avec une longueur incohérente, rend `ZeroQuantum` ; q non nul avec 14 à 17 octets rend `Truncated` |
| `mv3` | longueur `18 + 16·n` pour `n = 0..3` ; chaque préfixe d'un littéral valide : `Truncated` ; surplus : `TrailingBytes` ; `n = u32::MAX` avec peu d'octets : `Truncated` sans panique |
| `mv4` | ordre d'octets : q et `n` en little-endian |
| `mv5` | manifestes v1 (42 octets et **10 octets vides**) et versions `0`, `1`, `3`, `0x0100`, `0xFFFF` : `Version` ; signature altérée : `Signature` avant `Version` ; moins de 6 octets : `Truncated` |
| `mv6` | identifiants dupliqués : `DuplicateId` |
| `mv7` | équivalence du helper : mêmes entrées et mêmes erreurs de région pour `Manifest::parse` (v1) et le lecteur v2 |
| `mv8` | `Manifest::parse` sur un manifeste v2 : `Version` ; sur v1 : inchangé |
| `mv9` | assertions de compilation des constantes v2 ; valeurs littérales `18`, `2`, `16`, `*b"VMAN"` |
| Inchangés | `m1`–`m8`, `vm_1`, `t6_4`, tous les littéraux v1 |

### 10.3 V2-P (`package.rs`, tests Assets)

| Test | Énoncé |
|---|---|
| `pk1` | paquet littéral du § 3.4, table de formats `[Sprite; 2]` : dépôts attendus par slot, q rendu |
| `pk2` | `n' = 0` : manifeste = reste ; `Ok(q)`, aucun dépôt ; `len ≤ 10` : `Truncated` |
| `pk3` | **classification exacte des préfixes du littéral du § 3.4** (`T = 26`, `s_0 = 76`, `s_1 = 90`, `len = 104`) : longueurs `0..=26` : `Truncated` ; `27..=90` : `Offsets` ; `91..=103` : `Load(Artifact { index: 1, error: Truncated })` ; `104` : succès. Zéro dépôt pour tout échec |
| `pk4` | v1 nu comme paquet : `Signature` ; chaque octet de signature altéré : `Signature` ; versions d'enveloppe `0`, `2`, `0x0100`, `0xFFFF` : `Version` |
| `pk5` | offsets : égaux, décroissants, `s_0 = T`, `s_0 < T`, dernier `= len`, dernier `> len`, `u64::MAX` : `Offsets` ; `n' = u32::MAX` avec peu d'octets : `Truncated` sans boucle ni panique |
| `pk6` | `n' ≠ n` avec section de longueur exacte pour `n` : `Load(Binding(EntryCountMismatch))` (A-1, option X) |
| `pk7` | manifeste v1 dans une enveloppe valide : `Load(Manifest(Version))` ; q nul dans une enveloppe valide : `ZeroQuantum`, zéro dépôt |
| `pk8` | différentiel : mêmes entrées et artefacts via `load` (v1) et `load_package` : mêmes dépôts ; chaque erreur des niveaux 2 et 3 (artefact tronqué, référence non résolue, format de slot) identique, enveloppée |
| `pk9` | tout-ou-rien : chaque niveau en échec laisse un compteur de dépôts nul |
| `pk10` | `load` (v1) avec un manifeste v2 : `Manifest(Version)` et zéro dépôt ; avec un paquet : `Manifest(Signature)` |
| `pk11` | **section d'artefact non vide mais courte** (1 à 9 octets, puis préfixe complet à charge utile tronquée) : `Load(Artifact { index, Truncated })` ; **section d'artefact de longueur nulle** (offsets égaux, ou dernier offset `= len`) : `Offsets` |
| `pk12` | `package::SIGNATURE`, `VERSION`, `HEADER_LEN`, `OFFSET_WIDTH` : valeurs littérales et cohérence |
| `pk13` | `Envelope::artifact` : pour `n' = 0, 1, 2`, chaque `i < n'` rend la tranche attendue ; `i = n'` et `i = usize::MAX` rendent `None` (contrôle associé à la preuve du § 4.2) |
| `pk14` | **cas limites de la section de manifeste** : les 12 cas du tableau du § 4.5.3 (cas 10 et 11 : A-1, option X ; cas 1, 2, 3 et 12 : A-2) |
| doctests | `compile_fail` : noyau, extraction, lecteur v2 inaccessibles de l'extérieur (codes à épingler au constat du rouge) ; `compile_fail` : libérer le tampon après `load_package` puis utiliser un `Asset` déposé ; jumeau valide |
| Inchangés | `l1`–`l5`, `tb1`, `ld1`, `sf1`–`sf6`, `rf1`–`rf6`, `scr_l*`, doctests LT-5n/5t, `lt5` |

### 10.4 V2-F (`forge/tests/package.rs`, en mémoire ; après résolution de O-1)

| Test | Énoncé |
|---|---|
| `pf1` | lot de référence : paquet égal au littéral du § 3.4 |
| `pf2` | aller-retour : `compile_package` → `load_package` : mêmes slots que T6.1 ; q égal |
| `pf3` | entrées du manifeste v2 et artefacts égaux à ceux de `compile_assets` du même lot |
| `pf4` | permutation de l'entrée : mêmes identifiants et slots, `artifact_index` permutés ; q différent : paquet différent **exactement aux 8 octets de q**, offsets et artefacts identiques |
| `pf5` | `q = 0` : `ZeroValue("q")`, aucune sortie ; **priorité** sur une source invalide |
| `pf6` | mêmes erreurs que `compile_assets` pour un lot invalide avec `q ≠ 0` |
| `pf7` | lot vide : 28 octets ; `load_package` : `Ok(q)`, aucun dépôt |
| `pf8` | `compile_fail` : `CompiledPackage` sans constructeur, sans champ public, sans `manifest()` ni `artifacts()` |
| Inchangés | `f2`, `f3`, `f7`, `f8`, `vm_1`, `t6_4`, `t6_6`, `scr_f5`, doctest de `compile_assets` |

### 10.5 Mutations (copies isolées, une à la fois ; une erreur de compilation ne compte pas comme détection)

| Unité | Mutation | Détecteur |
|---|---|---|
| V2-L | q big-endian ; en-tête lu sur 10 octets ; version comparée à `MANIFEST_VERSION` | `mv1`, `mv4` |
| V2-L | q nul accepté ; contrôle de q déplacé après la disponibilité de `n` ou après la longueur | `mv2` |
| V2-L | disponibilité des 18 octets exigée avant la version | `mv5` (v1 vide) |
| V2-L | helper sans `checked_mul` ou sans contrôle de doublons ; `parse` acceptant v2 | `mv3`, `mv6`, `mv8`, `m3`, `m5` |
| V2-P | `<` → `≤` dans la chaîne ; `s_0 ≤ T` accepté ; dernier `≥ len` accepté | `pk5`, `pk11`, `pk14` |
| V2-P | offsets relatifs ; largeur 4 au lieu de 8 ; `end` du dernier artefact ≠ `len` ; `i` au lieu de `i + 1` | `pk1`, `pk13` |
| V2-P | `T ≥ len` accepté ; signature ou version d'enveloppe non contrôlée | `pk2`, `pk4` |
| V2-P | section courte d'artefact classée `Offsets` ; section nulle acceptée ; `len ≤ T` classé `Offsets` ; `s_0 ≤ T` classé `Truncated` ; contrôle de la table placé avant le contrôle physique (A-2) | `pk3`, `pk11`, `pk14` |
| V2-P | extraction qui recalcule la longueur du manifeste et classe l'écart `Offsets` (**option Y, refusée**) ; section trop longue classée `Offsets` | `pk6`, `pk14` |
| V2-P | conversion saturante (`unwrap_or(MAX)`) à la place d'une erreur typée | `pk5` |
| V2-P | compte du noyau pris sur le manifeste au lieu de l'enveloppe | `pk6` |
| V2-P | dépôt avant la fin du niveau 3 ; niveaux inversés ; formats de slot après le niveau 3 | `pk8`, `pk9`, `l3`, `sf6`, `rf6` |
| V2-P | q nul accepté dans le chemin paquet ; `load` v1 avec accès décalé | `pk7`, `l1`–`l5` |
| V2-F | q non écrit ou écrit deux fois ; offsets calculés avec `10 + 16·n` ; artefacts permutés ; table big-endian | `pf1`, `pf2` |
| V2-F | contrôle de q après la compilation ; q nul accepté | `pf5` |
| V2-F | entrées issues d'une autre compilation ; accesseurs séparés exposés | `pf3`, `pf8` |

### 10.6 Critères de sortie et portes

Pour chaque unité : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, exécutés par l'utilisateur ; MSRV en deux commandes (`cargo +1.85.0 build --workspace`, puis `cargo +1.85.0 test --workspace`) ; pas de chaînes `if let … &&` ; `cargo tree -p verbum-assets` sans dépendance, dev comprises.

| Unité | Acceptation |
|---|---|
| V2-L | `mv1`–`mv9` verts ; `m1`–`m8` inchangés ; `expect(dead_code)` locaux seulement ; Miri `-p verbum-assets` intégral |
| V2-P | `pk1`–`pk14` et doctests verts ; suites historiques inchangées ; **`expect(dead_code)` retirés** ; Miri intégral Assets |
| V2-F | **O-1 résolu** ; `pf1`–`pf8` verts ; `cargo tree -i toml --workspace` inchangé ; `cargo tree -i verbum-forge --workspace -e normal` sans dépendant ; Miri Forge **partiel** (exclusions connues de l'étape 5, rapporté comme tel) |
| V2-Z | VQ-1 à VQ-7 et VQ-9 verts ; consignations additives ; liste « non démontré » ; **P-E** |

**G-CM** : arrêt avec rapport si un niveau est créé, supprimé ou réordonné, si un dépôt précède la fin du niveau 3, si une allocation entre dans le chargement, si le modèle d'identité change, ou si un octet v1 produit change. **R-ADAPT / GV-1 à GV-5** : aucune adaptation attendue ; toute modification d'assertion ou de littéral v1 est un arrêt. **G-S6** : `forge/src/script.rs` et les tests `scr_*` ne sont pas modifiés.

---

## 11. Risques, limites, non démontré

### 11.1 Limites de la garantie de provenance

La garantie de V2 protège **la chaîne normale de production et de chargement** : la Forge produit un paquet unique, le chargement n'accepte qu'un paquet, q n'est obtenu que de ce chargement. Elle **ne protège pas** contre la fabrication volontaire d'un paquet structurellement valide :
- les constantes de layout sont publiques (comme celles de v1) : des octets valides peuvent être écrits à la main ;
- aucun digest, aucune signature : l'intégrité vérifiée est **structurelle**, non cryptographique ;
- q n'est lié à aucun contenu d'artefact (F-10) : le chargeur ne peut pas détecter qu'un artefact a été compilé pour un autre q ; la garantie ne gagne en substance qu'avec F2.

### 11.2 Risques

| # | Risque ou limite | Traitement |
|---|---|---|
| RK-1 | Refactor de `load.rs` (classé S) | corps du noyau = corps actuel ; suites `l*`, `sf*`, `rf*` inchangées ; mutations § 10.5 ; G-CM |
| RK-2 | L'extraction devient un niveau déguisé | une seule boucle, structurelle ; ne lit pas le manifeste ; ARB-1 ; option Z rejetée |
| RK-3 | Attribution d'erreur différente de v1 : table incohérente, artefact vide ou manifeste déclaré vide → `Offsets` ; troncature dans le manifeste ou un artefact intermédiaire → `Offsets` (les offsets dépassent `len`, § 4.5.3) ; seule la troncature du dernier artefact atteint le niveau 3 | classification explicite (§ 4.5), `pk3`, `pk11`, `pk14` |
| RK-4 | Un glissement vers l'option Y (extraction qui recalcule la longueur du manifeste) rendrait `EntryCountMismatch` inatteignable et dupliquerait la règle ; priorité A-2 mal ordonnée | A-1 option X et A-2 clos ; `pk6`, `pk14`, mutations § 10.5 |
| RK-5 | Offsets `u64` sur cible plus étroite : un offset non représentable est rejeté (`Offsets`) | `pk5` ; aucune cible 32 bits vérifiée |
| RK-6 | Constantes `HEADER_LEN` sous des chemins distincts | usages qualifiés ; littéraux écrits à la main |
| RK-7 | Deux instances du noyau : taille de binaire | acceptée (hors hot path) |
| RK-8 | `PackageError` `non_exhaustive` : bras par défaut pour les `match` externes | aucun `match` externe dans le dépôt lu |
| RK-9 | Prémisse « octets immuables » (P5, R14) étendue à la table, lue plusieurs fois | inchangée ; à reconsidérer avec un accès projeté en mémoire modifiable |
| RK-10 | `expect(dead_code)` : un oubli de retrait en V2-P fait échouer `clippy` | voulu |
| RK-11 | Premiers modules publics d'Assets : surface d'API élargie par les `pub` internes aux deux modules | éléments internes `pub(crate)` ; revue de l'API publique en V2-L et V2-P |
| RK-12 | O-1 : représentation des erreurs Forge | arbitrage avant V2-F |
| RK-13 | Collision de signature `VPKG` | recherche négative **déclarée** (F-14), dossier de preuve à consigner (§ 3.5) ; limite : fichiers binaires non vérifiés ; P-1 non adoptée |

**Non démontré** : exécution de tout test, de Miri et de la MSRV ; codes d'erreur des doctests `compile_fail` ; comportement sur cible 32 bits ; performance du chargement ; correspondance q ↔ artefacts (F2) ; authenticité du paquet ; absence de `VPKG` dans les fichiers binaires ; la recherche textuelle (F-14) n'est pas une vérification binaire et n'est pas vérifiée par la session.

---

## 12. Effets prévus sur le corps du plan V2 (non appliqués)

À P-CONC, le plan V2 pourrait être mis à jour (aucune modification faite ici) : § 4.5 (règle de séparation des constantes : espaces de noms) ; § 4.8 (sujets désormais décidés) ; § 8 (B-1, B-4 : E3) ; § 9 (R-C1 à R-C13 résolus par cette note, sauf O-1) ; § 10.3 à 10.5 (fichiers) ; § 14 (décisions ouvertes : D-N4, B-1, B-4, noms, emplacement des tests ; ajout de O-1 ; A-1 et A-2 clos). La roadmap reste compatible : V2-L livre le lecteur v2 additif, V2-P l'enveloppe, le noyau partagé et le retour de q.

---

## 13. Critères P-CONC et suite

### 13.1 Critères proposés pour franchir P-CONC [Proposition]

| # | Critère | Section |
|---|---|---|
| 1 | Layout exact, schéma byte-offset, exemple chiffré | § 3 |
| 2 | Algorithmes d'extraction et de lecture dans leur ordre ; arithmétique sans saturation, troncature ni `as` | § 4 |
| 3 | Classification des erreurs, avec parcours du lecteur et cas limites explicites ; A-1 (option X) et A-2 clos | § 4.5 |
| 4 | Noyau commun : conditions de la refactorisation, trois `unreachable!`, preuve structurelle d'inatteignabilité | § 4.2, § 4.4, § 5 |
| 5 | Signatures d'API, constantes et visibilités, durées de vie, erreurs | § 6 |
| 6 | Production de la Forge sans modification de `compile_assets` ; nomenclature acceptée ; O-1 identifié | § 7 |
| 7 | Tests et mutations couvrant VQ-1 à VQ-7 et VQ-9, avec correspondance explicite | § 10 |
| 8 | Garanties 1 à 9, limites de provenance, risques, non démontré | § 9, § 11 |
| 9 | Décisions adoptées distinguées des propositions et des points ouverts | § 1 |
| 10 | Aucun code, test, `Cargo.toml`, ADR modifié ; aucun commit | § 0 |

### 13.2 À traiter, et clos

**À traiter avant P-CONC :**
1. **P-1** : `VPKG` et version `1`, **en attente du dossier de preuve** (§ 3.5) ; ne peut être marquée adoptée avant sa consignation.

**Ouvert hors P-CONC :** **O-1**, avant V2-F seulement.

**Clos (arbitrés)** : P-2 (sauf les valeurs soumises à P-1), A-1 (option X), A-2 (avec priorité), preuve du § 4.2, P-3, P-4, P-5 (nomenclature, contrat), P-7, documentation de `Version`.

### 13.3 Suite prévue (sans autorisation tant que P-CONC n'est pas franchie)

1. **Clôture V2-C** : examen de la version finale (r5), consignation du dossier de preuve de P-1, arbitrage de P-1, validation P-CONC.
2. **V2-L** (session dédiée) : lecteur de manifeste v2 additif, helper commun, documentation de `Version`, constantes `manifest_v2`.
3. **V2-P** (session dédiée) : enveloppe, extraction, noyau partagé, `load_package`, `PackageError`.
4. **V2-F** (session dédiée) : voie de compilation Forge, après résolution de O-1.
5. **V2-Z** (session dédiée) : régression globale et clôture.

P-CONC n'est **pas franchie**. Aucune implémentation V2-L ou V2-P n'est autorisée avant votre validation explicite.
