# Plan de construction — étape 3 : Shell minimal (`verbum-shell`) — r3

## Statut
Document de planification interne, non normatif, révisable sans ADR (même statut que `ROADMAP.md`). Détaille l'étape 3 en unités. N'introduit aucun invariant : chaque unité cite la source du corpus qui la justifie. **Aucun code, aucune modification du dépôt, aucun commit, aucune commande de validation exécutée pour ce plan.**

**État de validation : plan non validé pour implémentation.** Un seul arbitrage architectural reste suspendu : **A4** (§4). Aucune unité ne démarre avant la résolution d'A4 et l'approbation explicite du plan. Le document d'exécution (équivalent de `02-verbum-forge-execution.md`) n'est rédigé qu'ensuite.

Historique : r1 (plan initial) → r2 (intégration d'ADR-005) → **r3 : intégration des arbitrages clos retransmis par l'utilisateur le 18 octobre 2026 (A1 = O2, A2, A3, A5 i et ii, A6, C7, saturation L4/C12) ; lecture intégrale d'ADR-003 et ADR-006 ; question à soumettre aux sessions d'origine pour A4 (§4.2).** L'analyse `03-verbum-shell-A4-analyse.md` est conservée telle quelle, comme annexe de l'état d'A4 avant les décisions ; la seule de ses lacunes aujourd'hui close est L4.

Légende : **[Lu]** texte vérifié dans un document lu · **[Déduit]** conséquence que je tire d'un texte lu · **[Imposé]** texte normatif cité · **[L]** choix local, non invariant, révisable · **[C]** contradiction, ambiguïté ou lacune signalée · **[Clos]** décision de l'utilisateur · **[A4]** arbitrage suspendu.

## 0. Sources et limites

**[Lu]** `ROADMAP.md`, `ADR-S01`, `ADR-000`, `ADR-003`, `ADR-004`, `ADR-005`, `ADR-006`, `ADR-A01`, `ADR-R03`, `ADR-AU01`, `ADR-N01`, `ADR-SV01`, `ADR-Sc01`, `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, `00-verbum-ecs.md`, `01-verbum-scheduler.md` (r5, en entier), `01-verbum-scheduler-sessions.md`, `HANDOFF-stage-2-forge.md`, `02-verbum-forge.md` (r8) et `02-verbum-forge-execution.md` (modèles de structure et de discipline), `tree.md` du 18 octobre 2026 (fait foi sur les chemins). Code réel : `crates/schedule/src/scheduler.rs` (`Scheduler`, `run_tick`, `TickError`), signatures publiques de `world.rs` et `command.rs`, `examples/headless.rs` (début), `Cargo.toml` racine, `crates/verbum/src/lib.rs`.

**Non lus** : `ADR-001`, `002`, `ADR-R01`, `R02`, `README.md` d'architecture, `HANDOFF-stage-0/1`, brouillons LPC, `docs/reflexivity/`. Leur contenu n'est connu que par les citations de `01-verbum-scheduler.md` et `02-verbum-forge.md`. Aucun n'est identifié comme susceptible de changer ce plan.

**ADR-S02 à S05 n'existent pas** : `tree.md` ne liste que `ADR-S01` ; ADR-S01 les annonce (« Suite prévue ») sans les rédiger.

## 1. Objectif et critères de sortie

### 1.1 Contrat de la Roadmap [Lu]
- Contenu : « boucle `while OS vivant → tick fixe → scheduler.run() → fin`. Console suffit, aucune fenêtre. Le tick fixe n'est ici qu'une politique provisoire de validation — elle ne préjuge pas de la politique temporelle réelle, explicitement réservée à une future ADR-S04 (ADR-S01, point 5). »
- Dépendance : « Étape 1 (`scheduler.run()` doit exister comme point d'appel). »
- Test d'invariant : « Aucune information ne revient vers une itération déjà en cours — même en boucle console, le principe généralisé en ADR-S01 point 6 doit être vérifiable, quitte à le simuler avec une fausse source d'entrée asynchrone. »
- Artefact : « premier runtime complet piloté par le Shell (World + Scheduler + boucle), toujours sans rendu. »
- Principe directeur : un jalon n'est validé que si aucun invariant architectural n'a dû être modifié pour le faire fonctionner.

### 1.2 Périmètre [Déduit], à confirmer à la revue
L'étape 3 construit : (a) le cycle de vie minimal du processus (démarrage, boucle, arrêt propre, libération) ; (b) une politique de cadencement provisoire, isolée ; (c) une boîte de réception provisoire, à sens unique, à capacité fixe, qui rend vérifiable l'invariant d'ADR-S01 point 6 ; (d) un exécutable console qui assemble `World`, `Scheduler` et boucle. Tout le reste est hors périmètre (§10).

### 1.3 Critères de clôture

| # | Exigence démontrée | Source | Preuve |
|---|---|---|---|
| S1 | Le Shell possède démarrage, boucle, arrêt propre, libération | ADR-S01 pt 1 | LC-1 à LC-4 |
| S2 | Le Shell invoque une infrastructure entière comme boîte noire ; il ne nomme ni système, ni phase, ni contrat, et ne dépend pas du noyau | ADR-S01 pt 3 ; [Clos] A1 | `cargo tree -p verbum-shell -e normal` (aucune dépendance) ; revue de l'API publique ; AC-2 |
| S3 | Aucune information ne revient vers une itération en cours | ADR-S01 pt 6 ; Roadmap | AQ-1, AQ-1b, AQ-3 ; AC-1 |
| S4 | Le tick fixe est une politique provisoire, isolée, remplaçable ; le `World` n'observe jamais le temps réel | ADR-S01 pt 5 ; `00-principes.md` inv. 3 ; Roadmap | PC-1 à PC-3 ; AQ-5 |
| S5 | Les dépendances de plateforme n'existent que dans `verbum-shell` ; aucun crate du noyau ne dépend du Shell | ADR-S01 pt 4 ; ADR-000 pts 1 à 3 | `cargo tree -i verbum-shell --workspace` ; `cargo tree` des crates du noyau inchangés |
| S6 | L'assemblage des infrastructures n'existe qu'à un endroit | ADR-S01 pt 7 | Revue de `examples/console.rs` ; aucun test automatisé |
| S7 | Artefact : exécutable console, plusieurs ticks, sans rendu, code retour 0 | Roadmap | AC-3 |
| S8 | Saturation de la boîte de réception : refus explicite, événements présents conservés dans l'ordre, aucune perte silencieuse, aucun écrasement, aucune croissance | [Clos] L4/C12 ; ADR-000 pt 4 | AQ-6, AQ-6b |
| S9 | Erreur de tick : arrêt de la boucle, erreur d'origine propagée, buffer non remis à zéro | [Clos] A5 (i) | LC-4 ; AC-4 |
| S10 | Aucun invariant ni fichier interdit modifié (K8 de l'étape 2, transposé) | Roadmap, principe directeur | Contrôles manuels (§6, U5) |

## 2. Exigences documentées et couverture à l'étape 3

| Exigence | Source | Couverture à l'étape 3 |
|---|---|---|
| Cycle de vie complet du processus | ADR-S01 pt 1 | Testée (U2) |
| Traduction des signaux bruts en données d'Acquisition, sans interprétation | ADR-S01 pt 2 | **Non couverte** : mécanisme réservé à ADR-S03. L'étape 3 ne fournit que la boîte de réception provisoire portant des événements déjà formés. Précédent voisin : ADR-A01 pt 5 (une Ressource chargée « devient une nouvelle donnée d'Acquisition d'une itération future ») |
| Orchestration d'infrastructures entières ; modèle d'exécution jamais influencé de l'extérieur | ADR-S01 pt 3 ; ADR-004 pt 5 | Couture O2 (fermeture opaque) ; AC-2 : `Scheduler::model()` égal avant et après N ticks |
| Dépendances de plateforme confinées au Shell | ADR-S01 pt 4 ; ADR-000 pt 2 | Structure (dépendances de crates) |
| Le Shell possède les sources de temps réelles ; politique temporelle non tranchée | ADR-S01 pt 5 | Politique provisoire isolée (U3) |
| Aucune infrastructure ne modifie une itération en cours | ADR-S01 pt 6 ; Roadmap | Testée (U4) sur le canal du Shell, avec une source scriptée sans thread (limite R5) |
| Le Shell crée, détruit, choisit et relie les infrastructures, en un seul endroit | ADR-S01 pt 7 ; ADR-000 pt 2 | Structure ; revue de l'exemple |
| Le Shell construit le noyau et fournit sa mémoire initiale (capacités fixées avant simulation) | ADR-000 pt 2 | L'exemple instancie `World<…>` aux capacités fixées par const-génériques |
| Données franchissant la frontière : « ni propriété partagée ni référence à un type de la Plateforme » | ADR-000 pt 3 | Événements `Copy` ; revue (« partiellement vérifiable par les types », ADR-000) |
| Capacités fixées avant simulation ; dépassement = bug de configuration, jamais croissance dynamique | ADR-000 pt 4 | Boîte de réception à capacité fixe ; AQ-6 |
| Chargement des artefacts (accès disque) par une infrastructure assemblée par le Shell | ADR-A01 pts 2 et 5 ; ADR-000 pt 2 | **Hors étape 3** : [Clos] A5 (ii), attribution à l'étape 5 (§3, C5) |
| Données d'Extraction vers Renderer et Backend | ADR-S01 pt 5 ; ADR-000 pt 3 | **Non couvertes** : étape 4 |

## 3. Contradictions, ambiguïtés et lacunes : état r3

| # | Nature | Statut r3 |
|---|---|---|
| **C1** | **Écart de vocabulaire et d'interface.** Normatif : `world.execute()` (ADR-S01 pt 3, ADR-000 pt 3) ; Roadmap : `scheduler.run()`. Réel : `Scheduler::run_tick(&self, &mut World, &mut CommandBuffer)` ; `World`, `CommandBuffer` et `Scheduler` sont trois objets distincts, le buffer appartenant à l'appelant (`01-verbum-scheduler.md` §6.2). **`World::execute()` et `Scheduler::run()` n'existent pas.** | **[Clos] pour l'étape 3** : la couture O2 est le point d'entrée opaque. La fermeture n'est pas `world.execute()` et ne prétend pas l'être : l'adaptateur d'application appelle `run_tick`. Aucune modification de `World`, du Scheduler, des ADR ou de la Roadmap. L'écart est consigné ici. |
| **C2** | Point ouvert normatif d'ADR-000 : la forme de l'interface de franchissement « reste à arbitrer avant l'intégration du Shell et du Renderer » ; la Roadmap ne prévoit aucun arbitrage. | **Non tranché, consigné.** La couture O2 est provisoire et ne préjuge pas de cette interface. Seule la moitié « entrée » est exercée ; la moitié « sortie » (Extraction, `renderer.compose()`) relève de l'étape 4 [Déduit]. |
| **C3** | « `while OS vivant` » sans condition d'arrêt définie. | **[Clos] A2** : arrêt demandé par l'appelant, observé entre les ticks ; un tick engagé se termine ; aucune gestion de signal OS. |
| **C4** | Tension apparente : ADR-S01 pt 5 (« n'impose pas silencieusement un modèle à tick fixe ») contre Roadmap (tick fixe provisoire). | Compatibles si la politique est nommée provisoire, isolée, remplaçable (S4, PC-3). Vigilance, pas arbitrage. |
| **C5** | ADR-A01 pt 2 : l'accès disque et le cast des artefacts relèvent d'une infrastructure assemblée par le Shell ; aucune étape de la Roadmap ne le crée (étape 2 : fonction pure `load`). | **[Clos] A5 (ii)** : pas d'accès disque réel à l'étape 3. **Attribution à l'étape 5, consignée dans ce plan seulement ; la Roadmap n'est pas modifiée.** Écart signalé : le texte de la Roadmap pour l'étape 5 ne mentionne pas explicitement cet accès. |
| **C6** | Façade `verbum` `no_std` contre Shell `std`. | **[Clos] A6** : façade inchangée, toujours `no_std`. |
| **C7** | Le corps de boucle du Shell est-il sur le hot path ? Aucun compteur d'allocations sans `unsafe` (interdit). | **[Clos]** : aucune allocation dynamique sur le chemin d'un tick ; capacité fixée avant la simulation ; aucune croissance. Contrôle retenu : **revue du code** ; aucun test automatisé. |
| **C8** | Porteur de la donnée d'Acquisition lue par les systèmes. | **[A4], suspendu** (§4). Non tranché par ADR-003, 005, 006 (annexe d'analyse). |
| **C9** | Politique d'erreur de tick de l'appelant (`01-verbum-scheduler.md` §6.2 : « non spécifiée »). | **[Clos] A5 (i)** : sur erreur, arrêt de la boucle et propagation de l'erreur d'origine, sans reprise automatique ni remise à zéro silencieuse du buffer. |
| **C10** | ADR-S02 (boucle), S03 (acquisition), S04 (temps), S05 (orchestration) absents. | Tout ce que l'étape 3 écrit dans ces domaines est provisoire, nommé comme tel, et ne les préjuge pas. |
| **C11** | « Console suffit » : hôte sans fenêtre, ou sortie console ? | Lecture retenue [Déduit] : hôte sans fenêtre ; aucune sortie requise ; code retour 0. À confirmer à la revue. |
| **C12** | Saturation de la boîte de réception (capacité fixe, ADR-000 pt 4). | **[Clos] L4** : refus du nouvel événement avec erreur explicite ; événements déjà présents conservés dans leur ordre d'arrivée ; aucune perte silencieuse, aucun écrasement, aucune croissance dynamique. |

## 4. Arbitrages

### 4.1 Arbitrages clos (retransmis le 18 octobre 2026)

| # | Décision | Conséquence sur le plan |
|---|---|---|
| **A1** | **O2** : fermeture `FnMut` comme interface opaque vers l'infrastructure. Pas de trait public supplémentaire. Pas de dépendance normale du Shell vers le noyau. | `verbum-shell` n'a aucune dépendance normale ; `verbum-ecs` et `verbum-schedule` y sont des dev-dependencies, ajoutées en U5 seulement. Le Shell ne détient jamais le `World` : **il ne peut donc pas écrire lui-même dans une Ressource** ; tout dépôt dans le `World` est fait par le code d'application à l'intérieur de la fermeture [Déduit de A1]. |
| **A2** | Arrêt demandé par l'appelant, observé entre les ticks ; un tick engagé se termine ; pas de signal OS. | LC-2, LC-3. Mécanisme d'observation : choix local, fermeture `FnMut() -> bool` (§5). |
| **A3** | Source scriptée déterministe, sans thread réel, capable de simuler un dépôt pendant un tick. **Ce test n'est pas une preuve de sûreté concurrente réelle** et n'est jamais présenté comme telle. | AQ-1 à AQ-6 ; §8 énumère la sûreté concurrente comme non démontrée. |
| **A5 (i)** | Arrêt de la boucle sur erreur de tick ; erreur d'origine propagée ; ni reprise automatique, ni remise à zéro silencieuse du buffer. | LC-4 ; AC-4. |
| **A5 (ii)** | Pas d'accès disque réel à l'étape 3 ; attribution explicite à l'étape 5, Roadmap non modifiée. | C5 ; §10. |
| **A6** | Façade `verbum` inchangée, `no_std`. | `crates/verbum/**` interdit sans exception (§11). |
| **C7** | Aucune allocation dynamique sur le chemin d'un tick ; capacité fixée avant la simulation ; revue du code comme contrôle. | §5 ; contrôle de revue en U2, U4, U5 ; §8. |
| **L4 / C12** | Saturation : refus explicite, conservation de l'existant dans l'ordre, aucune perte silencieuse, aucun écrasement, aucune croissance. | AQ-6, AQ-6b ; S8. |

### 4.2 [A4] — arbitrage suspendu : porteur de l'Acquisition

**État.** ADR-003, ADR-005 et ADR-006 ne tranchent pas A4 (annexe `03-verbum-shell-A4-analyse.md`). Ce plan ne retient aucune lecture et ne présente aucune hypothèse comme exigence. Lacunes subsistantes de l'annexe : L1 (écrivain hors phase), L2 (propriétaire de durée de vie), L3 (borne de durée de vie). L4 est close (§4.1).

**Ce qui est imposé, rappelé pour la question [Imposé].**
- « Aucune Ressource n'est accessible par un système sans figurer dans son interface déclarative » (ADR-005 pt 2) ; « Un système ne raisonne que sur les données explicitement fournies par son interface déclarative » (ADR-003 pt 5).
- « au sein d'une phase, une Ressource persistante a un unique propriétaire en écriture » (ADR-005 pt 5) : borné à une phase et aux Ressources persistantes.
- Une donnée transitoire a « un propriétaire de durée de vie » (`01-runtime-ecs.md` §4), « une durée de vie bornée et un état initial déterministe avant toute lecture » ; mécanisme et stockage en implémentation.
- « Le Shell reçoit des signaux bruts […] et les transforme en données d'Acquisition standardisées […], consommées ensuite par le `World` » (ADR-S01 pt 2) ; l'entrée franchit la frontière à l'invocation de l'infrastructure `World`, forme non fixée (ADR-000 pt 3).
- Précédent, non règle : « le chargement runtime (infrastructure assemblée par le Shell) la charge et la libère ; les systèmes ne font que la lire » (ADR-A01 pt 4) ; le résultat chargé « devient une nouvelle donnée d'Acquisition d'une itération future » (ADR-A01 pt 5).

**Vocabulaire de la question.**
- **Entrée (E)** : le lot d'événements traduits, remis par le Shell à l'invocation de l'infrastructure. **Porteur de E** : ce qui le tient depuis sa remise jusqu'à la lecture par un système.
- **Donnée transitoire d'Acquisition (T)** : la catégorie déclarée (composant ou Ressource) que lisent les systèmes de la phase d'Acquisition, **s'il en existe une distincte de E**. **Porteur de T** : la catégorie elle-même.
- **Propriétaire de durée de vie** : le rôle de `01-runtime-ecs.md` §4, pour E d'une part, pour T d'autre part.

**Question à soumettre aux sessions d'origine du corpus** (formulation autonome) :

> Contexte : à l'étape 3 (Shell minimal), le Shell remet à une fermeture opaque (l'infrastructure `World` côté application) un lot d'événements d'Acquisition à chaque tick ; le Shell ne détient jamais le `World`. Les systèmes ne lisent que des catégories déclarées (ADR-003 pt 5, ADR-005 pt 2).
>
> Q1 — **Écriture hors phase.** L'infrastructure `World` exécutante (assemblée par le Shell) peut-elle écrire une Ressource transitoire entre deux ticks, hors de toute phase, de sorte que T = E soit lisible par les systèmes d'Acquisition ? ADR-005 pt 5 et ADR-004 pt 3 ne régissent que l'écriture « au sein d'une phase ». Le précédent d'ADR-A01 pts 4 et 5 (Ressource d'asset déposée par une infrastructure de plateforme) s'applique-t-il aux Ressources transitoires, ou seulement aux Ressources d'asset ?
>
> Q2 — **Entrée à un système d'Acquisition.** Ou bien le Shell doit-il seulement fournir E à un système d'Acquisition qui produit ou résout T ? Dans ce cas, par quelle voie E atteint-il ce système sans contrevenir à ADR-003 pt 5 : E doit-il lui-même être une catégorie déclarée, donc déposée par un acteur extérieur aux systèmes (ce qui ramène à Q1) ?
>
> Q3 — **E et T.** E et T sont-ils une seule catégorie ou deux ? Si deux, laquelle est écrite par qui, et laquelle est lue par les systèmes ?
>
> Q4 — **Durée de vie.** Qui est propriétaire de durée de vie de E, et de T s'ils sont distincts ? À quoi la durée de vie est-elle bornée (la phase d'Acquisition, l'itération, le tick) ? (Le caractère déterministe de l'état initial est imposé ; le mécanisme de remise à l'état initial est une décision d'implémentation.)
>
> Q5 — **Producteurs multiples.** D'autres sources alimentent l'Acquisition (réseau, ADR-N01 ; chargement, ADR-A01 pt 5 ; replay). Si T a plusieurs producteurs, qui en est l'unique résolveur (« produire est libre ; résoudre est unique ») ?
>
> Réponse attendue : le passage normatif qui tranche, ou « non tranché par le corpus ». Aucune modification d'ADR n'est demandée.

**Effet de la réponse sur le plan.** U1 à U4 n'en dépendent pas : sous O2, le Shell ne touche jamais T. **U5** (acceptation avec vrai `World`, adaptateur d'application, exemple console) en dépend : la manière dont la fermeture rend E lisible aux systèmes (dépôt direct dans une catégorie, ou autre) est fixée par la réponse. Aucune unité ne démarre avant la réponse et l'approbation du plan.

## 5. Choix locaux proposés [L] (aucun n'est un invariant)

- Crate `crates/shell`, package `verbum-shell` ; `std` autorisé (ADR-000 pts 2 et 5) ; héritage du workspace (`edition`, `rust-version = 1.85`, `[lints] workspace = true`, donc `unsafe_code = "deny"`) ; **aucune dépendance normale** ; **aucune dépendance externe**.
- Forme **indicative** de la boucle (signatures fixées aux squelettes de U2, sans trait public ajouté) : une fonction d'exécution qui reçoit (a) une fermeture de cadencement `FnMut()` qui attend le prochain tick, (b) une fermeture d'arrêt `FnMut() -> bool` observée au début de chaque itération, (c) la boîte de réception, (d) la fermeture d'infrastructure `FnMut(&[Ev]) -> Result<(), E>` appelée une fois par itération avec le lot prélevé ; elle rend `Err(E)` inchangée à la première erreur, sinon `Ok` avec le nombre d'itérations achevées.
- Itération : attente de cadencement ; observation de la demande d'arrêt ; prélèvement du lot ; appel de l'infrastructure. Un tick engagé n'est jamais interrompu.
- Événement `Ev : Copy` (données simples, sans propriété partagée ni type de plateforme, ADR-000 pt 3).
- Boîte de réception : capacité fixée à la construction, jamais agrandie ; dépôt par une poignée clonable ; sur saturation, le dépôt rend une erreur explicite et ne modifie rien ; prélèvement de l'ensemble du contenu, dans l'ordre d'arrivée, dans un lot de même capacité alloué à la construction. Les allocations de construction sont permises (ADR-000 pt 2) ; aucune allocation dans l'itération (revue).
- La capacité de la catégorie du `World` qui reçoit le lot est celle de l'application ; un lot qui ne tient pas est une erreur de configuration retournée par la fermeture (A5 i), jamais une croissance.
- La boucle exécute exactement **une** itération d'infrastructure par tick. Pas de rattrapage, pas de 0..N itérations, pas d'interpolation, pas de présentation : contenu de la politique réelle (ADR-S04).
- Le `World` ne reçoit jamais de durée ni d'instant.
- Modules provisoires : `lib.rs`, `shell.rs` (boucle), `pacing.rs` (cadencement à pas fixe provisoire), `inbox.rs` ; tests `tests/{lifecycle,pacing,acquisition,acceptance}.rs` ; `examples/console.rs`. Noms provisoires ; tout ajustement est listé au rapport d'unité, sans changement de sémantique.

## 6. Unités

Discipline reprise de l'étape 2 : une unité à la fois, validation explicite avant la suivante ; tests avant implémentation, rouge constaté contre des squelettes `unimplemented!()` (un échec de compilation n'est pas un rouge) ; lorsqu'un test n'exerce que des API déjà implémentées, aucun rouge n'est possible : vert au premier passage plus sensibilité par mutation sur copie isolée hors de l'arbre de travail, jamais présentée comme rouge initial ; quatre contrôles du workspace à chaque unité avec résultats réels ; Miri et MSRV 1.85 exécutés par l'utilisateur ; aucun commit par la session ; gabarit de rapport de `02-verbum-forge-execution.md` §4 ; arrêt immédiat en cas de contradiction ou de divergence entre un fait [Lu] et le code réel.

Ordre : **U1 → U2 → U3 → U4 → U5**. U1 à U4 ne dépendent pas d'A4 ; U5 en dépend.

### U1 — Crate `verbum-shell` vide
- **Fichiers** : `Cargo.toml` racine (ajout à `members`), `crates/shell/Cargo.toml`, `crates/shell/src/lib.rs` (documentation de crate, aucune logique). `Cargo.lock` modifié par Cargo seulement.
- **Invariants visés** : ADR-S01 pt 4 ; ADR-000 pt 3 (le noyau ne nomme pas le Shell) ; A1.
- **Tests** : aucun comportement ; pas de rouge applicable.
- **Contrôles propres** : `cargo tree -p verbum-shell` (aucune dépendance) ; `cargo tree -i verbum-shell --workspace` (aucun dépendant) ; `cargo tree -p verbum-ecs` et `-p verbum-schedule` identiques à l'état de clôture de l'étape 2.
- **Critère** : quatre contrôles verts ; aucun fichier interdit touché.

### U2 — Cycle de vie et boucle bornée
- **Fichiers** : `crates/shell/src/{lib.rs, shell.rs}`, `crates/shell/tests/lifecycle.rs`.
- **Invariants visés** : ADR-S01 pts 1 et 3 ; ADR-000 pt 2 ; A1, A2, A5 (i). Une itération commencée est menée à son terme.
- **Tests d'acceptation** (infrastructure factice qui compte ses exécutions et ses libérations par `Drop` ; arrêt demandé par fermeture scriptée) :
  - **LC-1** : N itérations puis arrêt demandé → exactement N exécutions, dans l'ordre ; une seule libération, après la dernière.
  - **LC-2** : arrêt demandé avant la première itération → 0 exécution ; libération effectuée.
  - **LC-3** : arrêt demandé **pendant** une exécution → cette exécution s'achève (témoin de fin écrit) ; aucune suivante.
  - **LC-4** : une exécution échoue → la boucle s'arrête ; l'erreur d'origine est rendue **sans transformation** (égalité avec la valeur émise) ; aucune réexécution ; libération effectuée.
- **Rouge** : applicable ; motif attendu : panique du squelette.
- **Contrôles propres** : `cargo tree -p verbum-shell -e normal` (aucune dépendance) ; revue : l'API publique ne nomme aucun type du noyau ; revue : aucune allocation dans le corps d'itération (C7).
- **Critère** : LC-1 à LC-4 verts ; contrôles conformes.

### U3 — Cadencement provisoire
- **Fichiers** : `crates/shell/src/{pacing.rs, shell.rs}`, `crates/shell/tests/pacing.rs`.
- **Invariants visés** : ADR-S01 pt 5 ; `00-principes.md` inv. 3 ; Roadmap (politique provisoire).
- **Tests d'acceptation** :
  - **PC-1** : intervalle nul → N itérations sans attente.
  - **PC-2** : pour une même séquence d'Acquisition, la suite des lots observés par l'infrastructure est identique avec un intervalle nul et un intervalle non nul de l'ordre de la milliseconde. Aucune assertion sur la durée réelle.
  - **PC-3** : la boucle accepte une seconde politique de cadencement scriptée (déclenchement à la demande) sans modification du reste : la politique à pas fixe n'est pas figée.
- **Rouge** : applicable.
- **Contrôles propres** : revue de nommage (« provisoire » figure dans la documentation de l'élément public) ; l'API publique n'expose ni `Instant` ni `Duration` à l'infrastructure.
- **Critère** : PC-1 à PC-3 verts ; revue de nommage acceptée.

### U4 — Boîte de réception et test d'invariant de la Roadmap
- **Fichiers** : `crates/shell/src/{inbox.rs, shell.rs}`, `crates/shell/tests/acquisition.rs`.
- **Invariants visés** : ADR-S01 pt 6 ; `02-concepts.md` (principe de circulation : deux flux indépendants, aucune corrélation) ; ADR-000 pts 3 et 4 ; A3 ; L4.
- **Principe** : la source scriptée dépose à tout moment ; au début de chaque itération, le Shell prélève le lot en une fois et le remet à l'infrastructure ; rien d'autre n'entre pendant l'exécution.
- **Source scriptée (A3)** : un dépôt effectué par une fermeture du test, appelée **depuis l'infrastructure factice** (dépôt pendant un tick) ou **depuis la fermeture de cadencement scriptée** (dépôt entre deux ticks). Aucun thread. **Ces tests vérifient la logique de visibilité (quel lot voit quel tick) ; ils ne démontrent aucune sûreté concurrente.**
- **Tests d'acceptation** :
  - **AQ-1** (Roadmap) : un événement déposé **pendant** l'exécution du tick N n'est observé par aucun système du tick N ; il l'est au tick N+1.
  - **AQ-1b** : un événement déposé entre la fin du tick N−1 et le prélèvement du tick N est observé au tick N.
  - **AQ-2** : ordre d'arrivée conservé dans le lot d'un tick.
  - **AQ-3** : sur K ticks, aucun événement perdu ni dupliqué.
  - **AQ-5** [Déduit, retirable à la revue] : une même séquence scriptée (événements attachés à des ticks) rejouée avec deux cadencements donne les mêmes lots.
  - **AQ-6** (saturation, [Clos] L4) : capacité C ; C dépôts réussis ; le dépôt C+1 rend une erreur explicite ; le lot suivant contient exactement les C premiers événements, dans l'ordre ; l'événement refusé n'y figure pas ; la capacité n'a pas changé.
  - **AQ-6b** : après le prélèvement, la capacité est de nouveau disponible : un nouveau dépôt réussit.
- **Rouge** : applicable. **Sensibilité par mutation** en complément : M-S1 prélèvement après l'exécution (AQ-1b doit échouer) ; M-S2 dépôt direct dans le lot en cours d'exécution (AQ-1 doit échouer) ; M-S3 perte d'un événement au prélèvement (AQ-3) ; M-S4 inversion d'ordre (AQ-2) ; M-S5 saturation par écrasement du plus ancien (AQ-6) ; M-S6 saturation silencieuse (dépôt rendant `Ok` sans conserver l'événement) (AQ-6) ; M-S7 croissance de la capacité (AQ-6).
- **Contrôles propres** : revue : le type d'événement est `Copy`, sans type de plateforme ; revue : aucune allocation hors construction (C7).
- **Limite assumée** : un système qui capturerait un handle externe (état global à mutabilité intérieure) n'est pas empêché par les types (`01-verbum-scheduler.md` §2.4, point 3).
- **Critère** : AQ-1 à AQ-3, AQ-5, AQ-6, AQ-6b verts ; sept mutations détectées.

### U5 — Acceptation, exemple console, clôture (**dépend d'A4**)
- **Fichiers** : `crates/shell/Cargo.toml` (dev-dependencies `verbum-ecs`, `verbum-schedule`), `crates/shell/tests/acceptance.rs`, `crates/shell/examples/console.rs`.
- **Invariants visés** : ensemble S1 à S10.
- **Tests d'acceptation** (vrai `World`, vrai `Scheduler`, vrai buffer ; l'adaptateur d'application, écrit dans la fermeture, rend le lot lisible aux systèmes selon la réponse à A4) :
  - **AC-1** : plusieurs ticks, source scriptée ; l'état final égale la valeur attendue, avec le décalage d'un tick pour les événements déposés pendant un tick.
  - **AC-2** : `Scheduler::model()` égal avant et après N ticks ; buffer vide après chaque tick réussi ; le Shell n'a fourni que le lot.
  - **AC-3** : `cargo run -p verbum-shell --example console` : plusieurs ticks, aucune sortie, code retour 0. L'exemple contient seul l'assemblage (S6).
  - **AC-4** : une erreur de tick réelle (par exemple `BufferFull`, ou un `apply` rejeté) arrête la boucle ; l'erreur `TickError` d'origine est rendue inchangée ; le buffer **n'est pas vidé** (`len()` inchangé) ; un nouvel appel sans `clear` rend `BufferNotEmpty`.
- **Contrôles propres** : `cargo tree -i verbum-shell --workspace` ; `cargo tree -p verbum-shell -e normal` (toujours aucune dépendance) ; `cargo tree -p verbum-ecs`, `-p verbum-schedule`, `-p verbum-assets`, `-p verbum-forge` inchangés ; aucune modification de `crates/ecs`, `crates/schedule`, `crates/assets`, `crates/forge`, `crates/verbum`, ADR, `ROADMAP.md`, plans `00` à `02` ; **Miri sur `verbum-shell` et MSRV 1.85 par l'utilisateur** (sans thread réel, donc sans réserve propre aux threads ; `-Zmiri-disable-isolation` seulement si l'horloge l'exige, à constater).
- **Critère** : S1 à S10 renseignés test par test dans le rapport (tableau, comme K1 à K8) ; éléments non démontrés énumérés (§8) ; l'étape est déclarée close par l'utilisateur seulement.

## 7. Contrôles communs
Quatre contrôles à chaque unité : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`. Résultats réels uniquement ; les contrôles non exécutés (Miri, MSRV, doctests nocturnes) sont signalés comme tels. `tree.md` est régénéré par l'utilisateur.

## 8. Ce qui reste non démontré à la clôture
- **Sûreté concurrente réelle** de la boîte de réception (threads réels) : non démontrée ; la source scriptée ne l'établit pas (A3).
- Traduction sans interprétation des signaux de plateforme (ADR-S01 pt 2) ; politique temporelle réelle (ADR-S04) ; gestion des signaux de l'OS (A2).
- Chargement d'artefacts et accès disque (étape 5, A5 ii).
- Absence d'allocation sur le chemin d'un tick : revue seulement (C7).
- Absence d'effets externes cachés dans les systèmes.
- Moitié « sortie » de l'interface de franchissement (étape 4) ; C2 non tranché.
- Porteur de l'Acquisition (A4) tant que la réponse n'est pas rendue.
- Exclusivité de l'assemblage (S6) : revue seulement.

## 9. Risques
1. **Fermeture prématurée** de l'interface de franchissement : couture O2 minimale, à sens unique, provisoire, jamais présentée comme la décision d'ADR-000.
2. **Tick fixe devenu politique de fait** : isolation (PC-3), nommage, absence de temps dans le `World`.
3. **Sur-anticipation** (Renderer, Backend, Audio, chargement, parallélisme) : exclue ; tout ajout cite son invariant ou est écarté.
4. **Hypothèse présentée comme exigence** (A4) : aucune lecture n'est retenue ; U5 attend la réponse.
5. **Confusion sur la portée d'AQ-1** : la source scriptée vérifie la visibilité par tick, pas la concurrence réelle.
6. **Garantie limitée au canal du Shell** (effets externes cachés non empêchés par les types).
7. **Contrôle d'allocation par revue seulement** : un compteur d'allocations exige `unsafe`, interdit par le lint du workspace.

## 10. Hors périmètre
Fenêtre, rendu, Backend, `renderer.compose()`, `backend.present()` ; audio, réseau, sauvegarde, scripts ; politique temporelle réelle (découplage tick/présentation, rattrapage 0..N, interpolation, VSync) ; traduction d'événements OS et bibliothèque de fenêtrage ; **accès disque réel, chargement, streaming, hot-reload (attribué à l'étape 5 par ce plan seulement)** ; gestion des signaux de l'OS ; threads réels et sûreté concurrente ; capacité d'exécution abstraite pour le parallélisme intra-système (ADR-000 pt 1) ; rédaction d'ADR-S02 à S05 ; modification de la Roadmap, des ADR et des plans existants ; toute implémentation de `World::execute()` ou de `Scheduler::run()`.

## 11. Fichiers
**Créés** : `crates/shell/{Cargo.toml, src/lib.rs, src/shell.rs, src/pacing.rs, src/inbox.rs, tests/{lifecycle,pacing,acquisition,acceptance}.rs, examples/console.rs}`. **Modifiés** : `Cargo.toml` racine (`members`), `Cargo.lock` (par Cargo). **Interdits** : `crates/{ecs,schedule,assets,forge,verbum}/**`, ADR, `ROADMAP.md`, `docs/implementation/00` à `02`.

## 12. Fin de l'étape
L'utilisateur déclare l'étape close. Aucun travail de l'étape 4 n'est entrepris. Le document d'exécution de l'étape 3 est rédigé après résolution d'A4 et approbation de ce plan.
