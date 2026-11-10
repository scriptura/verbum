# Page d'axes — Shell, boucle native et présentation graphique

## 0. Statut et conventions

**Révision 2 (10 novembre 2026, après arbitrages).** **Non normative. Non adoptée. Aucun ADR, aucune spécification.** Elle regroupe les questions à fermer avant la rédaction éventuelle d'ADR-S02 (modèle de boucle), ADR-S04 (politique temporelle) et, si le découplage présentation / Frame s'y rattache, ADR-S05 (orchestration). Elle suit la voie « page d'axes » de `00-principes.md` (Grammaire des ADR).

Balises : **[Fait]** vérifié dans un fichier ou une documentation lus ; **[Invariant]** posé par un ADR adopté ; **[Interprétation]** lecture de ma part, à arbitrer ; **[Inconnu]** non vérifié ; **[Option]** possibilité techniquement réaliste, non retenue.

Sources de vérité : code actuel (comportements implémentés), ADR adoptés (invariants), documents d'implémentation (historique). Les orientations de GPT sont traitées comme des orientations à évaluer.

**Hypothèses non retenues par défaut** (conformément à votre cadrage) : `pump_app_events` (O3) ; R-b (relecture du `World` au redessin) ; trait public `Backend` ; méthodes publiques `resize`/`suspend`/`resume` ; second point d'entrée de présentation ; nouveaux types d'événements d'Acquisition ; `unsafe` ; structure auto-référentielle ; runtime asynchrone permanent ; type public `Shell` (un Shell à état n'implique pas un type public, la représentation de l'état reste ouverte) ; toute signature d'API ; toute politique de rattrapage ; toute présomption que N transmissions au Backend respectent « 0..1 ».

**Arbitrages intégrés dans cette révision** (orientations de GPT validées par vous, hors réserves signalées dans la réponse) :
1. Boucle : O1-Shell-driven privilégiée, à approfondir ; O2 conservée si difficulté concrète du monothread ; O3 écartée ; `run_loop` conservée (bloquant, headless) ; cœur d'itération idéalement commun ; l'adaptateur ne reproduit jamais les opérations internes de l'itération.
2. Pacing : propriété du Shell (ADR-S01) ; aucune politique de rattrapage retenue ; S04 distingue quantum logique, provenance (lien avec la Forge), échéance temps réel, nombre de ticks par progression, retard, présentation lente ou indisponible ; le quantum n'est pas le mécanisme de réveil de l'`EventLoop`.
3. Frame : préférence R-a (non adoptée) ; un unique chemin d'entrée des données de Frame vers le Backend, au point A-2 ; le Backend peut conserver la dernière Frame reçue ; entrée des données et exécution graphique restent distinctes ; « 0..N itérations pour 0..1 Frame transmise » à préserver, sans présumer que N transmissions le respectent ; R-b écartée par défaut.
4. Point d'entrée de la première PoC : `crates/games/demo`, futur binaire et membre du workspace ; aucun manifeste modifié, aucun découpage de crates décidé.
5. `PresentMode` : configuration du Backend, à distinguer de la politique de synchronisation éventuellement portée par le Shell ; aucun type wgpu dans les contrats indépendants de la plateforme.
6. Allocation : les allocations internes de wgpu ne sont pas automatiquement soumises à l'invariant du noyau ; les allocations évitables de notre propre code dans le chemin régulier de présentation restent maîtrisées.
7. ADR : S02, S04 et éventuellement S05 restent candidates, aucune n'est rédigée ; S02 et S04 en sessions dédiées ; S05 seulement si une décision autonome le justifie.
8. `frame.rs` : lu ; constatations validées intégrées (voir 1.6).

## 1. Constats de lecture qui conditionnent les axes

