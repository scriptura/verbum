# Plan de construction — étape 4 : Renderer minimal (`verbum-render`) — r3.2, 28 octobre 2026

## Statut
Document de planification interne, non normatif, révisable sans ADR (même statut que `ROADMAP.md` et `03-verbum-shell.md`). Il détaille l'étape 4 en unités. Il n'introduit aucun invariant : chaque exigence cite sa source, et tout ce qui ne découle pas du corpus est signalé comme tel. **Aucun code, aucune commande, aucun commit, aucune modification de la Roadmap, des ADR, du code ou des documents des étapes précédentes.** Le document d'exécution et le handoff sont rédigés : ils suivent la validation explicite de ce plan.

**État de validation : r3.2 (révision minimale du 28 octobre 2026, limitée à RP-5 : les codes d'erreur des doctests `compile_fail` ne sont pas épinglables sur Rust stable) ; r3.1 était validé ; r3.2 est soumis à la validation de l'utilisateur.**

Légende : **[Lu]** texte vérifié dans un document lu dans cette session · **[Imposé]** disposition normative citée · **[Déduit]** conséquence que je tire de textes lus · **[L]** choix local, non invariant, révisable · **[C]** ambiguïté, contradiction ou lacune réelle · **[A]** arbitrage requis de l'utilisateur · **[Ouvert]** laissé ouvert, non résolu par ce plan.

## 0. Sources et limites

**[Lu] normatif** : `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, ADR-001, ADR-003, ADR-004, ADR-005, ADR-R01, ADR-R02, ADR-R03, ADR-S01, ADR-S03, `README.md` d'architecture.
**[Lu] non normatif** : `ROADMAP.md` (étapes 0 à 4), `03-verbum-shell.md` (r4 révisé : §0, §1, §3, §7 à §12), `tree.md` du 26 octobre 2026 (fait foi sur les chemins).
**[Lu] code réel** : `crates/schedule/src/scheduler.rs` (`Scheduler`, `run_tick`, `TickError`), `order.rs` (règles de dépendance et de propriété, en-têtes), `model.rs` (en-têtes), `contract.rs`, `crates/ecs/src/resource.rs` (`ResourceSlot`), `crates/ecs/src/lib.rs`, `crates/schedule/src/lib.rs`, `crates/schedule/examples/headless.rs`. Le Shell (`run_loop`, `Inbox`, `FixedStepPacing`, tests et exemple) est connu par mon travail de l'étape 3.
**Mémoire de projet (déclarations de l'utilisateur, non relues dans le dépôt)** : l'évolutivité du Renderer vers plusieurs contributions visuelles ordonnées pour un même ensemble logique (cas cible multi-couches, moteur agnostique, aucun vocabulaire de ce cas dans le noyau) est un **point de vigilance pour le cadrage de l'étape 4**, pas une exigence ; ne pas présumer `FrameResource { layers: Vec<Layer> }`.

**Non lus dans cette session, donc non vérifiés** :
- **Lus depuis r1** : **ADR-000** (lu intégralement, version du 27 octobre 2026 ; les marques « second main » de r1 sont levées) ; **`02-verbum-forge-handoff..md`** (version du 16 octobre 2026, non commitée ; ses §4 et §5 sont traités en §11.1) ; `system.rs`, `access.rs`, `data.rs`, `command.rs` (lus pour la rédaction du document d'exécution).
- **Non lus** : ADR-002, ADR-006, ADR-A01, `02-verbum-forge*.md`, `world.rs` (signatures publiques seulement, par leurs usages), `HANDOFF-stage-0/1`, documents de réflexivité, brouillons multi-couches.
- Le `Cargo.toml` racine réel (avec `crates/shell` dans `members`) a été reçu le 27 octobre 2026 : `[workspace.dependencies]` ne liste que `verbum-ecs` et `verbum-schedule`.

**ADR-S02, ADR-S04, ADR-S05 n'existent pas** ; ADR-S03 est adoptée (19 octobre 2026). Aucune politique temporelle réelle n'existe : toute cadence reste celle, provisoire, de l'étape 3.

## 1. Objectif et critères de sortie

### 1.1 Contrat de la Roadmap [Lu]
- Contenu : `Systems → Frame Resource (composition) → Backend → présentation`. Un clear-color ou un carré suffit. Aucune caméra, animation, sprite réel. Backend headless préférable : « le test porte sur la circulation `FrameRepresentation → Backend → aucun retour`, pas sur l'ouverture d'une fenêtre » ; il démontre que Backend signifie « consommateur passif ».
- Dépendance : étapes 1 et 3 (« le Renderer s'invoque depuis le Shell, en aval du Scheduler »).
- Test d'invariant : « La représentation de frame est immuable une fois composée, et rien ne remonte du Backend vers l'ECS pendant l'exécution (ADR-R02 point 4 ; ADR-R03 point 5). »
- Artefact : « première frame produite et présentée (headless ou fenêtrée) ».
- Principe directeur : un jalon n'est validé que si aucun invariant architectural n'a dû être modifié pour le faire fonctionner.

### 1.2 Périmètre retenu [Déduit]
L'étape 4 construit : (a) une représentation minimale des contributions et de la frame, à capacité fixe ; (b) un pipeline `producteurs → composition unique → représentation` exécuté par le Scheduler réel sur le `World` réel ; (c) un Backend headless passif, invoqué par le code d'assemblage après le tick ; (d) l'assemblage sous la boucle du Shell de l'étape 3 ; (e) un exemple exécutable sans sortie. Tout le reste est hors périmètre (§12).

### 1.3 Critères de clôture
Voir §14. La clôture est déclarée par l'utilisateur seul.

## 2. Exigences documentées et source

| # | Exigence | Source | Statut | Test (§7) |
|---|---|---|---|---|
| E1 | La représentation de frame ne change plus une fois composée ; elle est transmise telle quelle en aval | ADR-R02 pt 4 ; table de classification (Invariant) | Imposé | RP-5, RP-7, RN-4, RA-4 |
| E2 | Aucune information ne remonte du Backend vers l'ECS pendant l'exécution d'une frame ; toute donnée de retour est une entrée d'Acquisition d'une itération ultérieure | ADR-R03 pt 5 ; ADR-S01 pt 6 ; `02-concepts.md` (principe de circulation) | Imposé | RA-2, RA-3 |
| E3 | Le Backend ne reçoit ni entités, ni composants, ni ressources, ni systèmes, ni phases, ni contrats : exclusivement la représentation de frame | ADR-R03 pt 1 ; `00-principes.md` (frontières) | Imposé | RA-2 (signature), revue |
| E4 | Le Backend applique ; il ne produit pas, n'interprète pas, ne décide ni layout ni ordre | ADR-R03 pt 2 | Imposé | RA-2, revue |
| E5 | Le Backend n'est ni contrat, ni dépendance, ni phase ; son point d'invocation est fixé à la fin de la phase d'Extraction | ADR-R03 pt 3 | Imposé | RN-2, RA-1 |
| E6 | Remplacer un Backend par un autre ne modifie ni la représentation, ni sa composition ; un Backend headless est admis | ADR-R03 pt 4 ; ADR-S01 pt 7 | Imposé | RA-3 |
| E7 | Les systèmes producteurs s'ignorent mutuellement (existence, contenu, ordre) | ADR-R02 pt 1 ; ADR-003 | Imposé | RN-1 (par construction), RN-2 |
| E8 | Production append-only ; aucune relecture avant la composition | ADR-R02 pt 2 | Imposé | RP-1, RN-2 |
| E9 | Composition unique ; dépendance producteurs → composition déduite des contrats, jamais déclarée | ADR-R02 pt 3 ; ADR-004 pt 3 | Imposé | RN-2, RN-3 |
| E10 | La composition agrège sans écarter ni arbitrer aucune contribution | ADR-R01 pt 4 ; `02-concepts.md` (« Composition ») | Imposé | RP-4, RN-1 |
| E11 | Un producteur ne connaît ni tri, ni regroupement, ni format de la représentation finale, ni Backend | ADR-R02 pt 5 | Imposé | RN-2, revue |
| E12 | La contribution est le contenu interne d'une Ressource transitoire ; ni persistante, ni adressable, ni requêtable | ADR-R01 pt 1 ; ADR-R02 (bilan) ; `02-concepts.md` (frontière structure interne) | Imposé | RP-1 (aucune clé), revue |
| E13 | Donnée transitoire : durée de vie bornée, état initial déterministe avant toute lecture | `01-runtime-ecs.md` §4 ; ADR-005 pt 4 | Imposé | RP-3, RP-6, RN-5 |
| E14 | Capacités fixées avant la simulation ; dépassement = bug de configuration, jamais croissance dynamique ni cas géré silencieusement en release | ADR-001 pt 5 ; ADR-000 pt 4 [Lu] ; `00-principes.md` inv. 4 | Imposé (stockages de composants : ADR-001 ; capacités en général : ADR-000 pt 4). Le **refus explicite** `Err(Full)` n'est pas exigé littéralement : c'est un choix [L] compatible, par analogie avec ADR-S03 pt 10 | RP-2 |
| E15 | Le Shell possède et assemble les infrastructures ; il les invoque comme des boîtes noires | ADR-S01 pts 3 et 7 | Imposé | RA-1, revue de l'assemblage |
| E16 | Aucune infrastructure ne modifie une itération en cours | ADR-S01 pt 6 | Imposé | RA-1, RA-3 |
| E17 | 0..N itérations de simulation pour 0..1 représentation transmise au Backend ; aucun rapport fixe présumé | ADR-S01 pt 5 | Imposé | aucun test n'affirme « une frame par tick » comme invariant (C6) |
| E18 | Un système ne voit une Ressource que si son contrat la déclare ; aucun état caché | ADR-003 pts 1, 2, 5 ; ADR-005 pt 2 | Imposé | RN-2 |
| E19 | Déterminisme : mêmes entrées, mêmes états | `00-principes.md` inv. 3 | Imposé | RN-6 |
| E20 | La `FrameRepresentation` relève de la strate « structure interne » ; le Backend de la Plateforme | `02-concepts.md` (grille) | Imposé | structure des crates (§13) |
| E21 | Zéro allocation sur le hot path | `00-principes.md` inv. 4 | Imposé | revue (aucun compteur sans `unsafe`, interdit par le lint du workspace) |

## 3. Ambiguïtés, contradictions et lacunes

| # | Constat | Traitement dans ce plan |
|---|---|---|
| **C1** | ADR-S01 pt 3 cite `renderer.compose()` comme infrastructure invoquée par le Shell ; ADR-R01 §5 et ADR-R02 pt 3 font de la composition **un système**. ADR-000 (Statut, point 3) désigne cette relation comme point ouvert à arbitrer avant l'intégration du Renderer. | **[Clos pour l'étape 4]** avec C13 (A-2) : composition interne au modèle (un système, ADR-R02 pt 3) ; aucun appel `renderer.compose()` distinct. Aucune fermeture générale pour l'avenir ; aucun texte n'est modifié. |
| **C2** | Où se trouve la représentation entre la composition et le Backend ? ADR-R02 (bilan) : « au niveau du modèle ECS, une seule donnée » (l'accumulateur) ; Roadmap (non normative) : « Frame Resource » ; ADR-R03 pt 1 : le Backend reçoit « la représentation » ; `02-concepts.md` la classe en structure interne. | **[Déduit]** : un système ne produit que des données déclarées ou des commandes (ADR-003 pt 1) ; la sortie de la composition ne peut donc être qu'une Ressource transitoire du `World` (aucun canal de sortie de système n'existe dans le corpus). **Deux Ressources** : l'accumulateur (produit librement, consommé par la composition) et la Frame (produite par la composition seule). Justification par la preuve : sans deux types distincts, « immuable une fois composée » se réduit à « personne n'ajoute plus rien », invérifiable par les types. Cette lecture n'est ni imposée ni contredite par ADR-R02 (voir A-2, clos). |
| **C3** | La Frame est transitoire (renouvelée à chaque itération) mais n'a qu'un producteur. Le Scheduler distingue `writes` (persistant, un seul écrivain par phase, appliqué par le modèle : `MultipleWriters`) et `produces` (transitoire, libre) ; une catégorie est écrite **ou** produite, jamais les deux (`MixedNature`). | **[C]** Déclarer la Frame en `produces` respecte sa nature mais le modèle n'impose alors pas l'unicité du composeur ; la déclarer en `writes` l'impose par phase mais contredit « persistantes » (`contract.rs`). Dans les deux cas l'unicité inter-phases n'est **pas** garantie par le modèle (précédent : C14 de l'étape 3). **[L]** `produces`, avec **contrôle de composition** (RN-2) et revue. À signaler dans chaque rapport d'unité. |
| **C4** | Qui possède la durée de vie de l'accumulateur et le remet à l'état initial ? `01-runtime-ecs.md` §4 exige un propriétaire sans le désigner ; ADR-R02 (bilan) nomme « le propriétaire unique de cette Ressource » comme celui qui applique la composition. | **[L]** Un système `Clear` (produit l'accumulateur : `insert` d'un accumulateur vide) en **phase 0** en est le propriétaire de durée de vie (motif de `headless.rs`). Le composeur **lit** l'accumulateur (`Read`) et **produit** la Frame ; il ne le vide pas. *Correction r3 :* le composeur ne peut pas lire et produire l'accumulateur dans un même accès (une catégorie ne peut figurer deux fois dans un accès, quel que soit le mode : erreur de compilation E0080) ; `Produce` n'ajoute aucune lecture au contrat (`MixedNature` ne porte que sur les écrivains/producteurs d'une catégorie sur tout le modèle, pas sur un accès). **Interprétation retenue pour l'étape 4 (décision du 28 octobre 2026, limitée à l'étape 4, sans amendement d'ADR-R01) :** la durée de vie *logique* d'une contribution prend fin lorsque `Compose` l'a consommée ; la présence physique d'anciennes valeurs dans l'accumulateur n'en fait pas des contributions vivantes, à condition (i) qu'aucun système autorisé ne puisse les relire, (ii) que l'état soit remis à l'état initial avant toute lecture ultérieure (`01-runtime-ecs.md` §4 laisse le mécanisme de remise à l'état initial à l'implémentation). *Contrôle :* `Compose` est l'unique lecteur de l'accumulateur ; `Clear` est en phase 0, strictement avant la phase de `Compose` : il précède, dans l'ordre du modèle, les lectures de l'accumulateur *déclarées par les systèmes du modèle* ; `Compose` s'exécute après les producteurs dans la phase d'Extraction, qui reste la dernière phase (RN-2, RN-3). **Limites :** (1) le contrôle de composition porte sur les systèmes assemblés et ne peut pas interdire une lecture directe de la ressource depuis le code extérieur au Scheduler ; (2) l'absence de lecture de l'accumulateur hors des systèmes (assemblage, fermeture d'infrastructure, instruments, observateurs) après `Compose` n'est établie ni par les types ni par les contrats du Scheduler : elle relève de la **revue de l'assemblage de référence** ; cette garantie n'est pas imposée par les types ; elle ne définit pas une sémantique générale des données transitoires. |
| **C5** | ADR-R03 pt 3 : invocation du Backend « à la fin de la phase d'Extraction ». Le Scheduler ne connaît pas de **type** de phase, seulement un nombre de phases et `run_tick` complet ; le Shell n'a pas de point d'accroche entre phases (et ne doit pas en avoir, ADR-S01 pt 3). | **[Déduit]** « fin de la phase d'Extraction » ≡ « fin du tick » **si et seulement si** la composition est dans la dernière phase. Contrôle de composition (RN-2) : le composeur est dans la dernière phase. |
| **C6** | ADR-S01 pt 5 : 0..1 représentation par 0..N itérations. La cadence provisoire de l'étape 3 donne 1 tick par itération. | Les tests peuvent compter une présentation par tick dans l'assemblage de test, jamais l'affirmer comme invariant. |
| **C7** | `System::run` rend `Result<(), BufferFull>` : un producteur n'a aucun canal d'erreur pour une saturation de l'accumulateur autre que ce type (sémantiquement celui du Command Buffer), un `panic` ou une perte silencieuse (interdite). Le crate du Scheduler est hors périmètre. | **[Ouvert]** au niveau général (confirmé par l'utilisateur : le Scheduler n'est pas modifié). **Obligation du document d'exécution** : aucun producteur du scénario accepté n'ignore silencieusement `Full` ni ne le traduit en erreur de Command Buffer ; la borne (contributions maximales par producteur et par tick, somme ≤ `CAP`) est explicitée ; la garantie n'est pas généralisée au-delà de ce scénario. |
| **C8** | ADR-R03 pt 1 : la représentation reçue est « déjà ordonnée ». Aucun critère d'ordre n'est donné : ADR-R01 pt 3 décrit la capture « dans l'ordre où elle a été émise » ; ADR-R02 pt 5 interdit aux producteurs de connaître le tri ; ADR-R02 (« Ce qui n'est pas tranché ») renvoie tri et clés « à documenter séparément, si un besoin concret l'exige, plutôt que par anticipation » ; ADR-004 : un système n'a pas d'ordre intrinsèque. | **[Ouvert]** Voir §4, Q2. Fait [Lu] dans le handoff de l'étape 2 (§4.4, test de S5) : l'ordre entre systèmes sans arête est l'ordre de **déclaration**, qui n'est pas une sémantique ; l'ordre d'émission en dépend donc. Question Q8 du handoff (qui porte l'ordre d'empilement : donnée, asset, composition du Renderer ?) : non tranchée par ADR-R01 à R03, laissée ouverte.  L'ordre de la séquence de la Frame est déterministe (ordre du modèle) mais **non contractuel** à l'étape 4. |
| **C9** | ADR-S03 pt 6 : aucune itération ne commence sans lot publié. Un assemblage sans aucun système lecteur du lot doit-il quand même publier ? | **[L]** L'assemblage de l'étape 4 reprend le motif de l'étape 3 (Ressource-lot initialisée vide à l'assemblage, publiée avant chaque `run_tick`), sans lecteur. Coût : recopie du motif ; aucun concept nouveau. Lecture la plus prudente du pt 6 ; consignée. |
| **C10** | « Rien ne remonte du Backend vers l'ECS » : ADR-R03 pt 5 vise l'ECS ; une erreur de présentation rendue au **Shell** ne l'est pas explicitement. | **[L]** `present` ne rend rien (`()`). La remontée d'erreurs de présentation vers le Shell n'est ni spécifiée ni nécessaire à la preuve ; **[Ouvert]**. |
| **C11** | Le mécanisme de réinjection d'une donnée de retour (picking, mesure) à l'Acquisition d'une itération ultérieure est « non tranché » (ADR-R03). | Aucun test n'utilise la boîte de réception comme **le** mécanisme de retour. Voir RA-3 (option non retenue). |
| **C12** | La Roadmap dit « clear-color **ou** carré ». Un clear-color est un attribut de la frame, non une contribution : il introduirait un élément de représentation qui n'est pas une contribution (ADR-R01 pt 1). | **[L]** Une contribution « carré » ; aucun clear-color. |
| **C13** | ADR-000 (Statut et point 3) désigne comme ouvertes la forme de l'interface de franchissement **et** la relation entre `renderer.compose()` et la composition interne ; le point 6 laisse la forme exacte à l'implémentation. C1 et C13 sont un seul point. Le droit de **lecture** d'une Ressource du `World` par le code d'assemblage n'est prévu nulle part (ADR-S03 pt 5 limite le droit d'assemblage au remplacement intégral du lot). | **[Clos pour l'étape 4]** A-2 : forme 2 (lecture de la Frame après `run_tick`, emprunt `&Frame` en lecture seule), choix d'implémentation de l'étape 4 décidé par l'utilisateur le 27 octobre 2026. |

## 4. Les cinq points à examiner

**Q1 — Représentation minimale pour l'invariant de frame immuable.**
Le corpus impose l'effet (E1), non la forme (ADR-R02 : forme mémoire « hors périmètre » ; `02-concepts.md` : structure interne libre). La preuve exige : (i) un instant « composée » identifiable ; (ii) un type dont l'API publique ne permet aucune modification après construction ; (iii) une copie de la valeur composée que rien ne partage avec la source. **[Déduit]** : une `Frame` à capacité fixe, construite d'un seul coup par la composition, **sans aucune méthode `&mut self`**, distincte de l'accumulateur modifiable. L'immuabilité est alors exprimée par l'absence de mutateurs (types) et par un contrôle des contrats (unique producteur), non par une convention. La représentation reste une **séquence** de contributions (ADR-R01 pt 2 : 0..N) : elle ne présume ni couche, ni clé de tri, ni forme multi-couches.

**Q2 — Propriété, consommation, ordre.**
- Accumulateur : producteurs (production libre, append-only, ADR-R02 pts 1 et 2) ; consommateur unique : le composeur ; propriétaire de durée de vie (remise à l'état initial) : le système `Clear`, phase 0 [L, C4].
- Frame : producteur unique : le composeur ; consommateur : le Backend, via le code d'assemblage ; tout système lecteur éventuel est en lecture seule et ordonné après le composeur (déduit des contrats).
- Ordre : **le corpus ne fournit aucune règle d'ordre indépendante de l'ordre d'émission** (C8). Il en découle : (a) un ordre fondé sur l'ordre d'exécution des producteurs dépendrait d'un ordre que ADR-004 déclare sans valeur intrinsèque ; (b) un critère d'ordre fondé sur un champ de la contribution (ADR-R01 pt 1 cite un « calque ») est une décision de tri que ADR-R02 renvoie explicitement à un besoin concret ultérieur. **Le plan ne retient aucune règle d'ordre** : la séquence de la Frame suit l'ordre d'émission, déterministe mais non contractuel ; aucun test ne l'affirme, aucun consommateur n'a le droit de s'y fier. Ce que cela laisse ouvert est consigné en §11.

**Q3 — Capacité et dépassement.**
- **Capacité : à spécifier dès cette étape [Déduit].** Le noyau est `no_std` sans allocation (`crates/ecs/src/lib.rs`) ; ADR-001 pt 5 et `00-principes.md` inv. 4 imposent des capacités fixées avant simulation ; ADR-000 pt 4 [Lu] l'étend (« un dépassement de capacité est un bug de configuration, jamais une croissance dynamique »). Un type de représentation sans capacité fixe est donc impossible dans ce cadre. La capacité est un paramètre constant des types ; l'accumulateur et la Frame partagent la même constante, ce qui rend le dépassement à la composition **impossible par construction** (relation d'ADR-S03 pt 7 transposée, sans contrôle d'exécution).
- **Dépassement au niveau du conteneur : à spécifier [Déduit]** (refus explicite, contenu déjà accepté conservé dans son ordre, aucune croissance, aucune perte silencieuse) : ADR-001 pt 5 et ADR-000 pt 4 excluent croissance et traitement silencieux ; la forme `Err(Full)` est un choix [L] compatible, par analogie avec ADR-S03 pt 10. Test RP-2.
- **Réaction d'un système producteur au refus : [Ouvert]** (C7). Aucune disposition ne la fixe ; aucun test ne la ferme ; les producteurs de test ne saturent pas.

**Q4 — Test synthétique de deux éléments indépendants et ordonnés.**
Deux demandes distinctes :
- **« Ordonnés » : non requis, et irait au-delà de la preuve.** Le test d'invariant de la Roadmap (E1, E2) ne dépend d'aucun ordre ; ADR-R02 renvoie le tri à un besoin concret ; l'affirmer créerait un invariant absent du corpus (C8). Le handoff de l'étape 2 (§5, Q9) posait cette question comme « à décider au cadrage » ; ADR-R01 à R03, désormais lus, n'attribuent pas l'ordre d'empilement au Renderer minimal.
- **« Indépendants » (deux producteurs, au moins deux contributions) : non requis par le texte du test d'invariant de la Roadmap, mais rendu nécessaire pour rendre la preuve sensible.** Avec une seule contribution, la Frame composée est indiscernable de la contribution ; un composeur qui écarte ou arbitre (E10), un Backend qui lit l'accumulateur au lieu de la Frame, ou une composition omise ne sont pas distingués. Deux producteurs coûtent un type de système et n'ajoutent aucun concept. A-1 (§5) : option Y retenue par l'utilisateur le 27 octobre 2026.

**Q5 — « Présentation » en headless.**
Le corpus définit le rôle, non l'observable : le Backend « applique la représentation qu'il reçoit » (ADR-R03 pt 2), invoqué à la fin de la phase d'Extraction (pt 3), sans retour (pt 5). **[L]** En headless, « présenter » = le Backend reçoit une référence partagée à la Frame complète, après la fin du tick, et l'applique à sa cible, qui est ici un enregistrement **côté Plateforme** (copie de la Frame dans une liste propre au Backend). Le test observe, depuis la Plateforme (le test, jamais un système) : (a) le nombre de présentations ; (b) le contenu présenté, égal à celui composé ; (c) la séquence : fin de tick *k* → présentation *k* → début de tick *k+1* ; (d) l'absence de toute différence observable de l'état du `World` imputable à la présentation. Le canal d'observation ne passe jamais par le `World` ni par un système : il serait sinon un canal de retour (E2).

## 5. Arbitrages (état)

**A-1 — Portée de la preuve de composition — [Clos] : option Y retenue par l'utilisateur le 27 octobre 2026.** Options examinées :
- **X** : un producteur, une contribution. Prouve E1 et E2 ; ne prouve pas E10 ; la composition est indiscernable d'un passage direct.
- **Y** : deux producteurs indépendants, au moins deux contributions ; assertions sur l'**ensemble** composé (aucune perte, aucun arbitrage), sans assertion d'ordre. Prouve E1, E2, E7, E9, E10 ; ne crée aucune règle.
- **Z** : Y plus une règle d'ordre affirmée. Crée une règle de tri absente du corpus (C8) ; ferme prématurément la vigilance multi-couches ; **non retenue**.
Option retenue : **Y**, pour la raison de Q4 ; passer à X supprime RN-1 et RN-3 en partie, sans autre effet.

**A-2 — Interface de franchissement, moitié sortie (ADR-000, Statut et points 3 et 6) — [Clos] par l'utilisateur le 27 octobre 2026.**
- **Décision** : forme 2. Le code concret d'assemblage lit la Ressource `Frame` après le retour de `run_tick` et la transmet au Backend par un emprunt temporaire `&Frame` en lecture seule. La composition reste interne au modèle (un système, ADR-R02 pt 3). Aucun trait `Backend` public, aucun canal de sortie alternatif.
- **Statut** : choix d'implémentation propre à l'étape 4, consigné dans ce plan une fois validé ; il **n'est ni provisoire ni non résolu pour cette intégration**. Aucune ADR ni aucun amendement d'ADR-000 n'est requis pour ce seul choix, tant que les autres contraintes d'ADR-000 sont respectées. Il **ne ferme pas** les questions futures sur l'interface de franchissement (formes 1 et 3 de l'ADR, politique temporelle, mécanisme de retour, Backend réel).
- **Passages déterminants [Lu]** : Statut d'ADR-000 : « la forme de l'interface de franchissement […] et la relation entre `renderer.compose()` et la composition interne au modèle (point 3) restent à arbitrer avant l'intégration du Shell et du Renderer » ; point 3 : « la représentation de frame peut être retournée par `world.execute()`, lue comme Ressource après son retour, ou produite par un appel distinct (`renderer.compose()` […]) » ; point 6 : « La forme exacte de l'interface de franchissement de `world.execute()` » est « volontairement laissée à l'implémentation ».
- **Compatibilité** : forme 2 = la deuxième des trois formes énumérées. La forme 1 exigerait que `run_tick` rende la frame (Scheduler validé, hors périmètre) ; la forme 3 placerait la composition hors du modèle, contre la lecture directe d'ADR-R02 pt 3, sans nécessité pour la preuve.
- **Conséquences tirées d'ADR-000** : (1) la composition est un système atteignable depuis `run_tick` : ses types sont dans le noyau (`no_std` sans `alloc`), **vérifié statiquement** dans les limites du point 4 (blocs `unsafe`, appels externes) ; (2) la lecture de la Frame et la présentation sont hors noyau, sur le hot path : l'absence d'allocation y est exigée par contrat et **vérifiée par revue seulement, non par preuve statique** (instrument préalloué, §6) ; (3) « ni propriété partagée ni référence à un type de la Plateforme » : `Frame` est une donnée plate ; `&Frame` est un emprunt temporaire, non un transfert ni un partage de propriété ; **aucune copie** n'est faite sauf si une contrainte réelle de durée de vie ou de typage l'impose, à signaler alors dans le rapport d'unité ; (4) `verbum-render` ne dépend ni du Shell ni d'aucune infrastructure de la Plateforme.

**A-3 — Emplacement des types et de l'acceptation (option (i) retenue par l'utilisateur).**
- Types (`Contribution`, accumulateur, `Frame`) dans un **nouveau crate `crates/render`, package `verbum-render`**, `#![no_std]`, sans allocation, **aucune dépendance normale**, dev-dépendances `verbum-ecs` et `verbum-schedule` ; façade `verbum` inchangée (comme A6 de l'étape 3). Ces types sont atteignables depuis `run_tick` (composition = système) : ils relèvent du noyau au sens d'ADR-000 pt 1.
- Acceptation sous boucle (RA-*) et exemple dans `crates/shell`. Justification : (a) **responsabilité de l'assemblage** : le Shell construit le noyau et est le seul lieu d'assemblage (ADR-S01 pt 7 ; ADR-000 pt 2) ; l'assemblage de test et l'exemple lui appartiennent ; (b) **direction des arêtes** : ADR-000 pt 3 interdit au noyau de nommer le Shell et l'encode dans le graphe de dépendances ; l'arête `shell → render` (dev) va de la Plateforme vers le noyau, comme les arêtes `shell → ecs` et `shell → schedule` déjà présentes depuis l'étape 3 ; l'arête inverse `render → shell` (dev) ferait nommer le Shell par un crate du noyau, même pour ses seuls tests ; (c) **instrument de contrôle** : le contrôle S5 de l'étape 3 (`cargo tree -i verbum-shell --workspace`) inclut les arêtes de dev par défaut ; sous (i) il reste lisible tel quel (aucun dépendant), sous (ii) il exigerait d'exclure les arêtes de dev (`-e normal`) et perdrait ce qu'il vérifie. Coût de (i) : `crates/shell/Cargo.toml` est modifié (dev-dépendances seulement) ; aucun fichier existant de l'étape 3 n'est touché.

**Décisions et confirmations (utilisateur, 27 octobre 2026)** : A-2 forme 2 (clos) ; A-1 option Y ; A-3 option (i) ; C3 (`produces` + contrôle de composition, unicité du composeur non garantie par les types) ; C4 (remise à l'état initial de l'accumulateur par un système `Clear` en phase 0 ; la réalisation « le composeur vide l'accumulateur », confirmée le 27 octobre, est irréalisable avec le Scheduler validé ; solution et interprétation de durée de vie limitée à l'étape 4 retenues le 28 octobre 2026, sans amendement d'ADR-R01) ; C9 (lot publié avant chaque tick, même sans lecteur) ; C12 (carré, pour exercer le parcours des contributions). C7 : réaction générale à la saturation laissée ouverte ; le document d'exécution explicite la borne du scénario accepté (C7).

## 6. Choix locaux proposés [L]

- `Contribution` : donnée `Copy` minimale (position entière, taille entière, couleur RGBA entière). Aucun flottant, aucune référence, aucun `EntityId` (E3). Champs exacts renvoyés au document d'exécution.
- Accumulateur : séquence à capacité fixe `CAP`, `push` rendant `Result<(), Full>`, lecture en tranche, remise à vide (par `Clear`) ; traits `Clone + PartialEq + Debug` (RP-3 : égalité avec l'état initial) ; **aucune clé, aucun accès par position publié comme identification** (E12).
- `Frame<CAP>` : séquence à capacité fixe, construite par `Frame::compose(&accumulateur)` (copie des contributions), état initial `Frame::empty()`, **aucune méthode `&mut self`**, `Clone + PartialEq + Eq + Debug`.
- Backend : **aucun trait public.** Toute fonction ou fermeture `FnMut(&Frame<CAP>)`, ou méthode d'un instrument de test, est un Backend aux yeux de l'assemblage (E6). Un trait `Backend` fermerait prématurément la forme de l'interface de franchissement (§11).
- Backend headless : instrument de test et d'exemple (côté Plateforme), non livré comme composant du crate. **L'enregistrement des Frames présentées se fait dans une capacité préallouée avant la boucle** (aucune allocation à la présentation : ADR-000 pt 1 soumet à l'invariant d'allocation, par contrat et revue, le code du hot path situé hors du noyau). `Frame` est une donnée plate (aucune référence, aucun handle, aucune propriété partagée) : ADR-000 pt 3.
- Ressources : `ResourceSlot<Accumulateur>` et `ResourceSlot<Frame>` dans l'ensemble statique de ressources de l'application, initialisés vides à l'assemblage (E13 ; `Read::get` rend `Option`, l'absence n'est donc **pas** interdite par les types : initialisation à l'assemblage, vérifiée par RN-5).
- Systèmes de test : `Clear` (produit l'accumulateur : remise à l'état initial), `ProduceA`, `ProduceB` (deux types distincts, contrat : production de l'accumulateur), `Compose` (**lecture** de l'accumulateur, production de la Frame), `Witness` (lecture seule de la Frame, copie dans une catégorie de test). Phases : `Clear` en phase 0 ; producteurs, `Compose` et `Witness` en phase 1 (dernière phase).
- Assemblage : fermeture d'infrastructure de `run_loop` : publier le lot → `run_tick` → lire la Frame → présenter ; première erreur rendue inchangée ; **la présentation n'a pas lieu si le tick échoue** (conséquence de « donnée déjà entière », ADR-R03 pt 1 [Déduit], vérifiée par RA-5).

## 7. Invariants, tests, API minimale

### 7.1 Représentation (U2) — `crates/render/tests/representation.rs` et doctests
| Test | Énoncé | Invariants | API rendue nécessaire |
|---|---|---|---|
| RP-1 | Jusqu'à `CAP`, `push` accepte ; la lecture rend les contributions dans l'ordre d'acceptation ; aucune clé n'identifie un élément | E8, E12 | `Contribution`, accumulateur : `new`, `push`, `as_slice`, `len`, `capacity` |
| RP-2 | Au-delà de `CAP`, `push` est refusé par `Err(Full)` ; contenu et ordre inchangés ; capacité inchangée | E14 | `Full` (`Debug`, `PartialEq`) |
| RP-3 | Après `clear`, le contenu est vide et l'accumulateur est réutilisable ; l'état initial et l'état vidé sont indiscernables | E13 | `clear` |
| RP-4 | `Frame::compose` conserve toutes les contributions, sans écart, sans arbitrage et **sans déduplication** : même longueur, mêmes valeurs, y compris pour des contributions identiques | E10 | `Frame::compose`, `as_slice`, `len` |
| RP-5 | Aucune modification d'une `Frame` à travers `&Frame` : doctests `compile_fail` **sans code d'erreur** (le code n'est pas contrôlé par rustdoc sur Rust stable) : (a) affectation à travers `&` ; (b) appel de `push` sur une `Frame` **possédée et mutable** : le doctest échoue à la compilation tant que l'API ne contient aucun mutateur, et c'est cette sensibilité (un `push(&mut self, …)` ajouté à `Frame` le fait compiler, donc échouer) qui est vérifiée ; (c) fermeture `FnMut(&Frame)` jouant le rôle d'un Backend qui tente de modifier la Frame ; l'API publique de `Frame` ne contient aucun `&mut self` (revue) | E1 | absence de mutateurs |
| RP-6 | `Frame::empty()` est l'état initial déterministe : vide, comparable, sans notion d'« absent » | E13 | `Frame::empty`, `PartialEq` |
| RP-7 | La Frame composée est une valeur indépendante de sa source : `clear` ou `push` ultérieurs sur l'accumulateur ne modifient pas la Frame | E1 | possession des copies (aucune référence dans `Frame`) |

### 7.2 Pipeline sous Scheduler (U3) — `crates/render/tests/pipeline.rs`
`World` et `Scheduler` réels ; `ProduceA`, `ProduceB`, `Compose`, `Witness`.
| Test | Énoncé | Invariants | API |
|---|---|---|---|
| RN-1 | Après un tick, la Frame lue dans le `World` contient exactement les contributions de A et de B, comparées comme **multiensemble** (tri côté test, **multiplicité conservée**, aucun doublon effacé), aucune absente, aucune ajoutée. Le scénario comprend au moins deux contributions **identiques** (par exemple une contribution de A égale à une de B) | E7, E9, E10 | — |
| RN-2 | **Contrôle de composition** sur les contrats assemblés (`SystemList::fill`, `DataSet::contains`, `contract_of`, comme AC-6) : (a) un seul système produit la Frame ; (b) aucun système n'écrit la Frame ou l'accumulateur autrement que par `produces` ; (c) **responsabilités de l'accumulateur** : `Compose` en est l'unique lecteur ; seuls `Clear`, `ProduceA` et `ProduceB` le produisent ; `Clear` est en phase 0, strictement avant la phase de `Compose` (donc avant les lectures de `ACC` déclarées par les systèmes du modèle ; une lecture directe de la ressource hors Scheduler n'est pas couverte, voir § 10) ; (d) `Compose` est dans la dernière phase, et aucun système n'est dans une phase postérieure ; (e) aucun système n'a de contrat sur le Backend (il n'en a pas de catégorie). Non imposé par les types (C3, C4) | E5, E7, E8, E9, E11, E13, E18, C3, C4, C5 | lecture des contrats existants |
| RN-3 | Dans l'ordre de la phase de `Compose`, déduit (`model.phase(p)`) : `ProduceA` et `ProduceB` précèdent chacun `Compose`, et `Compose` précède `Witness`, sans qu'aucun système ne déclare ces dépendances. **L'ordre relatif de `ProduceA` et `ProduceB` n'est pas affirmé** (ordre incident, non figé par un test) | E9 | — |
| RN-4 | `Witness` (lecture seule, après le composeur) copie la Frame ; la copie est égale à la Frame lue dans le `World` après le tick | E1 | — |
| RN-5 | Sur *N* ticks, la Frame de chaque tick compte exactement les contributions de ce tick (aucun cumul : `Clear` remet l'accumulateur à l'état initial avant toute lecture déclarée par un système) ; avant le premier tick, Frame et accumulateur sont vides | E13 | — |
| RN-6 | Deux exécutions indépendantes du même scénario donnent la même suite de Frames, ordre de séquence compris (reproductibilité du moteur, E19 ; aucun ordre particulier n'est affirmé, aucune sémantique n'est prêtée à l'ordre de déclaration) | E19 | — |

### 7.3 Acceptation sous la boucle du Shell (U4) — `crates/shell/tests/frame_loop.rs` (si A-3 (i))
Assemblage : `run_loop` + `Inbox` (étape 3), cadence = fermeture vide (aucune assertion temporelle ; `FixedStepPacing` réservé à l'exemple de U5), `World`, `Scheduler`, Backend headless d'instrument.
| Test | Énoncé | Invariants | API |
|---|---|---|---|
| RA-1 | Sur *N* itérations, le Backend a reçu *N* Frames, chacune égale à la Frame attendue ; journal d'événements de la Plateforme : fin de tick *k* < présentation *k* < début de tick *k+1* ; jamais de présentation pendant un tick | E5, E15, E16, E17 (compte *N* = propriété de l'assemblage de test) | — |
| RA-2 | **Aucun chemin de retour, structurel** : la fonction de présentation a pour seule entrée `&Frame` et ne rend rien ; compilation vérifiée par un test et par un doctest `compile_fail` (un Backend qui tenterait de modifier la Frame est rejeté) ; revue de l'assemblage : le Backend ne détient aucune référence au `World`, au buffer, à la boîte de réception | E2, E3, E4, E16 | signature de présentation |
| RA-3 | **Indépendance de la trajectoire** : deux Backends dont les comportements internes diffèrent (enregistreur / compteur sans copie) produisent la **même** suite d'états observables du `World` publiés pour l'aval (Frame, témoin ; l'accumulateur n'est pas relu après `Compose`, C4) et les mêmes Frames composées | E2, E6 | — |
| RA-4 | **Immuabilité à la remise** : copie du témoin (RN-4) = Frame lue dans le `World` après le tick = Frame reçue par le Backend = Frame relue dans le `World` après la présentation | E1 | — |
| RA-5 | Si `run_tick` échoue, aucune présentation n'a lieu pour ce tick ; l'erreur est rendue inchangée | E15, ADR-R03 pt 1 [Déduit] | — |

**Extension de RA-3 non retenue** — un Backend qui dépose un fait de retour dans la boîte de réception pendant `present` et dont l'effet n'apparaît qu'au lot suivant. Elle exprimerait la seconde phrase d'ADR-R03 pt 5, mais présenterait la boîte comme *le* mécanisme de retour, que ADR-R03 déclare non tranché (C11). À ajouter seulement sur décision de l'utilisateur.

### 7.4 Exemple (U5) — `crates/shell/examples/headless_frame.rs`
Assemblage complet, plusieurs ticks, Backend headless, aucune sortie, code 0 ; vérifie en interne les Frames présentées (artefact de la Roadmap : première frame produite et présentée). Seul point d'assemblage de l'étape 4 : exclusivité de l'assemblage (E15) vérifiée par revue.

### 7.5 Sensibilité des tests (mutations, sur copies isolées)
| Mutation | Détectée par |
|---|---|
| M-R1 : le composeur écarte la dernière contribution | RP-4, RN-1 |
| M-R2 : `Clear` ne remet pas l'accumulateur à l'état initial | RN-5 |
| M-R3 : `push` au-delà de `CAP` écrase ou ignore sans erreur | RP-2 |
| M-R4 : `Frame` partage ou référence l'accumulateur | RP-7 |
| M-R5 : un mutateur est ajouté à `Frame` et l'assemblage modifie la Frame entre le tick et la présentation | RA-4 (RP-5 pour le doctest) |
| M-R6 : présentation avant `run_tick` ou pendant le tick | RA-1 |
| M-R7 : `present` rend une valeur que l'assemblage écrit dans le `World` | RA-3 |
| M-R8 : un second système produit ou écrit la Frame | RN-2 |
| M-R9 : le composeur placé avant ses producteurs (autre phase) | RN-1, RN-2 (d) |
| M-R10 : présentation malgré l'échec du tick | RA-5 |
| M-R11 : le composeur déduplique les contributions identiques | RP-4, RN-1 (cas avec doublon) |
M-R7 et M-R5 modifient des signatures dans la copie : elles établissent la sensibilité des tests, non l'impossibilité d'un retour dans tout assemblage possible (revue).

## 8. Unités (sessions d'implémentation)

Méthode de l'étape 3 : rouge avant vert (squelettes `unimplemented!()`), une validation explicite par unité, quatre contrôles par unité, fichiers `.rs` livrés complets avec commentaire de chemin en première ligne, documentation en français, aucun commit par Claude, Miri et MSRV 1.85 exécutés par l'utilisateur.

| Unité | Contenu | Fichiers | Tests |
|---|---|---|---|
| **U1** | Crate `verbum-render` vide : `Cargo.toml` (héritage du workspace, aucune dépendance normale, dev-dépendances `verbum-ecs`, `verbum-schedule`), `lib.rs` `no_std`, membre du workspace | `crates/render/{Cargo.toml,src/lib.rs}` ; `Cargo.toml` racine (`members`) | aucun ; `cargo tree -i verbum-render --workspace` |
| **U2** | Représentation : `Contribution`, accumulateur, `Full`, `Frame` | `src/{contribution,accumulator,frame}.rs`, `tests/representation.rs` | RP-1 à RP-7 ; M-R3, M-R4 |
| **U3** | Pipeline sous Scheduler et contrôle de composition | `tests/pipeline.rs` | RN-1 à RN-6 ; M-R1, M-R2, M-R8, M-R9 |
| **U4** | Acceptation sous boucle, Backend headless d'instrument | `crates/shell/{Cargo.toml (dev-dépendances),tests/frame_loop.rs}` | RA-1 à RA-5 ; M-R5, M-R6, M-R7, M-R10 |
| **U5** | Exemple, contrôles de clôture, Miri, MSRV 1.85, rapport « non démontré » | `crates/shell/examples/headless_frame.rs` | exemple (code 0) |

Découpage en sessions proposé [L] : session A = U1 à U3 (types et pipeline, sans Shell) ; session B = U4 et U5 (assemblage). Le choix (une session ou deux) revient à l'utilisateur selon sa limite de contexte ; le document d'exécution le fixera.

## 9. Contrôles communs
Quatre contrôles à chaque unité : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`. Résultats réels de l'utilisateur uniquement ; contrôles non exécutés (Miri, MSRV 1.85, doctests `compile_fail` sous une autre chaîne) signalés comme tels. `tree.md` est régénéré par l'utilisateur. Les doctests `compile_fail` ne pinnent aucun code d'erreur (non contrôlé sur Rust stable) ; leur validité est confirmée par le jumeau qui compile et leur sensibilité par mutation. Les codes observés par compilation directe (E0594, E0599, E0594) sont un constat ponctuel de la session, non une assertion de test.

## 10. Ce qui reste non démontré à la clôture
- **Durée de vie logique de l'accumulateur** (C4) : interprétation propre à l'étape 4 (fin de vie à la consommation par `Compose`, remise à l'état initial par `Clear` avant les lectures déclarées par les systèmes) ; non imposée par les types ; contrôle de composition (RN-2 (c), (d)) et revue ; limites : le contrôle de composition porte sur les systèmes assemblés ; l'absence de lecture de l'accumulateur hors des systèmes (assemblage, fermeture d'infrastructure, instruments, observateurs) après `Compose` n'est établie ni par les types ni par les contrats du Scheduler : elle relève de la **revue de l'assemblage de référence** (aucune relecture de `ACC` par l'assemblage, ses instruments ou ses observateurs) ; aucun amendement d'ADR-R01, aucune sémantique générale des données transitoires prétendue.
- **Unicité du composeur et lecture seule des autres systèmes** : contrôle de composition (RN-2) et revue ; le modèle n'impose pas cette unicité entre phases (C3).
- **Absence d'état « absent » de la Frame et de l'accumulateur** : initialisation à l'assemblage et RN-5 ; non garantie par les types (`Read::get` rend `Option`).
- **Le Backend n'a aucun accès au `World`** : garantie structurelle de la signature de présentation ; l'absence d'un autre chemin dans un assemblage quelconque relève de la revue de `examples/headless_frame.rs` (E15).
- **Interdit « le Backend ne nomme ni entité, ni composant, ni système »** (ADR-000 pt 3 : encodé dans le graphe de dépendances pour que la violation ne compile pas) : satisfait pour les types de `verbum-render` (aucune dépendance), **non imposé par le graphe pour le Backend headless**, instrument situé dans un crate qui a `verbum-ecs` et `verbum-schedule` en dev-dépendances ; revue seulement pour cet instrument.
- **Allocation sur le hot path hors noyau** (ADR-000 pt 1 : par contrat et revue) : la lecture de la Frame et la présentation sont hors noyau mais sur le chemin de chaque itération ; l'instrument headless n'alloue pas à la présentation (§6) ; revue seulement.
- **Absence d'allocation sur le chemin d'une itération** : revue seulement.
- **Réaction d'un système à la saturation de l'accumulateur** (C7).
- **Règle d'ordre de la frame** (C8) ; clés de tri, regroupement, batching.
- **Fermeture générale de l'interface de franchissement** (C13) : seul le choix de l'étape 4 (forme 2) est arrêté ; formes 1 et 3, Backend réel, mécanisme de retour restent ouverts.
- Fenêtre, GPU, bibliothèque graphique ; présentation d'un Backend réel ; erreurs de présentation (C10) ; mécanisme de retour vers l'Acquisition (C11) ; politique temporelle (C6).
- Équivalence « fin de phase d'Extraction » ≡ « fin de tick » (C5) : contrôlée par la position du composeur, non par un type de phase.

## 11. Risques de fermeture prématurée
1. **Trait `Backend` public** : il figerait la forme de l'interface de franchissement (ADR-000 pt 3). Évité : fonction ou fermeture.
2. **`Frame` présentée comme la décision d'ADR-R02 sur la forme de la représentation** : elle est un minimum de preuve ; forme mémoire, tri, regroupement restent hors périmètre.
3. **Règle d'ordre implicite** : la séquence suit l'ordre d'émission ; un test ou un Backend qui s'y fie crée la règle (C8). Les tests comparent des ensembles.
4. **Forme multi-couches présumée** : aucun champ `layers`, aucune couche, aucun vocabulaire de ce cas ; une contribution reste une donnée plate. Vérification : §11.1.
5. **Clear-color ou attribut de frame** : écarté (C12) ; l'ajouter crée un élément de représentation qui n'est pas une contribution.
6. **Une présentation par tick devenue invariant** : C6 ; seule l'assemblage de test le compte.
7. **Choix de l'étape 4 présenté comme fermeture générale de l'interface de franchissement** : A-2 est un choix d'implémentation de l'étape 4, non une décision sur toutes les formes futures ni une modification d'ADR-000.
8. **Saturation fermée par commodité** (réutiliser `BufferFull`, perdre ou paniquer) : C7 non fermée.
9. **Observation de la présentation par un canal du `World`** : serait un canal de retour ; l'observateur reste côté Plateforme.
10. **Test de retour par la boîte de réception** présenté comme le mécanisme de retour (C11) : non retenu.
11. **Garantie d'immuabilité présentée comme imposée par les types dans tous les cas** : les types interdisent la modification de la valeur ; l'unicité du producteur reste un contrôle de composition.
12. **Déduplication ou normalisation à la comparaison** : comparer des ensembles qui effacent les doublons masquerait un composeur qui écarte une contribution identique à une autre (E10) ; les comparaisons conservent la multiplicité (RN-1, RP-4, M-R11).

### 11.1 Vérification de non-fermeture (handoff de l'étape 2, §4 et §5) [Lu]
Exigence du handoff : la représentation de frame doit pouvoir exprimer plusieurs éléments visuels ordonnés pour un même ensemble logique, sans qu'on la transforme en exigence de tri ou de modèle multi-couches à l'étape 4 ; ne pas présumer `FrameResource { layers: Vec<Layer> }`. Confrontation des familles du handoff (§4.2) au minimum de ce plan (une `Frame` = séquence à capacité fixe de contributions plates, composeur unique, aucun ordre contractuel) :
| Famille (§4.2) | Effet sur ce plan |
|---|---|
| 1. un élément = une image | non présumée : la contribution est un carré, pas une image ; une séquence admet N éléments par ensemble logique |
| 2. plusieurs éléments indépendants, ordre = clé explicite par élément | additif : un champ de clé dans `Contribution` et un tri dans le composeur ; `Frame` (séquence) et Backend inchangés |
| 3. tableau inline par élément | modifie `Contribution` seulement ; `Frame` reste une séquence |
| 4. plage dans un tampon plat | modifierait `Frame` et le Backend : non exclue, coût de migration plus élevé |
| 5. `Vec<Layer>` | écartée par ADR-000 (noyau `no_std` sans `alloc`) et par les critères §4.3 du handoff |
Critères §4.3 : immuabilité (types sans mutateur, indépendante de la forme), capacité fixée (`CAP`), absence d'allocation, Backend passif sans retour : satisfaits par le minimum ; **déterminisme de l'ordre (relation totale et stable)** : volontairement non traité (C8, Q8). Aucune contradiction concrète n'est apparue ; aucune exigence de tri ou de couche n'est introduite. L'arbitrage A-1 (Y) tient lieu de la décision demandée par le handoff §9.5 : pas de test à ordre affirmé à l'étape 4.

## 12. Hors périmètre
Caméra, animation, sprite réel, texture, atlas, asset (étape 5), fenêtre, GPU, bibliothèque de rendu, batching, instancing, tri par état GPU, règle d'ordre ou de calque, clear-color, plusieurs frames par itération, interpolation, VSync, politique temporelle (ADR-S04), mécanisme de retour vers l'Acquisition, rédaction d'ADR, modification de la Roadmap, des ADR, de `verbum-ecs`, `verbum-schedule`, `verbum-forge`, `verbum-assets`, de la façade `verbum`, des documents des étapes 0 à 3 ; toute implémentation de `World::execute()`, `Scheduler::run()` ou `renderer.compose()`.

## 13. Fichiers
**Créés** : `crates/render/{Cargo.toml, src/lib.rs, src/contribution.rs, src/accumulator.rs, src/frame.rs, tests/representation.rs, tests/pipeline.rs}` ; `crates/shell/tests/frame_loop.rs`, `crates/shell/examples/headless_frame.rs` (sous A-3 (i)).
**Modifiés** : `Cargo.toml` racine (`members` : ajout de `crates/render` ; `[workspace.dependencies]` : ajout de `verbum-render = { path = "crates/render", version = "0.1.0" }`, [L] par cohérence avec les entrées existantes `verbum-ecs` et `verbum-schedule`, que `crates/shell/Cargo.toml` consomme par `.workspace = true` ; l'alternative est une dépendance par `path` dans `crates/shell/Cargo.toml`, à trancher dans le document d'exécution) ; `crates/shell/Cargo.toml` (`[dev-dependencies]` seulement) ; `Cargo.lock` (par Cargo).
**Interdits** : `crates/{ecs,schedule,assets,forge,verbum}/**`, tous les fichiers existants de `crates/shell/src/**` et `tests/**` de l'étape 3, ADR, `ROADMAP.md`, `docs/implementation/00` à `03`, `docs/handoffs/**`.

## 14. Critères d'acceptation (clôture)
1. U1 à U5 validées une à une par l'utilisateur, quatre contrôles verts à chaque unité sur le dépôt réel.
2. RP-1 à RP-7, RN-1 à RN-6, RA-1 à RA-5 verts ; chaque mutation M-R1 à M-R11 détectée par le test indiqué, dans une copie isolée.
3. `cargo run -p verbum-shell --example headless_frame` : code 0, aucune sortie.
4. `cargo tree` : `verbum-render` sans dépendance normale ; aucun crate du noyau ne dépend du Shell ; `verbum-ecs`, `verbum-schedule`, façade inchangés.
5. Miri et MSRV 1.85 exécutés par l'utilisateur et verts, ou signalés non exécutés.
6. Le rapport de clôture reproduit §10 sans l'atténuer ; aucun invariant, ADR, Roadmap ni document antérieur n'a été modifié (principe directeur de la Roadmap).
7. Les arbitrages A-1 à A-3 et les confirmations de §5 sont consignés comme clos par l'utilisateur.

## 15. Fin de l'étape
L'utilisateur déclare l'étape close. Claude ne la déclare pas. Aucun travail de l'étape 5 n'est entrepris.
