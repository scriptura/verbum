# Verbum — Plan d'implémentation du chantier V2 : provenance du quantum q par un paquet `VMAN` v2 — r1

## 0. Statut, marqueurs, vocabulaire

**Non normatif. Plan d'implémentation, révision 1, version définitive (15 novembre 2026, corrigée le 16 novembre 2026).** Document directeur du chantier V2 (décision D-N3). P-V2 validée, au 17 novembre 2026.

**Ce document ne conçoit pas le paquet.** Le layout du paquet, son accès indexé, sa gestion mémoire, ses erreurs propres, les noms et la visibilité des constantes v2 et l'emplacement des tests additifs sont réservés à la note de conception V2-C (§ 9, § 10), annexée à ce plan avec un statut distinct.

### 0.1 Marqueurs

| Marqueur | Sens |
|---|---|
| **[Contrat]** | Posé par une ADR adoptée (référence donnée) |
| **[Fait]** | Constaté dans un fichier fourni ou le résultat d'une recherche (référence donnée) |
| **[Arbitré]** | Décision de l'utilisateur (référence : § 4.2) |
| **[Proposition]** | Choix de ce plan, à valider ; ne vaut pas décision adoptée |
| **[Ouvert]** | Non déterminé ; énoncé sans choix |
| **[Dépendance]** | Relève d'une autre décision, unité ou chantier |

### 0.2 Codes de risque d'adaptation

| Code | Sens |
|---|---|
| **S** | Sensible : octet, constante, offset ou assertion v1. Toute modification est un arrêt (G-CM ou R-ADAPT). Sous V-a, aucune modification n'est attendue |
| **C** | Conditionnel : dépend d'une décision de V2-C ou d'un arbitrage |
| **N** | Nul : aucun contact attendu |

### 0.3 Vocabulaire local

| Terme | Définition |
|---|---|
| **Quantum q** | Durée du tick logique, unique par configuration de jeu (ADR-S04 pt 2). Représenté en nanosecondes (`q_ns`, `u64`) dans l'en-tête v2 (D-1d) |
| **`VMAN`** | Signature du manifeste : table produite par la Forge qui associe un identifiant opaque d'asset à un indice d'artefact et à un indice de slot (ADR-A01 pt 3) |
| **Niveaux 1 à 4** | Contrôles de `load` : 1 analyse du manifeste ; 2 cohérence des liaisons, puis format attendu de chaque slot ; 3 premier décodage de chaque artefact et fermeture des références ; 4 redécodage et dépôt |
| **Paquet** | Conteneur unique qui porte le manifeste v2 et les artefacts (spéc. § 7.2, R-1). Sa forme est réservée à V2-C |
| **Enveloppe** | Ce qui, dans le paquet, permet d'en extraire le manifeste et les artefacts avant le niveau 1. Sa forme est réservée à V2-C |
| **Voie additive** | Chemin de code nouveau, de nom distinct, qui laisse intacts les symboles, octets et comportements v1 |
| **Compatibilité de lecture / d'exécution** | Lire un manifeste v1 est admis (V-a) ; l'accepter pour une exécution conforme à ADR-S04 ne l'est jamais (D-1b) |
| **G-CM, R-ADAPT, G-S6** | Portes de gouvernance définies en § 6 et § 11 |

---

## 1. Objet, périmètre, hiérarchie documentaire

### 1.1 Objet
Planifier le chantier V2 : faire provenir le quantum q d'un paquet d'assets unique, produit par la Forge et chargé sans allocation par `verbum-assets`, de sorte que le Shell n'exécute jamais silencieusement des artefacts compilés pour un autre quantum (ADR-S04 pt 2, R-1). V2-0 produit ce plan ; il n'autorise ni ne réalise aucune modification de `VMAN` (ADR-S04 pt 2, dernier alinéa).

### 1.2 Périmètre
| Zone | Contenu |
|---|---|
| Assets | lecteur v2 du manifeste, chargement à entrée unique, refus croisés (additifs) |
| Forge | voie de compilation additive qui reçoit q explicitement |
| Consommateurs v1 | protégés : aucun changement (§ 5, § 6, Annexe A) |
| Documents | ce plan ; chaîne exécution et handoff sur la même racine (§ 12) |

### 1.3 Hors périmètre
| Sujet | Renvoi |
|---|---|
| Layout du paquet, accès indexé, gestion mémoire, erreurs du paquet, noms et visibilité des constantes v2 | V2-C (D-N4) |
| Règle des durées non entières en ticks | F2 (D-N2) |
| Portée de `games/demo` | F3a (D-N5) |
| Valeurs de q, K, M de composition | F3c (D-N1) |
| VQ-8 (bout en bout Forge → paquet → chargement → `TemporalPolicy`) | **Dépendance de F3b**, hors critères de sortie de V2-L, V2-P et V2-F |
| `verbum-script`, `forge/src/script.rs`, tests `scr_*` et `scr_l*` | G-S6 : aucune modification sans arbitrage |
| Intégration native (`winit`, `wgpu`), politique de pause | hors chantier |

