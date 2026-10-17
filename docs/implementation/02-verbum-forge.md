# Plan de construction — étape 2 : Forge minimale (r8)

## Statut
Document de planification interne, non normatif, révisable sans ADR (même statut que `ROADMAP.md`). Détaille l'étape 2 en micro-phases. N'introduit aucun invariant : chaque phase cite l'invariant du corpus qui la justifie. **Aucun code, aucune modification du dépôt, aucune commande de validation exécutée.** Révision r8 du 17 octobre 2026.

**État de validation : plan non validé pour implémentation.** Aucune implémentation et aucune commande de validation avant l'approbation explicite du plan complet (tranches A et B). Après r8, l'utilisateur effectuera un contrôle de cohérence des trois corrections (non un nouvel audit architectural) ; l'approbation explicite du plan complet précède ensuite la phase 1.

Historique : r1 (plan initial) → r2 (Q-C, Q-B sur les textes) → r3 (Q-B interprétation 2) → r4 (Q-B interprétation A ; tranches A et B) → r5 (Q-I à Q-N ; phases 4 à 6 détaillées ; clôture corrigée) → r6 (confirmations ; deux niveaux de contrôle ; point de test privé pour la collision) → **r7 : précisions d'implémentation — référence FNV-1a vérifiée à la source (constantes et algorithme) ; stratégie de chargement tout-ou-rien sans allocation par double décodage, avec contrat du dépôt ; `Manifest<'a>` emprunte les octets ; exemple Unicode concret pour F-5.** Aucun arbitrage architectural (Q-I à Q-N) n'est rouvert ; décisions et structure de r6 conservées. → **r8 : trois corrections seulement — (1) références FNV-1a du RFC 9923 distinguées par section (§2, §5, §8.2.2, §8.3), divergence de numérotation et réserve sur les vecteurs supprimées, vecteur `""` intégré à F-1 ; (2) seconde lecture impossible traitée par `unreachable!` actif en production, statut `[H]` et analogie avec `World::apply` retirés ; (3) constructeur public `SlotIndex::new(u32)` ajouté à l'API.** Les autres décisions sont inchangées.

Légende : **[D]** décision architecturale (source normative citée) ; **[L]** choix local d'implémentation ; **[H]** hypothèse à confirmer. Niveaux de preuve : **[Lu]**, **[Déduit]**, **[Non vérifié]**.

## 0. Sources et limites

**[Lu]** `ROADMAP.md`, `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, `README.md` d'architecture, ADR-000, ADR-001, ADR-005, ADR-A01, ADR-R01, ADR-R02, ADR-R03, ADR-S01, `00-verbum-ecs.md`, `HANDOFF-stage-1.md`, `HANDOFF-stage-2-forge.md`, `tree.md`, `Cargo.toml` et `Cargo.lock` du workspace, `crates/ecs/Cargo.toml`, `lib.rs`, `resource.rs`, `world.rs`, `component_set.rs`.

**RFC 9923 (« The FNV Non-Cryptographic Hash Algorithm », Informational, février 2026)** : §2 pseudocode de FNV-1a ; §5 constantes FNV ; §8.2.2 code de référence FNV64 ; §8.3 code de test, qui contient les entrées `""`, `"a"` et `"foobar"` et leurs valeurs attendues. Les §2, §5 et §8.2.2 sont lus par moi ; le tableau de test du §8.3 est vérifié par l'utilisateur.

**Non lus, sans incidence sur ce plan** : ADR-002, 003, 004, 006 ; `entity.rs`, `storage.rs`, `values.rs`, `command.rs`, `query.rs`, `fixtures.rs` ; `01-verbum-scheduler*.md`. **Volontairement non lus** : spécifications LPC. **Non consultées par moi** : les sessions antérieures invoquées par l'utilisateur ; leurs conclusions sont prises telles que formulées.

**Aucune commande n'a été exécutée** ; aucun résultat de sandbox n'y figure.

## 1. Objectifs et critères de sortie

### 1.1 Contrat de la Roadmap [Lu]
- Contenu : « un pipeline complet mais étroit — `sprite.toml → Forge → sprite.bin → lecture binaire → Ressource`. Pas de validation exhaustive, pas de second type d'asset. Le chargement runtime peut être simulé par un appel direct en test, sans Shell réel. »
- Dépendance : « Étape 0 seulement » ; ni Scheduler ni Shell.
- Test d'invariant : « Le runtime ne parse jamais le `.toml` — seule la Forge le fait ; le runtime ne lit qu'un artefact binaire déjà validé (`00-principes.md`, invariant 1). »
- Artefact : « premier binaire produit par la Forge, relu et casté en Ressource ».
- Principe directeur : un jalon n'est validé que si **aucun invariant architectural n'a dû être modifié**.
- Étape 5 : « extension de la Forge et du chargement runtime établis en étape 2, sans changement de mécanisme ».

### 1.2 Lecture retenue pour ADR-A01 (arbitrage du 17 octobre 2026)
Le manifeste et l'identité opaque font partie du contrat d'ADR-A01 pour les ressources d'assets, **même pour un seul asset**. L'invariant n'est pas requalifié en exigence conditionnelle ; il est distingué de son calendrier de mise en œuvre. L'identité typée d'un `ResourceSlot<Sprite>` n'est pas l'identifiant opaque produit par la Forge.

Le texte de la Roadmap pour l'étape 2 ne mentionne ni manifeste ni identifiant. Le plan l'étend sans modifier la Roadmap (non normative) pour ne pas renvoyer le manifeste à l'étape 5. **Écart signalé** : le contenu de l'étape 2 dans ce plan est plus large que celui de la Roadmap ; la mise en cohérence éventuelle du texte de la Roadmap relève de l'utilisateur.

### 1.3 Structure : deux tranches, une clôture

| | Tranche A — tranche verticale initiale | Tranche B — identité opaque et manifeste |
|---|---|---|
| Contenu | `sprite.toml → Forge → sprite.bin → lecteur binaire → ResourceSlot<Sprite>`, sans manifeste (phases 1 à 3) | Manifeste, `AssetId` opaque, chaîne de résolution jusqu'aux slots statiques (phases 4 à 6) |
| Statut | **Jalon intermédiaire** : ne valide qu'une partie du contrat d'ADR-A01 | Mécanisme arbitré (§8.1) |
| Démontre | Compilation de la source humaine en artefact binaire ; lecture sans parsing de TOML ; intégration comme Ressource | Existence du manifeste, identité opaque produite par la Forge, résolution effective vers le bon artefact et le bon slot avant le hot path |

**Critères de la tranche A (jalon intermédiaire)** : (1) test d'invariant de la Roadmap passé (phases 1 à 3) ; (2) P1.6 vert ; (3) aucun ADR, document normatif ni `verbum-ecs` modifié ; (4) consigné, au journal et dans tout message de livraison, comme « **jalon intermédiaire : ne valide pas le contrat complet d'ADR-A01 (manifeste, identité opaque)** ». Il ne clôt pas l'étape 2.

**Critères de clôture de l'étape 2** : critères de la tranche A, **et la preuve par test** (non la seule énumération) de chacun des points suivants :

