# Verbum — Demo-0 : premier affichage natif — spécification r1

## Statut
Document de spécification, non normatif, révisable sans ADR. **Proposition, non adoptée** : rien ici ne vaut décision sans validation explicite de l'utilisateur. Ce n'est ni un ADR ni un plan d'exécution ; la chaîne plan → exécution → handoff reste à produire après validation. Aucun code, test, manifeste, ADR ou document existant n'est modifié par ce document.

Légende : **[Fait]** constaté dans un fichier fourni · **[Contrat]** posé par un ADR adopté · **[Proposition]** choix de ce document · **[Arbitrage]** décision à rendre (§ 9) · **[Inconnu]** non vérifié.

## 1. Objectif et résultat visuel
Une application `demo-0` ouvre une fenêtre native, affiche un fond `#333333` (RGB 51, 51, 51) et un carré `#FF8C00` (RGB 255, 140, 0), image statique conservée jusqu'à la fermeture de la fenêtre. Elle sert de première démonstration pédagogique durable : elle ne doit contenir aucun chemin propre à Demo-0 dans l'Engine.

Résultat attendu (conventions [Proposition]) : zone cliente de **800 × 600 pixels physiques** ; carré de **200 × 200** dont le coin supérieur gauche est en (300, 200) (centré à la création) ; origine en haut à gauche, Y vers le bas. Le carré garde sa position en pixels si la fenêtre est redimensionnée.

## 2. Périmètre et exclusions
Inclus : fenêtre, surface, première image, `Progressor` réel, `World` et `Scheduler` réels, `Frame` réelle, redessin sans nouveau contenu, redimensionnement et minimisation sans plantage, fermeture propre.
Exclus : entrée clavier/souris (l'`Inbox` est câblée, lots vides), animation, caméra, couches, ordre d'empilement, assets, Forge, VPKG, scripts, audio, résolution virtuelle, mise à l'échelle, sRGB géré, transparence, hot-reload, plateformes mobiles, toute API publique `Backend`.

## 3. Responsabilités et pipeline

### 3.1 Faits vérifiés qui conditionnent la conception
- [Fait] `verbum-shell` n'a aucune dépendance ; il ne contient ni fenêtre, ni surface, ni `winit`/`wgpu`. L'intégration native était hors chantier (roadmap § 1). Aucun crate Backend ni plateforme n'existe (`tree.md`, 21 novembre 2026).
- [Fait] `Progressor::progress()` (non bloquant, compte rendu `{iterations, wake, state}`), `TemporalPolicy::from_nanos(q, K, M)`, `MonotonicClock` (`start`, `elapsed`, `origin`), `Deadline::as_duration()`, `take_error()` existent et sont validés. Le pilote événementiel n'existe que sous forme de harnais de test (`temporal_drivers.rs`, `EventApp`).
- [Fait] `Frame<CAP>` : tableau fixe + longueur, `Clone`, `PartialEq`, `Eq`, aucun mutateur. `Contribution { x: i32, y: i32, width: u32, height: u32, color: [u8; 4] }`. Il n'existe ni couleur d'effacement, ni caméra, ni clé de couche, ni règle d'ordre (C8, C12 de l'étape 4).
- [Fait] Les systèmes `Clear`, `Compose`, `ProduceA/B` n'existent que dans les tests et exemples (`frame_loop.rs`, `headless_frame.rs`), non exportés.
- [Fait] Workspace : `members` sans `crates/games/*` ; `[workspace.dependencies]` sans `verbum-shell` ; `rust-version = "1.85"` ; `unsafe_code = "deny"`.

### 3.2 Pipeline
```
games/demo-0 (Application)
  ├─ construit : MonotonicClock, TemporalPolicy, Inbox, FrameSink, World, Scheduler,
  │              Progressor (fermeture d'infrastructure : publier lot → run_tick → lire Frame → FrameSink)
  └─ remet le contrôle à l'adaptateur de plateforme
engine/platform (Engine, rôle Shell — à créer)
  ├─ EventLoop + ApplicationHandler (winit), fenêtre, surface, device, Backend wgpu
  ├─ about_to_wait : progress() ; si iterations > 0 et Frame changée → request_redraw ;
  │                  ControlFlow::WaitUntil(origin + deadline)
  ├─ RedrawRequested : exécute la dernière Frame du FrameSink (ou fond seul s'il est vide)
  └─ CloseRequested : exit()
```
Répartition [Proposition, fondée sur ADR-S01 pt 1, 4, 7, ADR-S02 pt 8-9, ADR-R03, ADR-W01] :

