# Plan de construction — étape 2 : Forge minimale (r2)

## Statut
Document de planification interne, non normatif, révisable sans ADR (même statut que `ROADMAP.md`). Détaille l'étape 2 en micro-phases. N'introduit aucun invariant : chaque phase cite l'invariant du corpus qui la justifie. **Aucun code, aucune modification du dépôt.** Révision r2 du 16 octobre 2026.

**État de validation : non validé pour implémentation.** Une question reste ouverte et bloquante : **Q-B (identité des assets, §8.3)**. Les phases 1 à 3 sont rédigées sous l'hypothèse « interprétation 2 » de Q-B ; si l'interprétation 1 est retenue, la phase 1 est à revoir avant toute implémentation.

Historique : r1 (plan initial, questions Q-A à Q-H) → r2 : application des arbitrages Q-A, Q-D, Q-E, Q-F, Q-G, Q-H ; reprise de Q-C sur les textes exacts (§8.2) ; reprise de Q-B sur les textes exacts, avec retrait d'une affirmation de r1 trop forte (§2, §8.3) ; précision de la frontière structurel / sémantique (§8.4).

Légende : **[D]** décision architecturale (source normative citée) ; **[L]** choix local d'implémentation ; **[H]** hypothèse à vérifier. Niveaux de preuve : **[Lu]**, **[Déduit]**, **[Non vérifié]**.

## 0. Sources et limites

