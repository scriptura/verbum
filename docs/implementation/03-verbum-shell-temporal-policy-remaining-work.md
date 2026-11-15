# Verbum — Politique temporelle du Shell : travaux restants et découpage des sessions — 15 novembre 2026

**Mise à jour du 15 novembre 2026 (après arbitrage)** : matrice C-1 à C-16 acceptée, **J-A prononcé**, **T8 non retenue**, D-N6 et D-N3 (principe) arbitrés. Les lignes concernées sont mises à jour ci-dessous ; le reste est inchangé.

Non normatif. Dérivé de la roadmap r1.1 (`03-verbum-shell-temporal-policy-roadmap.md`), qui reste la référence ; aucune décision n'est tranchée ici. Les unités validées (T1 à T7) ne figurent pas dans l'inventaire. Les statuts « obligatoire / facultatif / conditionnel » sont ceux de la roadmap ; lorsque la roadmap ne se prononce pas, la mention est « non tranché ».

## 1. Inventaire des travaux restants

### 1.1 Clôture du chantier temporel du Shell (chantier A)

| Id et objectif | Caractère | État et prérequis | Porte / critère | Fichiers / crates | Nature | Dépendances |
|---|---|---|---|---|---|---|
| **T8** : exemple headless conforme S04, distinct des exemples historiques | **Non retenue** (décision du 15 novembre 2026 ; n'est pas une unité en attente) | Écartée | Code de sortie 0 ; horloge et attenteur scriptés ; valeurs de test (q = 10, K = 3, M = 5) ; aucune valeur de jeu. P-C ne l'exige que « si retenue » | `crates/engine/shell/examples/policy_headless.rs` (créé) | Implémentation | Aucune unité aval n'en dépend (F3c ne cite pas T8). Recouvrement partiel avec l'exemple headless de F3c (l'un en valeurs de test, l'autre en valeurs de composition) : à arbitrer, non tranché |
| **Matrice C-1 à C-16 → tests** | **Obligatoire pour J-A** (condition de § 11) ; reprise par G1 | **Acceptée** le 15 novembre 2026 (annexe A), y compris C-2, C-10 (branche impossible), C-15 et C-16 par structure ou revue | Acceptation obtenue | Document (aucun code) | Vérification / documentation | Précède J-A ; réutilisée par G1 |
| **J-A** : jalon « cœur temporel conforme (valeurs de test) » | **Obligatoire** (jalon) | **Prononcé** le 15 novembre 2026 (roadmap § 11.1) ; P-A, P-B, P-C, P-D franchies | Conditions de § 11 | — | Vérification | Prérequis du jalon « fin complète » |

### 1.2 Chantier B — V2 : provenance du quantum et paquet Forge/Assets

Tout V2 est obligatoire pour **J-V2** et pour la fin complète ; il est entièrement indépendant de la preuve du cœur temporel (déjà faite avec q de test). Il est placé sous **G-CM** (arrêt avec rapport si un mécanisme de l'étape 5 est touché) et **R-ADAPT** (aucune assertion d'étapes antérieures modifiée).

| Id et objectif | Caractère | État et prérequis | Porte / critère | Fichiers / crates | Nature | Dépendances |
|---|---|---|---|---|---|---|
| **V2-0** : plan V2, avec recensement complet des tests assertant des octets de manifeste (y compris `tests/pipeline.rs`), VQ-1 à VQ-9, R-ADAPT, véhicule d'enregistrement | Obligatoire | Démarrable, sans prérequis. **D-N3** : principe arbitré (document autonome non normatif dans `docs/implementation/`) ; nom et numéro à proposer puis valider avant rédaction. Fichiers listés « non fournis » par la roadmap à fournir : `forge/tests/pipeline.rs`, `assets/src/{map,font,tileset}.rs` | **P-V2** (validation du plan) | Nouveau document de plan V2 ; aucun code, aucun artefact | Documentation | Bloque V2-C, V2-L, V2-P, V2-F, V2-Z |
| **V2-C** : conception du paquet (layout, accès par index, offsets, absence d'allocation, réutilisation des niveaux 1 à 4, refus croisés, lecture de q, unique producteur) | Obligatoire (porte de conception) | Conditionnel : V2-0 validé. **D-N4** (layout) | **P-CONC** (note validée) | Note annexée au plan V2 ; aucun code | Conception | Bloque V2-L, V2-P |
| **V2-L** : lecteur `VMAN` v2 additif (`q_ns`, en-tête de 18 octets, q = 0 refusé) | Obligatoire | Bloqué : P-V2 et P-CONC | VQ-1 à VQ-4 et VQ-7 ; tests `m1` à `m8` inchangés ; Assets sans dépendance | `crates/engine/assets` : `manifest.rs`, `lib.rs` | Implémentation | Bloque V2-P |
| **V2-P** : paquet et chargeur v2, niveaux 1 à 4 réutilisés, refus croisés | Obligatoire | Bloqué : V2-L | VQ-6, VQ-9 (a, b, d) ; `l1` à `l5`, `sf*`, `rf*` inchangés ; Miri Assets intégral | `crates/engine/assets` : module de paquet, éventuellement `load.rs` (si V2-C la retient), `lib.rs` | Implémentation | Bloque V2-F |
| **V2-F** : voie de compilation Forge additive (q explicite, unique producteur du paquet) | Obligatoire | Bloqué : V2-P | VQ-2, VQ-5, VQ-9c ; `compile_assets` et v1 octet pour octet ; `cargo tree -i toml --workspace` inchangé ; Miri Forge partiel | `crates/engine/forge` : `batch.rs` ou module additif, `lib.rs`, `sprite.rs`, `tests/pipeline.rs` (additif) | Implémentation | Bloque V2-Z, F2 |
| **V2-Z** : clôture V2 (régression globale, consignation additive, « non démontré ») | Obligatoire | Bloqué : V2-F | **P-E** ; VQ-6 et VQ-7 sur l'ensemble | Assets, Forge (comptes rendus additifs) | Vérification | Bloque F3b, G1 |

### 1.3 F2 — conversion des durées non entières

| Id et objectif | Caractère | État et prérequis | Porte / critère | Fichiers / crates | Nature | Dépendances |
|---|---|---|---|---|---|---|
| **D-N2** : arbitrage de la règle des durées non entières (rejet, quantification déterministe, autre), périmètre de F2, autorisation de toucher des sources de l'étape 6 | Décision, prérequis de F2 | En attente de l'utilisateur ; non tranché | **G-ARB-F2** | Documentation probable (plan V2 ou note Forge) ; ADR non nécessaire a priori, à confirmer | Décision / documentation | Bloque F2 |
| **F2** : règle déterministe et son premier consommateur dans la Forge | **Conditionnel** : une clôture partielle « conforme S04 sans F2 » n'est possible que sur décision de l'utilisateur (ADR-S04 pt 2 exige une règle explicite) | Bloqué : V2-F et D-N2 | **P-E′** ; règle documentée et testée (entiers, non entiers, limites, déterminisme) | `crates/engine/forge` : `src/script.rs`, aiguillage de `batch.rs`, tests `SCR-F*` — **sources de l'étape 6, additif seulement, assertions intactes** (RK-4, G-S6, R-ADAPT) | Documentation + implémentation | Précède G1 si retenue |

### 1.4 F3 — assemblage dans `crates/games/demo`

| Id et objectif | Caractère | État et prérequis | Porte / critère | Fichiers / crates | Nature | Dépendances |
|---|---|---|---|---|---|---|
| **D-N5** : portée de `games/demo` (consommateur, production du paquet, nom du crate) | Décision, prérequis de F3a | En attente ; non tranché | — | — | Décision | Bloque F3a |
| **F3a** : promotion de `games/demo` au workspace (niveau Application) | Obligatoire pour J-F3 ; conditionnel à D-N5 | Aucune dépendance de code | **G-W01** (`cargo tree -i`), quatre contrôles, MSRV | `Cargo.toml` racine (`members`, `[workspace.dependencies]` pour `verbum-shell`), `crates/games/demo/{Cargo.toml, squelette, README.md}` | Implémentation (configuration, squelette sans logique) | Bloque F3b |
| **F3b** : provenance de q par le paquet chargé uniquement ; `TemporalPolicy` ; erreur typée avant toute progression | Obligatoire | Bloqué : T1 (✓), **V2-Z**, F3a | **VQ-8** ; aucune valeur de q par défaut | `crates/games/demo` | Implémentation | Bloque F3c |
| **D-N1** : valeurs q, K, M de composition | Décision, prérequis de F3c | En attente ; non tranché | — | — | Décision | Bloque F3c |
| **F3c** : assemblage `World`, `Scheduler`, `Inbox`, `Progressor`, pilote bloquant à politique ; exemple headless de code 0 | Obligatoire | Bloqué : F3b, D-N1 ; T5 et T6 satisfaits | **P-F** ; aucun type de fenêtrage ; R03/S05 non résolus | `crates/games/demo` | Implémentation | Précède G1 |

### 1.5 Validation intégrée et hors chantier

| Id et objectif | Caractère | État et prérequis | Porte / critère | Fichiers / crates | Nature | Dépendances |
|---|---|---|---|---|---|---|
| **G1** : matrice C-1 à C-16, non-régression de tous les consommateurs, graphe de dépendances, Miri et MSRV, « non démontré », exigences transmises à l'intégration native | Obligatoire (dernier) | Non démarré ; dépend de tout le reste (P-F, et P-E′ si F2 retenue) | **P-G** | Transversal ; rapport de clôture | Vérification | Dernier maillon |
| **Intégration native** (winit/wgpu, PO-3, R03/S05, expériences E1, E2, E8, E10, E14, E15) | **Hors chantier** | Non planifiée ici | — | Adaptateur natif, autre chantier | Conception puis implémentation | Reçoit la liste d'exigences produite par G1 |
| **D-N6** : granularité des sessions | Décision | **Arbitrée** (15 novembre 2026) : une unité par session par défaut, regroupement exceptionnel sur arbitrage explicite | — | — | Décision | Handoffs |

**Jalons** : J-A (§ 1.1) ; **J-V2** = P-V2, P-CONC, P-E ; **J-F3** = P-F ; **J-F2** = P-E′ (ou décision de livrer séparément) ; **fin complète** = J-A + J-V2 + J-F3 + J-F2 + P-G.

## 2. Recommandation sur les sessions

### 2.1 Dans la session actuelle

- *(Réalisé le 15 novembre 2026)* synchronisation documentaire, matrice acceptée, J-A prononcé, T8 écartée, D-N6 et D-N3 (principe) arbitrés.
- Prochaine session : **V2-0**, dans une session neuve (message d'ouverture dédié).

### 2.2 Nouvelles sessions avec handoff propre

| Session | Contenu | Pourquoi une session séparée |
|---|---|---|
| **S1** | **V2-0** (plan V2) | Documentaire, sans prérequis, plus longue échéance (la validation de V2 conditionne tout le chantier B). Exige un recensement complet : les fichiers « non fournis » (`pipeline.rs`, `map.rs`, `font.rs`, `tileset.rs`, ADR et plans des étapes 2, 5, 6) doivent être fournis, ce qui impose de répartir les téléversements (20 par tour) |
| **S2** | **V2-C** | Porte de conception à valider (P-CONC) avant tout code ; contexte distinct du plan |
| **S3** | **V2-L** | Première unité de code sous G-CM |
| **S4** | **V2-P** | Plus risquée (réutilisation de `load`, RK-2, RK-3) ; Miri Assets intégral |
| **S5** | **V2-F** | Forge additive |
| **S6** | **V2-Z** | Régression globale et consignation |
| **S7** | **D-N2 puis F2** | Arbitrage préalable ; touche des sources de l'étape 6 : session dédiée, additif seulement |
| **S8** | **F3a** | Petite unité de configuration ; à placer juste avant F3b ou plus tôt, car sans dépendance de code (décision D-N5 requise) |
| **S9** | **F3b**, puis **F3c** | F3c après D-N1 ; peuvent partager une session si F3b est validée avant |
| **S10** | **G1** | Rapport de clôture, contexte propre |
| — | ~~T8~~ | Non retenue (15 novembre 2026) |

Chaque session démarre par un handoff (modèle du handoff T6) : interdictions, état, API en place, fiche de l'unité, fichiers à fournir.

### 2.3 Ordre et dépendances bloquantes

1. **Maintenant** : arbitrages J-A / T8 / D-N3 / D-N6.
2. **V2-0 → P-V2 → V2-C → P-CONC** : chaîne documentaire, à lancer en premier (roadmap § 7.1).
3. **V2-L → V2-P → V2-F → V2-Z** : strictement séquentielle, bloquée par P-V2 et P-CONC (G-CM).
4. **F2** : après V2-F et D-N2 ; peut être livrée séparément (J-F2) ; sinon la clôture est partielle sur décision.
5. **F3a** (après D-N5) peut être faite à tout moment ; **F3b** après V2-Z ; **F3c** après F3b et D-N1.
6. **G1** en dernier.

Blocages durs : P-V2 et P-CONC (V2-L et suivantes) ; V2-Z (F3b) ; D-N2 (F2) ; D-N1 (F3c). Le chantier temporel du Shell n'est plus un blocage de quoi que ce soit : F3c a besoin de T5 et T6, validées.

## Annexe A — Proposition de matrice C-1 à C-16 → tests (pour J-A)

Établie à partir des noms et rôles des tests du dépôt (`src/policy.rs`, `src/schedule.rs`, `src/clock.rs`, `src/progressor.rs`, `tests/temporal_progress.rs`, `tests/temporal_equivalence.rs`, `tests/temporal_structure.rs`, `tests/temporal_drivers.rs`) et des lignes de la matrice de la spécification (§ 10). **Proposition à valider** : elle repose sur la lecture des tests et non sur une nouvelle exécution ; la colonne « Mode » distingue les contrats vérifiés par exécution de ceux vérifiés par structure ou revue.

| Contrat | Tests (identifiants de la spéc. § 10) | Mode |
|---|---|---|
| C-1 q, K, M positifs, finis, explicites ; K et M indépendants | A7 (`a7_zeros_rejetes`, `a7_bornes_acceptees`, `a7_k_et_m_independants`, `a7_q_hors_plage`), E11 | Exécution (types `NonZero`) |
| C-2 stables par instance, jamais par appel | Signature : `progress(&mut self)` sans paramètre ; politique fixée à `Progressor::new` ; D7 | **Structure / revue** |
| C-3 échéancier espacé de q, sans dérive | P2–P3, P8, `avancement_sans_derive` (A) | Exécution |
| C-4 échéancier initial avant les échéances dues | P0, P0b, `construction_sans_lecture_ni_invocation` | Exécution |
| C-5 progression en cinq temps | P1 à P7, S1, S2 | Exécution |
| C-6 ensemble fixé à l'entrée ; échéances nées en cours comptées dans R | P5, P6, S7 | Exécution |
| C-7 cœur d'itération, coupure propre, aucun lot rejoué, lots vides comptés | E5, E6, AQ-1 à AQ-7b et RA rejoués via `Progressor` (D8), équivalence sur énumération, D1 | Exécution |
| C-8 R1 : t_fin + q ; phase non conservée | S2, S5, S6b | Exécution |
| C-9 échéance abandonnée jamais exécutée | S3, S4 | Exécution |
| C-10 compte rendu ; échéance toujours restituée hors terminal | Contrôle de `wake` dans chaque cas P/S (`expect`), D2 ; état non terminal sans échéance : branche `unreachable!` du pilote | Exécution **sauf** la branche impossible (revue) |
| C-11 valeurs de réveil par cas (D vide, R = 0, 0 < R ≤ M, R > M) | P1, P5, S1, S2, S6a, S6b | Exécution |
| C-12 terminal : aucune décision ; progression suivante sans effet | E1, E3, E4, E7, E8, E10 | Exécution |
| C-13 rien dû : aucune coupure | P1, D9, E6 | Exécution |
| C-14 seconde lecture ; horloge substituable à l'assemblage ; t_entrée et t_fin scriptables | A6, E9, harnais d'horloge (suite décroissante refusée, lectures comptées) | Exécution |
| C-15 non-garanties (durée réelle, latence, retard maximal) | — | **Non testable** (énoncé de non-garantie) ; documentation |
| C-16 aucune durée au `World`/scripts ; aucun type `winit`/`wgpu` ; aucune valeur de q, K, M dans le contrat | D7 (`[dependencies]` vide, aucun `winit`/`wgpu`, `Instant` confiné) ; valeurs de test confinées aux tests | **Structure** (D7) pour le fenêtrage ; **revue** pour le `World` et les scripts |

Quatre lignes ne sont pas des tests d'exécution : C-2, C-15, C-16 et la branche impossible de C-10. Leur acceptation est une décision de l'utilisateur ; elle est nécessaire pour prononcer J-A.
