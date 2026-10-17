# Exécution — étape 2 : Forge minimale

Document d'exécution, non normatif. **Il ne contient aucune règle de conception.** Source de vérité unique : `docs/implementation/02-verbum-forge.md` (r8, approuvé). En cas de divergence entre ce document et r8, **r8 prévaut** ; la divergence est signalée à l'utilisateur, jamais résolue par la session. Ce document n'est modifié que sur décision explicite de l'utilisateur.

## 0. Consigne d'amorce (à coller au début de la session)

> Vous implémentez l'étape 2 de Verbum (Forge minimale). Lisez, dans cet ordre : `docs/implementation/02-verbum-forge-execution.md` (règles de session), puis `docs/implementation/02-verbum-forge.md` (r8, référence de conception, à lire une seule fois en entier), puis `tree.md` (fait foi sur les chemins). Les ADR ne se relisent que si une unité les cite. Ne lisez pas les spécifications LPC. N'écrivez aucun code avant que l'utilisateur ait dit : « démarrez l'unité V1 ». Accusez réception en cinq lignes maximum.

## 1. Règles de session

1. **Une unité à la fois.** Le § 3 découpe r8 en unités de validation V1 à V8. À la fin d'une unité, la session s'arrête et rédige le rapport (§ 4). Elle ne commence l'unité suivante que sur validation explicite de l'utilisateur.
2. **Rouge avant vert.** Pour chaque unité : écrire les tests listés, contre des squelettes d'API (signatures, corps `unimplemented!()`), **constater qu'ils échouent pour la raison attendue**, puis implémenter. Un échec de compilation n'est pas un rouge valide. Pour les doctests `compile_fail` (M-6, F-9), appliquer la méthode de r8 : codes d'erreur épinglés au constat du rouge. Le rouge est rapporté (test, motif d'échec observé) avant l'implémentation ; l'utilisateur n'a pas à le valider séparément, sauf si la session le demande.
3. **Contrôles réels.** À chaque unité : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`. Le rapport donne les lignes de résultat réelles (récapitulatif des tests, avertissements, codes de sortie) ; la sortie complète n'est fournie qu'en cas d'échec. Aucun résultat n'est déduit, estimé ni rapporté sans avoir été produit.
4. **Miri et MSRV 1.85 sont exécutés par l'utilisateur.** La session fournit les commandes exactes et attend les résultats avant de déclarer l'unité close.
5. **Pas de commit par la session.** Elle propose, en fin d'unité, le regroupement des changements par invariant démontré ; les messages et les commits restent à la charge de l'utilisateur. `tree.md` est régénéré par le script de l'utilisateur : la session signale quand des fichiers ont été créés, sans y toucher.
6. **Noms provisoires.** Les noms de r8 marqués provisoires peuvent être ajustés si nécessaire ; tout ajustement est listé dans le rapport. Aucun changement de sémantique.
7. **Économie de contexte** (limite de la session) : lire r8 une fois, puis seulement la section de l'unité en cours ; ne pas relire les fichiers que la session vient d'écrire ; rapports concis. Si la session constate qu'elle ne peut plus tenir r8 en mémoire de travail, elle le dit et s'arrête à la fin de l'unité en cours (§ 6).

## 2. Interdits sans arbitrage explicite de l'utilisateur

- Rouvrir un arbitrage de r8 (Q-I à Q-N et §8) ou modifier r8.
- Modifier les ADR, `ROADMAP.md`, `crates/ecs`, `crates/schedule`, `crates/verbum` (K8).
- Ajouter une dépendance autre que `toml` (forge) et `verbum-ecs` (dev-dependency de la forge) ; ajouter `alloc` ou une dépendance à `verbum-assets`.
- Élargir le périmètre (r8 §10) : registre, second type d'asset, CLI, `build.rs`, hash configurable dans l'API publique, collection dynamique dans le chargement.
- Étendre la portée d'un test ou d'une API au-delà de ce que r8 liste.

## 3. Unités

Chaque unité renvoie à r8 pour tout le détail (invariants, fichiers, interfaces, tests, critères). « Tests » désigne les identifiants de r8 ; ils sont tous obligatoires.

| Unité | r8 | Contenu | Tests | Contrôles propres | Passage à l'unité suivante |
|---|---|---|---|---|---|
| **V1** | §6 phase 1 | Crate `verbum-assets` (`no_std`, sans `alloc`, sans dépendance), `Sprite`, lecture, `ReadError` | P1.1 à P1.6 | `cargo tree -p verbum-assets` sans dépendance ; **Miri (utilisateur)** | P1.1 à P1.6 verts ; critère de sortie de la phase 1 |
| **V2** | §6 phase 2 | Crate `verbum-forge`, `compile_sprite`, `ForgeError` | P2.1 à P2.3 | `cargo tree -i toml --workspace` ne liste que `verbum-forge` ; **MSRV (utilisateur)** | P2.1 à P2.3 verts ; MSRV vert |
| **V3** | §6 phase 3 | `crates/forge/tests/pipeline.rs`, chaîne complète tranche A | P3.1, P3.2 | Avant d'écrire le test : relire les signatures réelles de `World`, `ResourceSlot`, `resources_mut` ([Lu] dans r8) ; divergence = arrêt | Critères de la tranche A (r8 §1.3). **Jalon intermédiaire** : le rapport l'énonce comme tel et ne revendique pas la validation complète d'ADR-A01 |
| **V4** | §6 phase 4, partie 1 | `AssetId` opaque, `ArtifactIndex`, `SlotIndex` (dont `SlotIndex::new`), `Manifest<'a>::parse`, `ManifestEntry`, `find`, `ManifestError` | M-1 à M-6, M-8 | `cargo tree -p verbum-assets` sans dépendance | Tests verts ; opacité établie par M-6 avec codes d'erreur épinglés |
| **V5** | §6 phase 4, partie 2 | `check_bindings`, `BindingError`, `load`, `LoadError` (niveaux 1 à 4, double décodage, dépôt) | M-7, L-1 à L-5 | **Miri (utilisateur)** | Tests verts ; critère de sortie de la phase 4 ; L-3 vérifie le compteur à zéro dans les quatre cas |
| **V6** | §6 phase 5, partie 1 | `id.rs` (FNV-1a), `batch.rs` : `compile_assets`, point de test privé à calculateur injectable, sorties nominales | F-1, F-2, F-3, F-5, F-7, F-8 | Les constantes attendues de F-1 sont les littéraux de r8, pas des valeurs calculées par la fonction testée | Tests verts ; l'API de la tranche A reste valide |
| **V7** | §6 phase 5, partie 2 | Rejets et collision : clé vide, dupliquée, slots, TOML invalide ; non-fuite du point de test | F-4, F-6, F-6b, F-9 | Relecture de l'API publique : `compile_assets` sans paramètre de hash ; `cargo tree -i toml --workspace` inchangé | Tests verts ; critère de sortie de la phase 5 |
| **V8** | §6 phase 6 | Extension de `pipeline.rs` : application de test à deux `ResourceSlot<Sprite>`, chaîne complète ; clôture | T6.1 à T6.6 | `cargo tree -p verbum-assets` ; `cargo tree -i toml --workspace` ; aucune modification de `crates/ecs`, `crates/schedule`, `crates/verbum`, ADR, Roadmap ; **Miri sur `verbum-assets` et MSRV (utilisateur)** | **Clôture de l'étape** (§ 5) |

Le découpage de la phase 4 et de la phase 5 en deux unités est un ordre d'exécution ; les phases de r8 ne sont ni renumérotées ni modifiées.

## 4. Rapport de fin d'unité (gabarit, rien de narratif)

1. **Unité** et fichiers créés ou modifiés (liste).
2. **Rouges observés** : test → motif d'échec constaté.
3. **Contrôles** : quatre commandes, résultat réel de chacune (lignes pertinentes).
4. **Contrôles propres à l'unité** (colonne du § 3) : résultat, ou « à exécuter par l'utilisateur » avec la commande exacte.
5. **Écarts par rapport à r8** : aucun, ou liste (noms ajustés, divergence constatée).
6. **Questions d'arbitrage ouvertes** : aucune, ou liste avec les options, sans choix.
7. **Regroupement de commit proposé** (périmètre seulement).
8. Fin : « En attente de la validation de l'unité Vn. »

## 5. Clôture de l'étape (V8)

Le rapport de V8 contient en plus un tableau K1 à K8 : pour chacun, l'identifiant du test qui l'établit et son résultat réel ; r8 §8.4 (éléments non démontrés) reproduit tel quel en complément. La session ne déclare pas l'étape close : l'utilisateur la déclare. Aucun travail de l'étape 3 n'est entrepris.

## 6. Arrêt, reprise

**Arrêt immédiat, sans choix de la session**, avec exposé des options : contradiction ou ambiguïté de r8 ; divergence entre un fait [Lu] de r8 et le code réel ; incompatibilité d'une dépendance (par exemple `toml` et MSRV 1.85) ; test dont le rouge ne peut pas être constaté pour la bonne raison ; besoin de modifier un fichier interdit (§ 2).

**Reprise dans une nouvelle session** (si la session se dégrade ou se coupe) : fournir ce document, r8, `tree.md` et la dernière phrase de rapport validée (« Unité Vn validée »). La nouvelle session relit le code déjà écrit seulement pour l'unité qu'elle reprend, constate l'état réel avec les quatre contrôles et reprend à l'unité suivante. Une unité interrompue en cours est refaite depuis ses tests.