**[Lu]** `ROADMAP.md`, `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, `README.md` d'architecture, ADR-000, ADR-001, ADR-005, ADR-A01, ADR-R01, ADR-R02, ADR-R03, ADR-S01, `00-verbum-ecs.md` (modèle de plan), `HANDOFF-stage-1.md`, `HANDOFF-stage-2-forge.md`, `tree.md`, `Cargo.toml` et `Cargo.lock` du workspace, `crates/ecs/Cargo.toml`, `lib.rs`, `resource.rs`, `world.rs`, `component_set.rs`.

**Non lus, sans incidence sur ce plan** : ADR-002, 003, 004, 006 ; `entity.rs`, `storage.rs`, `values.rs`, `command.rs`, `query.rs`, `fixtures.rs` ; `01-verbum-scheduler*.md`. **Volontairement non lus** : spécifications LPC.

**Aucune commande n'a été exécutée** pour établir ce plan ; aucun résultat de sandbox n'y figure.

## 1. Objectifs et critères de sortie (Roadmap) [Lu]

- Contenu : « un pipeline complet mais étroit — `sprite.toml → Forge → sprite.bin → lecture binaire → Ressource`. Pas de validation exhaustive, pas de second type d'asset. Le chargement runtime peut être simulé par un appel direct en test, sans Shell réel. »
- Dépendance : « Étape 0 seulement » ; ni Scheduler ni Shell.
- Test d'invariant : « Le runtime ne parse jamais le `.toml` — seule la Forge le fait ; le runtime ne lit qu'un artefact binaire déjà validé (`00-principes.md`, invariant 1). »
- Artefact : « premier binaire produit par la Forge, relu et casté en Ressource ».
- Principe directeur : un jalon n'est validé que si **aucun invariant architectural n'a dû être modifié** pour le faire fonctionner.

Critères de sortie (reprise, sans ajout) : (1) le test d'invariant ci-dessus passe ; (2) le binaire produit par la Forge est relu et intégré comme Ressource d'un `World` ; (3) aucun ADR, aucun document normatif, ni le noyau `verbum-ecs`, n'a été modifié ; (4) P1.6 (§6) est vert.

## 2. Corrections et précisions apportées au handoff

| Handoff | Constat après lecture des sources |
|---|---|
| §3.2 : « [Non vérifié] si ADR-A01 … en font un invariant » (index/manifeste) | ADR-A01 point 3 **[Lu]** : la Forge « produit également le manifeste qui permet au runtime d'adresser chaque Ressource via un identifiant opaque, résolu avant le hot path ». Dans la classification d'ADR-A01, l'invariant est la propriété de l'identifiant (« Adressage runtime par identifiant opaque résolu avant le hot path ») et le « layout du manifeste » est hors périmètre. **r1 affirmait « rien n'impose un manifeste pour un asset unique » : affirmation non établie par les textes, retirée.** Voir §8.3. |
| §3.1 / Q1 : format source, binaire, identité | Ni ADR-A01 ni `00-principes.md` n'imposent TOML, YAML ou PNG : `00-principes.md` cite « YAML, JSON, etc. » comme exemples. Les noms `AssetRegistry` et `FrameSequence` n'apparaissent dans aucune source lue. |
| Q3 : « minimal » inclut-il un décodage d'image ? | **[Déduit]** À terme, ADR-A01 (points 2-3) place tout décodage d'image côté Forge. Non requis à l'étape 2 : arbitré (Q-A). |
| Q6 : contraintes du binaire « casté » | ADR-000, point 3 et table du point 4 **[Lu]** : endianness, version et valeurs invalides vérifiées au chargement ; layout non conforme « rejeté au chargement, hors hot path » : conformité de format, « non une validation de cohérence métier ». Reprise détaillée en §8.2. |
| Q10 : capacités fixées par la Forge ? | ADR-001 point 5 **[Lu]** : mécanisme « privilégié », non obligatoire. Rien n'est fixé par la Forge à l'étape 2. |
| §2 et §10 : chemins `crates/verbum-ecs`, `crates/verbum-schedule` | `tree.md` (fait foi) : `crates/ecs`, `crates/schedule`, `crates/verbum`. Noms de packages inchangés. |

## 3. Invariants applicables

| # | Invariant | Source | Application à l'étape 2 |
|---|---|---|---|
| I1 | Seule la Forge lit et parse les formats humains ; le runtime ne fait que consommer des données précompilées | `00-principes.md` inv. 1 ; ADR-A01 pt 2 ; Roadmap (test) | Le lecteur ne reçoit que des octets d'artefact ; aucun parseur TOML dans son arbre de dépendances |
| I2 | Aucune validation de cohérence métier au runtime ; la Forge valide | `00-principes.md` inv. 1 et 5 | Validation minimale côté Forge ; côté runtime, uniquement la conformité de format (I5) |
| I3 | Artefact directement interprétable, sans étape de parsing ; layout explicite ; la règle « pas de parsing » porte sur les formats humains | ADR-A01 pt 3 ; ADR-000 pt 3 (« Cette règle porte sur les formats humains. ») ; `00-principes.md` inv. 5 | Layout fixe documenté ; endianness explicite |
| I4 | Aucune chaîne ni chemin ne franchit la frontière de compilation | ADR-A01 pt 3 | Artefact purement numérique, de longueur exacte |
| I5 | Layout non conforme rejeté au chargement ; endianness, version, valeurs invalides vérifiées au chargement ; pas une validation métier | ADR-000 pt 3 et pt 4 (table) | En-tête (identité du format, version, longueur de charge utile) vérifié par le lecteur |
| I6 | Un asset devient, après la Forge, une Ressource ordinaire | ADR-A01 pt 1 et 5 ; ADR-005 | Insertion dans un `ResourceSlot` du `World` |
| I7 | Clé d'une Ressource = clé unique ; forme concrète = implémentation (aujourd'hui : position statique) | ADR-005 pt 1 ; `02-concepts.md` ; `resource.rs` | Champ `ResourceSlot<Sprite>` dans un `R` de test |
| I8 | Seule l'infrastructure de chargement fait l'accès disque | ADR-A01 pt 2 | Lecteur = fonction sur `&[u8]` ; accès fichier simulé par le test (permis par la Roadmap) |
| I9 | La Forge est extérieure au runtime (frontière de compilation) | `02-concepts.md` ; ADR-000 pt 2 | Aucun crate runtime ne dépend de la Forge ; la Forge ne dépend pas du noyau en production |
| I10 | Noyau `no_std`, sans `alloc` | ADR-000 pt 1 et 4 | `verbum-ecs` inchangé |
| I11 | Un concept n'est codé que s'il est porté par un invariant ou rendu nécessaire par un test | `00-verbum-ecs.md`, règle gouvernante | Exclusions du §10 |
| I12 | Adressage runtime par identifiant opaque résolu avant le hot path ; forme opaque ; aucune chaîne ni chemin ne franchit la frontière ; l'ensemble des assets est connu de la Forge | ADR-A01 pt 3, pt 4 et classification | **Application partielle et ouverte : Q-B (§8.3)** |

Non applicables à l'étape 2 : ADR-A01 pt 4 (absence de propriétaire d'écriture au runtime : aucun système n'existe encore, cf. R6), pt 5 (streaming, déchargement), pt 6 (rejets).

## 4. Pipeline et frontières de crates (Q-E arbitrée : deux crates)

```
sprite.toml ──(Forge, build-time, std)──► sprite.bin
                                              │  octets fournis par l'appelant
                                              ▼  (accès fichier simulé par le test)
                            lecture runtime (no_std, sans alloc, sans dépendance)
                                              ▼
                                            Sprite ──► ResourceSlot<Sprite> du World (insertion directe)
