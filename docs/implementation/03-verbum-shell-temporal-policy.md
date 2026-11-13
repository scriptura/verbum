# Verbum — Spécification d'implémentation de la politique temporelle du Shell

## 0. Statut, marqueurs, périmètre

**Non normative. Non implémentée. Révision 3, consolidée (12 novembre 2026).** Traduction minimale et fidèle d'ADR-S04 (adoptée le 11 novembre 2026) dans `verbum-shell`, après les audits des 12 novembre. Aucun code, test, dépendance, ADR, format d'artefact ni `Cargo.toml` n'est créé ou modifié par cette mission. La porte G-CM (étape 5, `VMAN`) n'est pas contournée. **Aucune implémentation n'est autorisée par ce document.** La validation appartient à l'utilisateur ; la roadmap d'implémentation relève d'une session ultérieure.

| Marqueur | Sens |
|---|---|
| **[Contrat]** | Posé par une ADR adoptée (référence donnée). |
| **[Fait]** | Observé dans un fichier fourni (référence donnée). |
| **[Arbitré]** | Décision de l'utilisateur (12 novembre 2026). |
| **[Proposition]** | Choix d'implémentation de cette spécification, à valider. |
| **[Ouvert]** | Non déterminé par les textes ni par le code ; énoncé sans choix. |
| **[Dépendance]** | Relève d'une autre décision ou d'un autre chantier. |