| Question | Réponse |
|---|---|
| Qui crée et possède la fenêtre | l'adaptateur de plateforme, dans `resumed` (aucun code Games) |
| Qui initialise wgpu / le Backend | l'adaptateur de plateforme, après création de la fenêtre ; Games ne nomme aucun type `winit`/`wgpu` |
| Qui possède la boucle | winit pilote la distribution ; le Shell (`Progressor`) possède la sémantique d'exécution ; modèle monothread (S02 pt 9) |
| Comment l'application fournit les données | systèmes Games → `Frame` (Ressource) → fermeture d'infrastructure → `FrameSink` ; le Backend ne reçoit que la `Frame` |
| Comment l'image est produite et présentée | `RedrawRequested` : encodage et présentation de la dernière `Frame` retenue |
| Fermeture | `CloseRequested` → `event_loop.exit()` directement par l'adaptateur (S02 pt 7) ; Backend et surface détruits avant la fenêtre, puis `Progressor` |
| Retour du Backend | aucun vers l'ECS ; erreur fatale du Backend → adaptateur (même crate), qui arrête la boucle (ADR-R03 pt 5, S01 pt 6) |

`FrameSink` : emplacement monothread (`Rc<RefCell<…>>`) à `Frame<CAP>` copiée par `Clone` à chaque tick effectif ; un indicateur « changée » dérivé de `PartialEq`. Pas d'`unsafe`, pas de trait public. C'est le puits décrit par l'axe C (C1) de la page d'axes, non adoptée.

## 4. Représentation du carré : décision et justification
**Recommandation : option A, le carré est une `Contribution`.**

- Option B, impossible sans infrastructure nouvelle : le `Sprite` runtime est un descripteur `{width, height}` sans pixels ni couleur (étape 2, étape 5 A-1) ; aucun contrat ne relie un asset chargé à une `Contribution` ; le Backend ne reçoit que la `Frame` (ADR-R03 pt 1). Rendre un sprite orange exigerait données raster, référence d'image dans `Contribution` et upload GPU : décisions d'architecture absentes (étape 5 § 10, A-1 option 3).
- Option A utilise un contrat existant : un producteur émet une `Contribution { x: 300, y: 200, width: 200, height: 200, color: [255, 140, 0, 255] }`, composée par `Compose` selon ADR-R02. Aucun contournement.

**Le fond n'a pas de représentation dans le contrat actuel** [Arbitrage A-1]. Écarté : une contribution plein écran (recouvrement avec le carré, donc dépendance à un ordre d'empilement que le corpus n'a pas posé, C8) ; un attribut de `Frame` (clear-color écarté en C12, modifierait ADR-R01 pt 1). **Recommandation : constante de fond propre au Backend** (`#333333`), conforme à la convention locale de la page d'axes (axe I). Conséquence assumée : le fond n'est pas piloté par l'application ; une démo ultérieure qui veut un fond variable devra décider d'un mécanisme.

Conventions du Backend [Proposition] : unités des contributions = pixels physiques de la surface ; format de surface non sRGB préféré (les octets sont écrits tels quels) ; alpha opaque seul ; rectangle borné à la surface.

## 5. Organisation dans le workspace
Chemin proposé : **`crates/games/demo-0/`**, package `verbum-demo-0`, binaire, membre du workspace, `publish = false`. Le chemin « `games/demo-0/` » suggéré se confronte à la réalité `crates/games/…` (`tree.md`). `crates/games/demo/` (README vide, hors workspace) **n'est ni renommé ni supprimé** par cette spécification : D-N5/F3a (nom, portée, promotion de `games/demo`) reste ouvert ; ce choix de chemin en tranche **le nom pour Demo-0 seulement** et doit être validé comme tel [Arbitrage A-5].