### 1.4 Hiérarchie documentaire
| Niveau | Responsabilité |
|---|---|
| **ADR adoptées** | Décisions normatives (ADR-000, A01, S04 et autres) |
| **Spécifications et plans techniques validés** | Références techniques dans leur périmètre, subordonnées aux ADR (spécification `03-verbum-shell-temporal-policy.md` r3, plans des étapes 2, 5 et 6 avec leurs documents d'exécution) |
| **Roadmap** (`03-verbum-shell-temporal-policy-roadmap.md` r1.1) | Ordre, dépendances et portes |
| **Plan V2 (ce document)** | Référence d'organisation du chantier V2, **non normative** |
| **Handoffs** | Consignes opérationnelles par session |
| **Comptes rendus d'exécution** | État factuel des réalisations et des vérifications |

La roadmap ne modifie pas les algorithmes d'une spécification validée. Le plan V2 ne modifie aucune décision déjà adoptée.

### 1.5 Contradictions
Une contradiction entre documents, ou avec le code, est signalée à l'utilisateur et attend son arbitrage ; elle n'est jamais résolue par la session. Les écarts constatés à ce jour figurent en § 5.6 et § 15.

---

## 2. Sources examinées et limites de preuve

### 2.1 Documents

| Catégorie | Documents |
|---|---|
| **Lus et exploités** | ADR-000, ADR-A01, ADR-S04 ; `02-verbum-forge.md` (r8) et `-execution` ; `05-verbum-assets.md` (r1.3 ; statut : voir § 2.5) et `-execution` ; `06-verbum-scripts.md` (r1.2) et `-execution` ; `03-verbum-shell-temporal-policy.md` (r3) ; `03-verbum-shell-temporal-policy-roadmap.md` (r1.1) ; résultats de recherche sur `docs/` (G2, G3) |
| **Lu à titre indicatif** | `03-verbum-shell-temporal-policy-remaining-work.md` (mémoire de travail qui ne remplace pas la roadmap) |
| **Reçus, non exploités par le recensement** | ADR-001 ; ADR-S01 |
| **Reçus après la validation du recensement consolidé, non exploités par le recensement, non requis** | `docs/handoffs/05-verbum-assets-implementation-handoff.md` ; `ADR-Sc01-scripts.md` |

### 2.2 Code lu
| Crate | Fichiers |
|---|---|
| `verbum-assets` | `Cargo.toml`, `src/{lib, manifest, header, format, load, sprite, tileset, map, font, script}.rs` |
| `verbum-forge` | `Cargo.toml`, `src/{lib, batch, id, sprite, map, tileset, font, script}.rs`, `tests/pipeline.rs` |
| `verbum-script` | `Cargo.toml`, `src/{lib, emissions, state, step, table}.rs` |
| `verbum-shell` | `Cargo.toml`, `src/{lib, policy}.rs`, `tests/{borrowed_resource, script_loaded, level_loaded, script_suspend}.rs`, `tests/script_support/mod.rs`, `examples/{script_headless, level_headless}.rs` |
| Racine | `Cargo.toml`, `Cargo.lock`, `tree.md` (régénéré le 15 novembre 2026, 04:37) |

### 2.3 Recherches textuelles
Commandes exactes et périmètres en Annexe C. Cinq recherches : G1 (symboles du format et du chargement), G2 (documents citant `VMAN`), G3 (lignes citant `VMAN` dans les handoffs et notes), G4 (littéraux hexadécimaux de la signature), G5 (variantes d'erreur et `non_exhaustive`).

### 2.4 Limites
| Limite | Conséquence |
|---|---|
| Une recherche textuelle ne détecte que ce que son motif cible | Littéraux répartis sur plusieurs lignes, en chaîne d'échappement, assemblés ou générés à l'exécution : non vus |
| La couverture repose sur la lecture intégrale des fichiers fournis et sur les cinq recherches | **Ce n'est pas une preuve d'exhaustivité.** Elle est jugée suffisante pour rédiger ce plan |
| Les numéros de ligne viennent des recherches du 15 novembre 2026 | Ils peuvent dériver ; les symboles et tests nommés font foi |
| G5 ne contient que les formes `Énumération::Variante` | Un usage par import global (`use X::*`) n'est pas détecté ; aucun n'a été vu dans les fichiers lus |
| Aucune commande n'a été exécutée | Aucun résultat de test, de Miri ou de MSRV n'est affirmé |

### 2.5 Faits non vérifiés
| Fait | Statut |
|---|---|
| Contenu courant du dépôt réel au-delà des fichiers fournis | Non vérifié |
| `TemporalPolicy` et `PolicyError::ZeroQuantum` dans le code du Shell | **Vérifié** par relecture de `crates/engine/shell/src/policy.rs` et `src/lib.rs` (16 novembre 2026) ; API consignée en § 4.9. Aucune exécution |
| Statut de validation des plans des étapes 2 et 5 | Les en-têtes de `02-verbum-forge.md` et `05-verbum-assets.md` étaient historiquement périmés ; l'utilisateur les a corrigés lui-même le 16 novembre 2026 (`02-verbum-forge.md` : adopté ; `05-verbum-assets.md` : adopté). Ces documents n'ont pas été relus ni modifiés par cette mission. Traçabilité en § 5.6 |
| Faisabilité d'une production v1 octet pour octet identique avec une voie additive | Condition posée par la spécification (§ 7.2) ; à établir en V2-F |
| Faisabilité du chargement à entrée unique sans allocation | À étudier en V2-C |

### 2.6 Écarts documentaires historiques
Voir § 5.6.

---

## 3. Invariants v1 à préserver

### 3.1 Constantes du manifeste v1 [Fait]
Source : `assets/src/manifest.rs` (l. 21-35), exportées par `lib.rs`.

| Constante | Valeur | Assertion associée |
|---|---|---|
| `MANIFEST_SIGNATURE` | `b"VMAN"` | — |
| `MANIFEST_VERSION` | `1` (`u16`) | — |
| `MANIFEST_HEADER_LEN` | `10` | égale signature (4) + `u16` + `u32` (`const _`) |
| `MANIFEST_ENTRY_LEN` | `16` | égale `u64` + 2 × `u32` (`const _`) |

Épinglées par `vm_1` (`forge/tests/pipeline.rs`) et utilisées pour produire le manifeste (`forge/src/batch.rs`, l. 222-236).

### 3.2 `Manifest::parse` [Fait]
| Rang | Contrôle | Erreur |
|---|---|---|
| 1 | en-tête de 10 octets présent | `Truncated` |
| 2 | signature | `Signature` |
| 3 | version égale à `MANIFEST_VERSION` (**avant** la longueur) | `Version` |
| 4 | `n` (`u32`) converti en `u64`, `checked_mul` par 16, conversion vérifiée ; octets restants inférieurs ou supérieurs | `Truncated`, `TrailingBytes` |
| 5 | unicité des identifiants | `DuplicateId` |

Conséquence [Fait] : un manifeste dont la signature est valide et dont la version n'est pas 1 est rejeté par `ManifestError::Version` **sans dépendre de sa longueur**. Cela suppose que le champ de version reste, dans un en-tête v2, à la position qu'il occupe en v1 (D-1d le place immédiatement après la signature).

### 3.3 `load` historique [Fait]
| Élément | Contenu |
|---|---|
| Signature | `load<'a>(manifest: &[u8], artifacts: &[&'a [u8]], slot_formats: &[Format], deposit: impl FnMut(SlotIndex, Asset<'a>)) -> Result<(), LoadError>` |
| Niveau 1 | `Manifest::parse`, erreur `LoadError::Manifest` |
| Niveau 2 | `check_bindings(artifacts.len(), slot_formats.len())` (nombre d'entrées, capacité, bornes, doublons d'artefact puis de slot), puis `SlotFormat` pour une identité connue différente du format attendu |
| Niveau 3 | décodage de chaque artefact, fermeture des références des cartes, résultat abandonné |
| Niveau 4 | redécodage et dépôt, seulement si les niveaux 1 à 3 ont réussi |
| Garanties | tout-ou-rien, aucune allocation (`no_std` sans `alloc`, `[dependencies]` vide) |
| `unreachable!` | trois sites : seconde lecture du niveau 4, `expected_format`, `artifact_bytes` |

### 3.4 Forge v1 [Fait]
| Élément | Contenu |
|---|---|
| `compile_assets` | signature `fn(&[AssetSource<'_>], usize) -> Result<CompiledSet, ForgeError>` épinglée par doctest ; un troisième argument est refusé (E0061) |
| Accès | `compile_assets_with` (calculateur de hash injectable) et `fnv1a_64` sont `pub(crate)` ; les modules `batch` et `id` sont privés (doctests E0603 et E0432) |
| Ordre de production | contrôles (clé vide, clé dupliquée, slot hors capacité, slot dupliqué, collision d'identifiants), compilation des sources, résolution des références, **puis manifeste** |
| `CompiledSet` | champs privés `manifest` et `artifacts`, exposés **séparément** ; aucun constructeur public |
| Artefacts | aucune dépendance à q : `compile_script` écrit `ticks` en brut |

### 3.5 Identité [Contrat][Fait]
`AssetId` opaque (constructeur `pub(crate)`, `Debug` sans valeur, doctests `compile_fail`) ; `ArtifactIndex` et `SlotIndex` distincts ; aucune chaîne ni chemin ne franchit la frontière de compilation (ADR-A01 pt 3).

### 3.6 Artefacts [Fait]
Préfixe commun de 10 octets (identité, version `u16`, longueur de charge utile `u32`) ; version 1 pour chacun des cinq formats. Les layouts d'artefacts ne changent pas dans ce chantier (arbitrage ARB-9).

### 3.7 Constantes v1 et v2
Voir § 4.5.

---

## 4. Architecture additive v2 (sans layout du paquet)

### 4.1 Principes arbitrés [Arbitré]
| Réf. | Décision (spéc. § 7.1) |
|---|---|
| D-1 | q est une métadonnée globale de la configuration du jeu, portée par un champ global de l'en-tête de `VMAN` |
| D-1b (V-a) | Le lecteur supporte v1 et v2 ; les nouvelles compilations destinées à S04 produisent du v2 ; la compilation historique reste disponible et produit les **mêmes octets** ; un v1 n'est **jamais** accepté silencieusement pour une exécution conforme à S04 (aucun q par défaut) |
| D-1c | Chantier V2 : plan → exécution → handoff, pleinement soumis à G-CM, validation avant toute modification de `VMAN` ou de la Forge |
| D-1d | `q_ns: u64` (nanosecondes), **immédiatement après la version et avant le nombre d'entrées** ; en-tête de 18 octets ; longueur totale `18 + 16·n` ; q jamais dans une entrée ni dans un artefact ; ordre d'octets little-endian, aligné sur v1 (arbitrage ARB-10) |
| R-1 (M-1) | Paquet unique manifeste + artefacts, sous réserve que V2 garantisse son intégrité structurelle |

Ce plan **ne précise rien** sur la forme du paquet au-delà de ces décisions.

### 4.2 Arbitrages consignés

| Réf. | Énoncé | Traitement dans le plan |
|---|---|---|
| ARB-1 | Pas de nouveau niveau de validation. L'extraction de l'enveloppe est une préparation technique préalable au niveau 1 ; les niveaux 1 à 4 restent inchangés ; la forme exacte relève de V2-C | § 4.6, R-C1 |
| ARB-2 | La contrainte no-allocation est maintenue ; la signature `&[&[u8]]` de `load` est un problème de conception de V2-C, non une justification d'allouer | § 4.7, R-C4 |
| ARB-3 | Constantes et valeurs v1 inchangées ; constantes v2 distinctes, additives ; aucune constante historique réutilisée avec une nouvelle signification | § 4.5 |
| ARB-4 | R-ADAPT s'applique aux périmètres des étapes 2, 5 et 6 ; correspondance précise tests historiques / vérifications v2 ; aucune assertion modifiée pour faire passer v2 | § 6 |
| ARB-5 | Recensement des consommateurs et des `match` exhaustifs ; aucune variante d'erreur ajoutée par anticipation ; besoins présentés pour arbitrage | § 5.5, § 8, Annexe B |
| ARB-6 | `Manifest::parse` reste strictement v1 ; `ManifestError::Version` privilégiée pour une version inattendue ; pas de variante dédiée pour distinguer v1 et v2 ; toute autre erreur distincte doit être justifiée | § 8 |
| ARB-7 | VQ-8 est repris comme dépendance de F3b | § 7 |
| ARB-8 | Plan V2 = référence centrale ; comptes rendus Assets et Forge : synthèses et renvois à V2-Z, sans duplication | § 12.4 |
| ARB-9 | Les artefacts `Script` sont transportés par le paquet ; la compatibilité v1 couvre artefacts et octets de l'ancien chemin ; aucune modification de `forge/src/script.rs` ni des assertions `scr_*` sans arbitrage ; recensement de `verbum-script` inclus | § 5.2, G-S6 |
| ARB-10 | Little-endian rappelé comme contrainte, sans rouvrir ; layout du paquet intégralement à V2-C | § 4.1 |
| ARB-11 | D-N3 : plan autonome, non normatif, nommé `docs/implementation/03-verbum-shell-temporal-policy-vman-v2.md` | § 12.2 |
| ARB-12 | VQ-9b affecté à V2-P, vérification additive, `m4` inchangé ; V2-L vérifie la lecture et le rejet des versions, V2-P le comportement du chargement historique face à un manifeste v2 | § 7.2 |
| ARB-13 | q = 0 côté Forge : `ForgeError::ZeroValue("q")` (précédent `ZeroValue("ticks")`), aucune variante ajoutée | § 8 |
| ARB-14 | q = 0 côté Assets : ouvert, réservé à V2-C ; `Version` n'est pas détournée | § 8, R-C9 |
| ARB-15 | `vm_1`, `t6_4`, `f2`, `f3`, `f7`, `f8` et les littéraux recensés sont explicitement sensibles ; assertions intangibles ; ajouts v2 additifs ; distinction erreurs publiques / internes | § 5, § 6, Annexes A et B |
| ARB-16 | Recensement suffisant pour rédiger ; réserve d'exhaustivité à documenter avec sources et limites | § 2, Annexe C |
| ARB-17 | Annexes A, B, C intégrées au document ; glossaire facultatif ; document autonome | structure |
| ARB-18 | Statut de VQ-8 : « Dépendance de F3b, hors critères de sortie de V2-L, V2-P et V2-F » ; non compté comme validation V2 | § 7 |
| ARB-19 | La note de conception V2-C est annexée au plan V2, avec un statut distinct et une validation P-CONC avant toute implémentation V2-L ou V2-P ; elle pourra préciser layout, accès indexé, gestion mémoire et erreurs du paquet, **aucun de ces éléments n'étant anticipé ici** | § 9, § 10, § 11.3 |
| ARB-20 | Hiérarchie documentaire en six responsabilités | § 1.4 |
| ARB-21 | Le handoff Assets et ADR-Sc01 sont présentés dans une catégorie séparée | § 2.1 |
| ARB-22 | Règle de séparation sémantique v1 / v2 conservée ; noms exacts et visibilité des constantes v2 réservés à V2-C | § 4.5 |
| ARB-23 | R-ADAPT historique distingué des garde-fous spécifiques à V2 ; objectif zéro adaptation du chemin v1 ; autorisations historiques non élargies par interprétation | § 6 |

### 4.3 Ce qui est additif
| Élément | Nature |
|---|---|
| Lecteur v2 du manifeste | nouveau symbole, distinct de `Manifest::parse` |
| Chargement du paquet (entrée unique) | nouveau chemin ; le chargement historique reste réservé à v1 |
| Voie de compilation q | nouveau symbole de nom distinct ; `compile_assets` et son doctest intacts |
| Constantes v2 | nouvelles, distinctes (§ 4.5) |
| Tests v2 | nouveaux ; aucun test historique modifié |

### 4.4 Ce qui reste intact
Tout § 3 ; `Manifest::parse` strictement v1 ; `compile_assets` et son doctest ; octets produits par l'ancienne voie ; `load` à deux entrées pour v1 ; tests de l'étape 6 et sources de `forge/src/script.rs`.

### 4.5 Constantes v1 et v2 : règle de séparation
| Liste | Contenu | Règle |
|---|---|---|
| Constantes v1 | `MANIFEST_SIGNATURE`, `MANIFEST_VERSION`, `MANIFEST_HEADER_LEN`, `MANIFEST_ENTRY_LEN` et leurs assertions | **Conservées telles quelles** ; aucune ne reçoit une nouvelle signification |
| Constantes v2 | à identifier par V2-C | **Distinctes et additives** ; **noms exacts et visibilité publique réservés à V2-C** |
| Collision de noms | `HEADER_LEN` (`sprite.rs`, publique, préfixe d'artefact) et `MANIFEST_HEADER_LEN` | Ne doivent pas être réutilisées pour l'en-tête v2 |

### 4.6 Préparation technique avant le niveau 1 [Arbitré]
L'extraction de l'enveloppe du paquet n'est pas une étape de validation sémantique et n'ajoute aucun niveau. Les niveaux 1 à 4 conservent leur ordre, leur atomicité et leur comportement. La forme de cette préparation est réservée à V2-C (R-C1). Tout besoin qui obligerait à créer, supprimer ou réordonner un niveau est un arrêt G-CM (§ 11.2).

### 4.7 No-allocation [Arbitré]
Le noyau `verbum-assets` reste `no_std` sans `alloc`, avec `[dependencies]` vide, dev-dépendances comprises. Le fait que `load` reçoive `&[&[u8]]` est un problème de conception à résoudre en V2-C (R-C4) : il ne justifie ni allocation ni relâchement du contrat.

### 4.8 Décidé et réservé à V2-C

| Sujet | État |
|---|---|
| Position de q dans l'en-tête, largeur, ordre d'octets, longueur d'en-tête | **Décidé** (D-1d) |
| Paquet unique, absence de chemin v2 à deux entrées | **Décidé** (R-1) comme exigence ; **forme réservée** |
| Layout du paquet, accès aux artefacts, offsets, chevauchements, conversions | **Réservé V2-C** |
| Gestion mémoire du chargeur à entrée unique | **Réservé V2-C** |
| Réutilisation des niveaux 1 à 4 sans duplication | **Réservé V2-C** (exigence posée) |
| Noms et visibilité des constantes v2, nom de la voie de compilation | **Réservé V2-C** |
| Erreurs B-1 et B-4 | **Réservé V2-C** |
| Emplacement des tests additifs | **Réservé V2-C** |

### 4.9 Contrat aval — `TemporalPolicy` (API réelle, relue le 16 novembre 2026)

Source : `crates/engine/shell/src/policy.rs` et `src/lib.rs` (relus). **[Fait]**. Cette section consigne l'API existante ; elle n'en déduit aucun usage futur (F3b, VQ-8 : hors périmètre).

| Élément | Constat |
|---|---|
| Constructeur | `pub fn from_nanos(quantum_ns: u64, budget: u32, threshold: u32) -> Result<Self, PolicyError>` |
| Ordre des contrôles | q, puis K (`budget`), puis M (`threshold`) |
| q = 0 | `NonZeroU64::new(quantum_ns)` échoue → `Err(PolicyError::ZeroQuantum)` (premier contrôle) |
| K = 0, M = 0 | `Err(PolicyError::ZeroBudget)`, `Err(PolicyError::ZeroThreshold)` |
| Erreurs | `pub enum PolicyError { ZeroQuantum, QuantumOutOfRange, ZeroBudget, ZeroThreshold }` ; dérive `Debug, Clone, Copy, PartialEq, Eq` ; pas de `#[non_exhaustive]` |
| `QuantumOutOfRange` | Retournée seulement par `from_duration`, forme privée `#[cfg(test)]` ; non atteignable par `from_nanos` |
| Type | `pub struct TemporalPolicy { quantum_ns: NonZeroU64, budget: NonZeroU32, threshold: NonZeroU32 }` ; champs privés ; dérive `Debug, Clone, Copy, PartialEq, Eq` ; immuable, sans mutateur |
| Accesseurs | `quantum_ns() -> NonZeroU64`, `budget() -> NonZeroU32`, `threshold() -> NonZeroU32` (`#[must_use]`) |
| Export | `lib.rs` : `mod policy;` (privé) et `pub use policy::{PolicyError, TemporalPolicy};` |
| Dépendances | `[dependencies]` du crate Shell vide |
| Tests existants | `a7_valeurs_de_test_et_accesseurs`, `a7_zeros_rejetes`, `a7_ordre_des_controles`, `a7_bornes_acceptees`, `a7_k_et_m_independants`, `a7_q_hors_plage`, `a7_formes_de_q_equivalentes`, `e11_construction_invalide_ne_produit_aucune_politique` |

Conséquence pour V2 : q est un `u64` en nanosecondes, cohérent avec `q_ns: u64` du v2 (R-C13) ; le refus de q = 0 côté `TemporalPolicy` est déjà porté par T1. Aucune autre conséquence n'est tirée ici.

## 5. Recensement : producteurs, lecteurs, consommateurs

### 5.1 Producteurs et lecteurs de production [Fait]

| Fichier | Symbole | Donnée v1 épinglée | Risque |
|---|---|---|---|
| `forge/src/batch.rs` (l. 222-236) | écriture du manifeste dans `compile_assets_with` | signature, version, `n` `u32`, entrées de 16 octets ; longueur `10 + 16·n` | **S** : octets produits (G-CM). Unifier ce bloc avec une voie v2 est le risque principal de V2-F |
| `forge/src/batch.rs` (l. 72-97) | doctests de `compile_assets` | signature, E0061, E0603, E0432 | **S** |
| `forge/src/batch.rs` | `compile_assets_with` (`pub(crate)`) | ordre des contrôles, manifeste produit en dernier | **C** |
| `forge/src/batch.rs` | `CompiledSet` | champs privés, accesseurs séparés | **C** (base de VQ-9c) |
| `forge/src/lib.rs` | exports | — | **N** (ajouts additifs seulement) |
| `assets/src/manifest.rs` (l. 21-35) | constantes `MANIFEST_*` | § 3.1 | **S** |
| `assets/src/manifest.rs` (l. 220-255) | `Manifest::parse` | § 3.2 | **S** |
| `assets/src/manifest.rs` | `Manifest` (champ privé `entries`), `entries()`, `find()`, `check_bindings()` | entrées de 16 octets, indépendantes de l'en-tête | **C** |
| `assets/src/manifest.rs` (l. 179) | documentation de `ManifestError::Version` (« différente de `MANIFEST_VERSION` ») | — | **C** (formulation à ajuster si v2 la réutilise, sans variante) |
| `assets/src/load.rs` | `load`, niveaux 1 à 4 | § 3.3 | **S** ; refactor **C** (RK-3) |

### 5.2 Tests historiques épinglant du v1
Les littéraux d'octets et leurs tests utilisateurs sont en **Annexe A** ; ce tableau ne les répète pas.

| Groupe | Tests | Contrat épinglé | Risque |
|---|---|---|---|
| Assets, manifeste | `m1` à `m3`, `m5` à `m8` | layout v1, longueur, doublons, liaisons | **S** |
| Assets, manifeste | **`m4`** (signature ; versions `0x0000`, `0x0002`, `0x0100`, `0xFFFF` → `Version`) | refus de version | **S** : seul test qui épingle `Version` |
| Assets, chargement | `l1` à `l5`, `tb1`, `ld1`, `sf1` à `sf6`, `rf1` à `rf6` | niveaux 1 à 4, tout-ou-rien, format du slot, références | **S** |
| Assets, scripts | `scr_l1` à `scr_l3` | chargement de scripts | **S**, **G-S6** |
| Assets, artefacts | `p1_*`, `tls_r*`, `map_r*`, `fnt_r*`, `scr_r*` | artefacts seuls, aucun octet `VMAN` | **N** |
| Forge, lot | **`f2`**, **`f3`**, **`f7`**, **`f8`** | manifeste littéral, permutation, manifeste vide, longueur `10 + 16·n` | **S** |
| Forge, lot | `f4`, `f5`, `f6`, `f6b`, `an_entry_count_above_u32_max…` | erreurs `ForgeError`, identifiants | **N** |
| Forge, intégration | **`vm_1`** | constantes `MANIFEST_*` et en-tête littéral d'un lot de 3 | **S** |
| Forge, intégration | **`t6_4`** | décalages en dur du layout v1 ; `DuplicateId`, `DuplicateSlotIndex`, bornes | **S** |
| Forge, intégration | `t6_6`, `t6_2`, `ts_2`, `ms_2`, `fs_2` | manifeste vide de 10 octets ; identifiants de référence | **S** |
| Forge, intégration | `t6_5`, `ts_5`, `ms_5`, `fs_5`, `ts_4`, `ms_4_and_ms_7`, `fs_4` | refus de signature | **S** |
| Forge, intégration | `t6_1` à `t6_3`, `ts_1`, `ts_3`, `ts_6`, `ms_*`, `fs_*`, `map_f*`, `scr_c1` | flux v1 `compile_assets` → `load` | **S** (consomment les octets produits) |
| Forge, scripts | `scr_f5` (`compile_assets` et `Manifest::parse`, sans littéral) | octets produits, permutation | **S**, **G-S6** |
| Shell | doctests LT-5n et LT-5t, `lt5` | signature historique de `load`, manifeste littéral | **S** |
| Shell | `level_loaded`, `script_loaded`, `script_suspend`, `script_support`, exemples `level_headless` et `script_headless` | flux v1 sans littéral | **S** (octets produits), **G-S6** pour les fichiers de l'étape 6 |
| `verbum-script` | aucun usage de `Manifest`, `load`, `LoadError`, `compile_assets` ; consomme `Script`, `SlotIndex`, `Instruction`, `SCRIPT_*` | contrat `deposit(SlotIndex, Asset<'a>)`, `Script<'a>` empruntant les octets | **N** |

### 5.3 Littéraux v1
**Annexe A** (20 littéraux hexadécimaux, 2 constructeurs `b"VMAN"`, constantes).

### 5.4 Consommateurs des erreurs
**Annexe B.**

### 5.5 Erreurs publiques et internes [Fait]

| Catégorie | Éléments | `match` exhaustif |
|---|---|---|
| Publiques, réexportées | `ManifestError`, `BindingError`, `LoadError`, `ReadError`, `ReferenceError` (`verbum-assets`) ; `ForgeError` (`verbum-forge`) | **Aucun** dans le dépôt lu |
| Publiques, non erreurs | `Format`, `Asset`, `AssetKind` | `format.rs::decode` (sur `Format`), `batch.rs` (sur `AssetKind`), internes au crate |
| Internes | `Failure`, `Decoded` (`load.rs`, `format.rs`), `Pending` (`batch.rs`) | internes |
| Hors V2 | `Instruction` | `step.rs`, `forge/src/script.rs::encode_record` ; **portent sur `Instruction`, non sur `Asset`** |

Aucune de ces énumérations n'est `#[non_exhaustive]` (aucune occurrence dans les `*.rs` de `crates`, G5). Ajouter une variante reste donc une rupture de compatibilité source pour un consommateur externe, même si aucun `match` exhaustif n'existe dans ce dépôt. Précédent : `ReadError::Instruction` (étape 6), additif.

### 5.6 Écarts documentaires historiques (consignés, non corrigés)

| # | Écart | Source |
|---|---|---|
| 1 | Document d'exécution Forge F7 : dépendance par `path` direct ; le `Cargo.toml` réel utilise `.workspace = true` | `02-…-execution`, F7 |
| 2 | Listes de `members` périmées | F1 des exécutions 05 et 06 |
| 3 | Le document 06 cite « R-ADAPT du plan 05 § 3.1 » ; il est défini dans `05-verbum-assets-execution.md` § 3.1 | `06-…-execution` § 2 |
| 4 | R-ADAPT est défini pour des changements précis de signature (`load`, champ `kind`) | `05-…-execution` § 3.1 |
| 5 | La spécification (F-16, § 7.3) sous-évalue les littéraux v1 : 20 sont recensés ; `examples/level_headless.rs` y figure comme consommateur touché alors qu'il ne lit aucun manifeste | G4 ; lecture |
| 6 | La spécification (§ 7.2, point 5) laisse « à fixer au plan V2 » l'erreur de refus d'un v2 par le chemin historique ; `parse` rend déjà `Version` | `manifest.rs` |
| 7 | La définition de G-CM (`05-…-execution` § 3.2) cite « modification de `VMAN` (constantes, layout, octets produits) » ; la roadmap la qualifie (« layout `VMAN` v1 modifié, octets v1 changés ») | § 11.2 |

**Note de traçabilité (non écart).** Les en-têtes de `02-verbum-forge.md` (r8) et de `05-verbum-assets.md` étaient historiquement périmés par rapport à l'état de l'implémentation. L'utilisateur les a corrigés lui-même le 16 novembre 2026 : `02-verbum-forge.md` : adopté ; `05-verbum-assets.md` : adopté. Cette mission n'a modifié aucun de ces deux documents ; la hiérarchie de la mission (plans validés des étapes 2, 5 et 6) s'applique donc littéralement.

---

## 6. R-ADAPT et garde-fous spécifiques à V2

### 6.1 Périmètre
Étapes 2, 5 et 6 (ARB-4, exigence de J-V2). Tests sensibles : § 5.2 et Annexe A.

### 6.2 R-ADAPT historique [Fait]
Défini en `05-verbum-assets-execution.md` § 3.1 : seules sont admises, dans un test de l'étape 2, les modifications purement mécaniques imposées par la signature de `load` (table de formats à la place de `slot_count`, `deposit` recevant un `Asset`) ou par le champ `kind` d'`AssetSource`, consignées une à une ; toute modification d'une assertion, d'une valeur littérale attendue, du nom ou du motif d'un test est un arrêt G-CM. Ces autorisations portaient sur des changements passés.

### 6.3 Garde-fous spécifiques à V2 [Proposition]

| # | Garde-fou |
|---|---|
| GV-1 | Objectif sous V-a : **zéro adaptation** du chemin v1 |
| GV-2 | Aucune assertion, aucun littéral, aucun nom ni motif de test, aucun comportement v1 n'est modifié |
| GV-3 | Les vérifications v2 sont des **ajouts** ; elles ne remplacent ni ne reformulent un test historique |
| GV-4 | Aucune autorisation d'adaptation mécanique n'est ouverte par ce plan ; l'autorisation historique (§ 6.2) n'est pas reconduite par interprétation |
| GV-5 | Toute modification d'un test, d'une assertion, d'un littéral ou d'un comportement v1, ou toute adaptation de signature, entraîne **arrêt et arbitrage explicite** |
| GV-6 | Un ajout dans un fichier de tests existant ne modifie aucune assertion historique mais modifie le fichier : l'emplacement des tests additifs est **[Ouvert]**, réservé à V2-C et V2-P (R-C10) |
| GV-7 | La consignation d'une éventuelle exception, une à une, se fait au compte rendu de l'unité concernée |

### 6.4 Matrice de correspondance
Test historique sensible → contrat épinglé → vérification v2 additive → unité. Les littéraux détaillés sont en Annexe A.

| Test(s) historique(s) | Contrat épinglé | Vérification v2 additive (nom à fixer à l'exécution) | Unité |
|---|---|---|---|
| `m4` | refus de version par `parse` | VQ-4 (un v1 ne produit jamais q par le lecteur v2) ; VQ-3 pour le v2 | V2-L |
| `m1`–`m3`, `m5`, `m7` | layout v1, longueur, doublons, liaisons | VQ-1, VQ-3 (équivalents v2 : longueur `18 + 16·n`, ordre d'octets) | V2-L |
| `vm_1` | constantes v1 | constantes v2 distinctes (VQ-1, VQ-3) ; `vm_1` inchangé | V2-L |
| `l1`–`l5`, `tb1`, `ld1`, `sf1`–`sf6`, `rf1`–`rf6` | niveaux 1 à 4, tout-ou-rien, références | VQ-6 : mêmes propriétés pour le chemin v2 | V2-P |
| `l3` (niveau 1, `Signature`), tests `*_rejected` | rejet avant dépôt | VQ-9b : v2 refusé par le chargement historique, aucun dépôt | V2-P |
| LT-5n, LT-5t, `lt5` | signature historique de `load` | VQ-9a, VQ-9b | V2-P |
| `f2`, `f3` | octets v1 produits, permutation | VQ-5 : voie v1 identique ; permutation côté v2 | V2-F |
| `f7`, `t6_6` | manifeste vide de 10 octets | VQ-5 (cas vide v2, sans modifier ces tests) | V2-F |
| `f8` | longueur `10 + 16·n`, absence de chaînes | VQ-3, VQ-5 | V2-L, V2-F |
| `t6_4` | décalages v1 en dur | VQ-3 (altérations v2 additives) | V2-L |
| `REFERENCE_MANIFEST` et ses utilisateurs | identifiants de référence | VQ-5 : mêmes identifiants, q différent → manifeste différent | V2-F |
| `t6_5`, `ts_5`, `ms_5`, `fs_5`, `ts_4`, `ms_4_and_ms_7`, `fs_4` | refus de signature | VQ-4, VQ-9d | V2-L, V2-P |
| `scr_l*`, `scr_f5`, tests Shell de l'étape 6 | octets et chargement de scripts | VQ-7 (verts, inchangés) | toutes |
| Flux v1 du Shell et de `pipeline.rs` | `compile_assets` → `load` | VQ-7 | toutes |

### 6.5 Procédure d'arrêt
Constat factuel (source, texte, observation), exposé des options minimales, aucune modification d'office, reprise seulement sur instruction explicite.

---

## 7. VQ-1 à VQ-9

### 7.1 Matrice

| VQ | Énoncé (spéc. § 7.3) | Unité | Tests historiques voisins | Notes |
|---|---|---|---|---|
| VQ-1 | Manifeste v2 littéral écrit à la main, indépendant des constantes : `parse` v2 rend q attendu | V2-L | `m1` | — |
| VQ-2 | q = 0 refusé à trois frontières : octets (niveau 1), Forge, `TemporalPolicy` | V2-L (octets), V2-F (Forge) | `p2_3`, `scr_f3` (`ZeroValue`) | Octets : **B-1 [Ouvert]**. Forge : `ZeroValue("q")` [Arbitré]. `TemporalPolicy` : refus déjà porté par T1 (vérifié dans `policy.rs`, § 4.9 : `PolicyError::ZeroQuantum`) |
| VQ-3 | Longueur exacte `18 + 16·n` : tronqué, octets en trop, `n` incohérent ; ordre d'octets explicite | V2-L | `m2`, `m3`, `t6_4`, `f8` | little-endian (ARB-10) |
| VQ-4 | Un manifeste v1 ne produit **jamais** un q ; refus typé avant toute progression | V2-L | `m4` | `Version` réutilisée [Arbitré] |
| VQ-5 | Aller-retour Forge → paquet → lecture : q égal ; v1 : mêmes octets que `compile_assets` ; invariance à l'ordre ; q différent → manifeste différent, artefacts identiques | V2-F | `f2`, `f3`, `f7` | Condition : production v1 octet pour octet inchangée |
| VQ-6 | Régression : ordre et nombre des niveaux, tout-ou-rien, absence d'allocation, `cargo tree` inchangé | V2-P, V2-Z | `l1`–`l5`, `sf*`, `rf*` | — |
| VQ-7 | Suites des étapes 2, 5 et 6 : aucune assertion ni littéral modifié | toutes | tout § 5.2 | R-ADAPT |
| VQ-8 | Bout en bout : Forge (q) → paquet → chargement → politique ; v1 → erreur typée avant toute progression | **F3b** | — | **Dépendance de F3b, hors critères de sortie de V2-L, V2-P et V2-F.** Non comptée comme validation V2 |
| VQ-9 | Intégrité structurelle du couple : (a) pas de chemin public v2 à deux entrées ; (b) le chargement à deux entrées refuse un manifeste v2 ; (c) un paquet A/B recomposé ne peut pas être produit par la chaîne normale ; (d) un paquet v1 sans q n'est pas chargeable comme jeu conforme S04 | V2-P (a, b, d), V2-F (c) | tests de refus de signature | (c) : le constructeur de paquet est l'unique producteur |

### 7.2 VQ-9b

| Élément | Contenu |
|---|---|
| Constat [Fait] | `Manifest::parse` rejette déjà `version ≠ 1` par `ManifestError::Version` (G5 : `Version` n'est référencée que dans `manifest.rs`, constructeur et `m4`). **Ce refus n'est asserté nulle part à la frontière de `load`** |
| Répartition [Arbitré] | **V2-L** vérifie la lecture du manifeste et le rejet des versions incompatibles ; **V2-P** vérifie le comportement du chargement historique face à un manifeste v2, notamment l'**absence de dépôt** |
| Assertion visée [Proposition] | chargement historique + manifeste v2 → `LoadError::Manifest(ManifestError::Version)` ; compteur de dépôts nul |
| Contrainte [Arbitré] | vérification **additive** ; `m4` inchangé |
| Réserve | Si V2-C estime que `Version` ne correspond pas à la sémantique du rejet, toute autre erreur doit être **justifiée** et arbitrée (ARB-6) |
| Emplacement du test | **[Ouvert]** : V2-C et V2-P (GV-6) |

---

## 8. Besoins d'erreurs B-1 à B-6

| # | Besoin | État | Contenu et contraintes |
|---|---|---|---|
| B-1 | q = 0 refusé au niveau 1 (Assets) | **[Ouvert] V2-C** | Aucune variante de `ManifestError` ne convient (`Truncated`, `Signature`, `Version`, `TrailingBytes`, `DuplicateId`). `Version` n'est pas détournée (ARB-14). Contraintes : compatibilité source des énumérations publiques sans `#[non_exhaustive]` ; aucun `match` exhaustif dans le dépôt lu ; pistes à examiner **sans choix** : ajout d'une variante additive, type d'erreur propre au lecteur v2, autre représentation |
| B-2 | Refus d'un v2 par le chemin historique | Couvert par `Version` | Assertion à la frontière de `load` : VQ-9b (V2-P) |
| B-3 | Refus d'un v1 par le chemin v2 | `ManifestError::Version` [Arbitré] | Ajustement de la documentation de `Version` (cite `MANIFEST_VERSION`), sans variante |
| B-4 | Erreurs de l'enveloppe du paquet | **[Ouvert] V2-C** | Besoin décrit, sans variante ; à examiner avec B-1 |
| B-5 | q = 0 côté Forge | `ForgeError::ZeroValue("q")` [Arbitré] | Contrat actuel : « le champ vaut zéro » ; précédent direct : `script.rs`, `ZeroValue("ticks")` construit sans l'assistant `dimension` ; `OutOfRange` (documenté pour `u16`) ne convient pas ; aucune variante |
| B-6 | Paquet v1 ou sans q côté application | **[Dépendance] F3b** | Hors Assets et Forge (VQ-8) |

**Règles.** Aucune variante d'erreur publique n'est ajoutée par anticipation. Tout besoin avéré est présenté pour arbitrage, avec l'inventaire des consommateurs (Annexe B) et la mention de la rupture de compatibilité source possible.

---

## 9. Risques et questions réservés à V2-C

Aucune solution n'est proposée ici.

| # | Sujet réservé | Contrainte factuelle à rappeler |
|---|---|---|
| R-C1 | Préparation technique avant le niveau 1 | Aucun nouveau niveau (ARB-1) |
| R-C2 | Représentation du paquet, en-tête, ordre d'octets, largeurs | D-1d fixe seulement l'en-tête du manifeste |
| R-C3 | Accès aux artefacts, offsets, chevauchements, conversions vérifiées | `Manifest::parse` et chaque lecteur d'artefact exigent une longueur exacte (`TrailingBytes`) |
| R-C4 | Absence d'allocation face à `&[&[u8]]` | Dans `load`, `artifacts` n'est utilisé qu'à trois endroits (nombre d'artefacts, lecture par position dans `artifact_bytes`) ; toute refonte de `load.rs` relève de RK-3 |
| R-C5 | Réutilisation des niveaux 1 à 4 sans duplication | `Manifest` a un champ privé ; `entries()`, `find()`, `check_bindings()` ne dépendent pas de l'en-tête |
| R-C6 | Refus croisés (VQ-9a, b, d) | `Version` est déjà rendue par `parse` pour un v2 |
| R-C7 | Unique producteur du paquet (VQ-9c) | `CompiledSet` n'a pas de constructeur public ; `load` v1 accepte n'importe quel couple de tranches |
| R-C8 | Lecture de q ; noms et visibilité des constantes v2 | Ne pas réutiliser les constantes historiques ; collision `HEADER_LEN` |
| R-C9 | Erreurs B-1 et B-4 | Énumérations publiques sans `#[non_exhaustive]` ; aucun `match` exhaustif dans le dépôt lu |
| R-C10 | Emplacement des tests additifs | Éviter de modifier des fichiers de tests sensibles (GV-6) |
| R-C11 | Nom de la voie de compilation | `compile_assets` et son doctest restent inchangés ; modules `batch` et `id` privés |
| R-C12 | Prémisse « octets immuables » et accès par projection mémoire | Plan 02, R14 et § 5.4 ; ADR-A01 pt 2 évoque `mmap` ; non requis par V2 |
| R-C13 | Cohérence de la lecture de q avec `TemporalPolicy::from_nanos(u64, u32, u32)` | q en nanosecondes `u64` côté Shell (roadmap) |

---

## 10. Unités

Contraintes transverses héritées des étapes 5 et 6 : quatre contrôles à chaque unité (`cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`) exécutés par l'utilisateur, résultats de l'espace de travail de la session séparés de ceux du dépôt réel ; MSRV 1.85 en **deux commandes** (`cargo +1.85.0 build --workspace`, puis `cargo +1.85.0 test --workspace`) ; pas de chaînes `if let … &&` ; aucune constante utilisée seulement dans un `const _: () = assert!(…)` ; Miri exécuté par l'utilisateur. Les fichiers listés sont des **prévisions à confirmer**. Commits : regroupement proposé par la session, commits à la charge de l'utilisateur.

### 10.1 V2-0 — ce plan
| Rubrique | Contenu |
|---|---|
| Objectif | Produire le plan du chantier V2, soumis à G-CM, validé avant toute modification de `VMAN`, d'Assets ou de la Forge (D-1c) |
| Livrable | Ce document ; aucun code |
| Porte | **P-V2** (§ 11.1) |
| Non-décisions | Layout du paquet (V2-C), durées non entières (D-N2) |

### 10.2 V2-C — note de conception du paquet (annexe du plan V2)
| Rubrique | Contenu |
|---|---|
| Objectif | Établir, avant tout code du chargeur v2, la conception du paquet |
| Statut | **Annexe du plan V2 à statut distinct** (ARB-19) ; ne modifie pas le présent corps du plan |
| Sujets (liste de la roadmap, § 8) | layout, accès par index, limites et contrôles d'offset, absence d'allocation, réutilisation des niveaux 1 à 4 sans duplication, refus croisés, lecture de q et erreurs additives, unique producteur |
| Contraintes déjà établies | `Manifest::parse` strictement v1 (`m4` épingle `0x0002` → `Version`) ; lecteur v2 additif ; littéraux v1 intacts (V-a) |
| Peut préciser | layout, accès indexé, gestion mémoire, erreurs du paquet, noms et visibilité des constantes v2, emplacement des tests additifs |
| Sortie | Note validée = **P-CONC** (D-N4) |
| Prérequis | P-V2 |

### 10.3 V2-L — lecteur v2 du manifeste (`verbum-assets`)
| Rubrique | Contenu |
|---|---|
| Objectif | Lire un manifeste v2 portant q une seule fois (D-1d) ; refuser q nul au niveau des octets (**B-1 [Ouvert]**) ; refuser les versions incompatibles ; `parse` intact |
| Prérequis | P-V2 et P-CONC |
| Fichiers (prévisions) | `assets/src/manifest.rs` ou module additif (emplacement : V2-C) ; `assets/src/lib.rs` (exports additifs) ; `Cargo.toml` inchangé |
| Tests | VQ-1, VQ-2 (octets), VQ-3, VQ-4 ; VQ-7 : `m1` à `m8` inchangés |
| Familles de mutation | endianness de `q_ns` ; longueur d'en-tête ; q nul accepté ; v1 accepté comme v2 |
| Non-régression | § 5.2, groupes Assets ; `cargo tree -p verbum-assets` vide |
| Acceptation | VQ-1 à 4 et 7 verts ; aucune dépendance ; aucune allocation ; quatre contrôles |
| Miri | `cargo +nightly miri test -p verbum-assets` (suite complète) |
| Portes | G-CM, R-ADAPT, G-S6 |

### 10.4 V2-P — paquet et chargeur v2 (`verbum-assets`)
| Rubrique | Contenu |
|---|---|
| Objectif | Chargement du paquet par une entrée unique, réutilisant les niveaux 1 à 4 selon V2-C ; refus croisés ; **VQ-9b** |
| Prérequis | V2-L |
| Fichiers (prévisions) | module additif ; `assets/src/load.rs` **seulement si V2-C la retient** (niveaux, ordre, atomicité inchangés) ; `lib.rs` |
| Tests | VQ-6 ; VQ-9 (a, b, d) ; `l1`–`l5`, `sf*`, `rf*` inchangés |
| Familles de mutation | dépôt avant le niveau 3 ; niveaux inversés ; accès hors limite accepté ; chevauchement accepté ; chemin à deux entrées v2 conservé |
| Non-régression | § 5.2 ; `scr_l*` (G-S6) |
| Acceptation | VQ-6 et VQ-9 verts ; quatre contrôles ; Miri intégral Assets |
| Portes | G-CM (tout-ou-rien, ordre des niveaux, absence d'allocation), R-ADAPT, G-S6 |

### 10.5 V2-F — voie de compilation additive (`verbum-forge`)
| Rubrique | Contenu |
|---|---|
| Objectif | Recevoir q explicitement ; refuser q = 0 par `ForgeError::ZeroValue("q")` ; produire le paquet (unique producteur) ; laisser `compile_assets` et son doctest inchangés ; production v1 octet pour octet identique |
| Prérequis | V2-P |
| Fichiers (prévisions) | `forge/src/batch.rs` ou module additif ; `lib.rs` ; `forge/src/sprite.rs` seulement si une variante est arbitrée ; `tests/pipeline.rs` (ajouts seulement, GV-6) |
| Tests | VQ-2 (Forge), VQ-5, VQ-9c |
| Familles de mutation | q non écrit ; q écrit deux fois ; octets v1 altérés ; paquet recomposable hors constructeur |
| Non-régression | `f2`, `f3`, `f7`, `f8`, `vm_1`, `t6_4`, `t6_6`, `scr_f5` inchangés |
| Acceptation | VQ-2, 5, 9c verts ; `cargo tree -i toml --workspace` (inchangé, formulation de la roadmap) ; `cargo tree -i verbum-forge --workspace -e normal` (aucun dépendant) |
| Miri | `cargo +nightly miri test -p verbum-forge -- --skip p3_1 --skip p3_2 --skip t6_1 --skip t6_3 --skip t6_6` : **partiel** (exclusions connues de l'étape 5), rapporté comme tel |
| Portes | G-CM (octets v1 produits), R-ADAPT, G-S6 |

### 10.6 V2-Z — clôture
| Rubrique | Contenu |
|---|---|
| Objectif | Régression globale et consignation |
| Prérequis | V2-F |
| Contenu | VQ-6 et VQ-7 sur l'ensemble ; consommateurs § 5.2 verts ; `cargo tree` (Assets vide, Forge, `toml` isolé) ; Miri et MSRV ; synthèses et renvois dans les comptes rendus Assets et Forge, **sans duplication de ce plan** ; liste « non démontré » |
| Porte | **P-E** |

---

## 11. Portes et critères

### 11.1 Critères P-V2 [Proposition]
La validation de ce plan par l'utilisateur vaut P-V2 si les critères suivants sont satisfaits :

| # | Critère |
|---|---|
| 1 | Recensement et sources documentés, réserve d'exhaustivité formulée (§ 2, Annexe C) |
| 2 | Matrice R-ADAPT complète, tests sensibles identifiés, garde-fous V2 distincts du R-ADAPT historique (§ 6) |
| 3 | VQ-1 à VQ-9 reprises avec leur unité ; VQ-8 avec son statut de dépendance (§ 7) |
| 4 | Besoins d'erreurs B-1 à B-6 décrits, aucune variante ajoutée (§ 8) |
| 5 | Sujets réservés à V2-C énumérés, aucun layout, aucune structure mémoire, aucune représentation du paquet (§ 9) |
| 6 | Unités V2-0 à V2-Z, fichiers prévus, tests, mutations, portes (§ 10) |
| 7 | Chaîne d'exécution et de traçabilité (§ 12) |
| 8 | Arbitrages consignés, décisions ouvertes listées (§ 4.2, § 14) |
| 9 | Annexes A, B, C présentes ; document autonome |

### 11.2 G-CM
Source : plan 05 § 1.3 point 2 ; `05-…-execution` § 3.2 ; roadmap § 9.

| Élément protégé | Événement qui déclenche l'arrêt avec rapport | Test témoin |
|---|---|---|
| Niveaux de `load` | création ou suppression d'un niveau ; modification de leur ordre | `l1`–`l5`, `sf*`, `rf*` |
| Atomicité | dépôt avant la fin du niveau 3 | `l3`, `t6_4` |
| `VMAN` v1 | modification des constantes, du layout ou des octets produits pour un ensemble donné | `vm_1`, `f2`, `f3`, `m1`–`m8` |
| Allocation | allocation dans `load` | `[dependencies]` vide, `no_std` sans `alloc` |
| Identité | modification de `AssetId` opaque, de la distinction `ArtifactIndex` / `SlotIndex`, ou ajout d'une chaîne ou d'un chemin dans un artefact | doctests `compile_fail`, `f8` |

**Lecture retenue pour les constantes v2** [Arbitré, ARB-3 ; D-1c] : l'ajout de constantes v2 **distinctes et additives**, dans le cadre du chantier V2 soumis à G-CM, n'est pas une modification des constantes v1. Cette lecture n'élargit aucune autorisation historique (§ 5.6, écart 7).

### 11.3 Autres portes
| Porte | Condition |
|---|---|
| **P-CONC** | Note de conception V2-C (annexe à statut distinct) validée par l'utilisateur, **avant toute implémentation de V2-L ou V2-P** |
| **P-E** | V2-Z validée ; VQ-1 à VQ-7 et VQ-9 verts ; tests des étapes 2, 5 et 6 inchangés |
| **R-ADAPT** | § 6 |
| **G-S6** | Étape 6 close : aucune modification de ses documents ou sources ; toute incompatibilité concrète signalée et arbitrée |
| Contrôles communs | § 10 ; `cargo tree -p verbum-assets` sans dépendance, dev comprises |

---

## 12. Chaîne d'exécution et traçabilité

### 12.1 Chaîne (D-1c)
(1) Plan V2 validé (**P-V2**) ; (2) note V2-C validée (**P-CONC**) ; (3) document d'exécution validé ; (4) handoff validé ; (5) implémentation par unités V2-L, V2-P, V2-F, V2-Z sous G-CM ; (6) clôture avec rapport (**P-E**). F2 et F3 ne sont débloquées qu'à l'issue de la chaîne (F3 dès que la lecture de q du paquet est livrée).

### 12.2 Noms [Arbitré]
| Document | Chemin |
|---|---|
| Plan (ce document) | `docs/implementation/03-verbum-shell-temporal-policy-vman-v2.md` |
| Exécution | `docs/implementation/03-verbum-shell-temporal-policy-vman-v2-execution.md` |
| Handoff | `docs/handoffs/03-verbum-shell-temporal-policy-vman-v2-implementation-handoff.md` |

### 12.3 Règle de session
Une unité par session par défaut (D-N6) ; regroupement seulement après arbitrage explicite.

### 12.4 Enregistrement [Arbitré]
Le plan V2 est la référence centrale. À V2-Z, les comptes rendus Assets et Forge reçoivent des consignations additives sous forme de synthèses et de renvois vers ce plan, sans duplication.

### 12.5 Matrice de traçabilité

| Exigence | Source | Section | VQ | Unité | Porte |
|---|---|---|---|---|---|
| q unique, cohérence avant exécution | ADR-S04 pt 2 | § 4.1 | VQ-8, VQ-9 | V2-P, V2-F, F3b | G-CM |
| q global dans l'en-tête, `q_ns` `u64`, 18 octets | D-1, D-1d | § 4.1 | VQ-1, VQ-3 | V2-L | G-CM |
| q strictement positif | ADR-S04 pt 2 | § 8 | VQ-2 | V2-L, V2-F | — |
| v1 intact, mêmes octets, jamais accepté pour S04 | D-1b | § 3, § 6 | VQ-4, VQ-5, VQ-7 | V2-L, V2-F | G-CM, R-ADAPT |
| Chantier sous G-CM | D-1c | § 11 | VQ-6 | toutes | G-CM |
| Paquet unique | R-1 | § 4.1, § 9 | VQ-9 | V2-P, V2-F | P-CONC |
| Aucune allocation | ADR-000 pts 1 et 4 | § 4.7 | VQ-6 | V2-P | G-CM |
| Identité opaque, aucune chaîne ni chemin | ADR-A01 pt 3 | § 3.5 | VQ-6, VQ-7 | V2-P, V2-F | G-CM |
| `compile_assets` inchangé | D-1b, spéc. § 7.2 | § 3.4 | VQ-5 | V2-F | G-CM |
| Scripts non modifiés | ARB-9 | § 5.2 | VQ-7 | toutes | G-S6 |

---

## 13. Risques de fermeture prématurée

| # | Risque | Atténuation |
|---|---|---|
| RF-1 | Un layout, une structure mémoire ou un accès indexé est décidé hors de V2-C | § 4.8, § 9 ; P-CONC |
| RF-2 | La préparation avant le niveau 1 devient un niveau déguisé | ARB-1 ; § 11.2 |
| RF-3 | Une variante d'erreur est ajoutée par anticipation | § 8 ; ARB-5 |
| RF-4 | Un refactor de `load.rs` est jugé « sans changement de mécanisme » | RK-3 ; G-CM ; tests `l*`, `sf*`, `rf*` inchangés |
| RF-5 | Une constante v1 est réutilisée avec une nouvelle signification | § 4.5 ; `vm_1` |
| RF-6 | Un test historique est modifié pour faire passer v2 | GV-1 à GV-5 |
| RF-7 | L'exhaustivité du recensement est présumée | § 2.4 ; Annexe C |
| RF-8 | La compatibilité de lecture est confondue avec la compatibilité d'exécution | D-1b ; VQ-4, VQ-8, VQ-9d |
| RF-9 | VQ-8 est compté dans les critères V2 | § 7 ; ARB-18 |
| RF-10 | Une adaptation « mécanique » est supposée admise par analogie avec l'étape 5 | GV-4 |
| RF-11 | Les 20 littéraux sont supposés tous recensés | Annexe A, limites |

---

## 14. Décisions

### 14.1 Adoptées
ARB-1 à ARB-23 (§ 4.2) ; D-1, D-1b, D-1c, D-1d, R-1, D-N3, D-N6 (§ 4.1, § 12).

### 14.2 Ouvertes

| Décision | Réservée à | Remarque |
|---|---|---|
| D-N4 : layout du paquet | V2-C (P-CONC) | — |
| B-1 : q = 0 au niveau des octets | V2-C | `Version` exclue |
| B-4 : erreurs de l'enveloppe | V2-C | — |
| Noms et visibilité des constantes v2 ; nom de la voie de compilation | V2-C | ARB-22 |
| Emplacement des tests additifs, dont VQ-9b | V2-C et V2-P | GV-6 |
| Assertion exacte de VQ-9b | V2-P | `Version` proposée |
| D-N2 : durées non entières | F2 | G-ARB-F2 |
| D-N5 : portée de `games/demo` | F3a | — |
| D-N1 : valeurs q, K, M | F3c | — |

---

## 15. Vérification de cohérence et points non démontrés

### 15.1 Cohérence avec les documents de référence

| Référence | Constat |
|---|---|
| ADR-A01 pts 3 et 4 | Le layout du manifeste est « hors périmètre — implémentation » : aucune ADR n'est modifiée ; l'identité opaque et l'absence de chaîne sont préservées |
| ADR-S04 pt 2 | Le plan est le « cadre documentaire approprié » ; il ne réalise aucune modification de `VMAN` |
| ADR-000 pts 1 et 4 | No-allocation maintenue (§ 4.7) |
| Spécification r3 § 7 | D-1 à D-1d, R-1, VQ-1 à VQ-9 repris ; VQ-8 déplacé vers F3b conformément à la roadmap (D-7) ; écarts 5 et 6 du § 5.6 |
| Roadmap r1.1 | Fiches V2-0, V2-C, V2-L, V2-P, V2-F, V2-Z conformes ; V2-C annexée au plan ; portes P-V2, P-CONC, P-E |
| Plan 05 § 1.3, exécution § 3.1 et § 3.2 | G-CM et R-ADAPT reproduits ; écart 7 du § 5.6 |
| Plans 02 et 06 | Constantes, ordre des contrôles, tests recensés confirmés par le code lu ; `ReadError::Instruction` cité comme précédent additif |

### 15.2 Points non démontrés
Vérifications faites par relecture des fichiers fournis avant livraison : noms et lignes des tests de `load.rs`, `manifest.rs`, `batch.rs`, `pipeline.rs`, `script.rs` (Assets et Forge) ; offsets et contenu de `t6_4`, `t6_6`, `vm_1` ; usages des littéraux par test ; révisions et statuts d'en-tête des plans 02, 05 et 06 ; identifiants de la roadmap cités. Aucune de ces vérifications n'est une exécution.

| Point | Statut |
|---|---|
| Exhaustivité du recensement | Non démontrée (§ 2.4) |
| Exécution de contrôles | Aucune commande exécutée par cette mission |
| Dépôt réel postérieur aux fichiers fournis | Non vérifié |
| `PolicyError::ZeroQuantum` dans le code | Vérifié par relecture (§ 4.9) ; aucune exécution |
| Production v1 octet pour octet avec une voie additive | À établir en V2-F |
| Chargeur à entrée unique sans allocation | À établir en V2-C |
| Compatibilité source de B-1 et B-4 | Non évaluée |
| Durées de Miri des tests additifs | Non mesurées |

---

## Annexe A — Littéraux et constantes v1 (inventaire détaillé)

Lignes issues de G4 (littéraux hexadécimaux) et de G1 (`b"VMAN"`), au 15 novembre 2026.

### A.1 Littéraux hexadécimaux (20)

| # | Fichier | Ligne | Symbole | Tests utilisateurs | Contenu | Risque |
|---|---|---|---|---|---|---|
| 1 | `assets/src/manifest.rs` | 387 | `LITERAL` | `m1`, `m3` (3 tests), `m4` (2), `m5` | 42 octets, version 1, `n = 2` | S |
| 2 | `assets/src/load.rs` | 181 | `MANIFEST` | `l1`, `l2`, `l3` (×4), `l5`, `tb1`, `ld1`, `sf1`, `sf3` | 42 octets, `n = 2` | S |
| 3 | `assets/src/load.rs` | 343 | `empty` (inline) | `l4` (manifeste vide, collection vide) | 10 octets, `n = 0` | S |
| 4 | `assets/src/load.rs` | 359 | `single` (inline) | `l4` (capacité supérieure au nombre d'assets) | 26 octets, `n = 1` | S |
| 5 | `assets/src/load.rs` | 379 | `empty` (inline) | `l4` (manifeste vide, collection non vide) | 10 octets, `n = 0` | S |
| 6 | `assets/src/load.rs` | 426 | `SINGLE` | `sf2`, `sf4`, `sf5` | 26 octets, `n = 1` | S |
| 7 | `assets/src/load.rs` | 624 | `IDENTITY_MANIFEST` | `sf6` | 42 octets, `artifact_index = slot_index` | S |
| 8 | `assets/src/load.rs` | 685 | `FOUR` | `rf1`, `rf2`, `rf3`, `rf5`, `rf6` (via l'assistant `load_four`) | 74 octets, `n = 4` | S |
| 9 | `assets/src/load.rs` | 786 | `PAIR` | `rf4` | 42 octets, `n = 2` | S |
| 10 | `assets/src/script.rs` | 583 | `SINGLE` | `scr_l1` | 26 octets, `n = 1` | S, G-S6 |
| 11 | `assets/src/script.rs` | 643 | `MANIFEST` (inline) | `scr_l2` | 42 octets, `n = 2` | S, G-S6 |
| 12 | `assets/src/script.rs` | 680 | `MANIFEST` (inline) | `scr_l3` | 58 octets, `n = 3` | S, G-S6 |
| 13 | `shell/src/lib.rs` | 334 | `MANIFEST` (doctest) | LT-5n | 42 octets, `n = 2` | S |
| 14 | `shell/src/lib.rs` | 471 | `MANIFEST` (doctest) | LT-5t | 42 octets, `n = 2` | S |
| 15 | `shell/tests/borrowed_resource.rs` | 243 | `MANIFEST` | `lt5` | 42 octets, `n = 2` | S |
| 16 | `forge/src/batch.rs` | 285 | `expected_manifest` | `f2` | 42 octets, identifiants RFC 9923 | S |
| 17 | `forge/src/batch.rs` | 307 | `expected_manifest` | `f3` | 42 octets, entrées permutées | S |
| 18 | `forge/src/batch.rs` | 553 | `expected` | `f7` | 10 octets, `n = 0` | S |
| 19 | `forge/tests/pipeline.rs` | 328 | `REFERENCE_MANIFEST` | `t6_2`, `ts_2`, `ms_2`, `fs_2` | 42 octets, identifiants RFC 9923 | S |
| 20 | `forge/tests/pipeline.rs` | 991 | en-tête littéral (inline) | `vm_1` | 10 octets, `n = 3` | S |

### A.2 Autres épinglages du layout v1

| Fichier | Ligne | Symbole | Tests | Contenu | Risque |
|---|---|---|---|---|---|
| `assets/src/manifest.rs` | 411 | `b"VMAN"` dans `synthetic` | `m2`, `m5` | générateur à tampon `10 + 16·3` | S |
| `assets/src/manifest.rs` | 597 | `b"VMAN"` dans `manifest_of` | `m7` | générateur à tampon `10 + 16·3` | S |
| `assets/src/manifest.rs` | 522 | `MANIFEST_HEADER_LEN` | `m4` (signature) | seul usage d'une constante dans un test | S |
| `assets/src/manifest.rs` | 21-35 | constantes et `const _` | — | § 3.1 | S |
| `forge/src/batch.rs` | 222-236 | écriture du manifeste | `f2`, `f3`, `f7`, `f8` | production v1 | S |
| `forge/tests/pipeline.rs` | 972-975 | constantes dans `vm_1` | `vm_1` | `VMAN`, 1, 10, 16 | S |
| `forge/tests/pipeline.rs` | 417-485 | `t6_4` (et assistant `patched`) | `t6_4` | décalages en dur : entrée 0 à 10, entrée 1 à 26 ; champs +0, +8, +12 ; écritures aux offsets 26, 34, 38 | S |
| `forge/tests/pipeline.rs` | 502-512 | `t6_6` (`empty.manifest.len() == 10`) | `t6_6` | manifeste vide de 10 octets, dans un test de capacité | S |

### A.3 Réserve
L'inventaire repose sur G1 et G4 et sur la lecture des fichiers fournis ; il ne prouve pas l'absence d'autres littéraux (§ 2.4).

---

## Annexe B — Consommateurs des variantes d'erreur

Source : G5 (formes `Énumération::Variante`), au 15 novembre 2026. Aucun `match` exhaustif : usages par construction, `assert_eq!`, `matches!`, `let … else`, `map_err`, `.err()`.

| Énumération | Crate définisseur : constructions et tests | Autres crates | Observation |
|---|---|---|---|
| `ManifestError` | `manifest.rs` (constructions l. 222-251 ; tests l. 473-562) ; `load.rs` (l. 65, 308) | `forge/tests/pipeline.rs` : `DuplicateId` (l. 427), `Signature` (l. 493, 497, 695, 734, 738, 1174, 1223, 1227, 1451, 1533, 1537) | `Version` : `manifest.rs` seulement (l. 228, 536) |
| `BindingError` | `manifest.rs` (l. 290-322 ; tests l. 625-686) ; `load.rs` (l. 68, 323, 335, 389, 460, 575) | `forge/tests/pipeline.rs` (l. 434, 441, 448, 454, 458, 482, 701, 1180, 1457) | 6 variantes toutes assertées |
| `ReadError` | lecteurs `sprite.rs`, `tileset.rs`, `map.rs`, `font.rs`, `script.rs`, `header.rs` et leurs tests ; `load.rs` (l. 276, 293, 445, 596, 617) | `forge/tests/pipeline.rs` (l. 126, 464, 469, 724, 1202, 1474-1501) ; `shell/tests/script_loaded.rs` (l. 232, 238, 244) ; `shell/tests/level_loaded.rs` (l. 820) | `Instruction` : précédent additif de l'étape 6 |
| `LoadError` | `load.rs` (l. 65-75, 119-120 ; tests) ; `assets/src/script.rs` (l. 627, 705) | `forge/tests/pipeline.rs` ; `shell/tests/script_loaded.rs` (l. 216, 252) ; `shell/tests/level_loaded.rs` (l. 819, 835, 846) | `map_err(LoadError::Manifest)` dans `load.rs` |
| `ReferenceError` | `load.rs` (l. 140, 142 ; tests l. 748, 763, 777, 834) | `forge/tests/pipeline.rs` (l. 1214) ; `shell/tests/level_loaded.rs` (l. 847) | — |
| `ForgeError` | `forge/src/{script, tileset, font, map, sprite, batch}.rs` (constructions et tests) | `forge/tests/pipeline.rs` (`InvalidSource`, `ReferenceNotFound`, `ReferenceKind`, `CellOutOfRange`, l. 811-908) ; `shell/tests/script_suspend.rs` (l. 685, `BudgetExceeded`) | Non `Copy` (porte `String` et `Box<ForgeError>`) ; `ZeroValue` : `sprite.rs` (l. 120, via `dimension`), `script.rs` (l. 169, `"ticks"` direct) |
| `non_exhaustive` | — | — | **Aucune occurrence** dans les `*.rs` de `crates` |

---

## Annexe C — Sources, recherches et limites du recensement

### C.1 Recherches (exécutées par l'utilisateur, depuis la racine du dépôt)

| Id | Commande | Périmètre |
|---|---|---|
| G1 | `grep -rnE "Manifest\|MANIFEST_\|ManifestError\|LoadError\|compile_assets\|CompiledSet\|verbum_assets\|verbum_forge\|VMAN\|slot_formats\|b\"VMAN\"\|load\(" crates --include=*.rs --include=*.toml` | symboles du format et du chargement dans `crates` |
| G2 | `grep -rln "VMAN" docs` | documents citant `VMAN` |
| G3 | `grep -rn "VMAN" docs/handoffs docs/notes` | lignes citant `VMAN` dans les handoffs et notes |
| G4 | `grep -rnE "0x56, ?0x4D, ?0x41, ?0x4E" crates --include=*.rs` | littéraux hexadécimaux de la signature |
| G5 | `grep -rnE "(ManifestError\|BindingError\|ReadError\|LoadError\|ReferenceError\|ForgeError)::\|non_exhaustive" crates --include=*.rs` | variantes d'erreur et attribut `non_exhaustive` |

### C.2 Documents citant `VMAN` (G2, 14 fichiers)
`docs/handoffs/` : `05-verbum-assets-implementation-handoff.md`, `03-verbum-shell-temporal-policy-implementation-handoff.md`, `06-verbum-scripts-implementation-handoff-u5.md`, `06-verbum-scripts-implementation-handoff.md`. `docs/notes/` : `page-axes-shell-s-temporal-policy.md`. `docs/architecture/adr/` : `ADR-S04-shell-s-temporal-policy.md`. `docs/implementation/` : `05-verbum-assets.md`, `03-verbum-shell-temporal-policy-remaining-work.md`, `06-verbum-scripts.md`, `03-verbum-shell-temporal-policy.md`, `03-verbum-shell-temporal-policy-roadmap.md`, `06-verbum-scripts-execution.md`, `02-verbum-forge.md`, `05-verbum-assets-execution.md`.

Lus intégralement : `ADR-S04`, `05-verbum-assets.md`, `05-…-execution`, `06-…`, `06-…-execution`, `02-verbum-forge.md`, spécification, roadmap, remaining-work. Lignes seules (G3) : handoffs `03-…implementation`, `06-…`, `06-…-u5`, note `page-axes`. Le handoff 05 a été reçu après la validation du recensement (§ 2.1).

### C.3 Constats issus des recherches
| Constat | Source |
|---|---|
| 20 littéraux hexadécimaux, 2 constructeurs `b"VMAN"` | G4, G1 |
| Aucune occurrence de `non_exhaustive` | G5 |
| Aucun `match` exhaustif sur les énumérations d'erreur | G5 et lecture |
| `ManifestError::Version` référencée dans `manifest.rs` seulement | G5 |
| `ZeroValue` construite dans `sprite.rs` et `script.rs` | G5 |
| `verbum-script` sans usage de `Manifest`, `load`, `LoadError`, `compile_assets` | G1 |
| Faux positif : `schedule/tests/acceptance.rs` (`Ordering::load`) | G1 |

### C.4 Limites
Voir § 2.4. Le recensement est jugé suffisant pour rédiger ce plan [Arbitré, ARB-16] ; il ne constitue pas une preuve d'exhaustivité.