Les extraits Rust sont des **esquisses non normatives**. Vocabulaire local (repris d'ADR-S04) : **échéancier** (suite d'instants planifiés espacés de q) ; **échéance due** (planifiée ≤ t, ni exécutée ni abandonnée) ; **résiduel R** (échéances dues à t_fin) ; **réancrage** (nouvelle prochaine échéance après abandon) ; **progression** (demande d'exécuter les itérations dues) ; **checked** (arithmétique entière qui rend une erreur au lieu de déborder) ; **précondition** (condition que l'appelant garantit ; son non-respect n'est pas un cas de fonctionnement) ; **paquet** (conteneur unique manifeste + artefacts, § 7).

## 1. Fichiers lus et lacunes restantes

Lus : normatifs de la mission ; ADR-A01, S01, S02, S03, S04, W01 ; page d'axes ; note technologique ; `Cargo.toml` racine et Shell ; Shell (`shell.rs`, `pacing.rs`, `inbox.rs`, `lib.rs`), tests et exemples ; `assets/src/{lib,format,load,header,manifest}.rs` ; `forge/src/{lib,batch,script}.rs` ; `script/src/{step,state}.rs` ; plan étape 5 (§ 1.3, G-CM) et document d'exécution (§ 3.2, § 6) ; audits du 12 novembre.

Lacunes, sans effet sur les conclusions : `assets/src/script.rs` non fourni (la **valeur** de `SCRIPT_MAX_STEPS_PER_TICK` n'est pas observée ; elle ne conditionne pas K) ; tests Forge assertant des octets de manifeste et `shell/tests/{level_loaded,borrowed_resource}.rs` non lus (recensement relevant du plan V2, § 7.4).

## 2. Analyse de l'existant [Fait]

| # | Fait | Source |
|---|---|---|
| F-1 | `run_loop(cadence, stop, inbox, infrastructure) -> Result<u64, E>` est la seule boucle : `cadence()` → `stop()` (si `true`, `Ok(n)`) → `infrastructure(inbox.cut())?` → `n += 1`. Fermetures par valeur, `Inbox` par `&mut`, `Ev: Copy`. `cadence` est un `FnMut()` qui ne rend rien : **aucun instant**. Elle ne connaît ni K, ni M, ni échéancier : une itération par retour de `cadence`. | `shell.rs` |
| F-2 | `FixedStepPacing` lit `Instant::now()` (non substituable), dort par `thread::sleep`, mémorise `next: Option<Instant>`. Premier appel : échéance = maintenant + intervalle complet. Appel en retard : aucun sommeil, `next = now`. Se déclare provisoire. | `pacing.rs` |
| F-3 | `lib.rs` : « n'expose aucun trait public », « n'a aucune dépendance ». | `lib.rs` |
| F-4 | Suites appelant `run_loop` avec la signature actuelle : LC-1 à LC-4, PC-1, PC-3 (seconde cadence par fermeture), AQ-1, 1b, 2, 3, 5, 6, 6b, 7, 7b, AC-1, 2, 4, 5, 6, 7 (repérés), RA-1 à RA-5. | tests Shell |
| F-5 | Exemples appelant `run_loop` : `console`, `headless_frame`, `script_headless` (`FixedStepPacing` de 1 ms) ; `level_headless` (cadence `\|\| {}`, **sans attente**). Aucun ne met en œuvre une politique K/M. | exemples |
| F-6 | Shell : `[dependencies]` vide ; dev-dependencies assets, ecs, forge, render, schedule, script ; `[lints] workspace = true`. Racine : `resolver = "3"`, édition 2024, `rust-version = "1.85"`, `unsafe_code = "deny"` seul. | `Cargo.toml` ×2 |
| F-7 | Assets : `#![no_std]`, sans dépendance. `load(manifest: &[u8], artifacts: &[&[u8]], slot_formats, deposit)`, niveaux 1 à 4, tout-ou-rien. `LoadError` = {Manifest, Binding, SlotFormat, Artifact, Reference} : aucune variante temporelle. | `load.rs` |
| F-8 | `VMAN` v1 : `MANIFEST_VERSION = 1`, `MANIFEST_HEADER_LEN = 10` (signature 4, version `u16`, `n` `u32`), entrées de 16 octets : longueur exacte 10 + 16 n. Aucun champ temporel, aucune empreinte. `ManifestError` = {Truncated, Signature, Version, TrailingBytes, DuplicateId}. | `manifest.rs` |
| F-9 | Forge : `wait.ticks` ∈ `1..=u32::MAX`, aucune conversion durée → ticks. `compile_assets(&[AssetSource<'_>], usize) -> Result<CompiledSet, ForgeError>` : **signature épinglée par un doctest** ; délègue à `compile_assets_with` (crate-privé, calculateur de hash injectable). `CompiledSet` : champs privés `manifest: Vec<u8>` et `artifacts: Vec<Vec<u8>>`, exposés **séparément** (`manifest()`, `artifacts()`). | `batch.rs` |
| F-10 | Le Shell n'a pas accès à `verbum-assets` hors tests : q est obtenu dans le code d'assemblage (niveau Application, ADR-W01). Le Shell ne peut pas vérifier d'où provient le q qu'on lui remet. | `Cargo.toml` Shell |
| F-11 | Note technologique : `winit` 0.30.13, `about_to_wait`, `ControlFlow::WaitUntil(Instant)` ; sélection provisoire, micro-PoC non réalisée. | note technologique |
| F-12 | `World`, scripts : ticks seulement (`Wait { ticks: u32 }`). | `step.rs`, S04 pt 1 |
| F-13 | `Inbox<Ev: Copy>` : `Arc<Mutex<Content>>` + lot `Vec<Ev>` préalloué ; `cut(&mut self) -> &[Ev]` **emprunte** l'`Inbox` ; verrou libéré avant le retour ; allocation à la construction seulement. `DepositHandle<Ev>` clonable (`Arc`), obtenu par `Inbox::handle(&self)`, sans durée de vie liée à l'`Inbox`. | `inbox.rs` |
| F-14 | G-CM (plan étape 5 § 1.3 pt 2 ; exécution § 3.2) : « modification de `VMAN` (constantes, layout, octets produits pour un ensemble donné) » = **arrêt avec rapport**, sans contournement ; reprise « seulement sur instruction explicite » (§ 6). | plan, exécution |
| F-15 | R-ADAPT : seules des modifications **purement mécaniques** de signature sont admises dans les tests de l'étape 2 ; toute modification d'assertion ou de valeur littérale attendue = arrêt G-CM. | exécution § 3.1 |
| F-16 | Littéraux de manifeste v1 (42 octets) dans `load.rs` (tests) et dans les doctests LT-5n/LT-5t de `shell/src/lib.rs`. | `load.rs`, `lib.rs` Shell |
| F-17 | Aucun contenu d'artefact ne dépend de q aujourd'hui (F-9) : la dépendance apparaîtra avec la règle des durées non entières (F2). | `forge/script.rs` |

## 3. Contrat temporel [Contrat]

| # | Invariant | Réf. |
|---|---|---|
| C-1 | q, K, M sont **strictement positifs, finis, explicites** ; q est une durée ; K et M sont des entiers indépendants (pas de K = M/q). | S04 pt 2, 5 |
| C-2 | q, K, M sont **stables pour une instance**, fixés à sa composition, jamais fournis par appel de progression ni par l'adaptateur. | S04 pt 5, 13 ; S02 pt 5 |
| C-3 | Échéancier : échéances espacées de q, repère sur la prochaine non exécutée ni abandonnée ; sans réancrage, l'échéance suivante = précédente + q : aucune dérive. | S04 pt 3.1 |
| C-4 | Un échéancier initial est établi **avant** de déterminer les itérations dues ; mécanique distincte du réancrage. | S04 pt 3.2 |
| C-5 | Progression en cinq temps : (1) t_entrée et D ; (2) D vide → fin sans effet ; (3) au plus K itérations de D ; (4) t_fin et R ; (5) R ≤ M conservé, R > M abandonné en entier avec réancrage. | S04 pt 4 |
| C-6 | Ensemble exécuté fixé à l'entrée ; échéances nées pendant l'exécution comptées dans R seulement ; itérations = min(K, \|D\|). | S04 pt 4 |
| C-7 | Chaque itération applique le cœur d'ADR-S02 pt 2 avec sa propre coupure ; aucun lot rejoué ; lots vides comptés. | S04 pt 9.1 |
| C-8 | **R1** : si R > M, prochaine échéance = t_fin + q ; phase non conservée ; la prochaine itération n'est pas immédiatement due. | S04 pt 7 |
| C-9 | Une échéance abandonnée ne devient jamais une itération ; aucun tick sauté. | S04 pt 1 |
| C-10 | Compte rendu : itérations (≤ K), besoin de réveil, état terminal. Hors terminal, une échéance est toujours restituée. Rien sur les abandons. | S04 pt 8 ; S02 pt 6 |
| C-11 | Réveil : D vide → prochaine échéance (> t_entrée) ; R = 0 → prochaine échéance (> t_fin) ; 0 < R ≤ M → prochaine échéance non exécutée, **déjà due** ; R > M → t_fin + q. Une échéance restituée peut être antérieure à l'instant de restitution. | S04 pt 8 |
| C-12 | Progression terminale (arrêt, erreur, échec de publication) : aucune décision de conservation ni d'abandon. Progression suivante : aucune itération, état terminal inchangé. | S04 pt 4 ; S02 pt 4, 7 |
| C-13 | Progression sans effet si rien n'est dû : aucune coupure. | S02 pt 4 |
| C-14 | Seconde lecture d'horloge autorisée ; source substituable à l'assemblage, jamais par progression ; t_entrée et t_fin scriptables séparément. | S04 pt 4, 12 |
| C-15 | Non-garanties : durée réelle d'une progression, latence, retard maximal, convergence. K ne borne pas la durée ; M ne borne que R à t_fin. | S04 pt 5, 10 |
| C-16 | Aucune durée ni instant réel pour le `World`, les systèmes, les scripts ; aucun type `winit`/`wgpu` ; aucune valeur numérique de q, K, M dans le contrat. | S04 pt 13 |

## 4. Représentation arithmétique [Proposition]

### 4.1 Types

| Grandeur | Type | Justification |
|---|---|---|
| Instant interne | `u64` nanosecondes depuis une origine fixe de l'horloge | Entier exact, 584 ans de plage ; pas de flottant ; pas d'`Instant` dans le cœur (non substituable : F-2). |
| q | `NonZeroU64` ns | C-1 porté par le type ; division sans cas nul. Même représentation que `q_ns` du manifeste v2. |
| K, M | `NonZeroU32` chacun | C-1. |
| `next` | `u64` ns | Même domaine que l'instant. |
| Itérations rapportées | `u32` | ≤ K. |
| Échéance restituée | `Deadline(Duration)` depuis l'origine de l'horloge | Pas d'`Instant` dans le contrat ; le pilote convertit (§ 9). |

### 4.2 Conversions

| De → vers | Opération | Échec |
|---|---|---|
| Lecture `Duration` → `u64` ns | `u64::try_from(d.as_nanos())` | `TimeError::ClockOutOfRange` |
| q `Duration` ou `q_ns` → `NonZeroU64` | `NonZeroU64::new` (plage vérifiée à la conversion) | `PolicyError::QuantumOutOfRange` / `ZeroQuantum` |
| `u64` ns → `Deadline` | `Duration::from_nanos` | infaillible |
| `Deadline` → `Instant` (pilote seul) | `origine.checked_add(duration)` | traité par le pilote (PO-3) |

### 4.3 Formules (temps constant, sans boucle, sans flottant)

Soit `next` la prochaine échéance non exécutée, `t` un instant.

- **Échéances dues** : si `t < next` alors \|D\| = 0 ; sinon `s = (t − next) / q` et \|D\| = s + 1.
- **Itérations exécutées** : `e = 0` si `t < next` ; `e = K` si `s ≥ K` ; sinon `e = s + 1` (≤ K). Équivalent à min(K, \|D\|), sans calculer s + 1 (qui déborde si q = 1 et écart maximal).
- **R > M** à t_fin : `t_fin ≥ next` **et** `(t_fin − next) / q ≥ M`. R ≥ 1 ⇔ `t_fin ≥ next`.
- **Avancement** après chaque itération comptée : `next ← next.checked_add(q)`.
- **Réancrage** : `next ← t_fin.checked_add(q)`.

| Opération | Débordement possible | Traitement |
|---|---|---|
| `t − next` | non (test préalable) | « non dû » n'est pas une erreur |
| `(t − next) / q` | non (q ≠ 0) | — |
| `s + 1` | oui (q = 1, écart maximal) | évité par la formulation de `e` |
| `next + q`, `t_fin + q` | oui près de `u64::MAX` | `TimeError::ScheduleOverflow` |
| Lecture > 584 ans | oui | `TimeError::ClockOutOfRange` |

**Précondition de la source de temps.** La source est **monotone** (non décroissante), comme l'horloge réelle. Le comportement face à une source qui décroît n'est **pas spécifié** et n'est pas un cas de fonctionnement. Les sources substituées respectent la même précondition : le harnais de test vérifie la monotonie de ses séquences à leur construction et refuse toute séquence décroissante. L'implémentation évite toute soustraction non gardée (pas de panique), sans que cela définisse un comportement.

**Preuve exécutable** (réalisée pour cette spécification, modèle de référence) : comparaison par énumération exhaustive (q 1..5, next 0..19, t 0..59, K 1..5, M 1..7) à la définition naïve : `e` et « R > M » coïncident ; q = 1, next = 0, t = `u64::MAX` rend `e = K` sans débordement ; 100 000 progressions à retard nul ne dérivent pas.

### 4.4 Erreurs : construction et fonctionnement [Arbitré D-2]

| Famille | Erreur | Effet |
|---|---|---|
| **Construction** | `PolicyError::{ZeroQuantum, QuantumOutOfRange, ZeroBudget, ZeroThreshold}` | `TemporalPolicy` n'est pas créée ; **aucune instance** de `Progressor` ne peut exister. |
| **Fonctionnement** | `TimeError::{ClockOutOfRange, ScheduleOverflow}` | état **terminal** ; causes distinctes. |
| **Fonctionnement** | erreur d'infrastructure `E` (invocation ou publication) | état terminal ; valeur **inchangée** (S02 pt 7). |

L'enveloppe `ProgressError<E> { Infrastructure(E), Time(TimeError) }` ne fait que **transporter** : aucun `Clone`, aucune transformation, aucune exigence de trait sur `E`, aucune hiérarchie qui absorberait ou reformaterait les erreurs existantes. Ni saturation silencieuse, ni panique.

## 5. Le `Progressor` et la progression [Arbitré D-3 ; Proposition]

### 5.1 Objet persistant

Objet concret **public, générique, à champs privés**, nommé `Progressor` (nom de proposition accepté ; **aucun type public « Shell »** : S02 pt 10 ne l'exige pas). Esquisse non normative :

```rust
pub struct Progressor<Ev: Copy, E, C, S, I> { /* champs privés */ }

impl<Ev: Copy, E, C, S, I> Progressor<Ev, E, C, S, I>
where
    C: FnMut() -> Duration,            // horloge (depuis une origine fixe)
    S: FnMut() -> bool,                // demande d'arrêt
    I: FnMut(&[Ev]) -> Result<(), E>,  // publication + invocation
{
    pub fn new(policy: TemporalPolicy, clock: C, stop: S, inbox: Inbox<Ev>, infrastructure: I) -> Self;
    pub fn progress(&mut self) -> Progress;               // jamais un Result
    pub fn take_error(&mut self) -> Option<ProgressError<E>>;
}
```

**État conservé** : politique (q, K, M) ; échéancier (`Option<u64>`, § 6) ; source de temps ; arrêt ; `Inbox` ; infrastructure ; phase ; cause d'erreur (`Option<ProgressError<E>>`).

**API publique minimale** : construction, progression, compte rendu, récupération de l'erreur. Types associés : `TemporalPolicy`, `PolicyError`, `Progress`, `Phase`, `Wake`, `Deadline`, `ProgressError`, `TimeError`, `MonotonicClock`.

**Dispatch statique.** Aucun trait public ajouté (les bornes sont des traits `FnMut` de la bibliothèque standard), aucun `dyn`, aucun `Box`, aucune allocation de masquage. Conséquence assumée : les types de fermeture ne sont pas nommables ; un adaptateur qui stocke un `Progressor` est lui-même générique (ou utilise des `fn` / types nommés).

### 5.2 Compte rendu et état terminal [Arbitré D-2]

```rust
#[derive(Clone, Copy)]
pub struct Progress { pub iterations: u32, pub wake: Wake, pub phase: Phase }
pub enum Phase { Running, Stopped, Failed }
pub enum Wake { At(Deadline), NoDeadline }
```

- `progress()` **ne rend jamais** un `Result` : le nombre d'itérations **effectivement exécutées par cette progression** reste accessible même lorsqu'une erreur survient (par exemple trois itérations achevées, puis `ScheduleOverflow`).
- `Phase::Failed` ne porte pas la cause. Elle se récupère **séparément**, **une seule fois**, par `take_error()` (déplacement, aucun `Clone`). Un second appel rend `None` ; la phase reste `Failed`.
- **État terminal stable** : toute progression ultérieure rend `{0 itération, NoDeadline, phase inchangée}` sans lire l'horloge, sans coupure, sans relancer de travail ; `take_error` n'y change rien.
- Hors terminal, `wake` est toujours `At(..)` ; en terminal, `NoDeadline`.
- Pilote bloquant à politique : sur `Failed`, il appelle `take_error()` et rend l'erreur ; sur `Stopped`, il rend le total d'itérations.

### 5.3 Possession et durées d'emprunt, justifiées par `inbox.rs` (F-13)

- **`Inbox` possédée par valeur.** `cut(&mut self)` rend un `&[Ev]` emprunté à `self.batch` : l'emprunt vit le temps de l'invocation, entre deux champs disjoints (`inbox`, `infrastructure`) du même objet, sans conflit d'emprunt dans `iterate`.
- **`DepositHandle` obtenu avant** le transfert (`let h = inbox.handle();`), puis cloné vers l'adaptateur et les autres fils : c'est un `Arc` **sans durée de vie liée** à l'`Inbox` ni au `Progressor`. Aucun accesseur n'est requis sur le `Progressor`.
- **Pourquoi pas `&'a mut Inbox`** : un adaptateur possédant l'`Inbox` et le `Progressor` serait auto-référentiel, ce que S02 pt 10 exclut. La possession par valeur n'ajoute aucun paramètre de durée de vie.
- **Allocation** : celles de l'`Inbox` restent à sa construction ; `progress()` et `take_error()` n'allouent pas (compte rendu `Copy`).
- **Fermeture d'infrastructure** : peut emprunter `World`, `Scheduler`, tampon (comme `console.rs`), le `Progressor` étant alors lié à ces emprunts par inférence ; peut aussi les posséder (`move`), cas de l'adaptateur de plateforme.
- **Non-réentrance** (S02 pt 4) : garantie par possession (`progress(&mut self)`).

### 5.4 Algorithme de progression

```text
progress():
  si terminal                       → {0, NoDeadline, phase}              // aucune lecture d'horloge
  t_in ← read()                     // erreur → terminal Failed(Time), 0 itération
  next ← init_paresseuse(t_in)      // § 6 : si jamais initialisé, next = t_in
  e ← due_capped(next, q, t_in, K)
  si e = 0                          → {0, At(next), Running}              // aucune coupure, aucune 2e lecture
  done ← 0
  répéter e fois :
      iterate() :                   // cœur unique d'ADR-S02
        Stopped      → {done, NoDeadline, Stopped}      // aucune décision
        Failed(err)  → conserver err ; {done, NoDeadline, Failed}
        Completed    → done += 1 ; next ← checked_add(next, q)   (échec → terminal Failed(Time))
  t_fin ← read()                    // erreur → terminal Failed(Time) ; done reste rapporté
  si t_fin ≥ next et (t_fin − next)/q ≥ M :   // R > M
        next ← checked_add(t_fin, q)            // R1
  retourner {done, At(next), Running}
```

`iterate()` = observation de l'arrêt → `inbox.cut()` → publication+invocation → comptage : **seul** endroit où cette séquence est écrite (S02 pt 3, 13) ; `run_loop` et `progress` l'appellent.

### 5.5 Transitions d'état par chemin

| Chemin | Lectures | Itérations comptées | `next` final | Phase | Réveil | Décision M |
|---|---|---|---|---|---|---|
| A. Rien dû (échéancier déjà initialisé) | 1 | 0 | inchangé (> t_entrée) | Running | `next` | non |
| B. ≥ 1 itération, R = 0 | 2 | e | `next` > t_fin | Running | `next` | R ≤ M |
| C. ≥ 1 itération, 0 < R ≤ M | 2 | e | `next` ≤ t_fin (déjà dû) | Running | `next` | conservé |
| D. ≥ 1 itération, R > M | 2 | e | t_fin + q | Running | t_fin + q | abandon |
| E. Arrêt observé | 1 | achevées | non avancé après | Stopped | NoDeadline | **aucune** |
| F. Erreur infra / échec de publication | 1 | achevées (itération en échec exclue) | non avancé | Failed | NoDeadline | **aucune** |
| G. Erreur temporelle | 1 ou 2 | achevées (y compris avant l'erreur) | indéterminé (terminal) | Failed | NoDeadline | aucune |
| H. Progression après terminal | 0 | 0 | — | inchangée | NoDeadline | non |

## 6. Initialisation de l'échéancier [Arbitré : I-3]

S04 pt 3.2 laisse la mécanique à l'implémentation ; elle ne se déduit ni du réancrage ni de `FixedStepPacing` (F-2 : comportement provisoire, **aucune obligation n'en est déduite**).

**I-3 validée** : initialisation **paresseuse** à t_entrée de la **première progression** ; première échéance = t_entrée, **immédiatement due**.

- **La toute première progression a donc toujours une échéance due** (\|D\| ≥ 1, e ≥ 1 puisque K ≥ 1). Le cas « D vide » n'apparaît qu'à partir d'un échéancier **déjà initialisé** (chemin A du § 5.5).
- Aucune rafale liée au délai de construction ou au chargement : l'échéancier est établi à t_entrée, non à la construction ; la première progression exécute **une** itération, même lancée très tard.
- Aucune lecture d'horloge supplémentaire : l'initialisation réutilise t_entrée.
- **Strictement distincte de R1** : l'initialisation fixe la première échéance à t_entrée (une seule fois) ; R1 n'intervient qu'après un abandon et fixe t_fin + q. Deux chemins de code distincts.
- Alternatives écartées : à la construction (rafale possible, lecture d'horloge dans le constructeur) ; première échéance à t + q (latence du premier tick, sans exigence correspondante).

## 7. Provenance et cohérence de q (Forge → paquet → Shell)

**[Contrat]** (S04 pt 2) : q unique par configuration de jeu, connu de la Forge ; le runtime obtient q d'une donnée de build ou vérifie sa configuration contre celle des artefacts, avant exécution ; aucune exécution silencieuse d'artefacts compilés pour un autre quantum ; aucun q par script, système ou artefact ; aucune modification implicite de `VMAN`.

### 7.1 Décisions arbitrées [Arbitré]

| Réf. | Décision |
|---|---|
| **D-1** | O-1 : q est une métadonnée globale de la configuration du jeu, portée par un champ global de l'en-tête de `VMAN`. O-2 (artefact « temps »), O-3 (constante de build), O-4 (configuration externe répétée) et la copie de q dans chaque artefact sont écartées. |
| **D-1b** | **V-a** : le lecteur supporte `VMAN` v1 et v2 ; les nouvelles compilations destinées à une exécution conforme à S04 produisent du v2 avec q ; la compilation historique v1 reste disponible et produit **les mêmes octets** pour un ensemble donné ; fixtures et contrats protégés par R-ADAPT préservés. *La compatibilité de lecture n'est pas une compatibilité d'exécution* : un manifeste v1, dépourvu de q, **n'est jamais accepté silencieusement** pour une exécution conforme à S04 ; aucun q par défaut, aucun repli sur une constante de l'application. |
| **D-1c** | **V2** : chantier dédié (plan → exécution → handoff, même processus de validation que les étapes 3, 5, 6), pleinement soumis à G-CM, avec périmètre documentaire explicite, critères d'acceptation, vérifications G-CM, et **validation avant toute modification de `VMAN` ou de la Forge**. Ce n'est pas une dérogation à G-CM et ce n'est pas une réouverture de l'étape 5. |
| **D-1d** | `q_ns: u64` (nanosecondes) dans l'en-tête v2, **immédiatement après la version et avant le nombre d'entrées** : `[signature][version][q_ns][entry_count][entries…]` ; longueur d'en-tête 4 + 2 + 8 + 4 = **18 octets**, longueur totale 18 + 16 n ; ordre des octets **explicitement défini** au plan V2 (aligné sur celui du v1) ; **jamais** de q dans chaque entrée ni dans chaque artefact. |
| **R-1** | **M-1** : paquet unique manifeste + artefacts (§ 7.2). M-2 (q dans les `VSCR`) rejeté : contredit « pas de quantum par artefact ». M-3 (identifiant de build, empreinte) non retenu sans nécessité démontrée. R-1 se ferme par le contrat de paquet unique, **sous réserve que V2 en garantisse l'intégrité structurelle**. |

### 7.2 Contrat du paquet unique (M-1) [Arbitré]

Constat à l'origine de M-1 [Fait] : `CompiledSet` expose séparément `manifest()` et `artifacts()` (F-9) ; `load` reçoit deux entrées indépendantes (F-7) ; ni le manifeste (F-8) ni les artefacts ne portent de provenance : un manifeste de la compilation A et des artefacts de la compilation B de même structure franchissent les quatre niveaux. La cohérence n'est donc pas structurellement garantie **aujourd'hui**.

Garantie visée sur le **chemin normal** :

1. la voie de compilation v2 produit **un paquet unique** ;
2. le manifeste porte q **une seule fois** ;
3. le chargement v2 reçoit **une seule entrée**, représentant le paquet complet, dont il extrait le manifeste et les artefacts ;
4. il n'existe **aucun chemin de chargement v2 normal** permettant de fournir séparément un manifeste et des artefacts ;
5. le chargement historique à deux entrées reste **réservé à v1** et **refuse un manifeste v2** (erreur typée, à fixer au plan V2) ;
6. le Shell obtient q **du paquet chargé**, avant toute exécution.

**Portée** : la garantie concerne la provenance du paquet produit par la chaîne normale. Elle n'authentifie pas cryptographiquement des octets recomposés hors de cette chaîne.

Le **format du conteneur**, l'ordre d'octets et le nom du paquet relèvent du plan V2 ; ils ne sont pas décidés ici.

**Source unique de q** : une nouvelle voie de compilation **additive** reçoit q explicitement ; `compile_assets` et son doctest restent inchangés (voie v1). q entre **une fois** (paramètre de la Forge) et se retrouve dans l'en-tête v2 ; l'application ne reçoit jamais q ailleurs que du paquet chargé. Condition de faisabilité à vérifier au plan V2 : la production v1 d'un ensemble donné reste **octet pour octet identique** (G-CM, « octets produits », VQ-5).

**Limite structurelle [Fait F-10]** : le Shell ne peut pas vérifier d'où vient le q qu'on lui remet. La règle « jamais silencieux » repose sur le **chemin d'assemblage** (q obtenu uniquement du paquet, erreur typée sinon) et sur les tests VQ-8 et VQ-9, non sur un verrou de type.

### 7.3 Chaîne documentaire V2 et périmètre [Proposition]

**Chaîne** : (1) plan V2 validé ; (2) document d'exécution validé ; (3) handoff validé ; (4) implémentation par unités sous porte G-CM ; (5) clôture avec rapport. F1 à F3 ne sont débloquées qu'à l'issue de (5) (F3 dès que l'API de lecture de q du paquet est livrée).

**Fichiers et contrats touchés** (recensement partiel, à compléter au plan V2) :

| Zone | Fichiers | Contrat touché |
|---|---|---|
| Layout et lecteur | `assets/src/manifest.rs` (constantes, layout v2, `Manifest::parse`, accesseur de q, `ManifestError`) ; `assets/src/lib.rs` (exports) | layout `VMAN`, constantes `MANIFEST_*`, niveau 1 |
| Chargement | `assets/src/load.rs` (refus d'un manifeste v2 par le chemin à deux entrées ; nouveau chemin à paquet unique) | niveaux 1 à 4, tout-ou-rien |
| Forge | `forge/src/batch.rs` (voie additive q, production du paquet), `forge/src/lib.rs` (`ForgeError`) | signature épinglée de `compile_assets` (F-9), déterminisme, octets v1 identiques |
| Consommateurs | `shell/src/lib.rs` (doctests LT-5n/LT-5t), `examples/level_headless.rs`, tests qui lisent des manifestes | littéraux v1 (F-16) **non modifiés** sous V-a |
| Documents | plan étape 5 § 1.3, exécution § 3.2 et § 6 | définition de G-CM, R-ADAPT : **non modifiés** ; le plan V2 énonce ses propres critères |

**Conditions de validation et tests** (à intégrer au plan V2) :

| ID | Test |
|---|---|
| VQ-1 | Manifeste v2 **littéral écrit à la main** (indépendant des constantes) : `parse` rend le q attendu. |
| VQ-2 | q = 0 refusé au niveau 1 (octets), par la Forge et par `TemporalPolicy` : trois frontières (obligation « q > 0 » d'ADR-S04). |
| VQ-3 | Longueur exacte du layout v2 (18 + 16 n) : tronqué dans l'en-tête, octets en trop, `n` incohérent ; ordre d'octets explicite. |
| VQ-4 | Un manifeste v1 ne produit **jamais** un q ; le chemin S04 le refuse avec une erreur typée avant toute progression. |
| VQ-5 | Aller-retour Forge → paquet → `parse` : q égal ; v1 : mêmes octets que `compile_assets` actuel pour un ensemble donné ; invariance à l'ordre ; q différent → manifeste différent, artefacts identiques octet pour octet. |
| VQ-6 | Régression : ordre et nombre des niveaux, tout-ou-rien (aucun dépôt sur échec), absence d'allocation (`no_std`), `cargo tree` inchangé. |
| VQ-7 | Suites de l'étape 2 : sous V-a **aucune assertion ni littéral modifié** (R-ADAPT intact). |
| VQ-8 | Bout en bout : Forge (q) → paquet → chargement → politique : `TemporalPolicy` porte le q du paquet ; v1 → erreur typée **avant toute progression**. |
| **VQ-9** | **Intégrité structurelle du couple** : (a) le chargement v2 ne reçoit qu'une seule entrée ; il n'existe pas de chemin public v2 à deux entrées ; (b) le chargement à deux entrées **refuse** un manifeste v2 (erreur typée) ; (c) un paquet dont le manifeste est de la compilation A et dont les artefacts sont de B **ne peut pas être produit** par la chaîne normale (le constructeur de paquet est le seul producteur) ; (d) un paquet v1 sans q n'est pas chargeable comme jeu conforme S04. |

### 7.4 Détail laissé à V2 ; dépendances [Dépendance]

- Format exact du paquet, ordre d'octets, variante d'erreur de refus (point 5 du § 7.2), nom de la voie de compilation, recensement complet des tests assertant des octets de manifeste.
- Règle des durées non entières en ticks (F2) : Forge, après V2 ; hors de cette spécification (S04 pt 2).

## 8. Valeurs q, K, M : exemples et valeurs de test seulement

**Aucune composition de jeu n'est sélectionnée.** Aucune valeur n'est retenue pour `games/demo`.

| Grandeur | Statut | Ce qui la déterminerait |
|---|---|---|
| **q** | **ouvert** | besoins de simulation établis (résolution des durées de scripts, cadence des systèmes) : aucun n'est établi à ce jour |
| **K** | à calibrer | débit d'exécution durable (E14, non exécutée) ; sans effet sur la validité de l'algorithme |
| **M** | à calibrer | gigue de réveil et débit (E10, E14, non exécutées) ; idem |

E10 et E14 **n'ont pas été exécutées** et ne sont **pas** des prérequis à la validation mathématique.

**Quatre grandeurs à ne pas confondre** : (1) **quantum logique q** — durée de jeu d'un tick, espacement nominal des échéances (configuration de jeu / Forge) ; (2) **cadence nominale** 1/q — conséquence de q, **pas** une garantie de ticks exécutés (S04 pt 2) ; (3) **cadence de présentation** — frames transmises au Backend (R03 / S05) ; (4) **fréquence de réveil plateforme** — rappels de l'adaptateur (E10). La cadence d'attente des exemples historiques (`FixedStepPacing` de 1 ms, F-5) **ne définit pas** le quantum logique ; aucune valeur n'en est déduite.

**Valeurs de test** : q = 10 ns, K = 3, M = 5 ; extrêmes : q = 1 et `u64::MAX`, K = M = 1 et `u32::MAX`. Elles permettent de démarrer les travaux T1 à T7 sans aucune composition de jeu.

**Illustration non sélectionnée** (ordre de grandeur seulement, **non proposée comme configuration**) : q = 16 666 667 ns (≈ 60 Hz), K = 4, M = 8 donneraient un rattrapage d'au plus 4 itérations par progression et un résiduel conservé jusqu'à 8 échéances. Ces chiffres n'ont aucun statut décisionnel.

`SCRIPT_MAX_STEPS_PER_TICK` borne les pas d'**un script par tick** ; K borne les **itérations par progression** : indépendants.

## 9. Pilotes

### 9.1 Note d'application de S02 pour `run_loop` [Arbitré, sans modification de S02]

Textes de S02 sur lesquels elle s'appuie :
- **Point 1** : le pilotage est assuré « soit par `run_loop` (pilote bloquant), soit par un pilote événementiel ».
- **Point 3** : le pilote bloquant « attend selon la politique de cadencement, puis applique le cœur ; conservé pour le headless, les tests et toute application qui choisit une boucle bloquante. Son contrat observable actuel (suites AQ, LC, PC, RA) est préservé. »
- **Point 3** : « Que le pilote bloquant soit réécrit au-dessus du cœur ou aux côtés de celui-ci est une décision d'implémentation, tant que la sémantique n'existe qu'une fois. »
- **Point 4** : N est « fixé par la politique temporelle (ADR-S04) ». **Liste « laissé à l'implémentation »** : « réécriture ou non de `run_loop` au-dessus du cœur ».
- **S04 pt 12** : la politique est la même pour les deux pilotes ; la forme de l'adaptation du pilote bloquant est laissée à l'implémentation. S04 ne nomme pas `run_loop`.

**Constat (fait)** : `run_loop` ne peut pas appliquer S04 sans changer de signature (F-1 : `cadence` ne rend aucun instant ; q, K, M et une source de temps substituable y manquent), donc sans cesser de préserver son contrat observable (S02 pt 3). Les deux textes ne sont simultanément satisfaits que par l'organisation suivante (**option B**, retenue) :

| Rôle | Élément | Politique temporelle S04 |
|---|---|---|
| **Compatibilité historique** | `run_loop` : signature et contrat observable **strictement conservés** ; utilisable comme **boucle bloquante à cadence externe, hors politique temporelle S04**, pour les usages historiques, les tests et les applications qui conservent ce comportement ; il n'implémente pas K/M | non appliquée (la cadence est fournie par l'appelant) |
| **Pilotes conformes S02/S04** | un **nouveau pilote bloquant à politique temporelle** ; le **pilote événementiel** | appliquée, par le **même** `Progressor`, le **même** `progress()` et le **même** cœur `iterate()` |

- La sémantique d'itération n'existe **qu'une fois** (`iterate()`), appelée par `run_loop` et par `progress()` ; l'échéancier et la logique K/M n'existent **qu'une fois** (dans `progress()`) ; `run_loop` n'en contient aucune copie.
- `run_loop` n'est **pas qualifié de « non conforme »** : il est une boucle bloquante à cadence externe, hors politique temporelle S04. Cette qualification est portée par la documentation de `run_loop` (unité T4).
- **Exemples** : `console`, `headless_frame`, `script_headless`, `level_headless` (`run_loop` + `FixedStepPacing` ou cadence vide) sont des **exemples historiques à cadence externe** ; ils **ne démontrent pas** la conformité S04 et ne sont pas modifiés. Les **nouveaux exemples conformes S04** (T8) utilisent le pilote bloquant à politique avec horloge et attenteur scriptés.
- **Équivalence (S02 pt 3)** : pour une même séquence d'observations, le pilote bloquant à politique et le pilote événementiel produisent mêmes lots, mêmes invocations, même issue (TP-D1) ; les scénarios existants rejoués via `Progressor` (une échéance due par progression) donnent mêmes lots, arrêts et première erreur que `run_loop` (TP-D8).

### 9.2 Pilote bloquant à politique [Proposition]

Fonction distincte de `run_loop` : boucle `progress()` puis attend jusqu'à `wake` par une fermeture **injectée** (`wait_until`) ; rend le total d'itérations sur `Stopped`, l'erreur (`take_error`) sur `Failed`. Par défaut la fermeture dort `deadline − now` ; une différence négative signifie « déjà dû, ne pas dormir ». Les tests injectent un attenteur scripté : aucun `sleep`.

### 9.3 Pilote événementiel [Contrat S02 pt 8, 9 ; Proposition de mise en œuvre]

- Le `Progressor` est **possédé par l'adaptateur**, sur le fil de la boucle de plateforme : pas de fil dédié, pas de `pump_app_events`.
- Point d'accroche visé avec `winit` : `about_to_wait` : `progress()`, puis `ControlFlow::WaitUntil(origine + deadline)` si `Running`, sortie de la boucle sinon. À confirmer par la micro-PoC ; ne décide pas du rendu.
- Dépôts par `DepositHandle` ; un dépôt ne rend pas une itération due (S02 pt 8).
- `MonotonicClock { origine: Instant }` (`Copy`) fournit la fermeture d'horloge ; l'adaptateur en garde une copie pour convertir `Deadline → Instant`. Le `Progressor` ne manipule jamais d'`Instant`.
- Suspension, reprise, non-présentabilité : aucun effet propre sur la politique (S04 pt 11) ; [Dépendance] cycle de vie de la plateforme. Réveil sur échéance passée : comportement par plateforme **non vérifié** (E1).

### 9.4 Arrêt lorsque D est vide [Arbitré : comportement conservé]

Si aucune échéance n'est due : **aucune coupure, aucune itération, aucune observation de l'arrêt** (l'observation de l'arrêt fait partie de l'itération, S02 pt 2 et 4).

**Conséquence documentée** : une demande d'arrêt n'est observée par le `Progressor` qu'à la **prochaine occasion d'itération** (au plus un quantum plus tard pour le pilote bloquant). La **fermeture de fenêtre** peut être traitée **directement par l'adaptateur** de plateforme (S02 pt 7), sans attendre le Shell. **Aucun canal de réveil supplémentaire** n'est introduit.

## 10. Matrice de tests [Proposition]

**Harnais** : horloge = fermeture qui rend la suite scriptée de `Duration` et compte ses lectures ; **la suite doit être non décroissante, le harnais refuse (panique à la construction) toute suite décroissante** ; attenteur scripté ; aucun `sleep`, aucun `Instant`, aucune horloge murale. Base : q = 10, K = 3, M = 5 (valeurs de test).
**Préfixe Σ0** : première progression, lectures [100, 100] → 1 itération, réveil 110 ; `next` = 110 ensuite. **Chaque ligne des tableaux TP-P et TP-S est un scénario indépendant** : elle repart d'un `Progressor` neuf suivi de Σ0 (`next` = 110), sauf mention contraire. Valeurs issues du modèle de référence.

### TP-A — arithmétique pure

| ID | Scénario | Attendu |
|---|---|---|
| A1 | `e` pour next = 100, q = 10, K grand : t = 99, 100, 109, 110, 119, 120 | 0, 1, 1, 2, 2, 3 |
| A2 | K = 3 : t = 119 / 120 / 145 | 2 / 3 / 3 |
| A3 | q = 1, next = 0, t = `u64::MAX`, K = 7 | `e = 7`, sans panique ni erreur |
| A4 | next = 130, q = 10, M = 5 : t_fin = 129 / 179 / 180 | R = 0 / R = 5 conservé / R = 6 abandon |
| A5 | `next + q` et `t_fin + q` au-delà de `u64::MAX` | `ScheduleOverflow` |
| A6 | `Duration` > `u64::MAX` ns | `ClockOutOfRange` |
| A7 | q = 0, K = 0, M = 0, q hors plage rejetés ; K = M = `u32::MAX` acceptés ; (K = 1, M = 1000) et (K = 1000, M = 1) acceptés | erreurs de construction typées / acceptation |
| A8 | Énumération déterministe (q 1..5, next 0..19, t 0..59, K 1..5, M 1..7) contre la définition naïve | égalité sur tous les cas |

### TP-P — progression

| ID | Lectures | Itérations | Réveil | Observation |
|---|---|---|---|---|
| **P0** première progression | [100, 100] (Σ0 lui-même) | **1** | 110 | échéance **toujours due** à l'initialisation ; jamais « D vide » |
| **P0b** absence de rafale | [1 000 000 000, 1 000 000 000] | **1** | 1 000 000 010 | initialisation à t_entrée : une seule itération malgré le retard depuis l'origine |
| P1 avant l'échéance | [105] | 0 | 110 | 1 lecture ; **aucune coupure** (un dépôt préalable reste dans l'`Inbox`) |
| P2 échéance exacte | [110, 110] | 1 | 120 | 2 lectures |
| P3 retard < q | [115, 115] | 1 | 120 | |
| P4 plusieurs dues | [130, 130] | 3 | 140 | \|D\| = 3 = K |
| P5 plafond K | [155, 155] | 3 | 140 | \|D\| = 5 > K ; R = 2 ≤ M conservé ; réveil déjà passé |
| **P6** nées pendant l'exécution | [110, 135] | **1** | 120 | depuis `next` = 110, **trois échéances sont atteintes à t_fin = 135** (110, 120, 130), **dont une exécutée** (110) ; le **résiduel est de deux échéances** (120, 130) ≤ M, conservé ; l'ensemble exécuté reste fixé à l'entrée |
| P7 rattrapage chaîné | P5 puis [160, 160] (même instance) | 3 | 170 | le reliquat conservé est exécuté |
| P8 absence de dérive | 100 000 progressions à t = `next` | 1 chacune | — | `next` final = 110 + 10 × 100 000 |

### TP-S — seuil et réancrage

| ID | Lectures | Itérations | Réveil | Observation |
|---|---|---|---|---|
| S1 R = M conservé | [110, 169] | 1 | 120 | R = 5 |
| S2 R = M + 1 abandonné | [110, 170] | 1 | **180** | R = 6 ; réveil = t_fin + q, non immédiatement dû |
| S3 aucune échéance abandonnée n'est exécutée | S2 puis [179], puis [180, 180] (même instance) | 0 puis 1 | 180 ; 190 | |
| S4 interruption longue | [10110, 10110] | **3** | 10120 | K itérations tardives exécutées **avant** l'abandon (R = 998 > M) |
| S5 phase non conservée | [10113, 10113] | 3 | 10123 | 10123 mod 10 ≠ 0 alors que la grille initiale est ≡ 0 |
| **S6a** M = 1, R = 1 conservé | [110, 120] avec M = 1 | 1 | 120 | scénario **indépendant**, depuis `next` = 110 |
| **S6b** M = 1, R = 2 abandonné | [110, 130] avec M = 1 | 1 | 140 | scénario **indépendant**, **non** enchaîné à S6a, depuis `next` = 110 |
| S7 reliquat + nées | [110, 150] | 1 | 120 | depuis `next` = 110 : une exécutée (110), R = 4 (120, 130, 140, 150) ≤ M conservé ; réveil = prochaine échéance non exécutée (120), déjà due |

### TP-E — arrêt, erreur, acquisition, erreur terminale

| ID | Scénario | Attendu |
|---|---|---|
| E1 | Arrêt observé avant la première itération | 0 itération, `Stopped`, 1 lecture, `Inbox` intacte, aucune décision M |
| E2 | Arrêt observé après j itérations | j comptées, `Stopped`, **1 seule lecture**, aucune décision |
| E3 | Erreur d'infrastructure à la j-ième itération | `Failed` ; `take_error()` rend la valeur **inchangée** (identité, sans `Clone`) ; j − 1 comptées ; aucune invocation ensuite ; 1 lecture |
| E4 | Échec de publication avant le tick (cf. AC-7) | itération non lancée, non comptée ; même issue qu'E3 |
| E5 | Dépôt pendant une itération d'une progression de 3 | dans le lot de l'itération suivante ; un lot par itération, lots vides comptés, aucun lot rejoué (cf. AQ-1, AQ-7b) |
| E6 | Dépôt, progression « rien dû », puis progression avec itération | dépôt dans le lot de la première itération effective |
| E7 | Débordement de `next + q` après la j-ième itération (j ≥ 2) | `Failed` ; **`iterations = j` accessible** ; cause `Time(ScheduleOverflow)` via `take_error()` |
| E8 | Progression après terminal | 0 itération ; phase inchangée ; **0 lecture** ; aucune coupure ; `NoDeadline` |
| E9 | `ClockOutOfRange` à t_entrée / à t_fin (séquences monotones) | `Failed` ; causes **distinctes** de `ScheduleOverflow` ; compteur cohérent |
| E10 | `take_error()` appelé deux fois | `Some` puis `None` ; phase `Failed` stable ; aucune progression relancée |
| E11 | Erreur de construction (q = 0, K = 0, M = 0) | **aucun `Progressor` constructible** |

### TP-D — pilotes et non-régression

| ID | Scénario | Attendu |
|---|---|---|
| D1 | Même script d'observations via pilote bloquant à politique et via harnais événementiel | mêmes lots, mêmes invocations, même issue ; **même `progress()`** appelé |
| D2 | L'attenteur reçoit `wake` ; réveil déjà passé | attente nulle, aucune boucle active supplémentaire |
| D3 | Harnais événementiel : `Deadline → Instant` par `checked_add` | conforme |
| D4 | Suites F-4 (LC, PC, AQ, AC, RA) **non modifiées**, signature de `run_loop` inchangée | toutes passent |
| D5 | Revue structurelle : coupure, publication, invocation dans **une seule** fonction ; aucune copie d'échéancier ni de logique K/M dans `run_loop` | une occurrence |
| D6 | Dépôt depuis un autre fil entre deux progressions (jointure, sans `sleep`) | dans le lot de l'itération suivante |
| D7 | `[dependencies]` du Shell vide ; aucun trait public ajouté, aucun `dyn`, aucun `Box` ; aucun type de fenêtrage | inchangé |
| D8 | Scénarios existants rejoués via le `Progressor` (une échéance due par progression) | mêmes lots, arrêts, première erreur que `run_loop` |
| D9 | Arrêt demandé, D vide | non observé ; observé à la progression suivante qui exécute une itération (§ 9.4) |

## 11. Unités de travail

Les unités T1 à T7 peuvent démarrer avec les **valeurs temporelles de test** (§ 8), sans aucune composition de jeu ni V2. **Aucune n'est autorisée tant que cette révision n'est pas validée.** La séquence et les portes relèvent de la roadmap (session ultérieure).

**T1 — Politique** · *Objectif* : `TemporalPolicy` (q, K, M), `PolicyError`. · *Prérequis* : aucun. · *Fichiers* : `crates/engine/shell/src/policy.rs` (créé), `src/lib.rs` (exports). · *Invariant* : C-1, C-2. · *Tests* : TP-A7, TP-E11. · *Acceptation* : constructions invalides rejetées ; aucune dépendance ajoutée.

**T2 — Arithmétique d'échéancier** · *Objectif* : fonctions pures `due_capped`, `exceeds`, avancement, réancrage. · *Prérequis* : T1. · *Fichiers* : `src/schedule.rs` (créé). · *Invariant* : § 4.3 ; temps constant, `checked`, sans flottant ni allocation. · *Tests* : TP-A1 à A5, A8. · *Acceptation* : égalité avec la définition naïve ; extrêmes sans panique.

**T3 — Horloge** · *Objectif* : conversion `Duration → u64` ns, `TimeError`, `MonotonicClock`. · *Prérequis* : T2. · *Fichiers* : `src/clock.rs` (créé). · *Invariant* : C-14 ; précondition de monotonie. · *Tests* : TP-A6. · *Acceptation* : aucune lecture d'horloge hors de ce module et du pilote par défaut.

**T4 — Cœur d'itération extrait** · *Objectif* : `iterate()` ; `run_loop` réécrit au-dessus, **signature et comportement inchangés** ; documentation de `run_loop` : « boucle bloquante à cadence externe, hors politique temporelle S04 » (§ 9.1). · *Prérequis* : aucun. · *Fichiers* : `src/shell.rs`. · *Invariant* : S02 pt 2, 3, 13. · *Tests* : TP-D4, D5. · *Acceptation* : suites F-4 et exemples F-5 inchangés et verts.

**T5 — `Progressor`** · *Objectif* : objet persistant, `progress()`, compte rendu, phase, `take_error`, initialisation I-3. · *Prérequis* : T1 à T4. · *Fichiers* : `src/progressor.rs` (créé), `src/lib.rs`, `tests/temporal_progress.rs` (créé). · *Invariant* : C-3 à C-14 ; aucune allocation sur le chemin d'itération ; dispatch statique. · *Tests* : TP-P, TP-S, TP-E, TP-D7, D8, D9. · *Acceptation* : toutes les lignes attendues du § 10 ; compteur d'itérations conservé sur erreur.

**T6 — Pilote bloquant à politique** · *Objectif* : fonction à attenteur injectable utilisant `Progressor::progress`. · *Prérequis* : T5. · *Fichiers* : `src/driver.rs`, `tests/temporal_drivers.rs`. · *Invariant* : S02 pt 3 ; S04 pt 12. · *Tests* : TP-D1, D2. · *Acceptation* : aucun `sleep` dans les tests ; aucune copie de la logique K/M.

**T7 — Harnais du pilote événementiel** · *Objectif* : simuler `about_to_wait` sans `winit`, prouver l'équivalence. · *Prérequis* : T5, T6. · *Fichiers* : `tests/temporal_drivers.rs`. · *Tests* : TP-D1, D3, D6. · *Acceptation* : équivalence établie ; aucun type de plateforme.

**T8 (optionnelle) — Exemple conforme S04** · *Fichiers* : `examples/policy_headless.rs`. · *Prérequis* : T5, T6. · *Acceptation* : code de sortie 0 ; horloge et attenteur scriptés, valeurs de test ; distinct des exemples historiques.

**Bloquées par G-CM (chantier V2)**
- **F1 — Évolution de `VMAN` et paquet unique** : chaîne § 7.3 jusqu'à la clôture ; tests VQ-1 à VQ-9.
- **F2 — Durées non entières en ticks** : règle côté Forge ; après F1 ; hors Shell.
- **F3 — Assemblage applicatif** : q obtenu **du paquet chargé uniquement**, `TemporalPolicy` construite avec q, erreur typée si q absent. *Prérequis* : T1, F1.

## 12. Statut consolidé

### 12.1 Déterminé (arbitré ou contractuel)

| Sujet | Détermination |
|---|---|
| Contrat temporel | C-1 à C-16 (ADR-S04) |
| Provenance de q | D-1 (O-1), D-1b (V-a), D-1c (V2 sous G-CM), D-1d (`q_ns: u64`, 18 octets d'en-tête), R-1 (M-1, paquet unique, VQ-9) |
| Erreurs | D-2 : typées, terminales ; construction distincte ; compteur conservé ; cause récupérée une fois, sans `Clone` ; erreur d'infrastructure inchangée |
| Objet | D-3 : `Progressor` concret public générique à champs privés, dispatch statique, aucun trait public ni allocation de masquage |
| Initialisation | I-3 |
| `run_loop` | option B et note d'application de S02 (§ 9.1) |
| Arrêt quand D est vide | conservé, documenté (§ 9.4) |
| Valeurs | aucune composition sélectionnée (§ 8) |

### 12.2 Laissé à l'implémentation

Noms internes et organisation en modules ; type exact de `Deadline`/`Wake` ; représentation de l'état terminal ; forme du pilote bloquant à politique (§ 9.2) ; avancement de `next` après une erreur survenue après invocation (PO-1, sans effet observable car terminal) ; traitement d'un `Deadline` non représentable en `Instant` (PO-3, avec la micro-PoC).

### 12.3 Bloqué par G-CM

F1 (VMAN v2, paquet, provenance de q), F2 (durées non entières), F3 (assemblage conforme S04), et toute « composition de jeu conforme à S04 », tant que la chaîne de provenance de q n'est pas opérationnelle. Aucun format n'est modifié par cette mission.

### 12.4 Pouvant démarrer avec des valeurs de test (après validation)

T1 à T7 ; T8 optionnelle. Le chantier V2 n'est **pas** une dépendance du cœur temporel.

### 12.5 Points ouverts

- **PO-1** Avancement de l'échéancier après erreur survenue après invocation : S02, S03, S04 muets ; sans effet observable (terminal) ; choix proposé : non avancé.
- **PO-2** Valeur de `SCRIPT_MAX_STEPS_PER_TICK` non observée.
- **PO-3** `Deadline → Instant` non représentable côté pilote : traitement à fixer avec la micro-PoC.
- **PO-4** Sort de `FixedStepPacing` et de `run_loop` à long terme : hors de cette spécification.

(Les identifiants O-1 à O-4 du § 7.1 désignent les options de provenance de q ; les points ouverts portent le préfixe PO.)

### 12.6 Dépendances externes

R03 / S05 (désignation de la dernière itération d'une progression pour 0..1 frame ; cadence de présentation) ; cycle de vie de la plateforme (suspension, reprise, non-présentabilité) ; ADR-AU01 (application des commandes audio) ; ADR-N01 (alignement des ticks) ; micro-PoC `winit` + `wgpu` ; E10, E14, E15 non exécutées.

## 13. Verdict de maturité

Le contrat logique d'ADR-S04 est entièrement traduit ; aucune contradiction avec S01, S02, S03, A01 n'est relevée. Tous les arbitrages nécessaires sont fermés. **Prêtes après validation : T1 à T7** (T8 optionnelle), avec valeurs de test. **Bloquées : F1 à F3**, par G-CM, via le chantier V2.

## 14. Compte rendu de révision 3

1. **Document** : `spec-implementation-shell-temporal-policy.md`, révision 3, consolidée.
2. **Arbitrages intégrés** : D-1, D-1b (V-a), D-1c (V2), D-1d (`q_ns`), R-1 (M-1, VQ-9), D-2, D-3, I-3, option B pour `run_loop`.
3. **`run_loop`** : note d'application de S02 (§ 9.1), « boucle bloquante à cadence externe, hors politique temporelle S04 » ; exemples historiques distingués des exemples conformes S04.
4. **Erreurs et compte rendu** : § 4.4 et § 5.2.
5. **Valeurs** : exemples ou valeurs de test seulement.
6. **Aucun code, test, `Cargo.toml`, ADR ni format d'artefact n'a été modifié.** Aucune implémentation, aucune roadmap.