Graphe (ADR-W01, Application → Engine, jamais l'inverse) :
- `verbum-demo-0` → `verbum-ecs`, `verbum-schedule`, `verbum-render`, `verbum-shell`, `verbum-platform` ;
- `verbum-platform` (nouveau, Engine, `std`) → `verbum-shell`, `verbum-render`, `winit`, `wgpu`, un exécuteur ponctuel ;
- `verbum-shell`, `verbum-render` : inchangés, sans dépendance normale ; aucun Engine ne nomme l'Application.

Topologie [Arbitrage A-3] : **un seul crate de plateforme** (adaptateur + Backend + `FrameSink`, option G2 de la page d'axes), car cela évite de publier une API de Backend ou un second callable (axe D). Nom provisoire `crates/engine/platform`. Coût : winit et wgpu compilés par `cargo … --workspace`.

Code de `demo-0` : `Clear`, `Compose`, `ProduceSquare`, `Binding`, `Locate`, lot (motif C9 de l'étape 4/3) recopiés du motif de `headless_frame.rs`, sans durée de vie (pas d'assets, donc pas de contournement E-12). La duplication de `Clear`/`Compose` est assumée [Arbitrage A-7].

## 6. Préalables techniques réels

| # | Prérequis | État | Manquant / action |
|---|---|---|---|
| P1 | Adaptateur winit + Backend wgpu | **absent** | nouveau crate `crates/engine/platform` ; périmètre minimal : fenêtre, surface, clear + un rectangle, `FrameSink`, gestion `Resized`/taille nulle/statuts de surface |
| P2 | Plancher Rust | 1.85, wgpu 30.0.1 exige 1.87 | relever `rust-version` du `Cargo.toml` racine avant la première résolution ; réécrire les contrôles `+1.85.0` ; discipline « sans let-chains » valable jusqu'à 1.88 |
| P3 | Dépendances | aucune | `winit` 0.30.13, `wgpu` 30.0.1, exécuteur ponctuel pour `request_adapter`/`request_device` [Arbitrage A-6] ; `Cargo.lock` croît fortement |
| P4 | Transmission de la `Frame` (R03/S05 non résolu) | ouvert | D2 (transmission au tick, exécution sur `RedrawRequested`) retenu comme choix de Demo-0 [Arbitrage A-4] ; pas d'ADR rédigé |
| P5 | Valeurs q, K, M (D-N1) et provenance de q | ouvert | voir A-2 |
| P6 | Promotion workspace (D-N5/F3a) | ouvert | ajouter le membre `crates/games/demo-0` ; ajouter `verbum-shell` et `verbum-platform` à `[workspace.dependencies]` |
| P7 | `Deadline` non représentable en `Instant` (PO-3) | ouvert | `origin.checked_add(deadline)` ; sur `None`, arrêt avec erreur explicite (jamais de panique) |
| P8 | Systèmes réutilisables | non exportés | recopie dans l'Application (A-7) |
| P9 | Comportement wgpu 30 | **[Inconnu]** | `Arc<Window>` → `Surface<'static>`, futures prêtes au premier `poll`, API d'acquisition (`CurrentSurfaceTexture`) : à vérifier à l'implémentation ; aucune signature wgpu/winit n'est fixée ici |

Hors périmètre mais consignés : ADR-000 pt 3 (point ouvert sur l'allocation hors `world.execute()`) n'est pas fermé par Demo-0 ; le code propre à Verbum du chemin de présentation n'alloue pas par tick au-delà de ce que le contrat actuel impose (copie de `Frame` à capacité fixe) ; les allocations internes de wgpu sont hors invariant (page d'axes, arbitrage 6).

## 7. Critères d'acceptation

Observables (manuels, une machine desktop avec GPU) :
1. `cargo run -p verbum-demo-0` ouvre une fenêtre native sans autre action.
2. Fond uni `#333333` (51, 51, 51 relevé à la capture d'écran).
3. Un seul carré `#FF8C00` (255, 140, 0), 200 × 200, centré dans la zone cliente de 800 × 600 à l'ouverture.
4. L'image reste identique après plusieurs secondes, après masquage puis restauration de la fenêtre, et après un redimensionnement (fond et couleur inchangés, aucun plantage).
5. Sans événement, le processus ne consomme pas de CPU de façon continue (pas de boucle active ; réveils ≤ une fois par quantum).
6. Fermeture de la fenêtre : le processus se termine avec le code 0, sans panique ni message d'erreur.

Automatisés (`cargo test --workspace`, sans fenêtre) :
7. Un test de `demo-0` exécute le `Scheduler` réel sur le `World` réel pendant un tick et vérifie que la `Frame` est égale à exactement `[carré]` ; un test unitaire de la plateforme vérifie la conversion contribution → coordonnées de la cible (cas limites : rectangle partiellement hors surface, taille nulle).
8. `cargo tree -p verbum-shell -e normal` et `-p verbum-render -e normal` : aucune dépendance ; aucun crate Engine ne dépend de `verbum-demo-0`.
9. Aucun `unsafe` (lint du workspace) ; aucun type `winit`/`wgpu` dans `verbum-demo-0`, `verbum-render`, `verbum-shell`.
10. Quatre contrôles du workspace verts ; MSRV et Miri : à redéfinir après P2 (Miri n'est pas applicable au code natif graphique).

La conformité visuelle n'est pas vérifiable automatiquement : le test de rendu (lecture de pixels) est hors périmètre.

## 8. Ordre d'implémentation proposé
Une unité par session, validation entre chaque ; plan et exécution à rédiger après validation de cette spécification.
- **D0-0** Décisions préalables (§ 9) ; relèvement `rust-version` ; expériences E12 (futures prêtes), E1 (callbacks), à l'aide d'une copie hors dépôt.
- **D0-1** Crate `demo-0` sans fenêtre : systèmes, `Progressor` piloté par `run_blocking` et horloge scriptée ; critère 7.
- **D0-2** Crate `platform` : fenêtre, surface, fond seul (fond, fermeture, redimensionnement).
- **D0-3** Backend : rectangle (carte contribution → cible), statuts de surface, taille nulle.
- **D0-4** Intégration : `FrameSink`, `about_to_wait`, `WaitUntil`, redessin conditionnel.
- **D0-5** Clôture : critères 1 à 10, liste « non démontré » ; la clôture est déclarée par l'utilisateur.

## 9. Arbitrages à soumettre
- **A-1** Fond : constante du Backend (recommandé) ou attribut de `Frame` (modifie ADR-R01).
- **A-2** q, K, M et provenance : Demo-0 n'a aucun asset, donc aucun paquet ; la règle de l'unité F3b (q lu du paquet seul) ne s'applique pas telle quelle. Recommandé : `TemporalPolicy::from_nanos` avec constantes de composition documentées (illustration : q = 16 666 667 ns, K = 4, M = 8, déjà notée « non sélectionnée »), à valider comme D-N1 pour Demo-0 seulement. Alternative : paquet VPKG vide (28 octets, valide) mais sa production exigerait la Forge à l'exécution ou un littéral écrit à la main.
- **A-3** Un crate `platform` unique (G2) ou séparation adaptateur / Backend (G1, exige un second callable) ; nom du crate.
- **A-4** D2 retenu ; avec K > 1, la `Frame` est copiée dans le puits jusqu'à K fois par progression, une seule exécution graphique en résulte. L'invariant « 0..1 transmise » (S01 pt 5) n'est donc pas implémenté tel quel : à tolérer pour Demo-0, ou fixer K = 1. Une clarification d'ADR-R03 reste à rédiger plus tard.
- **A-5** Chemin `crates/games/demo-0`, `crates/games/demo/` laissé intact.
- **A-6** Exécuteur ponctuel : crate dédié (recommandé) ou `Waker::noop` maison selon E12 ; plancher MSRV exact.
- **A-7** Recopie de `Clear`/`Compose` dans l'Application, ou crate Engine/Extension à décider ultérieurement.
- **A-8** Conventions 800 × 600 physiques, carré 200 × 200 en (300, 200), pixels physiques sans mise à l'échelle.
