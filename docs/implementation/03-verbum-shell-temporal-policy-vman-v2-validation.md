# Verbum — Dossier de validation V2-Z du chantier VMAN v2

**Statut : dossier de validation et trace historique. Non normatif.** **Version définitive** du 21 novembre 2026, établie à partir des rapports Lot 1, Lot 2, complément `pf9`, Lot 3 et de l'inventaire des sorties de commandes, puis mise à jour avec les confirmations et arbitrages finaux de l'utilisateur (§ 6.2, § 7.4 à 7.7, § 8). Ce document ne crée aucun invariant, ne rouvre aucune décision et ne modifie rétroactivement ni le plan V2-0 ni l'annexe V2-C. Après la clôture, le code validé reste la source de vérité opérationnelle ; ce dossier conserve l'état vérifié à la date de clôture et n'a pas vocation à être resynchronisé, sauf correction factuelle ou besoin explicite de traçabilité.

Documents de référence (non reproduits ici) :
- plan : `docs/implementation/03-verbum-shell-temporal-policy-vman-v2.md` (V2-0, r1) ;
- conception : `docs/implementation/03-verbum-shell-temporal-policy-vman-v2-annex-v2c.md` (V2-C, r5.2).

Légende des niveaux de preuve, utilisée dans tout le document :

| Marqueur | Sens |
|---|---|
| **[R]** | Preuve réelle sur le dépôt : sortie de commande fournie par l'utilisateur |
| **[C]** | Confirmation de l'utilisateur sur le dépôt réel ; sorties brutes non transmises à Claude |
| **[W]** | Exécution par Claude sur un workspace reconstitué à partir des fichiers transmis |
| **[S]** | Inspection statique (lecture du code transmis) |
| **[D]** | Déduction ou raisonnement, sans exécution |
| **[N]** | Non vérifié |

---

## 1. Objet et raison d'être

### 1.1 Pourquoi V2-Z

Les unités V2-L (lecteur de manifeste VMAN v2), V2-P (enveloppe VPKG et chargement du paquet) et V2-F (producteur Forge) étaient terminées, de même que l'arbitrage O-1. V2-Z est l'unité de clôture prévue par le plan (§ 10.6) : régression globale, consignation, liste « non démontré », porte P-E.