```

| Crate | Chemin | Environnement | Contenu | Dépend de |
|---|---|---|---|---|
| `verbum-assets` | `crates/assets` | `no_std`, sans `alloc` | Layout de l'artefact sprite, type `Sprite`, lecture `&[u8]` → `Sprite` | rien |
| `verbum-forge` | `crates/forge` | `std` | Lecture de `sprite.toml`, validation minimale, écriture des octets | `verbum-assets`, crate `toml` ; en `dev-dependencies` : `verbum-ecs` |
| `verbum-ecs` | inchangé | | Aucune modification : `ResourceSlot<T>` n'impose aucune contrainte sur `T` **[Lu]** | |

Justifications :
- **Séparer le lecteur de la Forge** : le test d'invariant porte sur une frontière de connaissance. Elle s'exprime par le graphe de dépendances (ADR-000 pt 3 : interdits « X ne nomme pas Y » encodés dans le graphe de dépendances).
- **Layout défini côté `verbum-assets`** [L] : sens de dépendance Forge → format, seul compatible avec I9 ; la Forge possède la production des octets (ADR-000 pt 2), le runtime ne porte aucun code de production. Les octets littéraux des tests bornent le risque de dérive.
- **`verbum-assets` ne dépend pas de `verbum-ecs`** : l'insertion est faite par l'appelant (`resources_mut()`).
- **Test d'acceptation dans `crates/forge/tests/pipeline.rs`** (Q-F arbitrée), `verbum-ecs` en dev-dependency : évite un cycle de dev-dépendances entre `assets` et `forge`. La façade `verbum` est inchangée.

## 5. Choix concrets et statuts

| Élément | Statut | Détail |
|---|---|---|
| Source `sprite.toml` : deux champs entiers, largeur et hauteur en pixels (Q-A, option 1 arbitrée) | [L] | Aucune chaîne dans la source utile ; aucune durée (R4) ; aucun décodage d'image |
| Layout : en-tête (identité du format sur 4 octets, version `u16`, longueur de charge utile `u32`) suivi de la charge utile (largeur `u16`, hauteur `u16`), soit 14 octets | [L], provisoire | Voir la réserve ci-dessous : ce layout est celui de **ce** descripteur, non un gabarit imposé aux futurs assets |
| Little-endian explicite | [L] ; l'exigence « endianness vérifiée au chargement » est [D] (ADR-000) | |
| Lecture champ par champ (`from_le_bytes`), sans `unsafe`, sans dépendance | [L], **proposé suite à §8.2**, à confirmer | Écart au mécanisme « par défaut » (cast zéro-copie) consigné ici et justifié en §8.2 |
| Assertion à la compilation de la cohérence des constantes de layout (taille de l'en-tête + charge utile) | [L] | Équivalent, pour une lecture par champs, des « assertions de taille et d'alignement au build » d'ADR-000 pt 4 (table) |
| Parseur : crate `toml` (Q-D arbitrée) | [L], tracé ici | Décision d'implémentation ; MSRV 1.85 et Miri à vérifier (R1). Voir §8.5 |
| Nom `Sprite` : descripteur d'un sprite atomique | [L], provisoire | Ne désigne ni une frame composée ni un ensemble logique |
| Fonction de lecture unique, erreur énumérée `Copy` | [L] | Pas d'allocation côté runtime |
| Forge = bibliothèque appelée par les tests, sans CLI ni `build.rs` (Q-G arbitrée) | [L] | Le test vérifie l'écriture réelle puis la lecture réelle du fichier binaire (P3.1) |
| Insertion directe dans le `World` par `resources_mut()` | [L] | Cohérent avec la Roadmap et avec les Ressources déjà implémentées |

**Réserve (layout non généralisable).** Le layout du §5 est celui d'un descripteur de taille fixe. Il ne préjuge ni de la forme des futurs artefacts, ni de l'existence de charges utiles de longueur variable : le mécanisme générique Forge → artefact binaire → lecture runtime → Ressource doit pouvoir s'étendre à de telles charges. La longueur de charge utile figure dans l'en-tête parce que la vérification de conformité l'exige (I5), non parce qu'elle fixerait un gabarit.

## 6. Micro-phases

Méthode : invariant cité → test qui l'exprime (rouge confirmé avant l'API) → API minimale → implémentation. Un commit par invariant démontré (messages à votre charge). Contrôles de chaque phase : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check` ; Miri et MSRV 1.85 exécutés par vos soins.