| # | Invariant démontré | Test(s) de preuve |
|---|---|---|
| K1 | Le manifeste existe, est produit par la Forge, et ne contient aucun nom ni chemin (ADR-A01 pt 3) | F-2, F-8, M-1, M-2 |
| K2 | L'identifiant d'asset est opaque au runtime : non constructible, non lisible numériquement, non affiché hors du lecteur (ADR-A01 pt 3 ; ADR-000, types opaques) | M-6 |
| K3 | L'identifiant est produit par la Forge, de façon déterministe, indépendante du contenu et de la position (ADR-A01 pt 3 ; Q-K) ; la fonction de hash de production est FNV-1a 64 bits, non substituable par l'API publique | F-1, F-2, F-3, F-5, F-9 |
| K4 | La résolution `AssetId` → artefact → slot → `ResourceSlot<Sprite>` ciblé est effective, avant le hot path, et aboutit au bon contenu dans le bon slot (ADR-A01 pt 3, pt 4) | L-1, L-2, T6.1, T6.2 |
| K5 | Identité, ordre des artefacts et ordre des slots sont trois notions indépendantes ; une permutation de l'ordre de compilation ne change pas l'association identité → contenu | F-3, T6.3 |
| K6 | Tous les contrôles (manifeste, liaisons, artefacts) s'achèvent **avant** le premier dépôt dans un slot ; le chargement est tout-ou-rien, y compris avec un artefact mal formé, **sans allocation** | M-3, M-4, M-5, M-7, L-3, L-5, T6.4 |
| K7 | La capacité est fixée par la configuration statique des slots ; tout dépassement est rejeté ; une capacité supérieure au nombre d'assets est acceptée (ADR-001 pt 5 ; ADR-A01 pt 3) | M-7, L-4, F-4, T6.4, T6.6 |
| K8 | Aucun invariant d'une source normative, ni `verbum-ecs`, ni `World`, ni la façade, ni ADR-A01, ni la Roadmap, n'a dû être modifié | Contrôles manuels (§6, phase 6) |

**Complément, non substitut** : la clôture énumère aussi ce qui reste non démontré du contrat d'ADR-A01 (§8.4). Cette énumération ne remplace aucune des preuves K1 à K8. Aucun renvoi du manifeste à l'étape 5.

## 2. Corrections et précisions apportées au handoff

| Handoff | Constat après lecture des sources |
|---|---|
| §3.2 : « [Non vérifié] si ADR-A01 … en font un invariant » (index/manifeste) | ADR-A01 pt 3 **[Lu]** : la Forge « produit également le manifeste qui permet au runtime d'adresser chaque Ressource via un identifiant opaque, résolu avant le hot path ». Arbitrage du 17 octobre : manifeste et identité opaque dans le contrat, y compris pour un seul asset (§1.2). |
| §3.1 / Q1 : format source, binaire, identité | Ni ADR-A01 ni `00-principes.md` n'imposent TOML, YAML ou PNG. Les noms `AssetRegistry` et `FrameSequence` n'apparaissent dans aucune source lue. |
| Q3 : décodage d'image | **[Déduit]** À terme côté Forge ; non requis à l'étape 2 (Q-A). |
| Q6 : contraintes du binaire « casté » | ADR-000 pt 3 et pt 4 (table) ; reprise en §8.2. |
| Q10 : capacités fixées par la Forge ? | Capacité fixée par la configuration statique des slots (Q-M) ; mécanisme de test admis par ADR-001 pt 5. |
| §2 et §10 : chemins `crates/verbum-ecs`, `crates/verbum-schedule` | `tree.md` (fait foi) : `crates/ecs`, `crates/schedule`, `crates/verbum`. |

## 3. Invariants applicables

| # | Invariant | Source | Application à l'étape 2 |
|---|---|---|---|
| I1 | Seule la Forge lit et parse les formats humains ; le runtime consomme des données précompilées | `00-principes.md` inv. 1 ; ADR-A01 pt 2 ; Roadmap | Lecteurs sur octets ; aucun parseur TOML dans l'arbre de `verbum-assets` |
| I2 | Aucune validation de cohérence métier au runtime ; la Forge valide | `00-principes.md` inv. 1 et 5 | Validation sémantique du contenu côté Forge ; côté runtime, conformité binaire et cohérence des liaisons (I5, §5.3) |
| I3 | Artefact directement interprétable, sans étape de parsing ; layout explicite ; la règle porte sur les formats humains | ADR-A01 pt 3 ; ADR-000 pt 3 ; `00-principes.md` inv. 5 | Layouts fixes ; endianness explicite |
| I4 | Aucune chaîne ni chemin ne franchit la frontière de compilation | ADR-A01 pt 3 | Artefacts et manifeste sans nom ni chemin (la signature binaire `VMAN` est une constante de format, non un nom d'asset) |
| I5 | Layout non conforme rejeté au chargement ; endianness, version, valeurs invalides vérifiées au chargement ; pas une validation métier | ADR-000 pt 3 et pt 4 (table) | Contrôles du manifeste et du chargement (§5.3) |
| I6 | Un asset devient, après la Forge, une Ressource ordinaire | ADR-A01 pt 1 et 5 ; ADR-005 | Un `ResourceSlot<Sprite>` par asset (F1) |
| I7 | Clé d'une Ressource = clé unique ; forme concrète = implémentation (aujourd'hui : position statique) | ADR-005 pt 1 ; `02-concepts.md` ; `resource.rs` | Clé ADR-005 = champ statique de `R` ; **distincte** de l'`AssetId` |
| I8 | Seule l'infrastructure de chargement fait l'accès disque | ADR-A01 pt 2 | Lecteurs sur `&[u8]` ; accès fichier simulé par le test |
| I9 | La Forge est extérieure au runtime | `02-concepts.md` ; ADR-000 pt 2 | Aucun crate runtime ne dépend de la Forge |
| I10 | Noyau `no_std`, sans `alloc` ; capacités fixées, pas de croissance dynamique | ADR-000 pt 1 et 4 ; ADR-001 pt 5 | `verbum-ecs` inchangé ; `verbum-assets` `no_std` sans `alloc` ; chargement sans allocation (§5.4) |
| I11 | Un concept n'est codé que s'il est porté par un invariant ou rendu nécessaire par un test | `00-verbum-ecs.md`, règle gouvernante | Exclusions du §10 |
| I12 | Adressage runtime par identifiant opaque résolu avant le hot path ; forme opaque ; aucune chaîne ni chemin ne franchit la frontière ; ensemble des assets connu de la Forge ; manifeste produit par la Forge | ADR-A01 pt 3, pt 4, classification | Tranche B (K1 à K7) |
| I13 | Capacité fixée avant simulation ; dépassement = erreur de configuration | ADR-001 pt 5 ; ADR-A01 pt 3 | Q-M ; K7 |

Non applicables à l'étape 2 : ADR-A01 pt 4 (absence de propriétaire d'écriture au runtime, cf. R6), pt 5 (streaming, déchargement), pt 6 (rejets).

## 4. Pipeline, frontières de crates et chaîne de résolution

```
Tranche A :
sprite.toml ──(Forge, std)──► sprite.bin ──(octets)──► lecteur (no_std) ──► Sprite ──► ResourceSlot<Sprite>

Tranche B :
[ (clé logique, TOML, slot_index) ... ] en ordre explicite
        ──(Forge, compilation en lot, std)──► manifeste VMAN + artefacts (même ordre)
        ──(chargement, avant le hot path)──►
              1. parse du manifeste (octets empruntés, seuls)
              2. contrôles de liaison (nombre d'artefacts, indices, unicité, capacité)
              3. décodage de chaque artefact, sans dépôt et sans conservation du résultat
              4. second parcours : redécodage de chaque artefact et dépôt
                 ← seulement si 1, 2 et 3 ont réussi
```