L'objectif était de vérifier, avant de clôturer VMAN v2 :
1. les **invariants historiques v1** (octets produits, ordre des niveaux, tout-ou-rien, absence d'allocation, erreurs) ;
2. les **nouveaux contrats v2** (lecteur de manifeste, enveloppe, classification des erreurs, contrat d'erreur Forge, layout) ;
3. l'**intégration inter-crates** (graphe de dépendances, consommateurs, séparation des responsabilités).

### 1.2 Distinction avec les documents existants

| Document | Rôle | Ce dossier |
|---|---|---|
| Plan V2-0 | Organisation du chantier, garde-fous, unités, portes | Ne le remplace pas ; constate l'état atteint |
| Annexe V2-C | Conception technique : layout, algorithmes, erreurs, API | Ne la remplace pas ; ne duplique pas ses contrats |
| **Ce dossier** | Preuves recueillies, commandes, résultats, limites, réserves | Trace de validation, sans portée normative |

---

## 2. Périmètre et méthode

### 2.1 Crates examinés

| Crate | Contenu examiné |
|---|---|
| `verbum-assets` | 12 fichiers `src/` (`lib`, `header`, `format`, `sprite`, `tileset`, `map`, `font`, `script`, `manifest`, `manifest_v2`, `load`, `package`), `Cargo.toml` |
| `verbum-forge` | `src/` dont `batch.rs`, `package.rs`, `tests/pipeline.rs`, `tests/package.rs`, `Cargo.toml` |
| `verbum-script`, `verbum-shell` | manifestes, consommation des API d'Assets, sentinelles et exemples (Lot 3) |
| Workspace | `Cargo.toml` racine, `Cargo.lock`, `tree.md` |

Ces trois lots ne constituent **pas** un audit général de Verbum : ils ciblent le chantier VMAN v2 et ses frontières.

### 2.2 Les trois lots

| Lot | Objet | Nature de la vérification |
|---|---|---|
| Lot 1 | `verbum-assets` : invariants v1, V2-L, V2-P | Lecture, exécution sur copie isolée **[W]**, 14 mutations sur la copie |
| Lot 2 | `verbum-forge` et contrat avec Assets | Lecture **[S]**, aucune exécution ; mutations évaluées par raisonnement **[D]** |
| Complément `pf9` | Test hétérogène `compile_package` → `load_package` | Écriture d'un test, exécution **[W]** |
| Lot 3 | Intégration workspace, consommateurs, sentinelles Shell, clôture | Lecture **[S]** et sorties `cargo tree` / `rg` **[R]** fournies par l'utilisateur |

### 2.3 Méthode

Chaîne **invariant → code → test**. Pour chaque invariant de l'annexe V2-C et du plan, les rapports relèvent le code qui le porte et le test qui l'épingle. La couverture est qualifiée :
- **directe** : un test asserte l'invariant ;
- **indirecte** : couvert par un test voisin ou par un mécanisme partagé ;
- **structurelle / raisonnement** : garanti par la construction (par exemple `no_std` sans `alloc`) ou démontré par lecture, sans test dédié.

### 2.4 Limites des snapshots transmis

- Le snapshot est la seule source technique des lots 1 à 3 ; aucun accès direct au dépôt réel.
- **Aucune baseline antérieure à V2** n'a été transmise : « v1 inchangé » repose sur l'inspection du code, la conservation des tests historiques et leur exécution réelle confirmée par l'utilisateur, non sur un diff.
- Absents ou non transmis selon les lots : `README.md` des crates ; `shell/src/{policy, progressor, driver, …}.rs`, `schedule`, `render`, `ecs` (sans objet pour le Lot 3).
- Le Lot 1 disposait des tests Assets mais pas de `vm_1`, `t6_4`, `f2`, `f3`, `lt5`, LT-5n/5t ; ces fichiers ont été transmis ensuite (Lots 2 et 3).

### 2.5 Nature des exécutions

| Source des résultats | Marqueur | Détail |
|---|---|---|
| Dépôt réel, utilisateur | **[R]** | Sorties fournies : `cargo tree`, `rg`, recherche `VPKG` (§ 7.3) |
| Dépôt réel, utilisateur | **[C]** | Confirmation : `pf9` intégré et testé ; build, test, clippy, rustfmt, Miri, MSRV 1.85 verts (§ 7.4) |
| Workspace reconstitué, Claude | **[W]** | Copie isolée du crate Assets (Lot 1) ; workspace reconstruit pour `pf9` (cargo 1.97, `toml` 1.2.2 au lieu de 1.1.8 attendu ; `pipeline.rs` et `verbum-ecs` exclus) |
| Lecture | **[S]** | Lots 2 et 3 |
| Raisonnement | **[D]** | Mutations du Lot 2, argument 32 bits |

---

## 3. Historique des unités V2

Les dates d'achèvement de V2-L, V2-P et V2-F ne figurent pas dans les sources fournies ; elles ne sont pas reconstituées ici.

| Étape | Contenu | Décisions notables (détail : annexe V2-C) |
|---|---|---|
| **V2-0** | Plan du chantier (r1), validé P-V2 au 17 novembre 2026 | Paquet unique, v1 intact, q jamais accepté par défaut, aucune variante d'erreur ajoutée par anticipation |
| **V2-C** | Note de conception, révisions 1 à 5.2 ; **P-CONC franchie le 19 novembre 2026** | L3b (layout), A-1 option X, A-2 et sa priorité, P-1 (`VPKG`, version 1), P-2, P-3 à P-5, P-7, E3, N1 (noyau unique), preuve d'inatteignabilité du § 4.2 |
| **V2-L** | Lecteur de manifeste VMAN v2 additif (`manifest_v2.rs`), helper `Manifest::from_region`, documentation de `ManifestError::Version` | `Manifest::parse` strictement v1 ; `ZeroQuantum` interne au lecteur (P-4) |
| **V2-P** | Enveloppe VPKG (`package.rs`), noyau partagé `load_core`, `load_package`, `PackageError` | `load` devenu enveloppe mince ; aucun niveau créé, supprimé ni réordonné |
| **O-1** | Représentation des erreurs de taille côté Forge, clos le 19 novembre 2026 (r5.1) | **Option B** : `CompilePackageError { Forge, PackageTooLarge, InconsistentBatch }` ; `ForgeError` inchangé ; E3 non rouvert |
| **V2-F** | Producteur `compile_package` / `CompiledPackage` (`forge/src/package.rs`) | `compile_assets` inchangé ; layout pur isolé ; q nul testé avant toute compilation |
| **Corrections finales** | r5.1 et r5.2 de l'annexe (20 novembre 2026) : clôture de O-1, harmonisation documentaire, emplacement des tests V2-F arbitré (`forge/tests/package.rs`, `tests/pipeline.rs` intact), `pl1`–`pl4` maintenus dans V2-F | Les sources ne décrivent pas d'autre correction de code entre V2-F et V2-Z que l'ajout de `pf9` ; point non précisé par les confirmations finales ; aucune autre correction n'est portée à la connaissance de ce dossier |
| **V2-Z** | Vérification en trois lots + complément `pf9` | Objet de ce dossier |

Chaîne de production et de chargement, telle que définie par l'annexe : `AssetSource[] → Forge → VPKG → Assets → dépôt des ressources`.

---

## 4. Lot 1 — `verbum-assets`

Méthode : lecture des 12 fichiers `src/` et de l'annexe, exécution sur copie isolée **[W]**, mutations sur la copie uniquement. Aucun fichier du snapshot n'a été modifié.

### 4.1 Inventaire

| Élément | Constat |
|---|---|
| Fichiers | `Cargo.toml`, 12 fichiers `src/` ; aucun `tests/`, aucune fixture : **tous les tests sont inline** |
| Tests unitaires (107) | sprite 6, tileset 6, map 8, font 9, script 14, manifest 17, manifest_v2 9, load 24, package 14 |
| Doctests (12) | `AssetId` (`compile_fail` ×8, `manifest.rs`) ; `load_package` (4, `package.rs`) |
| Dépendances | `[dependencies]` vide, aucune dev-dépendance ; `Cargo.lock` concordant |
| `unsafe`, allocation | Aucun `unsafe`, `alloc`, `Vec`, `Box` ni `String` hors tests ; `#![no_std]` sans `extern crate alloc` |
| `expect(dead_code)` | Aucun : le retrait forcé en V2-P est effectif |
| Surface publique ajoutée | `pub mod manifest_v2` (4 constantes) ; `pub mod package` (4 constantes, `PackageError`, `load_package`) ; réexport racine de `PackageError` et `load_package`. `Envelope`, `read_manifest_v2`, `ManifestV2Error`, `load_core`, `Manifest::from_region` : `pub(crate)` |

### 4.2 Compatibilité v1

| Invariant | Couverture |
|---|---|
| Ordre de `Manifest::parse` (en-tête, signature, version avant la longueur, longueur, doublons) | **Directe** : `m3` (×3), `m4` (×2, `m4_unknown_version` épingle `Version`), `m5` |
| Format VMAN v1 (`10 + 16·n`, little-endian), constantes `MANIFEST_*` inchangées | **Directe** : `m1`, `m2` |
| `parse` strictement v1 | **Directe** : `m4`, `mv8` |
| Niveaux 1 à 4 de `load` | **Directe** : `l1`–`l5`, `tb1`, `ld1`, `sf1`–`sf6`, `rf1`–`rf6`, `scr_l1`–`scr_l3` |
| Tout-ou-rien (aucun dépôt avant la fin du niveau 3) | **Directe** : `l3` (×4), `rf1`–`rf6`, `sf1`, `scr_l3` |
| Priorité `Binding` > `SlotFormat` > `Artifact`/`Reference` | **Directe** : `sf3`, `sf6` |
| Erreurs et classifications historiques | **Directe** : `m7` (×5), `sf*`, `rf*` |
| Formats d'artefacts (VSPR, VTLS, VMAP, VFNT, VSCR) | **Directe** : `p1_*`, `tls_r*`, `map_r*`, `fnt_r*`, `scr_r*`, `scr_k1` |
| Partage de `from_region` et du noyau sans modification sémantique | **Directe** pour les tests listés ; **indirecte** pour le diff, l'état antérieur à V2-P étant absent ; `mv7` compare `parse` et le lecteur v2 sur 5 variantes |

Les noms des tests historiques cités existent dans le snapshot. `vm_1`, `t6_4`, `f*`, `lt5`, LT-5n/5t sont hors du crate Assets et traités aux Lots 2 et 3.

### 4.3 Lecteur VMAN v2 (V2-L, `manifest_v2.rs`)

| Invariant | Couverture |
|---|---|
| Module public, signature `VMAN`, version 2, en-tête de 18 octets, assertions `const _` | **Directe** : `mv9`, `mv1`. La publicité du module n'a pas de test externe au Lot 1 (voir C1, § 4.7) |
| `q_ns` strictement positif, little-endian | **Directe** : `mv1`, `mv2`, `mv4` |
| Longueur exacte `18 + 16·n`, tous les préfixes, surplus, `n` hostile | **Directe** : `mv3` (n = 0 à 3) |
| Doublons | **Directe** : `mv6` (adjacents) ; **indirecte** : `mv7`, `m5` (non adjacents) |
| Ordre champ par champ (6 octets, signature, version, 8 octets de q, q nul, 4 octets de n, longueur) | **Directe** : `mv2` (q nul avec 14 octets ou `n` hostile : `ZeroQuantum` ; 13 octets : `Truncated` ; q non nul de 14 à 17 octets : `Truncated`), `mv5` (version avant toute exigence de longueur ; v1 vide de 10 octets : `Version`) |
| Réutilisation de `Manifest<'a>` | **Directe** : `mv1`, `mv7` |
| Aucune allocation ni copie | **Structurelle** (crate `no_std` sans `alloc`) ; aucun test dédié |

### 4.4 Lecteur et enveloppe VPKG (V2-P, `package.rs`)

| Invariant | Couverture |
|---|---|
| Signature `VPKG`, version d'enveloppe 1 | **Directe** : `pk4`, `pk12` |
| `T = 10 + 8·n'` en `checked_*`, `n'` hostile rejeté en O(1) | **Directe** : `pk5`, `pk2` |
| Offsets `u64` little-endian absolus, strictement croissants, bornés | **Directe** : `pk5`, `pk1` |
| Sections d'artefact non vides | **Directe** : `pk11` |
| Extraction sans lecture du manifeste (A-1, option X) | **Directe** : `pk14` (cas 10 et 11 : `Load(Manifest(TrailingBytes))`) |
| Classification `Truncated` / `Offsets` / erreurs du lecteur (A-2 et sa priorité) | **Directe** : `pk3` (chaque préfixe de l'exemple chiffré), `pk14` (les 12 cas) |
| `n'` confronté à `n` par `check_bindings` | **Directe** : `pk6` |
| Noyau partagé sans contrôle dupliqué | **Directe** : `pk8` (différentiel v1/v2 sur dépôts et erreurs) |
| `Ok(q)` seulement après tous les dépôts ; aucun dépôt avant la fin du niveau 3 | **Directe** : `pk1`, `pk7`, `pk9`, `pk3`, `pk11` ; le succès tardif relève aussi de la structure du code (`load_core(..)?` puis `Ok(quantum)`) |
| Accès `Envelope::artifact(i)` : `None` hors table ; durée de vie `'a` indépendante de `&self` | **Directe** : `pk13` ; doctest `compile_fail,E0597` et jumeau valide |
| `unreachable!` d'`artifact_bytes` inatteignable (preuve § 4.2 de l'annexe) | **Raisonnement confirmé par lecture [S]** : `extract` établit P1 et P2, `check_bindings` établit P3, et précède le premier appel (P4) |
| Visibilité interne (`Envelope`, `read_manifest_v2`, `load_core`) | Doctests `compile_fail,E0603` : discriminants pour `Envelope`, partiels pour les deux autres (C1) |

### 4.5 Ordre des erreurs, emprunts, absence d'allocation

- **Frontières de modules** : graphe acyclique conforme (`package.rs` → `manifest_v2.rs`, `manifest.rs`, `load.rs`, `format.rs` ; aucune arête `load.rs` → `package.rs`) **[S]**.
- **Erreurs** : `PackageError` est `#[non_exhaustive]` et `Copy` ; `Load(LoadError)` encapsule les niveaux 1 à 4 ; `ManifestV2Error` reste interne ; `ZeroQuantum` est la seule erreur propre à V2 au niveau 1 **[S]**.
- **Durées de vie** : `Envelope` est `Copy` ; `artifact` rend `&'a [u8]` ; la fermeture `|i| envelope.artifact(i)` a le type `Fn(usize) -> Option<&'a [u8]>` **[S]** ; contrat d'emprunt couvert par le doctest `E0597` **[W]**.
- **Prémisse d'immuabilité des octets** (P5, R14) : valable pour des tampons ordinaires, comme en v1 ; à reconsidérer si un accès projeté en mémoire modifiable par un tiers était ajouté.
- **Conversions** : aucun `as` ni `unwrap_or(MAX)` dans `package.rs` et `manifest_v2.rs`. Deux branches typées sont inatteignables sur cible 64 bits, donc non testables : bloc de table différent de 8 octets ; échec de `usize::try_from(u32)`.

### 4.6 Mutations (sur la copie isolée uniquement) **[W]**

14 mutations ciblées sur `package.rs`, `manifest_v2.rs`, `manifest.rs`, `load.rs` : **13 détectées** par des tests ; la 14ᵉ est un **mutant équivalent** (aucun effet sémantique), qui survit par construction. Mutations nommées dans le rapport : dépôt avant le niveau 3 (19 tests détecteurs) ; signature d'enveloppe non contrôlée ; version d'enveloppe non contrôlée ; chaîne d'offsets `<` → `≤` ; dernier offset `>` au lieu de `≥` ; offsets big-endian ; `len ≤ T` ; compteur du noyau pris sur le manifeste ; q nul testé après `n` ; version 1 acceptée par le lecteur v2. Le rapport ne détaille pas les quatre autres.

### 4.7 Observations et réserves du Lot 1

| Réf. | Nature | Constat | Statut final |
|---|---|---|---|
| **E1** | Anomalie démontrée, non fonctionnelle | Lien rustdoc ``[`load`](crate::load)`` ambigu (fonction et module) dans la doc de `load_package` : `cargo doc` émet 2 avertissements **[W]** ; aucun effet sur build, test, clippy. `cargo doc` n'est pas dans les contrôles verts | Dette documentaire mineure **acceptée** (utilisateur) ; avertissement observé sur reconstitution, non confirmé sur le dépôt réel ; ne bloque pas la clôture |
| **C1** | Couverture | Les doctests `E0603` échouent aussi bien si l'élément est `pub(crate)` que si son module est privé ; pour `load_core`, le module `load` est privé. La publicité des constantes est établie par la compilation des consommateurs réels (Forge) | **Accepté** (utilisateur) ; risque faible |
| **C2** | Couverture | `pk8` ne compare que les `Sprite` déposés ; `pk9` : cas « liaisons » et « format de slot » en `is_err()` | **Accepté** (utilisateur) ; différentiel hétérogène comblé par `pf9` (§ 6) ; la faiblesse de `pk9` subsiste, risque faible (noyau unique) |
| **C3** | Non vérifié | Cibles 32 bits : résultats attendus identiques par raisonnement | Voir § 7.6 |
| **C4** | Non vérifié | Sentinelles Forge et Shell absentes du snapshot du Lot 1 | Présence établie aux Lots 2 et 3 ; exécution confirmée par l'utilisateur sur le dépôt réel **[C]** ; sentinelles Shell validées |
| O1 | Observation | Documentation de `ManifestError::Truncated` et `TrailingBytes` sur `10 + 16 × n` alors qu'elles servent aussi à `18 + 16 × n` ; seule la doc de `Version` a été mise à jour (annexe § 6.4) | Dette documentaire mineure **acceptée** (utilisateur) ; formulation historique conservée |
| O2 | Observation | `unwrap_or(u64::MAX)` dans `font.rs:83`, `map.rs:68`, `script.rs:150` ; quelques `as u64` / `as u32` sur constantes (dont `MANIFEST_ENTRY_LEN as u64` dans `from_region`). Aucun dans `package.rs` ni `manifest_v2.rs`. Antériorité non vérifiable (pas de baseline) | Consigné ; non traité |
| O3 | Observation | Quatrième `unreachable!` dans `manifest.rs::decode` (garanti par `chunks_exact`), absent de la preuve du § 4.2 | Consigné |
| O4 | Observation | Coquille dans la doc du module `package.rs` : « une section d'artefact est donc jamais vide » | Dette documentaire mineure **acceptée** (utilisateur) |
| O5 | Observation | `manifest_v2::HEADER_LEN` et `ENTRY_LEN` ne servent qu'à l'assertion de compilation et à `mv9` ; conforme à P-2 | Clos par `pf9` (§ 6.2) pour `ENTRY_LEN` |
| O6 | Observation | Doublons en O(n²), hérité de v1, borné par la taille du paquet | Consigné |

**Conclusion du Lot 1 : acceptable.** Aucune anomalie fonctionnelle démontrée ; traçabilité invariant → code → test établie pour tous les invariants v1, V2-L et V2-P.

---

## 5. Lot 2 — `verbum-forge`

Méthode : lecture intégrale **[S]** ; **aucune exécution** (ni `cargo`, ni Miri, ni mutation). Mutations du § 10.5 de l'annexe évaluées par raisonnement **[D]**. Pas de baseline antérieure à V2-F.

### 5.1 Inventaire

Forge `src/` : `lib.rs`, `batch.rs`, `id.rs`, `sprite.rs`, `tileset.rs`, `map.rs`, `font.rs`, `script.rs`, `package.rs`. Forge `tests/` : `pipeline.rs`, `package.rs`. `Cargo.lock` : Forge dépend de `toml`, `verbum-assets`, `verbum-ecs` ; aucune nouvelle dépendance.

### 5.2 `compile_assets` historique

- Signature, doctests (E0432, E0603, E0061) présents dans `batch.rs` **[S]**.
- Sentinelles présentes : `f2`, `f3`, `f7`, `f8` (`batch.rs`) ; `vm_1`, `t6_4`, `t6_6` (`pipeline.rs`) ; `scr_f5` (`script.rs`).
- `tests/pipeline.rs` ne contient aucune référence à `package`, `VPKG` ni `compile_package` : cohérent avec « intact » **[S]**.
- `CompiledSet` conserve `manifest()` et `artifacts()` ; `ForgeError` n'a aucune variante liée au paquet ; `batch.rs` n'importe rien de `package.rs` **[S]**.

### 5.3 `compile_package`, `CompiledPackage`, `CompilePackageError` (O-1, option B)

Conformité à l'annexe § 7.3 **[S]** :

- `CompilePackageError { Forge(ForgeError), PackageTooLarge, InconsistentBatch }` : `#[non_exhaustive]`, dérive `Clone`, `Debug`, `PartialEq`, `Eq` ; réexporté à la racine avec `compile_package` et `CompiledPackage`.
- **q nul** : testé en premier, avant `compile_assets` : `Forge(ZeroValue("q"))` ; `pf5` vérifie la priorité sur une source invalide.
- **Erreurs de `compile_assets`** : enveloppées par `map_err(Forge)` ; `pf6` couvre 7 lots invalides en comparant à `compile_assets` ; `TooManyAssets` n'est pas détourné.
- **Opacité de `CompiledPackage`** : champ privé, `as_bytes` seule, aucun constructeur ; doctests `compile_fail` (champ, `new`, `manifest()`, `artifacts()`, `quantum_ns()`) avec jumeau valide.

### 5.4 Layout pur et validations avant allocation

| Constat | Détail |
|---|---|
| Pureté | `package_layout` rend une `Layout` de quatre champs, sans collection ni allocation ; les longueurs sont parcourues deux fois via un itérateur `Clone` (P7, puis cumul) |
| Ordre | `package_layout(..)?` précède `Vec::with_capacity(layout.total)` |
| Plafond | `CAPACITY_LIMIT = isize::MAX.unsigned_abs()`, sans `as` ; P5 évalué avant l'allocation |
| Offsets | `next_offset` partagé par la validation et `write_offsets` ; écriture directe dans le tampon final, `u64` little-endian ; `s_0 = T + 18 + entries_len`, équivalent à `T + 18 + 16·n` car P7 impose `entries_len = 16·n` |
| Assemblage | Ordre enveloppe, table, manifeste v2, artefacts ; trois `debug_assert_eq!` (`table_end`, `first_artifact`, `total`) |
| Interdits | Aucun `as`, saturation, `unwrap_or(MAX)` ni `unreachable!` hors tests |

### 5.5 P1 à P7 (annexe § 7.3.2)

| # | Contrôle | Variante | Remarque du Lot 2 |
|---|---|---|---|
| P1 | `u32::try_from(artifacts.len())` | `InconsistentBatch` | Testé par `pl4` (`repeat_n`, 2³²) |
| P2 | `T = 10 + 8·n'` | `PackageTooLarge` | Atteignable seulement par la couture `ceiling` de `layout_within`, P1 plafonnant `n` à `u32::MAX` en 64 bits |
| P3 | `s_0 = T + 18 + 16·n` | `PackageTooLarge` | `pl2` aux seuils exacts via `ceiling` |
| P4 | offsets cumulés | `PackageTooLarge` | `pl2`, arithmétique réelle |
| P5 | longueur ou capacité > `isize::MAX` | `PackageTooLarge` | `pl3`, seuil exact et +1 |
| P6 | `u64::try_from(usize)` | `PackageTooLarge` | Inatteignable sur cible ≤ 64 bits ; documenté dans `pl2` |
| P7 | invariants de la sortie de `compile_assets` | `InconsistentBatch` | `pl4` (compte, longueur de manifeste, artefact vide, priorité sur P4) ; test complémentaire de `split_manifest` |

### 5.6 Tests

- `pf1`–`pf7` : présents dans `tests/package.rs`, conformes à l'annexe § 10.4. `pf8` (`compile_fail`) est porté par les doctests de `CompiledPackage`, comme l'annonce l'en-tête du fichier de tests.
- `pf1` : `REFERENCE_PACKAGE` est identique octet pour octet à `PACKAGE` d'Assets (`pk1`) : les deux littéraux écrits à la main concordent de part et d'autre du contrat.
- `pf4` : les deux quanta ont leurs 8 octets tous différents ; l'assertion « exactement 8 octets » est donc discriminante.
- `pl1`–`pl4` : unitaires dans `forge/src/package.rs` (layout privé) ; `pl1` valeurs exactes (vide, référence, 1 artefact, longueurs inégales) ; `pl2` P4, P2, P3 ; `pl3` seuil `isize::MAX` et offsets écrits au seuil ; `pl4` P1 et P7.
- Mutations V2-F évaluées par raisonnement **[D]** : q non écrit / écrit deux fois, offsets en `10 + 16·n`, artefacts permutés, table big-endian (`pf1`, `pf2`) ; contrôle de q après compilation (`pf5`) ; entrées d'une autre compilation, accesseurs exposés (`pf3`, `pf8`) ; seuil décalé, permutation des variantes, P7 par `unreachable!` (`pl3`, `pl4`). Deux cas non détectables par test : `as` ou saturation (mutant équivalent en 64 bits) ; allocation avant validation du layout (L2).

### 5.7 Constats du Lot 2

**Anomalies démontrées : aucune.**

| Réf. | Lacune | Gravité | Statut final |
|---|---|---|---|
| **L1** | Pas de test Forge → paquet → `load_package` pour Tileset, Map, Font, Script, ni avec artefacts de longueurs inégales | Modérée | **Comblée par `pf9`** (§ 6) |
| L2 | Mutation « allocation avant validation du layout » détectable par revue seulement ; `pl1`–`pl4` appellent `package_layout`, pas `compile_package` | Faible, inhérente | **Accepté** (utilisateur) |
| L3 | Le doctest `verbum_forge::package::package_layout` échoue en E0603 même si la fonction devenait `pub` (module privé) : non discriminant, comme C1 | Faible | **Accepté** (utilisateur) |
| L4 | P6 non testable sur cible ≤ 64 bits | Connu | Limite documentée |

Observations non bloquantes : `layout_within` expose un paramètre `ceiling` utilisé seulement par les tests (production : `usize::MAX`) ; la doc de `ForgeError::ZeroValue` dit « le champ vaut zéro » alors que q est un paramètre (cosmétique) ; `map.rs::encode_map` contient `unwrap_or(u32::MAX)` (hérité, déjà signalé à l'annexe § 7.3.5, hors périmètre) ; le critère `cargo tree -i verbum-forge -e normal` n'était pas vérifiable au Lot 2 (levé au Lot 3, § 6.4).

**Conclusion du Lot 2 : acceptable.**

---

## 6. Complément `pf9` et Lot 3

### 6.1 Origine de `pf9`

Le Lot 2 a proposé (L1) un test additif, sans modifier `pk8` ni les tests existants. `pf9` a été ajouté dans `forge/tests/package.rs` : `pf1`–`pf7` inchangés ; seuls `pf9`, ses constantes, ses helpers locaux (`hetero_sources`, `Typed`, `le32`/`le64`, `source`) et les imports nécessaires sont nouveaux. Aucune modification de `pipeline.rs`, de `package.rs`, des API Forge/Assets ni des dépendances.

### 6.2 Contenu du test hétérogène

Lot : tileset, carte, deux scripts, sprite, police (cinq formats, six artefacts).

| Élément | Valeur (rapports `pf9` et Lot 3) |
|---|---|
| Longueurs (ordre de sérialisation, artifact_index 0 à 5) | `HETERO_LENGTHS = [18, 34, 68, 14, 44, 38]`, toutes distinctes (une permutation d'offsets est détectable) |
| Offsets (idem) | `HETERO_OFFSETS = [172, 190, 224, 292, 306, 350]` ; total 388 ; `T = 58` ; manifeste 114 octets |
| Slots (ordre des sources du rapport `pf9`) | `HETERO_SLOTS = [2, 5, 4, 0, 1, 3]` ; relation avec `artifact_index` : voir la clarification ci-dessous |
| Calcul des offsets | Vérifié deux fois : valeurs littérales, puis recalcul à partir des longueurs réelles de `compile_assets` |
| Octets du paquet | Signature, version, `n'`, table lus littéralement ; sections contiguës, sans padding ; chaque section égale à l'artefact de `compile_assets` |
| Manifeste | q lu en little-endian, `n = n'`, entrées égales à celles de la v1 |
| Résolution Map → Tileset | Identifiant du tileset dans la carte égal à l'id de l'entrée 0 ; `map.tileset() == SlotIndex(2)`, c'est-à-dire le slot du tileset, distinct de son indice source (0) et du slot de la carte (5) ; son `artifact_index` vaut aussi 2 d'après les valeurs publiées (clarification ci-dessous) |
| Chaîne complète | `load_package` rend `Ok(Q)` avec 6 dépôts ; contenu vérifié pour chaque format (sprite 96×48 au slot 0, tileset au slot 2, carte 3×2 au slot 5, police au slot 3, scripts `[(4, intro), (1, outro)]`) |
| Équivalence | `load` sur le même `CompiledSet` produit un `Typed` égal à celui de `load_package` (couvre aussi un Script sur `load_package`) |
| Contrôle d'ordre | Selon le rapport `pf9`, le test échoue si l'ordre des artefacts est substitué à l'ordre des slots (manifeste, offsets ou résolution de la carte) : raisonnement **[D]**, sans mutation. Au vu de la clarification, la distinction `artifact_index` / `slot_index` n'est observable que sur les deux scripts ; celle entre ordre des sources et ordre de sérialisation l'est sur cinq entrées sur six |

Points résolus par `pf9` : L1 (Lot 2), C2 pour le différentiel de dépôt (Lot 1), et l'observation O5 sur `manifest_v2::ENTRY_LEN`, désormais consommée depuis un crate externe (`manifest_v2::ENTRY_LEN * count`) ; les huit constantes publiques de `package::` et `manifest_v2::` sont ainsi exercées par un consommateur externe (Lot 3).

**Clarification de présentation (arbitrage de l'utilisateur, 21 novembre 2026).** Trois notions distinctes : l'indice source (ordre du rapport `pf9`), l'`artifact_index` (ordre de sérialisation) et le `slot_index`. Le test `pf9` n'est pas modifié : il a été exécuté et validé sur le dépôt réel **[C]**.

*Provenance des valeurs ci-dessous.* Le code de `tests/package.rs` et les entrées de manifeste produites par `compile_assets` n'étaient pas disponibles à Claude lors de cette mise à jour. Les valeurs sont **déduites [D]** des longueurs, offsets et slots publiés et de la correspondance fournie par l'utilisateur ; elles n'ont pas été relues dans le code. Les offsets triés (172 < 190 < 224 < 292 < 306 < 350) fixent l'ordre de sérialisation.

Ordre du rapport `pf9` (indice source) :

| Indice source | Artefact | Slot | Longueur | Offset | `artifact_index` déduit |
|---:|---|---:|---:|---:|---:|
| 0 | Tileset | 2 | 68 | 224 | 2 |
| 1 | Map | 5 | 38 | 350 | 5 |
| 2 | Script intro | 4 | 34 | 190 | 1 |
| 3 | Sprite | 0 | 18 | 172 | 0 |
| 4 | Script outro | 1 | 44 | 306 | 4 |
| 5 | Font | 3 | 14 | 292 | 3 |

Ordre de sérialisation (`artifact_index`) :

| `artifact_index` | Artefact | Slot | Longueur | Offset |
|---:|---|---:|---:|---:|
| 0 | Sprite | 0 | 18 | 172 |
| 1 | Script intro | 4 | 34 | 190 |
| 2 | Tileset | 2 | 68 | 224 |
| 3 | Font | 3 | 14 | 292 |
| 4 | Script outro | 1 | 44 | 306 |
| 5 | Map | 5 | 38 | 350 |

Correspondance indice source → `artifact_index` : `[2, 5, 1, 0, 4, 3]`.

*Correction des rapports précédents.* L'énoncé des rapports `pf9` et Lot 3 selon lequel `artifact_index ≠ slot_index` serait vrai, ou asserté, pour chaque entrée **n'est pas retenu**. D'après les valeurs publiées, `artifact_index = slot_index` pour le Tileset (2), la Map (5), le Sprite (0) et la Font (3) ; ils diffèrent seulement pour Script intro (1 contre 4) et Script outro (4 contre 1). De même, le Tileset a pour indice source 0, `artifact_index` 2 et `slot_index` 2 : le « indice d'artefact (0) » du résumé du Lot 3 correspond à l'indice source, non à l'`artifact_index`. Le contenu exact des assertions de `pf9` n'a pas été relu.

*Conséquences, sans remise en cause du test.* Les trois notions restent distinctes, mais leurs valeurs numériques coïncident pour plusieurs entrées : seul Script intro a ses trois valeurs distinctes (source 2, artefact 1, slot 4). Pour le lien Map → Tileset, `pf9` distingue le slot (2) de l'indice source (0) mais ne discrimine pas, sur ce lien, slot et `artifact_index`. La distinction `artifact_index` / `slot_index` est observable sur les deux scripts de `pf9`, et sur le lot de référence de `pf1` / `pk1` (« a » : artefact 0, slot 1 ; « foobar » : artefact 1, slot 0). La relation entre l'ordre des sources et l'ordre de sérialisation dépend de `compile_assets`, dont le code n'a pas été relu ici ; elle ne doit pas être extrapolée au-delà de ce lot.

Exécution : § 7.2 (**[W]**, reconstitution) ; intégration et test de `pf9` sur le dépôt réel, tous contrôles applicables verts : confirmation de l'utilisateur **[C]** (§ 7.4). Mutations : non appliquées ; la détection des substitutions d'ordre est un raisonnement **[D]**.

### 6.3 Lot 3 — intégration workspace

Méthode : lecture des manifestes (`Cargo.toml` racine, assets, forge, shell, script), `Cargo.lock`, `tests/package.rs`, sources et tests du Shell, sources de `verbum-script` **[S]**, et sorties réelles **[R]** de l'inventaire (§ 7.3).

### 6.4 Graphe Cargo

| Crate | Normales | Développement | Source |
|---|---|---|---|
| `verbum-assets` | aucune | aucune | manifeste + lock **[S]** |
| `verbum-forge` | `verbum-assets`, `toml` | `verbum-ecs` | manifeste **[S]** |
| `verbum-script` | `verbum-assets` | aucune | manifeste **[S]** |
| `verbum-shell` | aucune | assets, ecs, forge, render, schedule, script | manifeste **[S]** |

Les sorties `cargo tree` **[R]** concordent : Forge sans consommateur normal (seul `verbum-shell` la consomme, en dev-dependency) ; Assets consommé normalement par Forge et Script, et en dev-dependency par Shell. Le critère « `cargo tree -i verbum-forge -e normal` sans dépendant » (annexe § 10.6), non vérifiable au Lot 2, est **confirmé par la sortie réelle**. Aucune dépendance ajoutée ; `Cargo.lock` cohérent ; `unsafe_code = "deny"` hérité du workspace par les quatre crates **[S]**. Aucune dépendance cyclique.

### 6.5 Consommateurs des API

| Chemin | Consommateurs (rg **[R]** + lecture **[S]**) |
|---|---|
| **v2** (`compile_package`, `load_package`, `PackageError`, `CompilePackageError`, constantes `manifest_v2::*`, `package::*`) | Uniquement : `forge/src/package.rs` (producteur), `forge/src/lib.rs` (réexport), `forge/tests/package.rs`, et côté Assets `package.rs`, `lib.rs`, `load.rs` (commentaire de documentation). Aucun autre crate, y compris Shell et Script |
| **v1** (`compile_assets`, `load`, `Manifest::parse`) | Shell (exemples `level_headless`, `script_headless` ; tests `level_loaded`, `script_loaded`, `script_suspend`, `borrowed_resource`, `script_support/mod.rs` ; doctests de `lib.rs`), Forge (`pipeline.rs`, `batch.rs`) ; signatures inchangées ; aucune migration V2 implicitement requise |

### 6.6 Sentinelles Shell

| Sentinelle | Emplacement | Constat |
|---|---|---|
| `lt5` | `shell/tests/borrowed_resource.rs` (ligne 289 au rg) | Manifeste littéral v1 de 42 octets (`n = 2`), `load` à quatre arguments, somme 7716 puis 7717 sur tampon altéré **[S]** |
| LT-5n | `shell/src/lib.rs`, doctest `compile_fail` | `drop(map_bytes)` après dépôt par `load`, même littéral **[S]** |
| LT-5t | `shell/src/lib.rs`, doctest jumeau valide | Identique sans `drop`, exécution attendue 7716 **[S]** |

Présence et localisation établies par `rg` **[R]** ; contenu par lecture **[S]** ; passage confirmé par l'utilisateur sur le dépôt réel **[C]** ; sentinelles validées par l'arbitrage final de l'utilisateur. Le même mécanisme d'emprunt est couvert côté v2 par le doctest `compile_fail,E0597` de `load_package`.

### 6.7 Script, Shell et absence d'adoption V2

- **`verbum-script`** : n'utilise que `Script`, `SlotIndex`, `Instruction`, `SCRIPT_MAX_STEPS_PER_TICK`, `SCRIPT_REGISTERS` (`table.rs`, `step.rs`, `state.rs`, `emissions.rs`, `lib.rs`) ; aucune référence à `Manifest`, `load`, `load_package` ni à Forge ; `no_std` sans `alloc` ; contrat `deposit(SlotIndex, Script<'a>)` inchangé ; aucune contradiction avec ARB-9 ni G-S6 **[S]**.
- **`verbum-shell`** : consommateur de développement uniquement ; aucun symbole v2 **[R][S]**. Cette absence est **normale et hors périmètre** : l'intégration de VPKG au Shell relève de travaux ultérieurs (VQ-8, F3b).
- Séparation des responsabilités : Assets définit layout, lecteurs, `load`, `load_package` ; Forge est seule productrice et consomme les constantes publiques d'Assets, sans lien inverse. Aucune contradiction inter-crates relevée **[S]**.

**Conclusion du Lot 3 : favorable à la clôture technique**, sous les réserves du § 7.7 et de la section 8.

---

## 7. Résultats et limites de validation

### 7.1 Principe de lecture

Un résultat n'a que la portée de son environnement. Les rapports distinguent les exécutions de Claude sur reconstitution **[W]** des résultats réels de l'utilisateur **[R]**. Les commandes sont reproduites telles que fournies ; lorsqu'une forme n'est connue que par l'annexe ou le plan, elle est signalée comme **prévue**, non comme exécutée.

### 7.2 Exécutions de Claude sur reconstitution **[W]**

| Lot | Commande (telle que rapportée) | Environnement | Résultat communiqué | Portée |
|---|---|---|---|---|
| 1 | `cargo test` | Copie isolée du crate Assets dans un workspace minimal ; toolchain 1.97 | 107 tests unitaires et 12 doctests verts | Crate Assets seul ; pas le dépôt réel |
| 1 | `cargo clippy --all-targets -D warnings` (forme du rapport) | idem | Propre | idem |
| 1 | `cargo fmt --check` | idem | Propre | idem |
| 1 | `cargo doc` | idem | 2 avertissements `crate::load is both a function and a module` (E1) | idem ; non exécuté sur le dépôt réel |
| 1 | 14 mutations manuelles (pas de commande unique) | Copie isolée | 13 détectées, 1 mutant équivalent | Mutations sur `package.rs`, `manifest_v2.rs`, `manifest.rs`, `load.rs` uniquement |
| `pf9` | `cargo test -p verbum-forge --test package` | Workspace reconstruit (cargo 1.97, `toml` 1.2.2 au lieu de 1.1.8 ; `pipeline.rs` et `verbum-ecs` exclus) | 8 tests passés, 0 échec (`pf1`–`pf7`, `pf9`) | Fichier `tests/package.rs` seul |
| `pf9` | clippy, rustfmt, Miri, MSRV 1.85, 32 bits, `pipeline.rs` | — | **Non exécutés sur la reconstitution** (contrôles sur le dépôt réel : § 7.4 ; 32 bits : non testé) | — |

Le Lot 2 n'a exécuté aucune commande. Le Lot 3 n'a exécuté aucune commande ; ses faits d'exécution viennent de l'utilisateur.

### 7.3 Résultats réels fournis par l'utilisateur **[R]** — inventaire du Lot 3

Environnement : dépôt réel de l'utilisateur (`~/Development/GitHub/verbum`), paquets en version 0.2.1. Les codes de retour ne figurent pas dans l'inventaire.

```
cargo tree -e normal -i verbum-forge
```
Résultat : la seule ligne `verbum-forge v0.2.1 (…/crates/engine/forge)`. **Aucun dépendant normal.**

```
cargo tree -e normal -i verbum-assets
```
Résultat : `verbum-assets v0.2.1` ← `verbum-forge v0.2.1` et `verbum-script v0.2.1` (`crates/extensions/script`).

```
cargo tree -e dev -i verbum-forge
```
Résultat : `[dev-dependencies]` ← `verbum-shell v0.2.1` seul.

```
cargo tree -e dev -i verbum-assets
```
Résultat : `[dev-dependencies]` ← `verbum-shell v0.2.1` seul.

```
rg -n \
  --glob '*.rs' \
  --glob 'Cargo.toml' \
  'verbum_assets|verbum_forge|compile_package|CompiledPackage|CompilePackageError|load_package|PackageError' \
  crates
```
Résultats significatifs :
- symboles v2 de production confinés à `forge/src/{lib,package}.rs`, `forge/tests/package.rs` et `assets/src/{lib,load,package}.rs` ; `forge/src/lib.rs:19` réexporte `CompilePackageError`, `CompiledPackage`, `compile_package` ; `assets/src/lib.rs:33` réexporte `PackageError`, `load_package` ;
- `forge/tests/package.rs` importe `load`, `load_package`, `manifest_v2`, `package as envelope` (lignes 14-16) et contient `pf9` (lignes 309 et 447) ;
- aucun symbole v2 dans `verbum-shell`, `verbum-script` ni `forge/tests/pipeline.rs` ;
- les consommateurs de `verbum_assets` dans `verbum-script` portent sur `Script`, `SlotIndex`, `Instruction`, `SCRIPT_*`.

```
rg -n 'lt5|LT-5n|LT-5t|load_package|compile_package' crates
```
Résultats significatifs : `lt5` à `shell/tests/borrowed_resource.rs:289` ; LT-5n/LT-5t dans `shell/src/lib.rs` (lignes 236, 240, 377) ; occurrences de `load_package`/`compile_package` limitées aux fichiers Forge et Assets listés ci-dessus.

**Portée.** `cargo tree` établit le graphe de dépendances effectif ; `rg` établit la présence et la localisation des symboles. Aucun des deux n'établit le contenu inchangé des fichiers v1 ni le passage des tests.

**Recherche de collision `VPKG` [R]** (consignée à l'annexe V2-C § 3.5, F-14 ; dépôt propre, commit `d30a1c725a33abbe48e41c83bb7744e68b263d7a`, hors `.git` et `target`) :

```bash
PATTERN='VPKG|0x56[[:space:]]*,[[:space:]]*0x50[[:space:]]*,[[:space:]]*0x4[Bb][[:space:]]*,[[:space:]]*0x47|\\x56\\x50\\x4[Bb]\\x47|86[[:space:]]*,[[:space:]]*80[[:space:]]*,[[:space:]]*75[[:space:]]*,[[:space:]]*71'
git grep -n -a -o -E "$PATTERN" HEAD -- .
grep -r -n -a -b -o -E \
  --exclude-dir=.git \
  --exclude-dir=target \
  "$PATTERN" .
```
Résultat : 15 occurrences, toutes documentaires dans la note V2-C ; aucune dans le code, les formats, les fixtures ou les binaires parcourus. Portée : fichiers compressés ou encodés non couverts ; recherche textuelle, non vérification binaire exhaustive. Elle précède l'adoption de P-1 et ne couvre pas l'état postérieur du dépôt.

### 7.4 Contrôles sur le dépôt réel, confirmés par l'utilisateur **[C]**

L'utilisateur confirme (21 novembre 2026) que `pf9` a été intégré et testé sur son dépôt réel, et que tous les contrôles applicables sont verts : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, Miri, MSRV Rust 1.85. Il s'agit d'une **confirmation de l'utilisateur** ; **Claude n'a pas consulté les sorties brutes**, qui ne lui ont pas été transmises.

Les quatre premières commandes ci-dessous sont celles citées par l'utilisateur. Les suivantes (MSRV, Miri) sont les formes **prévues** par l'annexe V2-C § 10.6 et le plan § 10.5-10.6 : l'utilisateur en confirme le résultat sans citer la commande exacte, qui n'est donc pas présentée comme exécutée.

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
# --- formes prévues par l'annexe ; commande exacte non citée par l'utilisateur ---
cargo +1.85.0 build --workspace
cargo +1.85.0 test --workspace
cargo +nightly miri test -p verbum-assets
cargo +nightly miri test -p verbum-forge -- --skip p3_1 --skip p3_2 --skip t6_1 --skip t6_3 --skip t6_6
```

Le Miri Forge est **partiel** par construction (exclusions connues de l'étape 5) ; il est à rapporter comme tel.

**Contrôles postérieurs à `pf9` : confirmés.** Le point laissé ouvert dans la version précédente du dossier est levé par la confirmation de l'utilisateur ci-dessus. Elle est cohérente avec l'inventaire, où `pf9` est présent dans le dépôt réel (`rg`, lignes 309 et 447 de `tests/package.rs`). La portée reste celle d'une confirmation, non d'une sortie examinée par Claude.

### 7.5 Points tranchés par l'utilisateur

- **Préservation v1 et baseline.** Aucun diff exhaustif entre le code actuel et une baseline antérieure à V2 n'a été établi dans V2-Z, et aucune nouvelle comparaison n'est demandée. La préservation v1 est étayée par l'inspection du code, la conservation des tests historiques et leur exécution réelle (§ 7.4). **Elle n'est pas démontrée par un diff.**
- **Contrôles postérieurs à `pf9`** : confirmés par l'utilisateur (§ 7.4).
- **`cargo doc`** : aucun contrôle sur le dépôt réel n'est demandé ; E1 reste observé sur la reconstitution seulement.
- **Corrections de code entre V2-F et V2-Z** autres que `pf9` : non précisées (§ 3).

### 7.6 Limites de validation

| Limite | Portée |
|---|---|
| **Cible 32 bits non testée** | Assets et Forge ; résultats attendus identiques au 64 bits par construction (arithmétique vérifiée) : argument de conception **[D]**, non validation (RK-5, RK-14, C3, L4) |
| **Pas de comparaison automatisée avec une baseline antérieure à V2** | « v1 inchangé » repose sur l'inspection du code, la conservation des tests historiques et leur exécution réelle confirmée **[C]** ; aucun diff exhaustif avec une baseline antérieure n'a été établi dans V2-Z (limite de preuve conservée) |
| **Recherche `VPKG`** | Fichiers compressés ou encodés non couverts ; recherche textuelle |
| **Authenticité** | Aucun digest ni signature ; l'intégrité vérifiée est structurelle, non cryptographique (annexe § 11.1) ; la garantie protège la chaîne normale, non la fabrication volontaire d'un paquet structurellement valide |
| **Correspondance q ↔ artefacts** | Non vérifiable : aucun contenu d'artefact ne dépend de q (F-10) ; relève de F2 |
| **VQ-8** | Bout en bout jusqu'à `TemporalPolicy` : dépendance de F3b, hors critères de V2-L, V2-P, V2-F et non comptée comme validation V2 |
| **Mutations** | Appliquées seulement au Lot 1 (copie isolée) ; Lot 2 par raisonnement ; `pf9` non muté |
| **Performance, durées Miri** | Non mesurées |
| **Workspace reconstitué** | Versions différentes du dépôt réel (cargo 1.97, `toml` 1.2.2) ; résultats **[W]** non transposables au dépôt |

### 7.7 Réserves résiduelles acceptées par l'utilisateur

Arbitrages finaux de l'utilisateur (21 novembre 2026) : **C1, C2, L2 et L3 acceptés** (doctests de visibilité non discriminants, atténués par la compilation des consommateurs réels ; `pk9` en `is_err()` ; mutation d'allocation détectable par revue seulement) ; **sentinelles Shell validées** ; **VQ-8 reporté à F3b** ; absence de validation 32 bits conservée ; authenticité cryptographique non démontrée ; absence de baseline pré-V2 conservée comme limite de preuve. Aucun nouveau test, audit ou contrôle n'est demandé.

---

## 8. Conclusion et état final

### 8.1 Unités validées

Les trois lots concluent à l'acceptabilité des unités **V2-L, V2-P, V2-F**, sans anomalie fonctionnelle démontrée. Le Lot 3 est favorable à la clôture technique de V2-Z, sous les réserves ci-dessous. Les réserves ayant été arbitrées par l'utilisateur (§ 7.7, § 8.3), V2-Z peut être formellement clôturée sur cette base (porte P-E) ; la décision de clôture appartient à l'utilisateur.

### 8.2 Invariants démontrés, avec leur niveau de preuve

- **Compatibilité v1** : ordre de `Manifest::parse`, niveaux 1 à 4, tout-ou-rien, erreurs historiques, formats d'artefacts : couverture directe par les tests historiques d'Assets, exécutés sur reconstitution **[W]** et confirmés verts par l'utilisateur sur le dépôt réel **[C]**. Préservation v1 : étayée par l'inspection du code, la conservation des tests historiques et leur exécution réelle ; **aucun diff exhaustif avec une baseline antérieure n'a été établi** ; elle n'est donc pas démontrée par un diff (§ 7.5).
- **Lecteur VMAN v2 et enveloppe VPKG** : ordre champ par champ, classification A-1/A-2, `n'` contre `n`, tout-ou-rien, absence d'allocation, emprunts : directe pour l'essentiel ; inatteignabilité du `unreachable!` d'`artifact_bytes` par lecture **[S]**.
- **Producteur Forge** : contrat O-1, layout pur sans allocation, validation avant allocation, P1 à P7, opacité de `CompiledPackage` : lecture **[S]** et tests présents ; exécution de `pf9` sur reconstitution **[W]** ; mutations Forge par raisonnement **[D]**.
- **Parcours complet** `compile_package → load_package` sur lot hétérogène (cinq formats, longueurs et offsets distincts, `artifact_index` / `slot_index` distincts sur les deux scripts, Map → Tileset, équivalence avec `load`) : `pf9` **[W]**, exécuté et validé sur le dépôt réel **[C]**.
- **Intégration** : graphe de dépendances sans cycle, Assets sans dépendance, Forge sans consommateur normal **[R]** ; Script et Shell sans symbole v2 **[R][S]**.

### 8.3 Dettes documentaires

| Réf. | Dette | Statut final |
|---|---|---|
| **E1** | Lien rustdoc ambigu `crate::load` dans la doc de `load_package` (correction triviale : `load()` ou `fn@load`) | Acceptée ; avertissement observé sur reconstitution, non confirmé sur le dépôt réel |
| **O1** | Doc de `ManifestError::Truncated` et `TrailingBytes` sur `10 + 16 × n` | Acceptée ; formulation historique conservée |
| **O4** | Coquille « une section d'artefact est donc jamais vide » (`package.rs`) | Acceptée |

Ces trois points n'ont aucun effet sur le comportement, le build, les tests ni clippy. L'utilisateur les accepte comme dette documentaire mineure (21 novembre 2026) : aucune correction et aucun nouveau contrôle `cargo doc` ne sont demandés, et ils ne bloquent pas la clôture de V2-Z.

### 8.4 Éléments reportés ou hors périmètre

- **VQ-8** et l'adoption applicative de VPKG par le Shell : F3b.
- Correspondance q ↔ artefacts : F2.
- Cibles 32 bits, fichiers compressés ou encodés, authenticité cryptographique : limites documentées, non traitées.
- `map.rs::encode_map` (`unwrap_or(u32::MAX)`) et conversions héritées de v1 (O2) : hors périmètre de V2, consignées.

### 8.5 Points non démontrés et anomalies

**Anomalie démontrée : une seule, non fonctionnelle (E1, documentation).** Tout le reste ci-dessus (couverture indirecte, doctests non discriminants, 32 bits, baseline absente, recherche `VPKG`, authenticité, VQ-8) est une **limite de preuve**, non un défaut fonctionnel.

### 8.6 Portée de la conclusion

Ce dossier atteste que, dans les sources fournies et avec les niveaux de preuve indiqués, aucune anomalie fonctionnelle n'a été relevée dans le chantier VMAN v2 et que les contrats de l'annexe V2-C sont respectés à la lecture et aux tests exécutés. Il n'atteste pas :
- l'absence de modification des fichiers v1 par comparaison à une baseline : aucun diff exhaustif n'a été établi dans V2-Z ;
- l'examen, par Claude, des sorties brutes des contrôles réels : ils reposent sur la confirmation de l'utilisateur **[C]** ;
- un comportement sur cible 32 bits ;
- une propriété d'authenticité du paquet.