### Phase 1 — Format de l'artefact et lecture runtime (`verbum-assets`)
**Invariants** : I3, I4, I5, I8, I10 (et I2 pour P1.6).
**Fichiers** : création de `crates/assets/Cargo.toml`, `crates/assets/src/lib.rs` ; modification de `Cargo.toml` (membres du workspace).
**Tests** (octets littéraux écrits à la main dans le test, valeurs asymétriques, par exemple largeur `0x0102` et hauteur `0x0304`, pour fixer l'ordre des octets) :
- P1.1 : l'artefact littéral est lu en `Sprite` aux valeurs attendues.
- P1.2 : toute entrée tronquée, de la longueur 0 à la longueur totale moins 1, est rejetée.
- P1.3 : une identité de format altérée est rejetée ; le texte d'un `sprite.toml` fourni au lecteur est rejeté.
- P1.4 : une version inconnue est rejetée.
- P1.5 : une longueur de charge utile différente de celle du layout, ou des octets en surplus, sont rejetés.
- **P1.6 (inclus, Q-H arbitrée)** : un artefact structurellement conforme portant une largeur nulle est **accepté** : le lecteur ne valide aucune cohérence métier (frontière précisée en §8.4).
**API minimale** : `Sprite` (donnée pure, `Copy`) ; fonction de lecture `&[u8]` → `Result<Sprite, ReadError>` ; constantes de layout publiques utilisées par la Forge ; `ReadError` énumérée.
**Critère de sortie** : P1.1 à P1.6 verts ; crate `#![no_std]` sans `extern crate alloc` ; `[dependencies]` vide ; Miri vert sur `verbum-assets` (vous).

### Phase 2 — Forge : source → artefact (`verbum-forge`)
**Invariants** : I1, I2, I4, I9, I11.
**Fichiers** : création de `crates/forge/Cargo.toml`, `crates/forge/src/lib.rs` ; modification de `Cargo.toml` (membre, dépendance `toml`) ; `Cargo.lock` modifié par Cargo seulement.
**Tests** :
- P2.1 : un `sprite.toml` valide produit exactement les octets littéraux attendus (second témoin indépendant de P1.1 : une erreur symétrique d'écriture et de lecture ne passe pas inaperçue) ; la longueur égale celle du layout (I4).
- P2.2 : le round-trip Forge → lecture runtime restitue les valeurs de la source.
- P2.3 : validation minimale refusée par la Forge, sans artefact produit : champ manquant, valeur nulle, valeur hors de l'intervalle de `u16`, type erroné, champ inconnu, TOML syntaxiquement invalide. Ensemble limité à ces cas (« pas de validation exhaustive »).
**API minimale** : fonction de compilation `&str` → `Result<Vec<u8>, ForgeError>` ; `ForgeError` énumérée. Aucun accès fichier dans cette phase.
**Critère de sortie** : P2.1 à P2.3 verts ; `cargo tree -i toml --workspace` ne liste que `verbum-forge` comme dépendant (contrôle manuel, à rapporter) ; MSRV 1.85 vert (vous).

### Phase 3 — Acceptation du pipeline (`crates/forge/tests/pipeline.rs`)
**Invariants** : I1, I6, I7, I8, critère de la Roadmap.
**Fichiers** : création de `crates/forge/tests/pipeline.rs` ; ajout de `verbum-ecs` en `dev-dependencies` de `crates/forge/Cargo.toml`.
**Tests** :
- P3.1 (chaîne complète sur disque) : écriture d'un `sprite.toml` dans un répertoire temporaire → Forge → écriture de `sprite.bin` → **suppression de `sprite.toml`** → lecture des octets (le test joue le rôle de l'infrastructure de chargement, ADR-A01 pt 2) → lecture runtime → insertion dans un champ `ResourceSlot<Sprite>` d'un `R` défini par le test, dans un `World` → `world.resources().<champ>.get()` renvoie le `Sprite` attendu. **Répertoire unique par test** (identifiant de processus et nom du test, `std` seulement), **nettoyé par un garde (`Drop`) y compris en cas d'échec d'assertion**.
- P3.2 : les octets du même fichier source sont acceptés par la Forge et rejetés par le lecteur ; les octets de l'artefact sont acceptés par le lecteur.
**Ce qui établit I1** : P3.2 et P1.3 ne sont **pas** une preuve suffisante à eux seuls. C'est la combinaison de la suppression de la source en P3.1, de la séparation des crates et de l'absence de parseur TOML dans l'arbre de dépendances de `verbum-assets` qui établit la frontière AOT / runtime.
**Contrôles manuels** (rapportés par vous) : `cargo tree -p verbum-assets` ne liste aucune dépendance ; aucune modification de `crates/ecs`, `crates/schedule`, `crates/verbum`, des ADR ni de la Roadmap.
**Critère de sortie** : P3.1 et P3.2 verts, quatre contrôles verts, Miri vert sur `verbum-assets`, MSRV vert. Miri sur `verbum-forge` : non prévu (accès fichier et `toml`), à confirmer (R1).

## 7. Risques

- **R1 — Dépendances du crate `toml`.** `Cargo.lock` ne contient aujourd'hui aucune dépendance externe. Arbre transitif, MSRV vis-à-vis de 1.85, comportement sous Miri : **[Non vérifié]**. Clause d'arrêt : si aucune version ne tient sous 1.85, arrêt et rapport.
- **R2 — Tension avec l'étape 5** (« aucun changement de mécanisme du pipeline Forge → Ressource »). Deux sources possibles : charges de longueur variable, et adressage de plusieurs assets (Q-B). Le plan n'anticipe aucune des deux (I11) et ne présente pas le layout du premier descripteur comme gabarit des futurs assets (§5). À rejuger au cadrage de l'étape 5.
- **R3 — Vigilance Renderer (pas une exigence)** : (a) `Sprite` est un descripteur atomique ; l'artefact ne porte aucune relation d'appartenance, d'ordre ou de regroupement ; (b) l'adressage par champ statique ne passe pas à l'échelle ; l'étape 4 ne doit pas en déduire qu'un asset correspond à une image rendue ; (c) la forme de la représentation de frame est hors de cette étape (ADR-R01 à R03).
- **R4 — Temps.** La conversion secondes → ticks suppose une fréquence de tick ; la politique temporelle est réservée à ADR-S04 (ADR-S01 pt 5). Aucune durée dans le sprite.
- **R5 — Frontières de dépendance.** `verbum-assets` ne dépend jamais de `verbum-ecs` ni de la Forge ; pas de cycle de dev-dépendances.
- **R6 — Écriture des Ressources d'asset.** ADR-A01 pt 4 interdit tout propriétaire d'écriture au runtime ; `ResourceSlot::get_mut` permet la mutation à tout porteur de `&mut R`. Seuls les contrats déclarés (Scheduler) pourront l'exprimer. Non testable à l'étape 2 **[Déduit]**.
- **R7 — Pression de sur-conception** : registre, manifeste, CLI, second type d'asset, couches (§10).

## 8. Arbitrages, clarifications et question ouverte

### 8.1 Arbitrages rendus (intégrés ci-dessus)

| Question | Arbitrage |
|---|---|
| Q-A | Option 1 : métadonnées à charge fixe (largeur, hauteur) ; aucun décodage PNG ; layout non généralisable (§5) |
| Q-D | Crate `toml`, sous réserve MSRV 1.85 et comportement des dépendances ; tracé en §5 (voir §8.5) |
| Q-E | Deux crates : `verbum-assets`, `verbum-forge` |
| Q-F | `crates/forge/tests/pipeline.rs`, `verbum-ecs` en dev-dependency ; façade inchangée |
| Q-G | Bibliothèque appelée par les tests ; ni CLI ni `build.rs` |
| Q-H | P1.6 inclus aux critères de sortie |

### 8.2 Q-C — nature du cast : textes et conclusion

Textes **[Lu]** :
- `00-principes.md`, inv. 5 : « accès directs (indices entiers, structures `#[repr(C)]` castées depuis des buffers) — jamais de résolution de références symboliques, de recherche par nom, ou de logique de validation de configuration ». Énumération d'exemples d'accès directs.
- ADR-A01, pt 2 : le chargement runtime est seul autorisé à « produire les Ressources correspondantes par cast zéro-copie ».
- ADR-A01, pt 3 : « Les garanties de layout mémoire concrètes (aujourd'hui obtenues via `#[repr(C)]` en Rust) relèvent de l'implémentation ; l'invariant architectural est l'absence de parsing, pas une convention de langage particulière. »
- ADR-A01, classification : les invariants listés sont « Aucun parsing, aucune recherche par nom sur le hot path » et « Seule la Forge parse des formats humains ». **Le cast n'y figure pas comme invariant.**
- ADR-000, pt 3 : « Cette règle porte sur les formats humains. » ; pour les binaires non produits par la Forge, « le cast zéro-copie ne lui est pas applicable par défaut » : le cast zéro-copie est donc la **posture par défaut** pour les artefacts de la Forge.
- ADR-000, pt 4 : `#[repr(C)]` « en est une instanciation possible, le choix concret relevant du point 6 ». Pt 6 : « bibliothèque de cast (`bytemuck` ou autre) » laissée à l'implémentation.

Conclusion **[Déduit]** :
1. Le cast zéro-copie est le mécanisme par défaut décrit par le corpus, **pas un invariant**. Les invariants sont : pas de parsing de formats humains, layout explicite, conformité vérifiée au chargement (endianness, version).
2. La lecture champ par champ est admissible, à condition que l'écart au mécanisme par défaut soit consigné comme choix local (fait en §5).
3. Pour `ResourceSlot<T>`, la différence de zéro-copie est sans objet : `ResourceSlot` stocke `Option<T>` **par valeur** (`resource.rs` [Lu]) ; les trois options (lecture par champs, `bytemuck`, `unsafe` local) aboutissent à la copie de la valeur dans l'emplacement. Le zéro-copie n'a de sens que pour une Ressource empruntant le tampon (charges volumineuses ou variables), hors étape 2.
4. La lecture par champs fixe l'endianness par construction, ce que ne fait pas un cast `#[repr(C)]` (ordre de l'hôte, à vérifier au chargement, ADR-000).
5. Cette conclusion ne vaut que pour ce descripteur. Pour des charges volumineuses ou variables (étape 5), la question du cast ou de l'emprunt revient ; elle n'est pas close par r2.
6. Doute résiduel : `00-principes.md` inv. 1 dit « aucun parsing » sans qualificatif ; ADR-000 pt 3 le restreint aux formats humains. La vérification de l'en-tête serait nécessaire avec n'importe quelle option, donc ce doute ne départage pas les options.

### 8.3 Q-B — identité des assets : textes, ce qu'ils établissent, **question ouverte**

Textes **[Lu]** :
- ADR-A01, pt 3 : la Forge « produit également le manifeste qui permet au runtime d'adresser chaque Ressource via un identifiant opaque, résolu avant le hot path [...]. La représentation concrète de cet identifiant (entier, hash, couple pack/offset, handle compressé...) est une décision d'implémentation ; l'invariant est que sa forme reste opaque au runtime et qu'aucune chaîne de caractères ni chemin de fichier ne franchit la frontière de compilation. »
- ADR-A01, pt 4 : « **Adressage stable** : par clé unique, comme toute Ressource (ADR-005), résolue au chargement, jamais recalculée sur le hot path. » ; « l'ensemble des assets utilisés par le jeu est entièrement connu à la compilation par la Forge ».
- ADR-A01, classification : « Adressage runtime par identifiant opaque résolu avant le hot path ; représentation concrète = implémentation » → **Invariant** ; « layout du manifeste » → hors périmètre.
- ADR-005, pt 1 : « identifiée par une clé unique plutôt que par `EntityId`. Le choix concret de cette clé (type Rust, `TypeId`, identifiant généré par la Forge...) est une décision d'implémentation. »
- `02-concepts.md`, Ressource : clé unique « (aujourd'hui, le type Rust concerné ; potentiellement, demain, une clé générée par la Forge ou tout autre mécanisme d'identification) ».

Ce que les textes **établissent** : (1) l'adressage runtime des assets passe par un identifiant opaque résolu avant le hot path (invariant) ; (2) rien de textuel ne franchit la frontière ; (3) la forme concrète de l'identifiant est libre ; (4) le pt 4 rattache cet adressage à la clé d'ADR-005, dont la forme actuelle (type Rust, position statique) est celle de l'étape 0 acceptée, et dont une forme « générée par la Forge » est une évolution possible.

Ce qu'ils **n'établissent pas** : que chaque asset reçoive, dès le premier, un identifiant produit par la Forge et résolu via un manifeste ; la classification ne classe ni l'existence ni la forme du manifeste.

**Correction de r1** : la justification de l'option « aucun identifiant » (« satisfait l'invariant d'identifiant opaque ») était trop forte. La position statique satisfait « rien de textuel » et « résolu avant le hot path » (résolution à la compilation), et c'est une clé ADR-005 valide ; elle ne démontre pas un identifiant **produit par la Forge**, mentionné au pt 3. Elle ne doit pas être présentée comme équivalente.

**Constat sur le noyau [Déduit, `resource.rs` et `world.rs`]** : aucune API ne permet d'adresser une Ressource par identifiant dynamique ; les Ressources sont des champs d'un `R` défini par l'appelant. Faire résoudre « identifiant → Ressource » à l'exécution exigerait soit une modification du noyau (exclue par le cadrage), soit une table de correspondance côté assets, c'est-à-dire un registre dont la forme serait conçue à partir d'un unique exemplaire.

Les deux interprétations restent **textuellement admissibles** ; les sources ne permettent pas de trancher.

| | Interprétation 1 : identité produite par la Forge exigée dès chaque asset | Interprétation 2 : exigée quand un ensemble d'assets doit être adressé |
|---|---|---|
| Étape 2 | La Forge attribue un identifiant opaque et produit un manifeste (une entrée) ; le runtime le résout. Le placement de l'identifiant (en-tête de `sprite.bin` ou artefact distinct) est **non fixé** par ADR-A01 et ne doit pas être choisi arbitrairement | Insertion directe dans un `ResourceSlot` statique (clé ADR-005 actuelle) |
| Coûts | Deuxième artefact ou champ, mécanisme de résolution identifiant → emplacement hors noyau (registre de fait), forme conçue sur un seul exemplaire | Invariant d'identité **non démontré** à l'étape 2 : à consigner explicitement comme tel |
| Risque | Forme spéculative, figée tôt | Étape 5 jugée « changement de mécanisme » (R2) |

**Arbitrage demandé (unique)** : laquelle des deux interprétations de « chaque Ressource » (ADR-A01 pt 3) retient l'étape 2 ? Si l'interprétation 1 : forme minimale souhaitée du manifeste et lieu de la résolution. Si l'interprétation 2 : acceptez-vous que le plan consigne « identité par manifeste : non démontrée à l'étape 2, reportée au premier besoin d'adresser plusieurs assets » ? Dans les deux cas, une éventuelle précision d'ADR-A01 reste votre décision ; elle n'est pas proposée ici.

### 8.4 Frontière entre invalide structurel et invalide sémantique (P1.6)

- **Structurel, rejeté par le runtime** : identité du format, version inconnue, longueur de charge utile, troncature, octets en surplus (ADR-000 pt 3 : « vérification de conformité de format, non d'une validation de cohérence métier »).
- **Sémantique, rejeté par la Forge seule** : largeur ou hauteur nulles, hors bornes (`00-principes.md` inv. 1 : aucune validation de cohérence métier au runtime).
- La table d'ADR-000 (pt 4) parle de « Valeurs invalides, endianness, version : vérifiées au chargement ». **[Déduit]** Dans cette colonne (résidu non couvert par les assertions de build sur le layout), « valeurs invalides » désigne des représentations invalides (motifs binaires hors du domaine du type, par exemple un `bool` ou une énumération), non des valeurs sémantiquement incohérentes. Pour `Sprite` (deux `u16`), tout motif binaire est une représentation valide : le lecteur n'a donc aucun contrôle de valeur. Le terme est défini nulle part dans l'ADR ; si vous le lisez comme incluant la cohérence métier, P1.6 serait en contradiction avec cette lecture.

### 8.5 Q-D — trace du choix de parseur

Choix tracé dans ce plan (§5) et à reporter au journal de construction. **Confirmation demandée** : `00-principes.md` (« Ce que ce document ne couvre pas ») indique que les choix de crates et bibliothèques « font l'objet d'ADR indépendantes (ADR-00X) », tout en les qualifiant de « décisions d'implémentation substituables ». `tree.md` ne montre aucune ADR de choix de crate ; ADR-001 nomme `bytemuck` sans ADR dédiée. Votre arbitrage (trace suffisante, pas d'ADR) est appliqué ; je consigne seulement cette formulation pour que le choix soit fait en connaissance du passage.

## 9. Fichiers créés ou modifiés

Créés : `docs/implementation/02-verbum-forge.md` (ce plan), `crates/assets/{Cargo.toml, src/lib.rs}`, `crates/forge/{Cargo.toml, src/lib.rs, tests/pipeline.rs}`.
Modifiés : `Cargo.toml` (membres, dépendances du workspace), `Cargo.lock` (par Cargo uniquement), `tree.md` (régénéré par votre script).
**Jamais modifiés** : `crates/ecs`, `crates/schedule`, `crates/verbum`, les ADR, `ROADMAP.md`.

## 10. Hors périmètre de l'étape 2

Registre runtime, manifeste de build (sous réserve de Q-B), second type d'asset, charges utiles de longueur variable, système de couches, notion de personnage composé, décodage d'image, durées et ticks, capacités de stockage fixées par la Forge, exécutable en ligne de commande, `build.rs`, mmap et accès disque runtime réel, Shell, Scheduler, hot-reload, streaming et déchargement, LPC.

## 11. Fin de l'étape

Si les trois phases sont vertes et qu'aucun invariant n'a dû être modifié, l'étape 2 est close. R3 est reporté tel quel au cadrage de l'étape 4 ; R2 et la question d'identité sont reportés au cadrage de l'étape 5 si Q-B est tranchée en interprétation 2.