**1.1 ADR-000 point 3 contient un point ouvert qui vise précisément cette intégration.** [Fait] Texte : la représentation de frame « peut être retournée par `world.execute()`, lue comme Ressource après son retour, ou produite par un appel distinct ». Si la composition s'exécute hors de `world.execute()`, le code concerné « relève du hot path (`00-principes.md`, point 4) et est soumis à l'invariant [d'allocation], par contrat et revue, sans entrer dans le noyau. Cette ADR ne tranche pas. Cette question reste un point ouvert distinct, à arbitrer avant l'intégration du Shell et du Renderer. »
- L'arbitrage A-2 (forme 2 = « lue comme Ressource après son retour ») est un choix d'implémentation de l'étape 4 « sans amendement d'ADR ». Le point reste donc formellement ouvert dans l'ADR.
- ADR-S03 le reprend dans ses non-tranchés (« moitié "Extraction" d'ADR-000 »).
- Dans le code actuel, `Compose` est un système de phase 1 (dans `world.execute()`), donc dans le noyau. Ce qui est hors noyau : la lecture de la Frame, sa transmission et l'exécution du Backend.
- [Interprétation] ADR-000 vise la composition, pas l'exécution du Backend. [Arbitré] Les allocations internes de wgpu ne sont pas automatiquement soumises à l'invariant d'allocation du noyau ; le code Engine contrôlé par Verbum doit continuer à respecter les invariants existants ; les allocations propres à l'exécution graphique sont un sujet distinct, à délimiter au regard du point ouvert d'ADR-000 (sujet F-A). Les allocations évitables de notre propre code dans le chemin régulier de présentation (copies, `Arc` par tick, etc.) restent à maîtriser.

**1.2 Portée d'ADR-R03 point 5.** [Fait] « Aucune information ne remonte du backend vers l'ECS pendant l'exécution d'une frame. » La règle vise l'ECS. ADR-S01 point 6 vise l'itération en cours. [Interprétation] Une erreur fatale du Backend vers le Shell (arrêt, journalisation) n'est interdite ni par la lettre de l'un ni par celle de l'autre. Elle ne doit jamais atteindre un système ni une itération.

**1.3 Contraintes d'ADR-S03 sur la séquence d'itération.** [Fait] Point 6 : « La publication et l'invocation de l'itération forment une seule séquence du code d'assemblage. Si la publication échoue, l'itération n'est pas lancée. » Point 7 : coupure unique « après la fin d'une itération et avant le début de la suivante ». Point 12 : « Aucune allocation ni croissance dynamique sur le chemin de l'itération. » Point 11 : le déterminisme est posé pour « une même séquence de lots effectivement publiés » ; la répartition des événements entre deux lots n'est pas présumée reproductible et relève d'ADR-S04.

**1.4 ADR-S01 point 3 cite `backend.present()`** comme appel de boîte noire. ADR-R03 point 3 ne nomme aucune méthode : « point d'invocation fixé par construction à la fin de la phase d'Extraction (`01-runtime-ecs.md`, section 5) — un point d'entrée unique, hors du système de contrats, pas un système de plus parmi d'autres ». ADR-S01 point 5 : « 0..N itérations de World::execute() » puis « 0..1 FrameRepresentation transmise au Backend ».

**1.6 `frame.rs` : lu, constatations validées.** [Fait] `Frame<CAP>` = tableau fixe `[Contribution; CAP]` + compteur `len: usize`, sans allocation dynamique ni référence. Taille = 20 × CAP + 8 octets (alignement 8, cible 64 bits) ; `Contribution` = 20 octets. `Clone` (dérivé) copie l'intégralité du tableau, indépendamment de `len`. `Frame` est `Send` + `Sync` (traits automatiques) mais pas `Copy`. Aucun mutateur ni constructeur à partir d'une tranche : l'API ne permet pas une copie bornée à `len` *dans une Frame*. Mesures sur une copie isolée hors dépôt (rustc 1.97, 64 bits, conteneur) : `clone()` ≈ 1,9 µs pour CAP = 4096 et ≈ 7,5 µs pour CAP = 16384, identique quel que soit `len`. Ces temps sont **indicatifs et ne constituent pas un argument de sélection architecturale**. Observations, sans nouvelle contrainte de capacité : `empty`, `compose` et `clone` renvoient par valeur, donc de grandes capacités (320 Ko pour CAP = 16384, 1,3 Mo pour 65536) transitent par la pile ; les limites de pile par défaut n'ont pas été vérifiées ; la capacité représentative du jeu n'est pas décidée (tests : CAP = 4).

**1.5 Vocabulaire.** `02-concepts.md` : `Event` est « non introduit ». Les entrées d'`Inbox` sont des événements bruts de Plateforme, pas des `Event` du modèle. La spécification devra éviter ce terme pour l'Acquisition.

## 2. Carte des axes et ADR cibles

| Axe | Sujet | ADR probable |
|---|---|---|
| A | Contrôle de la boucle et modèle de fils | S02 |
| B | Cœur commun d'itération et progression non bloquante | S02 |
| C | Possession et transport de la Frame (sans emprunt auto-référentiel ni `unsafe`) | S02 / S05 |
| D | Répétition de présentation et circulation du contrôle au redessin | S05 (ou clarification d'ADR-R03) |
| E | Cycle de vie du Backend et erreurs graphiques | S05 |
| F | Politique temporelle | S04 |
| G | Rôle Shell, répartition des crates, assemblage | S05 / W01 (application) |
| H | Initialisation wgpu, MSRV, dépendances | décision d'implémentation (dépendances), pas d'ADR par défaut |
| I | Périmètre de la première PoC et conventions locales | spécification |

Découpage proposé, à arbitrer : **S02** = qui pilote et comment une progression est exposée ; **S04** = quand un tick est dû, combien par progression, rapport tick / présentation, état non présentable ; **S05** = séquence d'invocation des infrastructures, déclencheurs d'exécution du Backend, canaux d'erreur entre infrastructures. [Arbitré] S02 et S04 : sessions dédiées. S05 n'est créé que si une décision architecturale autonome le justifie ; une clarification d'ADR-R03 ou la fermeture du point ouvert d'ADR-000 peuvent être préférables. Aucune n'est rédigée dans cette mission.

---

## Axe A — Contrôle de la boucle et modèle de fils (S02)

**1. Problème.** `run_loop` est bloquante, possède le thread et reçoit ses fermetures par valeur. winit exige de posséder la boucle (`EventLoop::run_app(self, &mut A)`, `EventLoop` ni `Send` ni `Sync`, exécution sur le thread appelant). Il faut décider qui pilote.

**2. Invariants et ADR.** ADR-S01 point 1 (le Shell possède la boucle principale) ; point 4 (fenêtrage, threads, timers confinés au Shell) ; « Ce qui n'est pas tranché » : « choix entre boucle bloquante ou pilotée par callbacks de plateforme : ADR-S02 ». ADR-S03 point 3 (dépôts possibles à tout moment).

**3. Options.**
- **A1 Shell bloquant** (existant) : seul mode headless et tests. Incompatible avec `run_app` seul.
- **A2 O1-Shell-driven** : winit pilote ; l'adaptateur demande une progression au Shell et programme l'attente avec l'échéance retournée.
- **A3 O2 Shell sur fil dédié** : `run_loop` inchangée ; winit sur son fil ; transport d'entrées par `DepositHandle` (déjà clonable, `Arc<Mutex>`) ; Frame transmise entre fils.
- **A4 Deux pilotes sur un cœur commun** (axe B) : A1 pour headless, A2 pour le natif.
- Écartée : O3 (`pump_app_events` : déconseillée par la doc winit, indisponible iOS/Web, artefacts de redimensionnement macOS signalés).

**4. Conséquences.**

| | A2 | A3 |
|---|---|---|
| Fils | un seul | deux, synchronisation |
| Mémoire | Frame copiée dans un emplacement local | emplacement partagé verrouillé, `Frame<CAP>` est `Send` + `Sync` (vérifié, pas un obstacle) |
| Propriété | Shell à état possédé par le handler | Shell possédé par son fil ; fermetures ou `thread::scope` si emprunts |
| Blocage en mode Fifo (notamment dans `Surface::get_current_texture()` quand la file est pleine ; `present` n'est pas supposé le seul point bloquant) | retarde la progression suivante | n'affecte pas les ticks |
| Arrêt / erreur | `run_app` a son propre `Result` ; deux terminaisons à unifier | coordination entre fils |
| Dépendances | aucune | aucune de plus (std) |
| `run_loop` et ses suites | partagés si cœur commun (axe B) | inchangés |

**5. Décisions manquantes.** [Arbitré] A2 privilégiée (à approfondir, non adoptée) ; A3 conservée si le modèle monothread rencontre une difficulté concrète ; `run_loop` conservée pour les usages bloquants et headless. Ouvert : critère de bascule vers A3 ; cœur commun (axe B) ; affinité de thread de `EventLoop` selon plateforme [Inconnu : contraintes macOS à vérifier] ; déclencheur de la progression : `about_to_wait` (après la distribution des événements du lot : les dépôts du lot sont dans la coupure) ou `new_events` (avant : latence d'une itération de plus). [Fait winit] `about_to_wait` n'est pas idéal pour piloter le rendu, mais la progression de simulation n'est pas du rendu si l'exécution GPU reste dans `RedrawRequested` (axe D).

**6. Expériences minimales.** E1 (ordre et fréquence des callbacks winit, réveil `WaitUntil`), E2 (mesure du blocage en mode Fifo, à l'acquisition de texture et à la présentation, sur le fil de boucle). Pour A3 : `Frame<CAP>: Send` est établi, aucune expérience nécessaire.

---

## Axe B — Cœur commun et progression non bloquante (S02)

**1. Problème.** Le corps de `run_loop` est : `cadence(); if stop() { return }; infrastructure(inbox.cut())?; n += 1`. `FixedStepPacing` n'expose que `wait()` (sommeil, un tick par appel, `next` privé, pas d'accesseur d'échéance). Une progression non bloquante a besoin de « un tick est-il dû à `now` ? » et de « prochaine échéance ». L'adaptateur ne doit pas reproduire `cut`, la publication, l'infrastructure, l'arrêt ni la progression temporelle.

**2. Invariants et ADR.** ADR-S01 points 5 et 6 ; ADR-S03 points 6 et 7 (une séquence coupure + publication + invocation, jamais de publication pendant une itération) ; PC-3 (pacing remplaçable par fermeture). `FixedStepPacing` est provisoire ; ADR-S04 n'existe pas.

**3. Options.**
- **B1 Cœur d'itération partagé** : une unité « arrêt observé → coupure → infrastructure → compteur ». Pilote bloquant = attente + cœur ; pilote événementiel = échéance + cœur répété 0..N fois. Les suites AQ, LC (arrêt, première erreur inchangée) s'appliquent au cœur.
- **B2 Deux implémentations parallèles** : duplication de sémantique ; écarte la demande.
- **B3 `run_loop` réécrite au-dessus de la progression** : une seule implémentation ; risque pour les suites existantes (à rejouer).

**4. Conséquences.** Le Shell devient un état possédé : pacing, `Inbox`, fermeture d'infrastructure (qui possède le `World`), état terminal. La représentation de cet état (type nommé, fermeture capturée, structure privée) reste ouverte ; aucun type public `Shell` n'est adopté ; aucune signature n'est adoptée. Cela correspond à `02-concepts.md` (le Shell « crée, assemble et orchestre » les infrastructures et le `World` en tant qu'exécutant). Aucun type du noyau dans le Shell (généricité par fermeture, comme aujourd'hui). `Instant` est de `std` : la propriété « zéro dépendance » est préservée. Changement additif dans `verbum-shell`, **non autorisé à ce stade**.

**5. Décisions manquantes.**
- Contenu du compte rendu : itérations exécutées, prochaine échéance (ou « aucune »), état terminal (arrêt demandé, erreur).
- Comportement après erreur ou arrêt (progression refusée ? état terminal explicite).
- Qui lit l'horloge : le Shell (conforme à S01 pt 5) ou un `now` injecté par l'adaptateur (tests sans sommeil ni horloge murale, atout vu les tests déjà écrits sans horloge pour Miri).
- Position de la transmission de Frame par rapport à N itérations (axe F4).
- Nature des suites à adapter (AQ, LC, PC).

**6. Expériences.** E3 : un banc de tests de progression avec `now` injecté, vérifiant mêmes lots, mêmes arrêts et même première erreur que `run_loop` sur les scénarios existants (pas d'horloge murale).

---

## Axe C — Possession et transport de la Frame

**1. Problème.** Le Shell possède le `World` (via la fermeture d'infrastructure). La Frame n'existe que comme Ressource du `World`. `&Frame` n'est valide que pendant l'appel A-2. Le Backend doit pouvoir réexécuter la dernière Frame sur `RedrawRequested` (appelé sur le handler winit). Pas d'`unsafe`, pas d'emprunt auto-référentiel.

**2. Invariants et ADR.** A-2 : « le code d'assemblage lit la Ressource `Frame` après le retour de `run_tick` et la transmet au Backend par `&Frame`. Aucun trait public `Backend` ni autre trait d'interface ; le Backend est une fermeture `FnMut(&Frame<CAP>)` ». ADR-R03 point 1 (le Backend reçoit exclusivement la représentation de frame), point 2 (passif). ADR-S01 point 3 (le Shell traite `world.execute()` comme une boîte noire). ADR-S03 point 12 (pas d'allocation sur le chemin de l'itération). `Frame<CAP>` : `Clone`, `PartialEq`, `as_slice`, `len` [Fait, `frame.rs` lu : tableau fixe + `len`, taille 20 × CAP + 8 octets, `Clone` = copie du tableau entier, `Send` + `Sync`, pas `Copy` ; voir 1.6]. R-a impose de conserver la dernière Frame, même en monothread [arbitré]. La Ressource `Frame` est mise à jour en interne à chaque tick (par `Compose`, dans `world.execute()`) ; seule sa transmission au Backend est visée par « 0..1 » : ce sont deux événements distincts.

**3. Structure de possession proposée à examiner (sûre).** Le handler `H` possède : le Shell (par valeur), la fenêtre, l'unité de présentation (ou une poignée vers elle). La fermeture d'infrastructure, possédée par le Shell, possède le `World` et un « puits » de transmission obtenu par clonage d'un `Rc` (ou d'un `Arc`). Aucun emprunt de `H` par le Shell : pas de cycle de propriété. `H::about_to_wait` n'emprunte que `self.shell` ; `H::window_event` n'emprunte que le reste.

**4. Options de stockage / transmission.**

| Option | Principe | Coût mémoire / exécution | Remarques |
|---|---|---|---|
| C1 Copie dans un emplacement fixe | Le puits copie `&Frame` dans un emplacement à capacité fixe (`Rc<RefCell<…>>` ou `Cell` si monothread) | copie de 20 × CAP + 8 octets par transmission, indépendante de `len` (`Clone` dérivé) ; aucune allocation si l'emplacement est préalloué | sûr, simple ; copie bornée à `len` impossible *dans une Frame* (aucun mutateur ni constructeur à partir d'une tranche) |
| C1b Conservation de la tranche seulement (option technique, non adoptée) | Le Backend retient les éléments de `frame.as_slice()` dans un emplacement fixe qui lui appartient, au lieu de cloner la Frame complète | copie de `len` × 20 octets ; aucune allocation dynamique ; aucune modification de l'API de `Frame` | le Backend retient une représentation, pas le type `Frame` ; structure exacte du stockage et propriétaire non décidés ; comparaison avec la Frame reçue sur la tranche |
| C2 Double emplacement avec bascule | deux emplacements, écriture sur l'arrière, bascule d'indice | 2 × la mémoire ; copie identique | utile surtout en O2 (éviter de copier sous verrou) |
| C3 Représentation GPU retenue | le puits téléverse dans un buffer GPU à chaque transmission ; le redessin ne fait que dessiner | téléversement par tick, même si non dessiné | une copie CPU reste nécessaire pour reconstruire après perte de surface ou de device (L4) |
| C4 `Arc<Frame>` immuable | partage sans copie | une allocation par tick (ou un pool) | en tension avec « zéro allocation sur le hot path » et ADR-000 point 3 |
| C5 Relecture du `World` (R-b) | le handler lit le `World` au redessin | aucun stockage | exige que le handler ou le Shell accède au `World` : en tension avec ADR-S01 point 3 (boîte noire) et avec la propriété par fermeture ; non retenue par défaut |

**5. Décisions manquantes.** Type du puits (`Rc` vs `Arc`) selon A2 / A3 ; copie complète (C1) ou tranche seule (C1b) ; partage de l'état de présentation entre la transmission A-2 et le redessin winit (question ouverte, non tranchée par les constatations sur `Frame`) ; contenu de l'emplacement avant la première Frame (rien : ne pas dessiner, ou fond constant local au Backend) ; propriétaire de l'emplacement (unité de présentation, voir axe G).

**6. Expériences.** E4 : établi par lecture de `frame.rs` (voir 1.6) ; aucune expérience supplémentaire demandée. `Send` n'est plus une inconnue pour O2.

---

## Axe D — Répétition de présentation et circulation du contrôle

**1. Problème.** Comment `RedrawRequested` déclenche-t-il l'exécution GPU de la Frame conservée sans introduire une seconde invocation de Frame incompatible avec ADR-R03 ?

**2. Notions (à ne pas confondre).**

| Notion | Définition |
|---|---|
| Production | `Compose` (système) écrit la Ressource `Frame` ; immuable ensuite (ADR-R03 point 1 : « composée, immuable — ADR-R02 ») |
| Première présentation | première transmission de cette Frame au Backend, selon A-2, à la fin du tick |
| Répétition | ré-exécution par le Backend de la dernière Frame reçue, sans nouvelle composition ni nouvelle transmission |
| Redessin autonome OS | `RedrawRequested` émis par l'OS sans tick (exposition, redimensionnement) ; winit agrège les doublons |

Transmission (donnée vers le Backend) et exécution (travail GPU) sont deux actes distincts ; A-2 ne couvre que la transmission.

**3. Dispositions applicables.**
- ADR-R03 point 3 : « point d'invocation fixé par construction à la fin de la phase d'Extraction — un point d'entrée unique, hors du système de contrats, pas un système de plus parmi d'autres ».
- ADR-R03 point 2 : le Backend « applique la représentation qu'il reçoit ; il ne la produit pas, ne l'interprète pas, ne la questionne pas ».
- ADR-R03 point 5 : rien ne remonte vers l'ECS.
- A-2 : voir axe C ; aucun trait public `Backend` ; fermeture `FnMut(&Frame<CAP>)`.
- ADR-S01 point 5 : « Aucun rapport fixe entre une itération de simulation et une présentation de frame n'est présumé » ; 0..N itérations, 0..1 Frame transmise.
- C6 (plan stage 4) : le nombre de présentations est une propriété du montage, pas un invariant. `01-runtime-ecs.md` §5 : l'Extraction « prépare » des données ; elle n'est pas définie comme dernière phase (C5 est une équivalence du montage).

**4. Lectures de « point d'entrée unique » [Interprétation].**
- L-1 : unicité du *callable* par lequel le Backend reçoit des données de Frame.
- L-2 : unicité du *moment* (une fois par tick, fin d'Extraction).
- L-3 : unicité *par construction, hors du système de contrats*, par contraste avec un système ; la phrase n'adresse alors pas la répétition.
Aucun texte n'interdit explicitement de ré-exécuter une Frame déjà reçue.

[Arbitré] Le Frame dispose d'un **unique chemin d'entrée** vers le Backend, au point A-2. Le Backend peut conserver une représentation du dernier Frame reçu et la réutiliser sur `RedrawRequested` sans nouvelle transmission de données de rendu. La distinction entre entrée des données et exécution graphique doit rester explicite. Une clarification d'ADR-R03 pourra être nécessaire (non rédigée). R-b reste écartée par défaut.

**5. Options.**
- **D1 Exécution dans l'appel A-2 (P1 strict)** : la fermeture exécute le GPU au tick. Répétition : aucune, ou ré-exécution interne sur demande. Conflit avec le modèle winit (rendu hors `RedrawRequested`, signalé comme non idéal ; artefacts macOS déjà documentés).
- **D2 Transmission au tick, exécution différée (R-a, préférence arbitrée, non adoptée)** : A-2 copie dans l'emplacement et demande un redessin ; `RedrawRequested` exécute l'état retenu. Un seul passage de données de Frame (A-2). L'exécution est déclenchée par la Plateforme, pas par le modèle. Compatible avec L-1 et L-3 si le déclencheur d'exécution n'est pas un point d'entrée de données. Contredit L-2 si « invocation » s'entend de toute exécution.
- **D3 Repli sur la prochaine transmission** : un redessin OS ne fait que marquer l'état « à redessiner », exécuté à la prochaine transmission A-2. Aucun second déclencheur, mais dépend de la cadence des ticks ; un redessin sans tick suivant (pause, occlusion) reste sans effet ; effet visuel sur plateformes à repeinte obligatoire [Inconnu].
- **D4 Fermeture à deux régimes** `FnMut(Option<&Frame>)` (`None` = répéter) : un seul callable, mais signature d'A-2 modifiée, et plusieurs moments d'invocation ; exige un arbitrage et une clarification d'ADR-R03.
- **D5 R-b** : non retenue par défaut (axe C5).

**6. Où se place le déclencheur d'exécution.** Dépend de l'axe G :
- Si l'unité de présentation et l'adaptateur sont dans le même crate (L1) : appel interne au crate, pas d'API publique de Backend.
- Si le Backend est un crate séparé (L3) : D2 exige un second callable public (ou un état partagé interrogé par une fonction publique), soit une « seconde entrée de présentation » au sens de votre cadrage ; D3 l'évite au prix de la dépendance à la cadence ; D4 l'évite en modifiant A-2.

**7. Décisions manquantes.** Lecture retenue de « point d'entrée unique » ; D2 / D3 / D4 ; statut du déclencheur d'exécution (donnée ou non) ; comportement avant la première Frame ; qui appelle `request_redraw` (le puits, ou l'adaptateur après lecture du compte rendu).

**8. Expériences.** E5 : sur chaque plateforme cible, observer ce qui se passe si l'on ne dessine pas sur un `RedrawRequested` OS (contenu conservé ou non) ; E6 : mesurer le délai entre transmission, `request_redraw` et `RedrawRequested`.

**Lien avec ADR-S01 point 5 (0..1) [arbitré : ne pas présumer que N transmissions respectent 0..1].** Quand une progression exécute N ticks, la Ressource `Frame` est mise à jour N fois (interne au `World`) ; la transmission au Backend doit rester unique, après la dernière itération. Sinon la transmission serait N fois effectuée même si une seule exécution graphique en résulte. Il faut donc que la séquence pilotée par le Shell distingue la dernière itération : fermeture d'infrastructure scindée (partie par itération, partie après la dernière) ou indicateur « dernière itération » connu du Shell avant la rafale ; la représentation n'est pas décidée. Avec le pacing actuel, N vaut 0 ou 1 : sans effet aujourd'hui.

---

## Axe E — Cycle de vie du Backend et erreurs graphiques

**1. Problème.** Redimensionnement, minimisation, suspension, reprise, invalidation de surface, erreurs GPU : aucune méthode publique, aucune seconde entrée de présentation, ADR-R03 non modifié.

**2. Trois catégories à ne pas mélanger.**
- **Cycle de vie de la plateforme** : `Resized(PhysicalSize<u32>)`, `ScaleFactorChanged { scale_factor: f64, inner_size_writer }` (ordre relatif non spécifié), `Occluded(bool)`, minimisation (taille nulle), `Suspended` / `Resumed` (suspension non émise en pratique sur desktop), `CloseRequested`.
- **État graphique interne** : surface, configuration, device, queue, image retenue ; statuts de surface (`Success`, `Suboptimal`, `Timeout`, `Occluded`, `Outdated`, `Lost`, `Validation`), perte de device, OOM, validation.
- **Frame** : unique source des données de rendu produites par le moteur ; aucune information de cycle de vie n'y entre.

**3. Options de transmission du cycle de vie.**
- **L1 Unité unique fenêtre + Backend (+ adaptateur)** : traitement interne au crate. Surface publique : fabrique, puits A-2, point d'entrée de lancement. Aucune méthode publique de Backend. Coût : winit et wgpu dans le même crate ; le Backend perd la propriété « dépend de `verbum-render` seul ».
- **L2 Le Backend consulte l'état** : relit `inner_size()` physique à chaque exécution, reconfigure sur `Outdated` / `Suboptimal`. Le Backend voit la fenêtre (handle). Évite toute notification, mais couple le Backend à la fenêtre.
- **L3 État de cible de présentation partagé** : données plates (taille physique, échelle, occlusion, génération) écrites par l'adaptateur, lues par le Backend. Flux unidirectionnel. Introduit un second canal de données à nommer ; le type doit vivre dans un crate visible des deux (le Backend, ou un crate tiers). Une structure publique en écriture libre est proche, par l'effet, de méthodes de cycle de vie : à arbitrer.
- **L4 Reconstruction par fabrique** : `Resumed` ou `Lost` détruisent et recréent le Backend. Perd caches et image retenue (écran vide jusqu'au tick suivant, sauf copie CPU de la Frame, axe C3).
- [Interprétation] Heuristique du « domaine manquant » (`00-principes.md`) et principe de circulation (`02-concepts.md`) : défavorables à L2 (deux domaines qui se connaissent), compatibles avec L1 et L3.

**4. Erreurs.**
- Statuts de surface : internes au Backend (reconfigurer, sauter la présentation, jamais d'effet sur l'ECS).
- Erreur fatale (device perdu non récupérable, OOM) : canal Backend → rôle Shell. A-2 renvoie `()` ; un canal latéral est nécessaire : cellule d'erreur partagée lue par l'adaptateur, qui demande l'arrêt. Variante : fermeture renvoyant `Result` vers l'infrastructure : modification d'A-2, non autorisée par défaut.
- [Inconnu] `set_device_lost_callback` : bornes `Send` / `Sync` du callback à vérifier ; elles peuvent forcer un puits atomique même en monothread.

**5. Conséquences.**
- Mémoire : un seul état partagé de quelques octets (L3) ; copie de Frame (axe C) pour L4.
- Propriété : le Backend possède son état GPU ; l'adaptateur possède la fenêtre ; la surface exige un handle de fenêtre quelle que soit l'option (`create_surface(impl Into<SurfaceTarget<'window>>)` est sûr ; `Arc<Window>` vers `Surface<'static>` [Inconnu : non confirmé dans la doc]).
- Dépendances : L1 regroupe ; L3 sépare.

**6. Décisions manquantes.** L1, L3 (ou autre) ; statut d'un état partagé public ; comportement en taille nulle (ne pas configurer) ; canal d'erreur fatale (latéral ou via `Result`) ; politique de reconstruction.

**7. Expériences.** E7 : redimensionnement continu (nombre de `Resized`, ordre avec `ScaleFactorChanged`) ; E8 : minimisation (taille 0 × 0), occlusion ; E9 : injection de `Outdated` / `Lost` si possible.

---

## Axe F — Politique temporelle (ADR-S04)

Le rattrapage 0..N n'est pas adopté ; aucune politique de rattrapage n'est retenue [arbitré]. ADR-S04 devra distinguer explicitement : quantum logique (F1), provenance et lien avec la Forge (F1), échéance en temps réel (F2), nombre de ticks par progression (F3), comportement en cas de retard (F4), politique si la présentation est lente ou indisponible (F7). Chaque sous-point est séparé.

**Cadre normatif.** [Invariant] `00-principes.md` inv. 3 : même séquence d'entrées, même séquence d'états ; exclus : `deltaTime` flottant, ordre non spécifié, scheduling dynamique dépendant du timing OS. ADR-S01 point 5 : la politique est une responsabilité du Shell ; rien n'est présumé. ADR-S03 point 11 : déterminisme pour une même séquence de lots effectivement publiés ; répartition des événements entre lots non reproductible, renvoyée à S04. `00-principes.md` inv. 1 : les secondes sont converties en ticks par la Forge (le pas de tick est donc aussi une donnée de build-time).

**Distinction à rendre explicite.** [Interprétation] Exécuter plusieurs ticks fixes ne remet pas en cause le déterminisme si la séquence de lots par tick est identique. Mais l'affectation des événements OS aux lots dépend de leur arrivée. Ne pas déduire une autorisation de rattrapage de l'invariant général.

| # | Question | Options | Décisions manquantes / expériences |
|---|---|---|---|
| F1 | **Quantum logique du tick** : valeur, provenance, lien avec la conversion temps → ticks de la Forge | constante de build partagée avec la Forge ; paramètre de runtime | où vit la durée d'un tick (couplage avec la conversion AOT) |
| F2 | **Échéance en temps réel** (distincte du quantum logique et du mécanisme qui programme le réveil de l'`EventLoop`) | `next += pas` (sans dérive) ; `next = now + pas` (dérive) ; sémantique « aucune échéance » (pause) | `FixedStepPacing` n'a ni accesseur ni `now` ; expérience : E10 précision de `WaitUntil` selon plateforme |
| F3 | **Nombre de ticks par progression** | exactement 0 ou 1 (existant) ; 0..N borné ; 0..N non borné | borne ; effet sur la latence |
| F4 | **Politique en cas de retard** | un tick par réveil (retard cumulé) ; rattrapage borné ; abandon des ticks perdus ; resynchronisation de l'échéance | gels de fenêtre, blocage de la présentation (Fifo), reprise après suspension ; pas de choix par défaut |
| F5 | **Affectation des lots aux ticks** | à la première itération d'une rafale, les suivantes vides (conséquence directe des coupures de S03 pt 7) ; répartition par instants (exige des instants dans la donnée, renvoyés à S04 par S03) | latence d'entrée ; reproductibilité au niveau des lots seulement |
| F6 | **Relation ticks / présentations** | présentation par tick ; découplée (D2) ; transmission après la dernière itération seulement | pour « 0..1 transmis » : fermeture d'infrastructure scindée (par itération / après la dernière), ou indicateur « dernière itération » fourni par le Shell ; N transmissions ne sont pas présumées conformes à 0..1 (arbitré) |
| F7 | **Présentation lente ou suspension** | mécanisme `PresentMode` : configuration du Backend [arbitré], à distinguer de la politique de synchronisation ou de cadence éventuellement portée par le Shell (S01 pt 5 : VSync) ; aucun type wgpu dans les contrats indépendants de la plateforme ; Fifo peut bloquer notamment à l'acquisition de texture quand la file est pleine ; état non présentable (occlus, minimisé, suspendu) : continuer, ralentir, geler | politique de gel ; resynchronisation à la reprise ; qui décide de l'abandon d'un `present` lent |
| F8 | **Interpolation** | reportée | hors périmètre de la première PoC |
| F9 | **Instants temps réel dans les données** | renvoyé par S03 | non nécessaire à la PoC |

**Allocation (F-A).** [Arbitré] Les allocations internes de wgpu ne sont pas automatiquement soumises à l'invariant d'allocation du noyau. Reste à délimiter, au regard du point ouvert d'ADR-000 point 3 (composition hors `world.execute()` ; le texte ne parle pas de l'exécution du Backend), ce que couvre « le chemin régulier de présentation » pour le code propre à Verbum (copie de Frame, état partagé, canal d'erreur) : ces éléments restent sans allocation évitable.

**Grammaire des ADR.** [Interprétation] Les six questions de `00-principes.md` ne sont pas toutes renseignables aujourd'hui pour S04 (F4, F5, F7 en particulier) : la page d'axes puis des expériences précèdent l'ADR.

---

## Axe G — Rôle Shell, crates, assemblage

**1. Problème.** Le **rôle** Shell (point de composition de la Plateforme) n'est pas le **crate** `verbum-shell` (aujourd'hui sans dépendance). Aucun type winit ou wgpu ne doit entrer dans les contrats indépendants de la plateforme (noyau ECS/Scheduler, Renderer/Frame).

**2. Invariants et ADR.**
- `02-concepts.md` : une infrastructure est « assemblée exclusivement par le Shell, seul point de composition de la strate » ; le Shell crée, assemble et orchestre les infrastructures et le `World`.
- ADR-S01 points 4 et 7 ; ADR-S03 point 1 (traduction côté Shell, aucun type de Plateforme dans la donnée) et point 5 (publication « au travers du code concret d'assemblage »).
- ADR-W01 (niveaux Engine / Extension / Application, selon mon relevé précédent) : la Plateforme est Engine ; l'Application compose systèmes et contrats ; l'Engine ne nomme ni Extension ni Application.
- ADR-000 point 3 : le noyau (graphe d'appel de `world.execute()`) ne nomme pas le Shell, et son périmètre est vérifié statiquement.

**3. Répartition des rôles (clarifiée).** Adaptateur winit : rôle Shell. Backend wgpu : infrastructure. `World` : exécutant créé et possédé dans la composition. Application : systèmes et contrats.

**4. Tension à arbitrer.** [Interprétation] Le code d'assemblage concret (S03 pt 5) nomme des types du noyau (Ressources, systèmes) ; or l'Engine ne peut pas nommer l'Application. Le point d'assemblage unique est donc physiquement au niveau Application (binaire ou exemple), construit avec des briques Engine du Shell. À écrire en S05 : « le Shell » comme rôle réparti sur plusieurs crates.

**5. Options de rattachement.**

| Option | Contenu | Avantages | Coûts |
|---|---|---|---|
| G1 | `verbum-shell` étendu (progression, sans dépendance) + crate adaptateur (winit, dépend de `verbum-shell`) + crate Backend (wgpu, dépend de `verbum-render`) | frontières nettes ; Backend sans winit | D2 sur Backend séparé exige un second callable ou D3 / D4 (axe D) ; trois unités à versionner |
| G2 | `verbum-shell` étendu + un seul crate « plateforme graphique » (adaptateur + Backend + fenêtre) | L1 : cycle de vie et redessin internes, aucune API publique de Backend | winit et wgpu ensemble ; Backend non réutilisable seul |
| G3 | tout dans un exemple de `verbum-shell` (dev-dépendances) | rien de plus dans le graphe normal | exemples hors périmètre d'ADR-W01 ; `cargo test --workspace` compile winit et wgpu ; pas de réutilisation |
| G4 | dans `verbum-shell` derrière une feature | lecture littérale de S01 pt 4 | perd « aucune dépendance » ; interaction features / `--workspace` |

Dans tous les cas : le Shell-crate reste sans type winit (la progression ne manipule que `std::time::Instant` et des fermetures) ; `unsafe_code = "deny"` reste applicable aux crates du workspace.

**Point d'entrée de la première application [arbitré].** `crates/games/demo` (aujourd'hui un README, hors workspace) deviendra, lorsque ce sera autorisé, un package binaire et un membre du workspace. Responsabilité : fournir les types concrets du jeu, les systèmes et contrats de l'application, puis démarrer l'exécution. Les infrastructures génériques de plateforme restent en strate Engine : `games/demo` n'est pas leur emplacement permanent. Aucun manifeste modifié, aucun découpage de crates décidé dans cette révision. Conséquence factuelle : ce binaire dépendra des crates de plateforme ; une fois membre, `cargo … --workspace` compilera winit et wgpu.

**6. Décisions manquantes.** Rôle ou crate ; G1 / G2 ; lieu physique de l'assemblage ; crate hébergeant le type d'état partagé (L3) ; effet sur les vérifications du workspace : `cargo … --workspace` inclut tous les membres quel que soit `default-members` ; un crate hors workspace ou exclu échappe à ces vérifications.

**7. Expériences.** E11 : dessiner le graphe de dépendances Cargo de G1 et G2 sur papier (aucun manifeste modifié) et vérifier qu'aucun crate Engine ne nomme Application ou Extension, et que `verbum-render` reste sans dépendance normale.

---

## Axe H — Initialisation wgpu, MSRV, dépendances

**1. Problème.** `request_adapter` et `request_device` sont asynchrones. La fenêtre n'est créée qu'après le premier `Resumed`. Le plancher Rust du workspace est 1.85 ; wgpu 30.0.1 exige 1.87.

**2. Faits.** [Fait] `Instance::request_adapter(&RequestAdapterOptions) -> impl Future<Output = Result<Adapter, RequestAdapterError>>` ; `Adapter::request_device(&DeviceDescriptor) -> impl Future<Output = Result<(Device, Queue), RequestDeviceError>>`. Séquence : `resumed` → fenêtre → `Instance` → surface → adaptateur (avec surface compatible) → device → configuration.

**3. Options d'attente ponctuelle.**
- **H1 Crate d'exécuteur minimal** (par exemple `pollster`, version et MSRV non vérifiés ici) : +1 crate dans `Cargo.lock`, code déjà éprouvé ; décision de dépendance non prise.
- **H2 Exécuteur écrit maison avec `std::task::Wake` + parking** : environ une vingtaine de lignes sûres ; coût : tests, maintenance, cas des futures qui répondent `Pending` ; justifiable seulement si la dépendance est refusée pour une raison établie.
- **H3 Boucle de scrutation avec `Waker::noop()`** : plus court, mais boucle active si un `Pending` survient ; la stabilité de `Waker::noop` (1.85, à ma connaissance) est à vérifier.
- Aucun runtime asynchrone permanent.

**4. Conséquences.** L'appel bloque brièvement la boucle winit, une fois (dans `resumed`). Erreur d'adaptateur ou de device : erreur fatale de démarrage vers l'arrêt du Shell (S01 pt 1). [Inconnu] Les futures natives sont-elles prêtes au premier `poll` ? Si oui, H3 suffit et H1 / H2 deviennent facultatifs.

**5. MSRV et lock [Fait].**
- `rust-version = "1.85"` défini une seule fois à `Cargo.toml:18` ; les huit membres (`assets`, `ecs`, `forge`, `render`, `schedule`, `shell`, `verbum`, `extensions/script`) l'héritent. Un relèvement = une ligne (non modifiée).
- `Cargo.lock` : 25 paquets, dont les 8 membres Verbum ; ni winit ni wgpu.
- Direction retenue : plancher uniforme ≥ 1.87. Le minimum réel est le maximum sur tout l'arbre, connu après résolution [Inconnu].
- **Ordre** [Inconnu : comportement du résolveur à vérifier] : à ma connaissance, `resolver = "3"` préfère des versions compatibles avec le `rust-version` du workspace. Relever le plancher avant la première résolution incluant wgpu évite une résolution calée sur 1.85.
- Les contrôles `cargo +1.85.0 … --workspace` du handoff de l'étape 4 sont à réécrire (historique). La discipline « sans let-chains » ne tombe qu'à partir de 1.88.
- Version de `rustc` installée et éventuel `rust-toolchain.toml` / CI : non vus.

**6. Décisions manquantes.** Plancher (1.87 ou plus) ; H1 / H2 / H3 ; sort de la discipline let-chains ; réécriture des contrôles `+1.85.0`.

**7. Expériences.** E12 : observer si `request_adapter` / `request_device` rendent `Ready` au premier `poll` ; E13 : résolution à blanc sur une copie hors dépôt pour mesurer le plancher réel (hors périmètre tant que rien n'est autorisé).

---

## Axe I — Première PoC : périmètre et conventions locales

**Point d'entrée** : application `crates/games/demo` (axe G), fournissant types concrets, systèmes et contrats.

**Périmètre [votre cadrage]** : fenêtre native ; initialisation wgpu ; première image ; progression du véritable Shell ; véritable Frame ; redessin sans nouveau tick ; fermeture ; redimensionnement ; attente et réveil programmés.

**Clavier : non requis.** `headless_frame` et `frame_loop` ne déposent aucun événement ; le lot n'a aucun lecteur ; `Ev = u8` est un type d'exemple. Aucun catalogue d'Acquisition (ADR-S03 : non tranché). La fermeture passe par `CloseRequested` vers l'arrêt du Shell, sans donnée d'Acquisition. **Limite à énoncer** : l'`Inbox` est câblée (lots vides) mais aucun dépôt issu de l'OS n'est exercé.

**Conventions locales à la PoC (pas des invariants).**
- Unités de `Contribution` = pixels physiques de la surface ; origine en haut à gauche ; Y vers le bas ; coin supérieur gauche.
- `[u8; 4]` écrit tel quel ; format de surface non sRGB préféré, sinon conversion locale ; alpha opaque seulement.
- Fond : constante locale (la Frame n'a ni couleur d'effacement, ni caméra, ni couche).
- Pas de recouvrement (aucune règle d'ordre, C8).
- Rectangles suffisamment grands pour être visibles (les 2 × 2 et 3 × 3 de l'exemple ne le seraient pas).
- `ScaleFactorChanged` : reconfiguration à la taille physique ; dimensions toujours relues via `inner_size()`.

**À décider séparément** : résolution virtuelle et mise à l'échelle ; rôle du facteur d'échelle ; axe Y du monde ; espace colorimétrique ; mélange alpha ; ordre de dessin ; découpage ; fond et caméra.

**Vérifications automatiques.** `cargo test --workspace` ne peut pas valider un rendu visuel ; la PoC visuelle est manuelle (exemple).

---

## 3. Matrice des décisions et dépendances

| # | Décision | ADR | Dépend de |
|---|---|---|---|
| 1 | Pilote du natif (A2 / A3) et coexistence avec `run_loop` | S02 | — |
| 2 | Cœur d'itération partagé, forme du compte rendu, état terminal | S02 | 1, F2, F3 |
| 3 | Qui lit l'horloge ; `now` injecté ou non | S02 / S04 | 2 |
| 4 | Pas de tick et échéance (F1, F2) | S04 | `00-principes.md` inv. 1 et 3 |
| 5 | Nombre de ticks par progression et retard (F3, F4) | S04 | 4 |
| 6 | Affectation des lots (F5) | S04 | 5, S03 pt 11 |
| 7 | Relation ticks / présentations, transmission « 0..1 » (F6) | S04 / S05 | 2, 5 |
| 8 | Lecture de « point d'entrée unique », D2 / D3 / D4 | S05 (ou clarification d'ADR-R03) | 7 |
| 9 | Point ouvert ADR-000 pt 3 : allocation hors `world.execute()` | S05 / ADR-000 | 8 |
| 10 | Rôle ou crate, G1 / G2, lieu de l'assemblage | S05 / W01 | 8 |
| 11 | Cycle de vie du Backend (L1 / L3), canal d'erreur fatale | S05 | 8, 10 |
| 12 | État non présentable et `PresentMode` (F7) | S04 | 5, 7 |
| 13 | Plancher MSRV, exécuteur ponctuel | décision d'implémentation | — (avant toute résolution wgpu) |

Ordre suggéré : 1 → 4, 5 → 2, 3 → 6, 7 → 8, 9, 10 → 11, 12. Les décisions 13 et la liste des conventions PoC sont indépendantes.

## 4. Expériences minimales (proposées, non lancées)

| # | Objet | Tranche | Besoin |
|---|---|---|---|
| E1 | Ordre et fréquence des callbacks winit, réveil `WaitUntil` | A, F2 | fenêtre |
| E2 | Blocage en mode Fifo (acquisition de texture, présentation) sur le fil de boucle | A, F4, F7 | fenêtre + GPU |
| E3 | Progression à `now` injecté contre `run_loop` | B | aucun (tests) |
| E4 | Représentation, `Clone`, taille et `Send` de `Frame<CAP>` : établi (voir 1.6) | C | fait |
| E5 | Redessin OS non traité : contenu conservé ? | D | fenêtre, par plateforme |
| E6 | Délai transmission → `request_redraw` → `RedrawRequested` | D | fenêtre |
| E7 | Redimensionnement continu, ordre `Resized` / `ScaleFactorChanged` | E | fenêtre |
| E8 | Minimisation, occlusion | E, F7 | fenêtre |
| E9 | Injection de `Outdated` / `Lost` | E | GPU |
| E10 | Précision de `WaitUntil` | F2 | fenêtre |
| E11 | Graphe de crates G1 / G2 sur papier | G | aucun |
| E12 | Futures wgpu prêtes au premier `poll` ? | H | GPU |
| E13 | Plancher MSRV réel après résolution | H | copie hors dépôt |

## 5. Non vérifié à ce stade

`Arc<Window>` → `Surface<'static>` ; futures wgpu natives prêtes immédiatement ; bornes `Send` / `Sync` du callback de perte de device ; limites de pile par défaut selon plateforme et fil (grandes capacités de `Frame`) ; comportement du résolveur `resolver = "3"` avec un plancher inférieur ; `StartCause::ResumeTimeReached` ; stabilité de `Waker::noop` ; contraintes d'affinité de thread de `EventLoop` hors desktop ; MSRV de winit et des dépendances transitives ; version de `rustc` locale ; point de vue exact d'ADR-W01 (relevé précédent, non relu dans cette passe).

## 6. Décisions qui restent ouvertes pour les sessions ADR

**Session ADR-S02 (modèle de boucle)**
- Critère de bascule de O1-Shell-driven vers O2 (difficulté concrète du monothread) ; affinité de thread de l'`EventLoop`.
- Cœur d'itération commun à `run_loop` et au pilote événementiel : périmètre exact (arrêt observé, coupure, publication, exécution, première erreur inchangée) ; sort des suites AQ, LC, PC.
- Représentation de l'état du Shell (type nommé, fermeture capturée, structure privée) ; aucun type public ni signature adoptés.
- Compte rendu d'une progression (itérations, échéance, état terminal) ; comportement après arrêt ou erreur ; qui lit l'horloge (Shell ou `now` injecté).
- Déclencheur de la progression côté winit (`about_to_wait` ou `new_events`) ; articulation arrêt / erreur avec le `Result` de `run_app`.
- Séquence pilotée par le Shell permettant une transmission unique après la dernière itération.

**Session ADR-S04 (politique temporelle)**
- Quantum logique du tick, provenance et lien avec la conversion temps → ticks de la Forge.
- Échéance en temps réel, distincte du mécanisme de réveil de l'`EventLoop`.
- Nombre de ticks par progression ; comportement en cas de retard (aucune politique de rattrapage retenue).
- Affectation des lots d'entrées aux ticks (reproductible au niveau des lots seulement, ADR-S03 point 11).
- Politique de synchronisation ou de cadence portée par le Shell, distincte du `PresentMode` (configuration du Backend) ; présentation lente ou indisponible (occlusion, minimisation, suspension).

**Sans ADR hôte désigné (à placer : S02, une clarification ou S05 si une décision autonome apparaît)**
- Lecture de « point d'entrée unique » d'ADR-R03 point 3 et statut du déclencheur d'exécution sur `RedrawRequested` (clarification d'ADR-R03 éventuelle, non rédigée).
- Fermeture du point ouvert d'ADR-000 point 3 : périmètre du « chemin régulier de présentation » pour le code propre à Verbum.
- Cycle de vie du Backend (L1 / L3), canal d'erreur fatale du Backend vers le rôle Shell, jamais vers l'ECS.
- Lieu physique du code d'assemblage (rôle Shell réparti sur plusieurs crates) et découpage des crates ; rôle de `crates/games/demo` limité à l'Application.

**Décisions d'implémentation hors ADR**
- Plancher MSRV (≥ 1.87) et ordre : plancher avant la première résolution incluant wgpu.
- Exécuteur ponctuel pour `request_adapter` / `request_device` (dépendance, code maison ou scrutation).
- Conservation de la dernière Frame : clone complet (C1) ou tranche seule (C1b) ; structure et propriétaire de l'emplacement ; partage de l'état de présentation entre la transmission A-2 et le redessin winit.

**Plus d'action préalable sur `frame.rs`** : lu et constaté (voir 1.6).