| Crate | Chemin | Environnement | Contenu | Dépend de |
|---|---|---|---|---|
| `verbum-assets` | `crates/assets` | `no_std`, sans `alloc`, sans dépendance | Tranche A : layout de l'artefact, `Sprite`, lecture. Tranche B : layout du manifeste, `AssetId`, indices typés, `Manifest<'a>`, chargement | rien |
| `verbum-forge` | `crates/forge` | `std` | Tranche A : compilation d'une source. Tranche B : FNV-1a 64, compilation en lot, production du manifeste | `verbum-assets`, `toml` ; en `dev-dependencies` : `verbum-ecs` |
| `verbum-ecs` | inchangé | | Aucune modification | |

### 4.1 Les trois notions, distinctes et non substituables

| Notion | Type | Rôle | Produite par | Où elle est utilisée |
|---|---|---|---|---|
| `AssetId` | opaque (valeur `u64` non exposée) | Identité logique de l'asset | Forge : FNV-1a 64 de la clé logique | Clé de recherche dans le manifeste, au chargement seulement |
| `artifact_index` | `ArtifactIndex` (enveloppe de `u32`) | Position de l'artefact dans la collection ordonnée fournie au chargement | Position dans l'entrée de la Forge | Sélection des octets de l'artefact |
| `slot_index` | `SlotIndex` (enveloppe de `u32`) | Ordinal dans la configuration statique des slots | Métadonnée de compilation fournie à la Forge | Sélection du champ `ResourceSlot<Sprite>` via la liaison statique |

`ArtifactIndex` et `SlotIndex` sont deux types distincts malgré une représentation binaire identique (arbitrage confirmé) ; leur valeur est lisible par un accesseur, ces indices n'étant pas opaques. **`SlotIndex` dispose en outre d'un constructeur public simple, `SlotIndex::new(u32)` (nom provisoire)**, car `verbum-forge` et l'application doivent pouvoir en fournir. Ce constructeur ne vérifie rien : il ne connaît pas la capacité. Les contrôles `slot_index < slot_count` et d'unicité des slots restent assurés par la Forge (F-4) et par le chargement (M-7). Les tests à ordres dissociés (Q-N) détectent de plus une confusion à l'exécution.

### 4.2 Chaîne de résolution complète

`AssetId attendu → entrée du manifeste → artifact_index → collection ordonnée d'artefacts → octets de l'artefact → Sprite décodé → slot_index → liaison statique → ResourceSlot<Sprite> ciblé`

- Les premières flèches (jusqu'au `Sprite` décodé et à l'envoi vers le dépôt) sont réalisées par `verbum-assets`, la recherche de l'entrée par identifiant ayant lieu pendant le chargement ; **aucune recherche sur le hot path**.
- **La liaison statique `slot_index → champ` ne peut pas vivre dans `verbum-assets`** : le crate ne dépend pas de `verbum-ecs` et ne peut pas nommer `ResourceSlot`. Elle est fournie par l'application (ici le test), sous la forme d'un **tableau statique d'accesseurs**, un par champ `ResourceSlot<Sprite>` de `R` ; `slot_count` est la longueur de ce tableau. **[Déduit]** conséquence : les deux dernières flèches sont du code de l'application, non de `verbum-assets`, et le test d'acceptation les représente explicitement.
- Le chargeur reçoit une fonction de dépôt `(SlotIndex, Sprite)` ; l'application y applique la liaison (contrat en §5.4).
- Une fois le contenu déposé, l'accès à la Ressource se fait par le champ typé ; aucun `AssetId` n'est consulté sur le hot path.

### 4.3 Source de vérité de la capacité
Le tableau statique de liaison est l'unique source de vérité de la capacité : `slot_count = tableau.len()`. Cette même valeur (a) est passée à la compilation en lot de la Forge, qui rejette tout `slot_index` invalide, et (b) est passée au chargement, qui rejette tout manifeste dépassant la capacité ou portant un indice invalide. Aucune constante indépendante n'est recopiée. **La capacité peut excéder le nombre d'assets effectivement chargés** : une capacité disponible n'implique pas que chaque slot soit utilisé.

## 5. Choix concrets et statuts

| Élément | Statut | Détail |
|---|---|---|
| Source `sprite.toml` : deux champs entiers (largeur, hauteur) | [L] (Q-A) | Aucune chaîne ; aucune durée (R4) ; aucun décodage d'image |
| Layout de l'artefact : en-tête (identité du format sur 4 octets, version `u16`, longueur de charge utile `u32`), charge utile (largeur `u16`, hauteur `u16`), 14 octets | [L], provisoire | Layout de **ce** descripteur ; non gabarit des futurs assets |
| Little-endian explicite ; lecture champ par champ (`from_le_bytes`) pour l'artefact **et** le manifeste | [L], confirmé (Q-C) | Ne préjuge pas des charges variables de l'étape 5 ; l'identifiant `u64` en little-endian est cohérent avec RFC 9923 §2.3 (stockage persistant) |
| Assertion à la compilation de la cohérence des constantes de layout | [L] | Équivalent des assertions de build d'ADR-000 pt 4 |
| Parseur : crate `toml` | [L], tracé ici, sans ADR dédiée (Q-D) | MSRV 1.85 à vérifier (R1) |
| Identifiant : FNV-1a 64 bits de la clé logique | [L] (Q-K) | Règle en §5.1 |
| Manifeste binaire distinct `VMAN` | [L] (Q-L) | Layout en §5.2 ; non gabarit imposé aux futurs types |
| Granularité F1 : un `ResourceSlot<Sprite>` par asset | [L] au regard d'ADR-005 pt 1 ; lecture littérale d'ADR-A01 (« chaque Ressource ») | Q-I |
| Capacité = cardinalité de la configuration statique des slots | [L] (Q-M) | §4.3 |
| Forge = bibliothèque ; ni CLI, ni `build.rs`, ni génération de code | [L] (Q-G, Q-J option C) | Nommage symbolique non démontré (§8.4) |
| `ArtifactIndex`, `SlotIndex` en types distincts ; `SlotIndex::new(u32)` public, avec accesseur | [L], confirmé ; constructeur ajouté en r8 | §4.1 ; le constructeur ne vérifie pas la capacité |
| `AssetId` : `Copy`, `Clone`, `PartialEq`, `Eq` ; `Debug` opaque (`AssetId(<opaque>)`) ; ni `Display`, ni accesseur numérique, ni `From<u64>`, ni constructeur public | [L], confirmé | Seuls le lecteur du manifeste et le calcul de la Forge manipulent la valeur |
| **`Manifest<'a>` emprunte les octets analysés ; ne les possède pas** | [L], précisé en r7 | `Manifest<'a>::parse(&'a [u8]) -> Result<Manifest<'a>, ManifestError>` ; itération et recherche sans allocation ; `ManifestEntry` est une valeur `Copy` sans référence |
| **Chargement tout-ou-rien par double décodage, sans allocation** | [L], arbitré en r7 | §5.4 |
| Entrée du manifeste ordonnée comme l'entrée de la Forge ; la résolution ne s'appuie pas sur cet ordre | [L] | Recherche par identifiant |
| Un manifeste vide (zéro entrée, longueur 10) est valide et correspond à une collection vide d'artefacts ; aucun dépôt | [L], confirmé | |
| Le nombre d'entrées égale le nombre d'artefacts fournis | [L], confirmé | Bijection avec des indices uniques et valides |
| Calculateur de hash : point de test **privé** (§6, phase 5) | [L] | Jamais paramètre de l'API publique |

### 5.1 Règle de calcul de l'identifiant (déterministe, consignée)
- **Clé logique** : chaîne fournie à la Forge **séparément** de la source TOML, comme métadonnée de compilation ; jamais écrite en clair dans le manifeste ni dans l'artefact.
- **Encodage** : octets UTF-8 exacts de la chaîne fournie ; sensible à la casse ; aucune normalisation implicite.
- **Algorithme [Lu], RFC 9923 §2** : `hash = offset_basis` ; pour chaque octet : `hash = hash XOR octet`, puis `hash = hash * FNV_Prime mod 2^64`. L'ordre XOR puis multiplication est celui de FNV-1a.
- **Constantes 64 bits [Lu], RFC 9923 §5 (tables 1 et 2), confirmées par le code de référence FNV64 du §8.2.2** : `offset_basis` = `0xCBF29CE484222325` (14 695 981 039 346 656 037) ; `FNV_Prime` = `0x00000100000001B3` (1 099 511 628 211, soit 2⁴⁰ + 2⁸ + 0xB3).
- **Vecteurs de référence, confirmés par le tableau de test FNV64 du RFC 9923 §8.3 (vérifié par l'utilisateur)** : `""` → `0xcbf29ce484222325` ; `"a"` (octet `0x61`) → `0xaf63dc4c8601ec8c` ; `"foobar"` → `0x85944171f73967e8`.
- **Vecteur de la chaîne vide** : `""` → `0xcbf29ce484222325` fait partie de F-1 ; il teste directement la fonction FNV-1a pure pour une entrée vide. Il ne modifie pas la règle distincte selon laquelle `compile_assets` refuse les clés logiques vides (voir « Validations de la Forge » ci-dessous). RFC 9923 §2.2 indique par ailleurs que la chaîne nulle donne l'`offset_basis`.
- **Stabilité** : l'identifiant ne dépend ni du contenu de l'asset, ni de sa position dans la collection, ni de son `slot_index`.
- **Validations de la Forge** : clé vide rejetée ; clés dupliquées rejetées ; toute collision d'identifiants entre clés distinctes rejetée. Clé dupliquée et collision entre deux clés distinctes sont deux cas distincts, testés séparément, la clé dupliquée étant détectée en premier.
- **Portée du contrôle de collision** : tous les assets pouvant coexister dans un même manifeste, c'est-à-dire l'ensemble fourni à un même appel de compilation en lot. **Le plan ne prétend pas traiter les collisions entre ensembles compilés séparément** puis réunis.
- FNV-1a n'est pas retenu pour une propriété cryptographique (le RFC en déconseille l'usage lorsque la résistance aux attaques est requise, §1.2) ; la détection des collisions est la garantie pertinente.

### 5.2 Layout du manifeste `VMAN` (choix local, non gabarit)

| Champ | Taille |
|---|---:|
| Signature `VMAN` | 4 octets |
| Version, `u16` little-endian | 2 octets |
| Nombre d'entrées `n`, `u32` little-endian | 4 octets |
| Par entrée : `AssetId`, `u64` little-endian | 8 octets |
| Par entrée : `artifact_index`, `u32` little-endian | 4 octets |
| Par entrée : `slot_index`, `u32` little-endian | 4 octets |

En-tête : 10 octets ; entrée : 16 octets ; longueur totale attendue exactement `10 + 16 × n`. Le calcul est effectué en `u64` avec opérations vérifiées (multiplication, addition) avant toute lecture des entrées ; la conversion en `usize` est vérifiée. La signature `VMAN` est une constante binaire du format ; le fichier n'est donc pas décrit comme composé exclusivement de champs numériques, mais il ne contient aucun nom lisible d'asset ni chemin.

**Contrat de `artifact_index`** : les entrées de compilation sont fournies à la Forge dans un ordre explicite ; les artefacts sont rendus dans ce même ordre ; `artifact_index` est l'ordinal base zéro de l'artefact dans cette collection. Le test d'acceptation conserve lui-même la liste ordonnée des fichiers correspondant aux positions retournées ; ni le manifeste ni le lecteur ne scannent un répertoire ; l'ordre du système de fichiers n'est jamais une source de vérité. Modifier l'ordre d'entrée modifie les `artifact_index`, jamais l'identité logique.

### 5.3 Séparation des contrôles : deux niveaux, déterminés par l'information disponible

| Niveau | Opération | Information disponible | Contrôles |
|---|---|---|---|
| 1 | `Manifest::parse(octets)` | Les seuls octets du manifeste | Signature ; version ; troncature ; longueur égale à `10 + 16 n` (surplus compris) ; décompte ; intégrité des entrées ; **unicité des identifiants** ; absence d'ambiguïté structurelle |
| 2 | Contrôles de liaison du chargement | Collection ordonnée d'artefacts et configuration des slots (`slot_count`) | **Nombre d'entrées égal au nombre d'artefacts** ; `n ≤ slot_count` (capacité) ; `artifact_index < nombre d'artefacts` ; `slot_index < slot_count` ; **unicité des `artifact_index`** ; **unicité des `slot_index`** |
| 3 | Premier décodage de chaque artefact | Octets de chaque artefact | Lecture de chaque artefact par le lecteur de la tranche A, sans dépôt et sans conserver le résultat |
| 4 | Second parcours et dépôt | — | Redécodage et dépôt, seulement si 1, 2 et 3 ont réussi (§5.4) |

Règles :
- **Tous les contrôles des niveaux 1 à 3 s'achèvent avant le premier dépôt dans un `ResourceSlot`.** Le chargement est tout-ou-rien, y compris lorsqu'un artefact est mal formé.
- Classification : ces contrôles relèvent de la **conformité binaire et de la cohérence des liaisons**, non d'une validation de cohérence métier (ADR-000 pt 3 : « vérification de conformité de format »). La validation sémantique du contenu d'un asset reste à la Forge (`00-principes.md` inv. 1) ; P1.6 en garde la frontière.
- Placement : l'unicité des liaisons est au niveau 2 par arbitrage, bien qu'elle soit détectable à partir des seuls octets du manifeste ; `find` n'en dépend pas.

### 5.4 Chargement tout-ou-rien sans allocation (stratégie arbitrée en r7)

**Problème** : valider tous les artefacts avant le premier dépôt exigerait, naïvement, de conserver les `Sprite` décodés (collection dynamique ou tampon de staging), ce qui est exclu pour un crate `no_std` sans `alloc` dont le nombre d'assets n'est pas une constante du crate.

**Stratégie retenue [L]** :
1. Analyser le manifeste (`Manifest::parse`) et vérifier toutes les liaisons (niveaux 1 et 2).
2. Parcourir tous les artefacts référencés et les décoder pour valider l'ensemble ; **chaque résultat est abandonné aussitôt** (niveau 3).
3. Seulement si tout a réussi, parcourir de nouveau les liaisons, **redécoder** chaque artefact à partir des mêmes octets et déposer le `Sprite` obtenu dans le slot ciblé (niveau 4).

**Justification [Déduit]** : le lecteur est une fonction pure et déterministe de ses octets ; les octets sont empruntés de manière immuable pendant l'appel (`&[u8]`, aucune mutation possible en Rust sûr). La seconde lecture d'un artefact ne peut donc pas échouer si la première lecture des mêmes octets a réussi. La stratégie n'alloue pas, n'a besoin d'aucun tampon de staging et ne conserve aucune collection de `Sprite`. Coût : chaque artefact est décodé deux fois (lecture de 14 octets) ; acceptable avant le hot path.

**Cas impossible de la seconde lecture** : une erreur lors du second décodage est une **violation interne impossible** dans les conditions du contrat (octets immuables pendant l'appel, lecteur déterministe, première lecture réussie) ; ce n'est **pas** une `LoadError` récupérable. Elle est traitée par `unreachable!` (ou une protection équivalente), **active également en version de production** ; `debug_assert!` seul n'est pas retenu. La panique correspondante est hors de la garantie tout-ou-rien (voir ci-dessous).

**Contrat du dépôt** :
- La fonction de dépôt a pour signature sémantique `(SlotIndex, Sprite)` **sans valeur de retour** : elle ne renvoie **aucune erreur récupérable**.
- Les slots ciblés sont **vides à l'entrée du chargement** : précondition de l'appelant, que `verbum-assets` ne peut pas vérifier (il ne nomme pas `ResourceSlot`). Un test d'acceptation l'établit avant le chargement (`get()` vaut `None`). Si un slot n'était pas vide, `ResourceSlot::insert` le remplacerait ; le contrat l'exclut.
- Le dépôt est appelé **exactement une fois par entrée du manifeste**, jamais deux fois pour un même slot (les `slot_index` sont uniques).

**Portée de la garantie tout-ou-rien** : elle s'applique aux **erreurs retournées par le chargement avant le début des dépôts** — manifeste, liaisons ou artefacts. Elle ne prétend **pas** garantir la récupération après une **panique**, qu'il s'agisse de la violation interne impossible ci-dessus ou d'une panique arbitraire de l'application (par exemple dans la fonction de dépôt, entre deux dépôts).

**Vigilance, hors étape 2 [Déduit]** : la prémisse « octets immuables » vaut pour des tampons mémoire ordinaires. Elle ne vaudrait plus pour un fichier projeté en mémoire modifiable par un tiers entre les deux parcours ; à reconsidérer si un accès par `mmap` est ajouté (ADR-A01 le mentionne comme mécanisme du chargement réel).

## 6. Micro-phases

Méthode : invariant cité → test qui l'exprime (rouge confirmé avant l'API) → API minimale → implémentation. Un commit par invariant démontré (messages à votre charge). Contrôles de chaque phase : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check` ; Miri et MSRV 1.85 exécutés par vos soins. Les erreurs sont des énumérations `Copy` côté `verbum-assets` (pas d'allocation). Les noms ci-dessous sont provisoires [L].

### Tranche A — jalon intermédiaire (ne valide pas le contrat complet d'ADR-A01)

#### Phase 1 — Format de l'artefact et lecture runtime (`verbum-assets`)
**Invariants** : I3, I4, I5, I8, I10 (et I2 pour P1.6).
**Fichiers** : création de `crates/assets/Cargo.toml`, `src/lib.rs`, `src/sprite.rs` ; modification de `Cargo.toml` (membres).
**Tests** (octets littéraux écrits à la main, valeurs asymétriques, par exemple largeur `0x0102`, hauteur `0x0304`) : P1.1 lecture de l'artefact littéral ; P1.2 toute troncature rejetée ; P1.3 identité de format altérée rejetée, y compris le texte d'un `sprite.toml` ; P1.4 version inconnue rejetée ; P1.5 longueur de charge utile erronée ou surplus rejetés ; **P1.6** largeur nulle structurellement conforme **acceptée**.
**API minimale** : `Sprite` (donnée pure, `Copy`) ; `Sprite::read(&[u8]) -> Result<Sprite, ReadError>` ; constantes de layout publiques ; `ReadError`.
**Critère de sortie** : P1.1 à P1.6 verts ; `#![no_std]` sans `extern crate alloc` ; `[dependencies]` vide ; Miri vert (vous).

#### Phase 2 — Forge : source → artefact (`verbum-forge`)
**Invariants** : I1, I2, I4, I9, I11.
**Fichiers** : création de `crates/forge/Cargo.toml`, `src/lib.rs`, `src/sprite.rs` ; modification de `Cargo.toml` (membre, `toml`) ; `Cargo.lock` par Cargo seulement.
**Tests** : P2.1 un `sprite.toml` valide produit exactement les octets littéraux attendus (témoin indépendant de P1.1) ; P2.2 round-trip Forge → lecteur ; P2.3 validation minimale refusée sans artefact produit (champ manquant, valeur nulle, hors `u16`, type erroné, champ inconnu, TOML invalide).
**API minimale** : `compile_sprite(&str) -> Result<Vec<u8>, ForgeError>` ; `ForgeError`.
**Critère de sortie** : P2.1 à P2.3 verts ; `cargo tree -i toml --workspace` ne liste que `verbum-forge` (contrôle manuel) ; MSRV vert (vous).

#### Phase 3 — Acceptation de la tranche A (`crates/forge/tests/pipeline.rs`)
**Invariants** : I1, I6, I7, I8, critère de la Roadmap.
**Fichiers** : création de `crates/forge/tests/pipeline.rs` ; `verbum-ecs` en `dev-dependencies`.
**Tests** : P3.1 chaîne complète sur disque (`sprite.toml` → Forge → `sprite.bin` → **suppression de la source** → lecture des octets → `Sprite::read` → insertion dans un `ResourceSlot<Sprite>` d'un `R` de test d'un `World` → lecture par le champ), répertoire temporaire unique nettoyé par un garde (`Drop`) même en cas d'échec ; P3.2 le même fichier source est accepté par la Forge et rejeté par le lecteur.
**Ce qui établit I1** : P3.2 et P1.3 ne suffisent pas seuls ; c'est leur combinaison avec la suppression de la source en P3.1, la séparation des crates et l'absence de parseur TOML dans l'arbre de `verbum-assets`.
**Critère de sortie** : critères de la tranche A (§1.3), consignés comme **jalon intermédiaire**.

### Tranche B — identité opaque, manifeste, chaîne de résolution

#### Phase 4 — Manifeste, identifiant opaque et chargement (`verbum-assets`)
**Invariants** : I12, I13, I4, I5, I10.
**Fichiers** : ajout de `crates/assets/src/manifest.rs`, `src/load.rs` ; modification de `src/lib.rs`.
**Interfaces (provisoires, sans code)** :
- `AssetId` : type opaque (champ privé ; constructeur non public hors du crate ; `Debug` opaque). Seul le lecteur du manifeste le construit.
- `ArtifactIndex`, `SlotIndex` : enveloppes de `u32` avec accesseur (nom provisoire `get`). **`SlotIndex::new(u32)` est public** (fourni par la Forge et par l'application) et **ne vérifie pas la capacité** ; les contrôles `slot_index < slot_count` et d'unicité des slots restent à la Forge et au chargement.
- **`Manifest<'a>`** : vue qui **emprunte** les octets analysés (durée de vie `'a`) ; elle ne les possède pas. Création : `Manifest<'a>::parse(octets: &'a [u8]) -> Result<Manifest<'a>, ManifestError>` ; **niveau 1 uniquement**. Méthodes : nombre d'entrées, itération sur les entrées, `find(AssetId) -> Option<ManifestEntry>` (recherche linéaire sans allocation, réservée au chargement). `ManifestEntry` : valeur `Copy` (identifiant, `ArtifactIndex`, `SlotIndex`) construite par lecture champ par champ, sans référence aux octets.
- `ManifestError` (niveau 1) : troncature, signature, version, longueur (formule, surplus compris), identifiant dupliqué.
- `Manifest::check_bindings(&self, artifact_count, slot_count) -> Result<(), BindingError>` : **niveau 2**. `BindingError` : nombre d'entrées différent du nombre d'artefacts, capacité dépassée, `artifact_index` hors limites, `slot_index` hors limites, `artifact_index` dupliqué, `slot_index` dupliqué.
- `load(octets_du_manifeste, artifacts: &[&[u8]], slot_count, dépôt) -> Result<(), LoadError>` : enchaîne les niveaux 1, 2, 3, puis 4 (§4, §5.3, §5.4). Les octets du manifeste et des artefacts sont empruntés pour la durée de l'appel. `dépôt` : fonction `(SlotIndex, Sprite)` sans valeur de retour. `LoadError` : erreur de manifeste, erreur de liaison, ou erreur de lecture d'un artefact avec son `ArtifactIndex`. **Le dépôt n'est jamais invoqué si l'un des niveaux 1 à 3 échoue.**
**Tests** (unitaires dans le crate, où un constructeur privé de `AssetId` sert aux valeurs attendues ; identifiants synthétiques arbitraires dans les octets littéraux) :
- **M-1** manifeste littéral à deux entrées (artefacts 0 et 1, slots 1 et 0) : `parse` réussit ; nombre, indices et liaisons attendus ; `find` retrouve chaque entrée par son identifiant.
- **M-2** formule de longueur `10 + 16 n` pour n = 0, 1, 2, 3 ; le manifeste vide est accepté par `parse`.
- **M-3** toute troncature du manifeste littéral rejetée par `parse` ; `n` démesuré (`u32::MAX`) avec peu d'octets rejeté sans panique ni débordement ; surplus rejeté.
- **M-4** signature altérée rejetée, y compris le texte d'un fichier TOML ; version inconnue rejetée.
- **M-5** (niveau 1) identifiants dupliqués rejetés par `parse`.
- **M-6** opacité de `AssetId`, par doctests `compile_fail` : pas de construction depuis un entier, pas d'accesseur numérique, pas de conversion `From<u64>` ; le `Debug` ne contient pas la valeur numérique (assertion sur la sortie formatée) ; codes d'erreur épinglés au constat du rouge.
- **M-7** (niveau 2) `check_bindings` : nombre d'entrées ≠ nombre d'artefacts ; `n > slot_count` ; `artifact_index` hors limites ; `slot_index` hors limites ; `artifact_index` dupliqués ; `slot_index` dupliqués : chacun rejeté avec l'erreur attendue ; **un manifeste dont `slot_count` excède `n` est accepté**.
- **M-8** `SlotIndex::new(n)` restitue `n` par l'accesseur, y compris pour `0` et `u32::MAX` ; la construction n'est jamais rejetée (elle ne connaît pas la capacité) ; le rejet d'un indice hors capacité est observé en M-7 (chargement) et F-4 (Forge).
- **L-1** `load` nominal : deux artefacts littéraux (32×16 et 96×48), manifeste littéral (slots 1 et 0) : le dépôt reçoit (slot 1, 32×16) et (slot 0, 96×48), **exactement deux appels, un par slot**.
- **L-2** résolution par identifiant : `find` de chaque identifiant attendu donne le bon `artifact_index` et le bon `slot_index` ; les artefacts ainsi désignés se décodent au contenu attendu.
- **L-3** tout-ou-rien, avec un **compteur d'appels du dépôt** : (a) artefact tronqué à l'`artifact_index` 1, l'artefact 0 étant valide ; (b) artefact tronqué à l'`artifact_index` 0 ; (c) manifeste invalide au niveau 1 ; (d) liaison invalide au niveau 2 avec des artefacts valides : dans chaque cas `load` échoue et le compteur vaut **zéro**. Toute erreur détectée par la validation préalable provoque zéro appel au dépôt.
- **L-4** manifeste vide avec collection vide et `slot_count` = 2 : `load` réussit, **aucun dépôt** ; manifeste à une entrée avec `slot_count` = 2 : réussit, un seul dépôt, l'autre slot non sollicité ; manifeste vide avec collection non vide : rejeté (nombre d'artefacts).
- **L-5** prémisse du double décodage : `Sprite::read` appliqué deux fois aux mêmes octets donne deux résultats identiques (réussite comme échec) ; les valeurs reçues par le dépôt en passe 4 sont exactement celles de la validation de la passe 3 (comparaison avec des valeurs littérales) ; le chargement ne comporte ni allocation ni conservation de collection (garanti par `#![no_std]` sans `alloc`, vérifié à la compilation).
**Critère de sortie** : tests verts ; `verbum-assets` sans dépendance ; Miri vert (vous).

#### Phase 5 — Forge : compilation en lot, FNV-1a et manifeste (`verbum-forge`)
**Invariants** : I12, I2, I4, I9.
**Fichiers** : ajout de `crates/forge/src/id.rs` (FNV-1a, contrôle de distinction des identifiants), `src/batch.rs` ; modification de `src/lib.rs`.
**Interfaces (provisoires)** :
- Entrée par asset : clé logique, texte TOML, `SlotIndex`. Entrée du lot : tranche ordonnée de ces éléments, plus `slot_count`.
- `compile_assets(...) -> Result<CompiledSet, ForgeError>` : **API publique, qui utilise toujours FNV-1a 64 bits ; aucun paramètre de hash**. `CompiledSet` : octets du manifeste et artefacts **dans l'ordre d'entrée** (`artifact_index = i`).
- **Point de test privé** : une fonction interne au crate (visibilité restreinte au crate, non exportée) reçoit un calculateur de hash déterministe en paramètre ; `compile_assets` l'appelle avec FNV-1a. Elle n'est ni `pub`, ni réexportée, ni configurable par l'API de production.
- **Tout-ou-rien** : une erreur n'émet ni manifeste ni artefact.
- `ForgeError` (extension) : source TOML invalide (avec la position de l'entrée), clé vide, clé dupliquée, collision d'identifiants, `slot_index ≥ slot_count`, `slot_index` dupliqué.
- Réutilise `compile_sprite` de la tranche A.
**Tests** :
- **F-1** FNV-1a 64, appliqué directement à la fonction de production, comparé aux **constantes indépendantes du §8.3 du RFC 9923** : `""` → `0xcbf29ce484222325` ; `"a"` → `0xaf63dc4c8601ec8c` ; `"foobar"` → `0x85944171f73967e8`. L'entrée vide teste la fonction pure ; elle est sans lien avec le rejet des clés vides par `compile_assets` (F-4). Ces tests vérifient **l'implémentation propre à Verbum** ; ils ne dépendent pas des tests de compilation de la Forge, et sont nécessaires même si les constantes de l'algorithme sont vérifiées à la source.
- **F-2** manifeste littéral attendu pour le jeu de référence : clé `"a"` → sprite 32×16 → slot 1 ; clé `"foobar"` → sprite 96×48 → slot 0 ; ordre d'entrée [a, foobar] ; octets du manifeste et des artefacts égaux à des littéraux écrits à la main, dont les identifiants proviennent des constantes de F-1 et non de la Forge.
- **F-3** permutation : même jeu compilé dans l'ordre [foobar, a], correspondances clé → slot inchangées : les identifiants sont identiques à F-2, les `artifact_index` sont permutés, les slots inchangés, les artefacts rendus dans le nouvel ordre.
- **F-4** rejets sans aucune sortie : clé vide ; **clé dupliquée** ; `slot_index` dupliqué ; `slot_index ≥ slot_count` ; une source TOML invalide parmi plusieurs sources valides.
- **F-5** stabilité et absence de normalisation : (a) même clé avec un contenu différent donne le même identifiant ; (b) `"a"` et `"A"` donnent des identifiants différents ; (c) **exemple Unicode concret** : les clés `"\u{00E9}"` (« é » précomposé, 2 octets UTF-8) et `"e\u{0301}"` (« e » suivi de l'accent aigu combinant, 3 octets UTF-8) représentent le même texte visuel mais pas la même séquence UTF-8 ; le test vérifie d'abord leurs longueurs en octets (2 et 3), puis que **leurs identifiants sont distincts**, puis qu'un lot contenant ces deux clés est **accepté** (ni « clé dupliquée », ni « collision »). Les deux littéraux sont écrits avec des **séquences d'échappement `\u{...}`**, jamais avec les caractères eux-mêmes, afin qu'aucun éditeur ni outil ne puisse normaliser le fichier source.
- **F-6** **collision entre deux clés distinctes** : le point de test privé reçoit un calculateur constant (même identifiant pour tout) et **deux clés logiques distinctes** : la compilation est rejetée avec l'erreur de collision, sans sortie.
- **F-6b** précédence : avec le même calculateur constant et deux clés **identiques**, l'erreur est « clé dupliquée », non « collision ».
- **F-7** lot vide : manifeste de 10 octets accepté par `Manifest::parse` et par `load` avec une collection vide.
- **F-8** aucune chaîne : la longueur du manifeste est exactement `10 + 16 n` ; chaque artefact a la longueur de la tranche A.
- **F-9** non-fuite du point de test : doctests `compile_fail` établissant que la fonction à calculateur injectable n'est pas accessible depuis l'extérieur du crate ; la signature publique de `compile_assets` ne comporte aucun paramètre de hash (vérifié aussi par relecture de l'API publique au critère de sortie).
**Critère de sortie** : tests verts ; l'API de la tranche A reste valide ; `cargo tree -i toml --workspace` inchangé.

#### Phase 6 — Acceptation de la clôture (`crates/forge/tests/pipeline.rs`, extension)
**Invariants** : I12, I13, I6, I7, I8.
**Prérequis de l'application de test** [Lu] : un `R` à deux champs distincts de type `ResourceSlot<Sprite>` (`first`, `second`) se déclare et s'instancie sans modifier `verbum-ecs` : `ResourceSlot<T>` n'impose aucune contrainte sur `T` et est `Default` sans `T: Default` ; `World::new` n'exige que `R: Default` ; `resources_mut()` rend `&mut R`. Liaison statique : tableau de deux accesseurs, slot 0 → `first`, slot 1 → `second` ; `slot_count = 2`.
**Tests** :
- **T6.1** chaîne complète sur disque : **précondition : `first` et `second` sont vides avant le chargement** ; sources TOML écrites → compilation en lot (`slot_count` = longueur du tableau) → manifeste et artefacts écrits ; le test conserve la liste ordonnée des chemins d'artefacts → **suppression des sources** → relecture du manifeste et des artefacts dans l'ordre conservé → `load` avec un dépôt appliquant la liaison statique → **`second` contient 32×16 et `first` contient 96×48** (clé `"a"` → slot 1, clé `"foobar"` → slot 0).
- **T6.2** résolution par identifiant attendu, indépendante de `load` : les identifiants attendus sont ceux d'un manifeste littéral indépendant (valeurs issues des constantes de F-1, non de la Forge) lu par `Manifest::parse` ; le test cherche chacun de ces identifiants dans le manifeste **produit par la Forge**, vérifie `artifact_index` et `slot_index`, décode l'artefact désigné et vérifie son contenu.
- **T6.3** permutation : compilation dans l'ordre inverse ; les mêmes identifiants aboutissent au même contenu dans le même slot ; les `artifact_index` ont changé (assertion explicite) ; l'état final de `first` et `second` est identique à T6.1.
- **T6.4** rejets avant dépôt : manifeste produit puis altéré (identifiant dupliqué, `slot_index` dupliqué, `slot_index` hors limites, `artifact_index` hors limites, nombre d'artefacts différent de `n`) ; **un artefact mal formé parmi des artefacts valides** ; un manifeste compilé avec `slot_count = 3` pour trois assets et chargé avec une capacité de 2 : chaque cas échoue, le dépôt n'est jamais appelé, **`first` et `second` restent vides**.
- **T6.5** le texte TOML d'une source fourni à `Manifest::parse` est rejeté (signature) ; le runtime n'interprète jamais la source.
- **T6.6** capacité supérieure au nombre d'assets : un seul asset compilé et chargé avec `slot_count = 2` ; l'autre slot reste vide ; manifeste vide et collection vide : aucun dépôt, les deux slots vides.
- Répertoire temporaire unique par test, nettoyé par un garde (`Drop`) même en cas d'échec.
**Contrôles manuels** (rapportés par vous) : `cargo tree -p verbum-assets` sans dépendance ; `cargo tree -i toml --workspace` ne liste que `verbum-forge` ; aucune modification de `crates/ecs`, `crates/schedule`, `crates/verbum`, des ADR ni de la Roadmap (K8).
**Critère de sortie** : critères de clôture du §1.3 (K1 à K8) ; Miri vert sur `verbum-assets` ; MSRV vert.

## 7. Risques

- **R1 — Dépendances du crate `toml`.** Arbre transitif, MSRV vis-à-vis de 1.85, comportement sous Miri : **[Non vérifié]**. Clause d'arrêt : si aucune version ne tient sous 1.85, arrêt et rapport.
- **R2 — Étape 5.** Le risque résiduel est que l'étape 5 (charges variables, plusieurs types) exige d'en modifier la forme. Les layouts de la tranche B sont des choix locaux, non gabarit (§5).
- **R3 — Vigilance Renderer (pas une exigence)** : (a) `Sprite` est un descripteur atomique ; l'artefact et le manifeste ne portent aucune relation d'appartenance, d'ordre d'empilement ou de regroupement ; (b) `AssetId`, `artifact_index` et `slot_index` ne sont ni un ordre d'empilement ni un regroupement ; (c) F1 n'impose rien à la représentation de frame (ADR-R01 à R03) ; l'étape 4 ne doit pas en déduire qu'un asset correspond à une image rendue.
- **R4 — Temps.** Aucune durée dans le sprite (ADR-S04 non tranché).
- **R5 — Frontières de dépendance.** `verbum-assets` ne dépend jamais de `verbum-ecs` ni de la Forge ; pas de cycle de dev-dépendances.
- **R6 — Écriture des Ressources d'asset.** ADR-A01 pt 4 interdit tout propriétaire d'écriture au runtime ; `ResourceSlot::get_mut` permet la mutation à tout porteur de `&mut R`. Non testable à l'étape 2 **[Déduit]**.
- **R7 — Pression de sur-conception.** Registre général, table d'assets, génération de code, second type, charges variables : exclus (§10).
- **R8 — Écart de portée avec la Roadmap** (§1.2).
- **R9 — Collisions entre ensembles compilés séparément.** Non traitées ; le plan ne le prétend pas (§5.1).
- **R10 — Fuite du point de test de hash.** Garde-fous : visibilité restreinte au crate, F-9, relecture de l'API publique.
- **R11 — Liaison statique côté application.** Les deux dernières flèches de la chaîne ne sont pas portées par `verbum-assets` (§4.2).
- **R12 — Coût du chargement.** Contrôles de doublons quadratiques (sans allocation) et double décodage de chaque artefact : bornés par `slot_count`, exécutés avant le hot path ; mesure d'abord si `slot_count` devient grand.
- **R13 — Référence FNV.** Algorithme et constantes vérifiés à la source (RFC 9923 §2, §5, §8.2.2) ; vecteurs `""`, `"a"` et `"foobar"` confirmés par le tableau de test du §8.3. Aucune valeur de référence ne provient de l'implémentation testée.
- **R14 — Prémisse « octets immuables » du double décodage.** Valable pour des tampons mémoire ordinaires ; à reconsidérer si un accès par `mmap` est ajouté (§5.4).
- **R15 — Panique.** Violation interne impossible du second décodage, ou panique de l'application dans le dépôt : hors garantie tout-ou-rien (§5.4), qui porte sur les erreurs retournées avant le début des dépôts.

## 8. Arbitrages et clôtures

### 8.1 Arbitrages rendus (intégrés)

| Question | Arbitrage |
|---|---|
| Q-A | Métadonnées à charge fixe ; aucun décodage PNG |
| Q-B | Interprétation A : manifeste et identité opaque dans le contrat d'ADR-A01 ; tranche A = jalon intermédiaire ; clôture exige la tranche B ; pas de renvoi à l'étape 5 |
| Q-C | `from_le_bytes`, choix local |
| Q-D | Crate `toml`, sans ADR dédiée |
| Q-E | Deux crates |
| Q-F | `crates/forge/tests/pipeline.rs`, `verbum-ecs` en dev-dependency ; façade inchangée |
| Q-G | Bibliothèque ; ni CLI ni `build.rs` |
| Q-H | P1.6 inclus |
| Q-I | F1 : un `ResourceSlot<Sprite>` distinct par asset ; trois notions distinctes ; pas de F2 |
| Q-J | Option C : identifiants lus dans le manifeste par les tests ; désignation symbolique non démontrée |
| Q-K | FNV-1a 64 bits de la clé logique (UTF-8 exact, fournie hors TOML) ; type opaque ; doublons et collisions rejetés par la Forge |
| Q-L | Manifeste binaire distinct `VMAN` |
| Q-M | Capacité = cardinalité de la configuration statique des slots |
| Q-N | Deux `Sprite` (32 → slot 1, 96 → slot 0), ordres dissociés, test de permutation |
| r6 | Vecteurs FNV confirmés ; deux niveaux de contrôle ; choix ajoutés confirmés ; test de collision par point de test privé |
| **r7, point 1** | **Référence RFC 9923 vérifiée : constantes et algorithme présentés comme vérifiés à la source ; F-1 conservé** |
| **r7, point 2** | **Chargement tout-ou-rien par double décodage sans allocation ; contrat du dépôt (sans erreur récupérable, slots vides à l'entrée) ; portée limitée aux erreurs retournées** |
| **r7, point 3** | **`Manifest<'a>` emprunte les octets ; `parse(&'a [u8])`** |
| **r7, point 4** | **F-5 : `"\u{00E9}"` et `"e\u{0301}"`, identifiants distincts, UTF-8 exact sans normalisation** |
| **r7, point 5** | **F-6 par calculateur privé confirmé ; aucune API publique ne l'expose** |
| **r8, point 1** | **RFC 9923 distingué par section (§2, §5, §8.2.2, §8.3) ; vecteurs `""`, `"a"`, `"foobar"` confirmés par le tableau de test du §8.3 ; `""` intégré à F-1 ; divergence de numérotation et réserve supprimées** |
| **r8, point 2** | **Seconde lecture impossible = violation interne, `unreachable!` actif en production, hors `LoadError` et hors garantie tout-ou-rien ; `[H]` et analogie avec `World::apply` retirés** |
| **r8, point 3** | **`SlotIndex::new(u32)` public avec accesseur ; sans vérification de capacité ; test M-8** |

### 8.2 Q-C — nature du cast (inchangé)
Le cast zéro-copie est le mécanisme par défaut décrit par le corpus, non un invariant (ADR-A01 pt 3 et classification ; ADR-000 pt 3 : « Cette règle porte sur les formats humains. »). La lecture champ par champ est admissible ; `ResourceSlot<T>` stockant `Option<T>` par valeur (`resource.rs`), les options copient de toute façon la valeur dans l'emplacement. La conclusion ne vaut que pour ces descripteurs ; elle ne préjuge pas des charges variables de l'étape 5.

### 8.3 Q-D — trace du choix de parseur (inchangé)
Aucune ADR dédiée au choix du crate `toml` (décision d'implémentation substituable, `00-principes.md`) ; choix consigné ici et au journal ; compatibilité MSRV 1.85 à vérifier (R1).

### 8.4 Ce qui reste non démontré à la clôture (complément, non substitut aux preuves K1 à K8)
- **Désignation symbolique d'un asset depuis le code du jeu** (Q-J option C) : les identifiants sont obtenus en lisant un manifeste, non par un nom du code du jeu.
- **Adressage d'une Ressource par `AssetId` après le chargement**, par exemple par un système : F1 n'offre aucune telle API, par décision (aucun accès dynamique à `World`) ; l'identifiant n'est exploité qu'au chargement.
- **Collisions entre ensembles compilés séparément** (§5.1).
- **Collision réelle d'identifiants entre deux clés** : le rejet est prouvé par injection, non par une paire réelle de clés.
- **Récupération après panique de l'application pendant le dépôt** (§5.4).
- **Un second type d'asset, des charges de longueur variable, un accès par `mmap`** : hors périmètre ; les layouts de la tranche B ne sont pas un gabarit.

## 9. Fichiers créés ou modifiés

Créés : `docs/implementation/02-verbum-forge.md` (ce plan) ; `crates/assets/{Cargo.toml, src/lib.rs, src/sprite.rs, src/manifest.rs, src/load.rs}` ; `crates/forge/{Cargo.toml, src/lib.rs, src/sprite.rs, src/id.rs, src/batch.rs, tests/pipeline.rs}`.
Modifiés : `Cargo.toml` (membres, dépendances du workspace), `Cargo.lock` (par Cargo uniquement), `tree.md` (régénéré par votre script).
**Jamais modifiés** : `crates/ecs`, `crates/schedule`, `crates/verbum`, les ADR, `ROADMAP.md`.

## 10. Hors périmètre de l'étape 2

Registre ou table d'assets (F2), second type d'asset, charges de longueur variable, système de couches, notion de personnage composé, décodage d'image, durées et ticks, génération de code, `build.rs` consommateur, exécutable en ligne de commande, accès par `AssetId` après le chargement, calculateur de hash configurable dans l'API publique, tampon de staging ou collection dynamique dans le chargement, `mmap` et accès disque runtime réel, Shell, Scheduler, hot-reload, streaming et déchargement, LPC.

## 11. Fin de l'étape

L'étape 2 est close lorsque la tranche A est verte **et** que les preuves K1 à K8 sont toutes établies par les tests listés, l'énumération du §8.4 étant fournie en complément. La fin de la tranche A seule est un jalon intermédiaire. R3 est reporté au cadrage de l'étape 4. **Aucune implémentation ni commande de validation avant l'approbation explicite de ce plan.**
