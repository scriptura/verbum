# Plan de construction — étape 5 : Assets réels — r1.3, 30 octobre 2026

## Statut
Document de planification interne, non normatif, révisable sans ADR (même statut que `ROADMAP.md` et les plans des étapes 2 à 4). Il détaille l'étape 5 en unités. Il n'introduit aucun invariant : chaque exigence cite sa source, et tout ce qui ne découle pas du corpus est signalé comme tel. **Aucun code, aucune modification d'ADR, aucune modification d'un document validé.** (r1.3 : seule exception de périmètre, `crates/schedule/src/system.rs`, voir §13.)

**État : validé (r1.2) ; révision r1.3 soumise à validation.**

**Révision r1.3 — passages modifiés, et seulement ceux-ci (décision de l'utilisateur du 30 octobre 2026, après l'échec constaté de G-LT sur le dépôt réel) :** (1) en-tête et état ; (2) §0, « Non lus » : la faisabilité de la vue empruntée avec le Scheduler était **[Non vérifié]** et s'est révélée fausse tel quel (E0478, E0491) ; (3) §5, A-2 option a, réserve ; (4) §7.2, preuve G-LT ; (5) §8, ligne U1, porte d'arrêt ; (6) §11 risque 2 ; (7) §13, **exception explicite et strictement limitée à `crates/schedule/src/system.rs`** (trois bornes de durée de vie, voir ci-dessous) ; (8) §14, critère 6 ; (9) §15. Layouts, unités, arbitrages A-1 à A-6, autres résultats attendus : inchangés ; aucun ADR modifié.

*Nature du correctif de `system.rs`* : `Binding::Parts<'w>` reçoit `where Self: 'w` ; `System::Access<'w>` reçoit `where B: 'w` ; `contract_of` devient `contract_of<'a, B: Binding + 'a, S: System<B>>()` et nomme `S::Access<'a>` au lieu de `S::Access<'static>`. Ces bornes ne changent ni les contrats Read/Write/Produce, ni les dépendances, ni l'ordre d'exécution ; les impls existantes (sans paramètre de durée de vie, donc `'static`) les satisfont sans modification. Aucune modification de `crates/ecs`, de `World`, de `ResourceSlot`, ni des autres fichiers de `crates/schedule`. Ce correctif **ne valide pas U1** : G-LT est réexaminé conjointement après livraison.

**Révision r1.2 — passages modifiés, et seulement ceux-ci (décisions de l'utilisateur du 29 octobre 2026 après audits de cohérence) :** (1) en-tête et état ; (2) §5, A-3 option i : précision du dispatch par identité (le format est identifié par l'identité lorsque le préfixe de 10 octets est complet et l'identité connue, sinon le format attendu du slot est retenu, ce qui reproduit le chemin d'erreur de l'étape 2 ; un seul décodeur ; contrôle de niveau 2 : identité connue différente du format attendu → `SlotFormat`) ; (3) §7.2, « charge variable » : « charge de longueur zéro conforme » reformulé en charge **contenant zéro élément variable**, la charge utile de longueur nulle restant invalide ; (4) §8 (ligne U1) et §13 : précision sur la factorisation du préfixe commun (fonction interne partagée ; adaptation interne de `Sprite::read` ; API et assertions de non-régression inchangées) ; (5) §15. Layouts, résultats attendus des tests, unités, arbitrages : inchangés.

**Révision r1.1 — passages modifiés, et seulement ceux-ci :** (1) en-tête et cet état ; (2) §1.3, définition de G-CM réécrite (distinction validations internes aux niveaux / modification de mécanisme) ; (3) §5, ajout de l'en-tête « Arbitrages retenus » ; (4) A-4, R1 : reformulation de la conséquence sur le niveau 3 ; (5) A-3 : conséquence sur le niveau 2 reformulée ; (6) §7.2, test de fermeture des références précisé ; (7) §8, portes U1, U3 et U5 ; (8) §11 risque 1 ; (9) §15, décisions restantes. La porte nommée `C-M` en r1 est renommée **G-CM** dans tout le document (aucun changement de portée). Périmètre, unités, choix locaux, critères d'acceptation : inchangés.

Légende : **[Lu]** texte vérifié dans un document lu dans cette session · **[Imposé]** disposition normative citée · **[Déduit]** conséquence que je tire de textes lus · **[L]** choix local, non invariant, révisable · **[C]** ambiguïté, contradiction ou lacune réelle · **[A]** arbitrage demandé à l'utilisateur · **[Non vérifié]** non lu ou non exécuté.

## 0. Sources et limites

**[Lu] normatif** : `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, ADR-000, ADR-004, ADR-005, ADR-A01, ADR-R01, ADR-R02, ADR-R03, `README.md` d'architecture.
**[Lu] non normatif** : `ROADMAP.md`, `02-verbum-forge.md` (r8), `02-verbum-forge-execution.md`, `HANDOFF-stage-2-forge.md` (§3 à §5, §8), plan, exécution et handoff de l'étape 4 (r3.2), `tree.md` régénéré le 29 octobre 2026 (12:13) complété de la partie `crates/assets`.
**[Lu] code réel** : `crates/assets/{Cargo.toml, src/lib.rs, src/load.rs, src/manifest.rs, src/sprite.rs}` ; `crates/forge/{Cargo.toml, src/lib.rs, src/id.rs, src/sprite.rs, src/batch.rs, tests/pipeline.rs}` ; `crates/ecs/src/resource.rs` (`ResourceSlot<T>` : aucune contrainte sur `T`) ; en-têtes de `world.rs` (`World<_, _, R>`, `R: Default`) ; code de l'étape 4 (livré par cette session).

**Non lus, donc non vérifiés** :
- **Le `Cargo.toml` racine courant.** La copie reçue liste `ecs, schedule, verbum, assets, forge, shell` en `members` et `ecs, schedule` en `[workspace.dependencies]` : elle ne contient pas `crates/render`, alors que `tree.md` le contient. Elle est donc **antérieure à l'étape 4** ou incomplète. À fournir avant le document d'exécution.
- ADR-001, ADR-002, ADR-006 (relus seulement par les citations des plans 02 et 04). **r1.3** : ADR-003, ADR-004, ADR-005, `01-verbum-scheduler.md` (r5) et `01-verbum-scheduler-sessions.md` relus : aucune contradiction avec le correctif de `system.rs` (la représentation Rust y est hors périmètre ; la faisabilité du §10 de `01-verbum-scheduler.md` avait été vérifiée sur des substituts, sans Ressource empruntée ni vrai `World`).
- **r1.3 — constat** : la réserve ci-dessous (effet sur `World`, Scheduler, contrats) a été levée par exécution : G-LT a **échoué** sur le dépôt réel avec les bornes d'origine, puis est vérifiable avec les trois bornes du §13.
- Les contraintes exactes que `World`, `Scheduler` et les contrats posent sur un type de Ressource (en particulier : un paramètre de durée de vie dans `R`, voir A-2 option a) — **[Non vérifié]**.
- L'usage effectif de `verbum-assets` / `verbum-forge` par `crates/shell` : `tree.md` n'en montre pas ; à confirmer par `Cargo.toml` de `crates/shell`.

## 1. Objectif et critères de sortie

### 1.1 Contrat de la Roadmap [Lu]
« **Contenu** : sprites, tilesets, maps, fonts — extension de la Forge et du chargement runtime établis en étape 2, sans changement de mécanisme. **Dépendance** : étapes 2 et 4. **Test d'invariant** : aucun nouveau — chaque nouveau type d'asset doit passer le même test qu'en étape 2, sans modification du pipeline Forge → Ressource. **Artefact produit** : premier niveau chargé et affiché. » Règle de jalon : validé seulement si aucun invariant architectural n'a dû être modifié.

### 1.2 Périmètre retenu [Déduit]
Trois nouveaux types (tileset, map, font) s'ajoutent au sprite existant. Les types sont introduits par tension croissante avec le mécanisme de l'étape 2 (charge utile fixe → charge utile variable → référence entre assets), pas par ordre de la Roadmap. Le « premier niveau chargé et affiché » est un assemblage de ces types, mesuré par le test d'invariant de l'étape 4 (circulation `Systems → Frame → Backend`), sans nouveau mécanisme.

### 1.3 Ce que « sans changement de mécanisme » désigne [Déduit, à valider]
Mécanisme (invariant de l'étape) : la chaîne de l'étape 2 — source humaine lue par la Forge seule (I1) → artefact binaire précompilé, layout explicite (I3) → manifeste `VMAN` produit par la Forge (I12) → chargement tout-ou-rien en niveaux 1 à 4 sans allocation (I5, I10) → dépôt dans une Ressource ordinaire à clé statique (I6, I7) → `AssetId` opaque résolu avant le hot path.
Variable par type d'asset (extension admise) : format de la source, layout de l'artefact, lecteur, type de la Ressource, compilateur Forge.
**Définition de la porte G-CM (r1.1).**
1. **Admis sans G-CM** : les validations propres aux nouveaux types, ajoutées **à l'intérieur des niveaux de contrôle existants** (1 à 4), sans créer de nouveau niveau, sans modifier leur ordre, l'atomicité du chargement, le layout `VMAN`, l'absence d'allocation ni le modèle d'identité. Relèvent de cette catégorie : le contrôle du format du slot au niveau 2 (A-3), la fermeture et le type des références au niveau 3 (A-4 R1), les contrôles de longueur propres à chaque layout au niveau 3.
2. **Soumis à G-CM, arrêt et rapport** : toute modification de l'un de ces éléments — création ou suppression d'un niveau, changement de leur ordre, perte du tout-ou-rien (un dépôt avant la fin du niveau 3), modification du layout `VMAN`, introduction d'une allocation dans le chargement, modification du modèle d'identité (`AssetId` opaque, `ArtifactIndex` et `SlotIndex` distincts, aucune chaîne ni chemin).
Cette clarification ne préjuge ni du résultat de la preuve G-LT, ni de celui des tests de référence : elle fixe seulement ce qui compte comme changement de mécanisme.

## 2. Exigences documentées et source

| Id | Exigence | Source | Statut |
|---|---|---|---|
| X1 | Les invariants I1 à I13 du plan 02 s'appliquent à chaque type d'asset sans exception | Roadmap étape 5 ; plan 02 §3 | Imposé |
| X2 | Seule la Forge lit un format humain ; `verbum-assets` n'a aucune dépendance (arbre vide) | I1, I9 ; `crates/assets/Cargo.toml` | Lu |
| X3 | Aucune chaîne ni chemin ne franchit la frontière de compilation | I4 ; ADR-A01 pt 3 | Imposé |
| X4 | Un asset devient une Ressource ordinaire, cardinalité 1, clé statique distincte de l'`AssetId` | I6, I7 ; ADR-005 | Imposé |
| X5 | Chargement tout-ou-rien, sans allocation, double décodage (niveaux 1 à 4) | I5, I10 ; plan 02 §5.4 ; `load.rs` | Lu |
| X6 | Capacité fixée avant simulation ; dépassement = erreur de configuration | I13 ; ADR-001 pt 5 (cité) | Imposé |
| X7 | Aucune validation de cohérence métier au runtime : la Forge valide ; le runtime vérifie la conformité binaire et la cohérence des liaisons | I2 ; plan 02 §5.3 | Imposé |
| X8 | `Contribution` ne porte ni entité, ni composant, ni ressource ; le Backend ne reçoit que la `Frame` | E3 (plan 04) ; ADR-R03 pt 1 | Imposé |
| X9 | La `Frame` est immuable une fois composée ; rien ne remonte du Backend | ADR-R02 pt 4 ; ADR-R03 pt 5 | Imposé |
| X10 | Aucun vocabulaire ni règle propre à un jeu ou à un jeu d'assets (LPC) dans le noyau ni dans les tests génériques | Handoff étape 2 §4.5 ; consigne de l'utilisateur | Imposé |
| X11 | Un concept n'est codé que s'il est porté par un invariant ou rendu nécessaire par un test | I11 ; règle gouvernante | Imposé |

## 3. Ambiguïtés, contradictions et lacunes

- **[C1] « Réel » et « affiché » ne sont pas définis.** Le `Sprite` de l'étape 2 est un descripteur `{width, height}` sans pixels ; la `Contribution` de l'étape 4 est un rectangle coloré ; le Backend est headless (Roadmap étape 4 : « headless préférable »). Aucun document lu ne dit par quel chemin un contenu raster atteindrait un Backend, ni si l'étape 5 l'exige. → A-1.
- **[C2] Le mécanisme de l'étape 2 est prouvé pour une charge utile fixe de 4 octets** (`PAYLOAD_LEN`, `Sprite` `Copy`, dépôt par valeur). Une map et une font sont, par nature, de longueur variable. L'absence d'allocation (I10) et le dépôt par valeur n'ont jamais été éprouvés sur ce cas. → A-2.
- **[C3] `load` est typé `Sprite`** (`deposit: impl FnMut(SlotIndex, Sprite)`) ; le manifeste ne porte aucun type (`VMAN` : id, artifact_index, slot_index) ; `AssetSource` ne porte aucun type. Avec plusieurs types apparaît une **classe d'erreur nouvelle : liaison d'un artefact à un slot d'un autre type**, inexistante à l'étape 2 (un seul type). → A-3.
- **[C4] Une map référence un tileset.** Les tilesets et les maps sont nommés ensemble par la Roadmap : la référence entre assets est **inhérente** au périmètre, et non une anticipation. Le handoff de l'étape 2 (§5, ligne étape 5) la désigne comme le point de tension avec « sans changement de mécanisme ». Aucun document ne dit comment une référence franchit la frontière de compilation sans chaîne ni chemin (I4). → A-4.
- **[C5] Ordre d'empilement (vigilance, Q8).** Voir §4.

## 4. Point de vigilance : composition visuelle et couches

### 4.1 Faits établis [Lu]
- `Frame<CAP>` = séquence à capacité fixe de `Contribution` plates (`x, y, width, height, color`), `Copy`, sans clé, sans lien vers un ensemble logique (E3). Égalité de `Frame` : **multi-ensemble** (A-1 du plan 04 = Y). L'ordre de la séquence = ordre d'émission, **déterministe mais non contractuel** (C8). Aucun test ni Backend ne doit s'y fier.
- ADR-R02 : les producteurs sont indépendants et ne connaissent ni le tri, ni le regroupement, ni le backend ; un seul composeur consomme l'accumulateur ; tri et clés sont renvoyés « à documenter séparément, si un besoin concret l'exige ». ADR-004 : un système n'a pas d'ordre intrinsèque.
- Plan 04 §11.1 a confronté la `Frame` aux cinq familles du handoff étape 2 §4.2 : aucune contradiction ; la famille 2 (clé explicite par contribution + tri dans le composeur) est **additive**.

### 4.2 Réponses [Déduit]
1. **Le modèle actuel peut-il représenter une apparence composée de N contributions visuelles ?** Oui. Rien dans `Frame`, `Contribution` ou le pipeline n'impose « une image par ensemble logique » : un ou plusieurs producteurs peuvent émettre N contributions pour un même ensemble. Le regroupement logique n'est pas représenté (volontairement, E3).
2. **Avec un ordre déterministe lorsque nécessaire ?** **Pas comme contrat.** L'ordre d'émission est reproductible de fait (accumulateur en ajout), mais (i) entre systèmes il dépend de l'ordonnancement, que ADR-004 déclare sans valeur intrinsèque ; (ii) il est explicitement non contractuel (C8) ; (iii) le Backend ne doit pas s'y fier. Un besoin d'ordre ne peut donc pas être **énoncé** dans le modèle actuel.
3. **Est-ce une incompatibilité avec l'étape 4 ?** **Non, démontré par examen** : aucune disposition d'ADR ni aucun invariant de l'étape 4 n'est contredit ; la règle d'ordre a été volontairement non posée, conformément à ADR-R02. C'est **une décision absente**, pas un défaut. Le corpus ne la rend pas nécessaire pour les critères de l'étape 4.
4. **Décision précise qui manque** (trois éléments liés) :
   - **D-ord-1** : l'existence d'une relation d'ordre de composition portée comme **donnée** de la contribution (clé), et non déduite de l'ordre d'émission.
   - **D-ord-2** : la règle de départage des clés égales (relation totale et stable, critère §4.3 du handoff) ; sans elle, le tri n'est pas déterministe.
   - **D-ord-3** : qui possède la valeur de la clé (Q8) : une donnée ECS lue par le producteur ; un ordinal intrinsèque à l'asset ; ou une règle du composeur. Le tri lui-même est du ressort du composeur (ADR-R02 pt 5), pas du producteur ni du Backend.

### 4.3 Conséquences par couche [Déduit]
| Couche | Effet |
|---|---|
| Modèle d'assets (étape 5) | Le format d'artefact (identité, version, longueur de charge utile, charge utile) n'impose pas « une image par ensemble ». L'option A-4 (référence = identifiant opaque résolu au chargement) se généralise à N références sans changement. Un ordinal intrinsèque à un asset serait une donnée de plus dans un layout, sans sémantique côté moteur. |
| Pipeline de production (Forge) | La Forge valide existence et type des références (I2). Elle n'attribue aucune sémantique d'ordre. |
| Renderer existant (étape 4, clos) | Familles du handoff : **2 (clé + tri)** = additive : un champ dans `Contribution`, un tri dans le composeur, `Frame` et Backend inchangés en forme ; elle impose en revanche de **redéfinir l'égalité de `Frame`** (multi-ensemble → séquence, ou clé incluse) et d'ajouter des tests RN/RP. **3** modifie `Contribution` seulement. **4** modifierait `Frame` et le Backend. **5** reste écartée par ADR-000. |

### 4.4 Où le besoin apparaît, ou non, à l'étape 5 [Déduit]
Un niveau typique superpose une couche de tuiles et des sprites : si leurs contributions **se recouvrent** et que le résultat dépende de l'ordre, le besoin est réel. Mais le Backend de l'étape 4 est headless et compare des multi-ensembles : l'étape 5 **ne peut pas démontrer** ce besoin, seulement le rencontrer dans la conception de la scène d'acceptation. Introduire la clé sans test qui l'exige violerait X11.

### 4.5 Arbitrage proposé (A-5)
- **O0 (recommandée)** : aucune modification de l'étape 4. La scène d'acceptation de l'étape 5 est conçue de sorte que son verdict ne dépende d'aucun ordre (contributions disjointes, ou assertions sur multi-ensemble). La décision D-ord-1 à D-ord-3 est consignée comme **point ouvert avant tout Backend graphique** (§10).
- **O1** : ouvrir un correctif additif ciblé de l'étape 4 (famille 2) maintenant. Requiert votre autorisation explicite et un plan de correctif séparé ; coût : crate clos, redéfinition de l'égalité de `Frame`, tests RN/RP, nouveau jalon de clôture.
- **Porte G-ORD** (dans O0) : si, en cours d'étape 5, une exigence de la scène d'acceptation ne peut être énoncée sans ordre, **arrêt et rapport** avec les conséquences et O1 comme option minimale. Aucune modification d'office.

### 4.6 Vérification de non-fermeture des choix de l'étape 5 [Déduit]
Chaque arbitrage A-1 à A-4 est confronté à la capacité future de composer N éléments ordonnés par ensemble logique : aucun des choix recommandés ne fige l'hypothèse « une image par ensemble » (la référence est à cardinalité N par construction ; la vue empruntée admet des listes ; aucun type n'est ajouté à `Contribution`). **Vocabulaire** : seuls les termes génériques « ordre d'empilement », « clé », « ensemble logique » sont employés ; aucune fixture ni aucun test ne portera de nom de couche, de partie ou de convention de jeu (X10).

## 5. Arbitrages demandés

**Arbitrages retenus par l'utilisateur (r1.1)** : A-1 option 1 ; A-2 option a, avec G-LT comme porte d'arrêt réelle en U1 ; A-3 option i ; A-4 option R1 ; A-5 O0 avec G-ORD ; A-6 font incluse. Les options non retenues restent consignées ci-dessous comme historique de la décision. Précisions de l'utilisateur : (A-1) le critère « niveau chargé et affiché » désigne la circulation des contributions jusqu'au Backend headless, non un rendu à l'écran, raster et Backend graphique hors périmètre ; (A-2) si la durée de vie ne peut pas être représentée avec le `World` et le Scheduler existants, **arrêt sans contournement** ; (A-4) validation de la fermeture et du type du référent **avant tout dépôt** ; (A-5) aucune modification de l'étape 4, aucune scène de l'étape 5 ne dépend de l'ordre d'émission, le besoin d'ordre d'empilement reste ouvert avant tout Backend graphique ou toute scène qui en dépend.

Pour chaque point : options, conséquences, recommandation. Une recommandation n'est pas une décision.

### A-1 [A] — Sens de « affiché » et présence de données raster *(bloquant : à trancher avant le document d'exécution)*
- **Option 1 (recommandée)** — *Assets structurels, affichage headless.* Les artefacts portent la structure (dimensions, grille de tuiles, indices, métriques de glyphes), **sans pixels**. Le « niveau affiché » = la `Frame` produite par des systèmes lisant les Ressources chargées, observée par le Backend headless. `Contribution` et Backend inchangés. Conséquence : « affiché » vaut « circule jusqu'au Backend sous forme de contributions », exactement le critère de l'étape 4 ; rien n'est visuellement montré. Les formats de source restent des TOML de structure.
- **Option 2** — *Artefacts avec pixels, affichage headless.* Charge utile raster chargée en Ressource, mais aucun consommateur : variable, volumineuse, sans test qui l'exige (X11). Non recommandée.
- **Option 3** — *Référence d'image dans `Contribution` + chemin d'upload vers un Backend graphique.* Conflit potentiel avec E3 et ADR-R03 pt 1 (« ne reçoit que la représentation de frame ») ; ajoute un Backend graphique absent de la Roadmap actuelle. Hors périmètre sans décision d'architecture préalable.
Si vous retenez l'option 1, le terme « assets réels » est interprété comme « types d'assets réels (tileset, map, font) avec leur structure réelle », ce que ce plan consigne comme limite (§10).

### A-2 [A] — Charge utile de longueur variable, sans allocation
- **a (recommandée, sous réserve [Non vérifié])** — *Vue empruntée validée.* Les octets de l'artefact restent possédés par l'infrastructure de chargement (Shell) ; la Ressource porte une vue `{champs d'en-tête décodés, &'a [u8] de charge utile}` dont la longueur est validée au niveau 3 ; les accès décodent à la demande (`from_le_bytes`, comme `Sprite::read`). Aucun `unsafe`, aucun cast, aucune allocation ; le double décodage reste valable (construction pure). **Réserve** : le type de Ressource porte un paramètre de durée de vie ; `ResourceSlot<T>` ne contraint pas `T`. **r1.3** : l'effet sur le Scheduler a été **constaté** : sans bornes `Self: 'w` / `B: 'w` sur `Binding::Parts<'w>` et `System::Access<'w>`, un `Binding` générique sur `'a` est refusé (E0478, E0491). Le correctif est limité à `crates/schedule/src/system.rs` (§13) ; `World`, `ResourceSlot` et `crates/ecs` sont inchangés. La preuve exécutable reste la porte G-LT.
- **b** — *Types inline à capacité const-générique* (grille `W×H` maximale). Dépôt par valeur, aucun emprunt ; mais capacité maximale gaspillée, taille de type dépendante du niveau, dépassement = erreur de configuration (I13) à définir par type.
- **c** — *Arène à capacité fixe fournie par l'application ; la Ressource porte une plage `(offset, longueur)`.* Même philosophie que la famille 4 du handoff ; ajoute un paramètre à `load` et une Ressource d'arène : plus proche d'un changement de mécanisme.
- **d** — *Allocation dans l'infrastructure de chargement (`std`, hors noyau).* Permise par ADR-000 pt 5 pour la Plateforme, mais **renverse la décision r7 de l'étape 2** (chargement sans allocation, I10) : changement de mécanisme, **écartée** sauf décision explicite.

### A-3 [A] — Généralisation de `load` et typage des liaisons
- **Option i (recommandée)** — *Manifeste unique, dispatch par identité d'artefact, table de formats par slot.* Les artefacts partagent déjà le préfixe `identité (4) + version (2) + longueur de charge utile (4)` [Lu, `sprite.rs`, choix local], **factorisé** en une fonction interne partagée par tous les lecteurs (U1). `load` **dispatche sur l'identité** : le format d'un artefact est celui que désigne son identité lorsque son préfixe de 10 octets est complet et son identité connue ; sinon le format attendu par la table pour son slot est retenu, et son lecteur rend alors l'erreur de l'étape 2 (`Truncated`, `FormatIdentity`). **Un seul décodeur** (format, octets) sert aux niveaux 3 et 4 ; il dépose une valeur d'une énumération fermée de types décodés. L'application fournit, **à la place de `slot_count`**, la table des formats attendus par slot (sa longueur = la capacité). Nouvelle classe d'erreur « format du slot », validation ajoutée **à l'intérieur du niveau 2** existant (liaisons ; §1.3, point 1) : une identité **connue** différente du format attendu du slot produit `SlotFormat` ; une identité inconnue ou un préfixe incomplet suit le chemin d'erreur de l'étape 2. `VMAN` inchangé (aucun tag de type : l'identité de l'artefact fait foi). Conséquence : signature de `load` modifiée (appelants : tests de l'étape 2, à confirmer) ; les niveaux 1 à 4, leur ordre, le tout-ou-rien et l'absence d'allocation inchangés.
- **Option ii** — *Un manifeste et un `load` par type.* Aucun changement de signature de principe ; l'inadéquation de type est déjà attrapée par `ReadError::FormatIdentity`. Mais l'espace d'`AssetId` et de slots est partitionné, ce qui rend la référence inter-assets (A-4) plus lourde et heurte I12 (« ensemble des assets connu de la Forge », manifeste unique par ensemble).
- **Option iii** — *Ajouter un tag de type dans `VMAN`.* Modifie le layout validé de l'étape 2 : changement de mécanisme, **écartée**.
Côté Forge : `AssetSource` doit connaître le type de chaque source — champ explicite de `AssetSource` ou discriminant dans le TOML : **[L]**, côté Forge uniquement, sans effet sur le runtime.

### A-4 [A] — Références entre assets (map → tileset)
- **Option R1 (recommandée, couplée à A-3 i)** — *La référence est un `AssetId` dans l'artefact, résolu au chargement en `SlotIndex`.* La Forge vérifie existence et type du référent (I2) ; au niveau 3, la fermeture de référence est vérifiée (`Manifest::find` puis format du slot référent = format attendu, grâce à la table de A-3) ; la valeur décodée porte un `SlotIndex`, pas un identifiant ni une chaîne (I4). Aucune référence n'est suivie au runtime ; pas de cycle possible à ce niveau. Conséquence : la fermeture des références est une validation propre au type, ajoutée **à l'intérieur du niveau 3 existant**, avant tout dépôt : aucun nouveau niveau, aucun changement de `VMAN`, aucune allocation (§1.3, point 1). C'est l'ajout **le plus proche de la frontière G-CM** ; ses propres tests (§7.2) doivent démontrer qu'elle reste à l'intérieur.
- **Option R2** — *La Forge écrit directement un `SlotIndex` dans l'artefact.* Le runtime ne vérifie que la borne ; l'artefact devient dépendant de l'affectation des slots, déjà portée par `AssetSource.slot`. Moins de logique runtime, mais le type du référent n'est plus vérifié côté runtime.
- **Option R3** — *Pas de référence au niveau asset* : la liaison map ↔ tileset est faite par l'application (donnée de composant ou de Ressource). Évite l'extension, mais déplace dans l'application une cohérence que la Forge doit valider (I2) et que le moteur ne vérifie plus.

### A-5 [A] — Ordre d'empilement
Voir §4.5. Recommandation : **O0**, avec la porte G-ORD.

### A-6 [A] — Périmètre et ordre des unités
Quatre types dans l'ordre : sprite (régression), tileset (charge fixe), map (charge variable + référence), font (charge variable). La font est maintenue car nommée par la Roadmap ; si vous préférez la différer, l'étape se clôt sur tileset + map avec la font consignée comme non démontrée (**[A]** à trancher).

## 6. Choix locaux proposés [L]
- Préfixe d'artefact commun conservé (identité, version `u16` = 1 par type, longueur de charge utile `u32`), little-endian explicite, lecture champ à champ, sans cast.
- Un identifiant de format par type (4 octets), constante de format non nom d'asset (I4).
- Tileset (A-1 option 1) : `{tile_width, tile_height, columns, count}` entiers non signés, charge fixe.
- Map : en-tête `{width, height, référence de tileset}` + `width × height` indices de tuile `u16` ; font : en-tête `{glyph_count, …}` + table de métriques. **Les champs exacts sont renvoyés au document d'exécution.**
- Source Forge : un TOML par type, validé sémantiquement par la Forge (dimensions non nulles *selon la Forge*, indices dans les bornes, taille de grille cohérente) ; le runtime n'applique aucune de ces règles (X7 ; cf. P1.6 de l'étape 2 : une largeur nulle reste binaire-conforme).
- Erreurs : énumérations `Copy`, sans allocation, une variante par contrôle (comme `ReadError`).

## 7. Invariants, tests, API minimale

### 7.1 Batterie commune par type (« même test qu'en étape 2 »)
Pour chaque type, sur le modèle des preuves de l'étape 2 (P1.x pour le lecteur, K1 à K8 pour la chaîne) : lecture d'un artefact littéral écrit à la main, indépendant des constantes ; rejet de **chaque préfixe tronqué** ; identité altérée octet par octet ; version inconnue ; longueur de charge utile erronée et octets excédentaires ; valeurs limites structurellement conformes acceptées (aucune validation métier au runtime) ; déterminisme de la sortie de la Forge (mêmes sources → mêmes octets) ; aucune chaîne ni chemin dans l'artefact ni le manifeste ; erreurs de liaison (index hors bornes, doublon d'identifiant, slot dupliqué) ; tout-ou-rien (aucun dépôt si un niveau 1 à 3 échoue) ; absence de dépendance de parsing dans l'arbre de `verbum-assets` ; chargement sans allocation (même contrôle qu'à l'étape 2, à reprendre du document d'exécution 02).

### 7.2 Tests propres aux nouveautés
- **Charge variable (U3, U4)** : longueur de charge utile incohérente avec `width × height` ou `glyph_count` → rejet niveau 3 ; charge **contenant zéro élément variable** conforme (carte `0 × 0` : partie fixe de 12 octets ; font avec `glyph_count = 0` : partie fixe de 4 octets) ; une charge utile **de longueur nulle** reste invalide ; accès par vue ne dépasse jamais la charge ; capacité dépassée = erreur de configuration, jamais silencieuse (X6).
- **Format du slot (U1)** : artefact d'un type lié à un slot d'un autre type → rejet niveau 2, avant tout dépôt.
- **Références (U3)** : référent absent du manifeste, référent d'un autre type, référence à soi-même, référence valide → `SlotIndex` attendu ; aucune référence suivie au runtime. **Le test de fermeture doit démontrer** (i) qu'un référent absent ou d'un type incorrect provoque un échec **avant tout dépôt** (compteur de dépôts nul), (ii) que cet échec n'implique **aucune nouvelle allocation** (même contrôle d'absence d'allocation que le chargement), (iii) que **`VMAN` est inchangé** (mêmes constantes de layout, mêmes octets de manifeste pour un même ensemble qu'en r1, test de non-régression du layout).
- **Régression de l'étape 2 (U1)** : l'intégralité des tests de l'étape 2 passe, **sans changement de leur sémantique** ; seules des adaptations de signature (A-3 i) sont admises et consignées une à une.
- **Preuve de durée de vie (U1, porte G-LT)** : une Ressource portant une vue empruntée est insérée dans un `World` du Scheduler de l'étape 1 et lue par un système sous `run_tick`. **r1.3** : les octets sont construits à l'exécution (jamais un littéral promu), possédés par l'appelant et déclarés avant le `World`. Tests : **LT-1** (montage réel exécuté sous `run_tick`, plusieurs ticks) ; **LT-2** (la sortie dépend des octets empruntés : deux tampons distincts, deux sorties distinctes) ; **LT-3** (doctest `compile_fail` : libérer les octets après le dépôt est refusé) ; **LT-3t** (jumeau valide, identique sans la ligne `drop`, compile et s'exécute) ; **LT-4** (cas `'static` : compile et s'exécute, **non compté** comme preuve G-LT). LT-1, LT-2 et LT-4 dans `crates/shell/tests/borrowed_resource.rs` ; LT-3 et LT-3t en doctests de `crates/shell/src/lib.rs` (documentation seule). Invariance : suites existantes de `ecs` et `schedule` inchangées et vertes ; contrats, dépendances et ordre d'exécution inchangés. Mutations : M-1.5 (suppression du `drop` de LT-3 → LT-3 échoue) et M-1.6 (ajout du `drop` à LT-3t → LT-3t échoue).

### 7.3 Acceptation : niveau chargé et affiché (U5)
Assemblage d'exemple dans `crates/shell/examples/` + test d'acceptation dans `crates/shell/tests/` : sources → Forge → manifeste et artefacts → `load` → Ressources ; N ticks du Shell ; systèmes producteurs lisant les Ressources et émettant des contributions ; composeur de l'étape 4 ; Backend headless. Le contenu attendu de la `Frame` est calculé **à la main, en littéral**, indépendamment du code testé, et comparé en multi-ensemble (O0). Contrôles : `verbum-render` et ses tests inchangés ; aucun chemin de retour (RA-2 inchangé).

### 7.4 Sensibilité des tests (mutations, sur copies isolées, une à la fois)
Protocole des étapes 2 et 4. Familles retenues : omission d'un contrôle de niveau 3 (troncature, surplus, longueur) ; dépôt avant la fin du niveau 3 ; table de formats ignorée ; fermeture de référence non vérifiée ; décalage d'un index de résolution ; vue débordant la charge ; endianness inversée sur un champ ; tri ou ordre introduit dans la scène (doit rester sans effet). Les mutations exactes, leurs détecteurs et le critère « une erreur de compilation ne compte pas comme détection » sont renvoyés au document d'exécution.

## 8. Unités (une session d'implémentation chacune ; validation de l'utilisateur entre unités)
| Unité | Contenu | Porte d'arrêt |
|---|---|---|
| **U1** | Généralisation du chargement (A-3), préfixe commun factorisé (fonction interne partagée ; adaptation interne de `Sprite::read`, API publique et assertions des tests de l'étape 2 inchangées), table de formats par slot, preuve G-LT ; **sprite seul** ; régression complète de l'étape 2 | Si un test de l'étape 2 doit changer de **sémantique** → arrêt (G-CM) ; **si G-LT échoue** (la durée de vie de la vue ne peut pas être représentée avec le `World` et le Scheduler **tels que corrigés en r1.3**) → **arrêt et rapport, sans contournement** (pas de repli sur b, c ou d sans décision) |
| **U2** | Tileset : Forge + lecteur + batterie commune | — |
| **U3** | Map : charge variable (A-2) + référence au tileset (A-4) + batterie commune et tests propres | Si la résolution de référence exige de sortir des limites du §1.3 point 1 (nouveau niveau, ordre modifié, atomicité perdue, `VMAN` modifié, allocation, modèle d'identité modifié) → arrêt (G-CM) |
| **U4** | Font : charge variable + batterie commune (ou différée, A-6) | — |
| **U5** | Niveau chargé et affiché (§7.3), porte G-ORD, contrôles de clôture | Si une exigence de la scène suppose un ordre → arrêt (G-ORD) |

## 9. Contrôles communs
`cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo test --workspace` (dont doctests) ; `cargo tree -p verbum-assets` (aucune dépendance) et `cargo tree -p verbum-forge` ; contrôle `no_std` sans `alloc` repris de l'étape 2 ; Miri sur les crates sans `unsafe` comme à l'étape 4 ; revue de diff : aucun fichier de l'étape 4 modifié (`crates/render/**`), aucun test de l'étape 2 modifié hors signature (consigné). Les résultats de l'espace de travail de la session sont toujours séparés de ceux du dépôt réel.

## 10. Ce qui reste non démontré à la clôture (à consigner)
- Le contenu raster et son chemin vers un Backend graphique (A-1).
- La règle d'ordre d'empilement (D-ord-1 à D-ord-3, Q8) : point ouvert **avant tout Backend graphique ou toute scène dont le résultat dépend de l'ordre**.
- Le streaming et le déchargement d'assets (ADR-A01, hors étape 2 et hors étape 5).
- La cohérence métier au-delà de ce que la Forge valide.
- Le comportement sur des assets de taille réaliste (performance non mesurée).

## 11. Risques de fermeture prématurée
1. **Validation de fermeture de référence ou de format de slot débordant des niveaux existants** (nouveau niveau, dépôt anticipé, allocation, effet sur `VMAN`) sans être reconnue comme changement de mécanisme : atténuation, définition de G-CM (§1.3), tests dédiés (§7.2) et porte G-CM.
2. **Vue empruntée acceptée sans preuve exécutable** : atténuation, porte G-LT en U1. **r1.3** : le risque s'est réalisé (G-LT échouée avec les bornes d'origine) ; la preuve exécutable a joué son rôle de porte d'arrêt.
3. **Table de formats ajoutée par commodité** : elle répond à une classe d'erreur réelle (C3), testée en U1 ; à défaut, elle serait écartée (X11).
4. **Sur-anticipation du multi-couches** (clé, tri, regroupement) : atténuation, O0 et G-ORD.
5. **Pollution du moteur par un vocabulaire de jeu** : atténuation, X10 ; revue des noms de types, champs et fixtures.
6. **« Affiché » interprété plus fort que le test ne le prouve** : atténuation, A-1 et §10.

## 12. Hors périmètre
Backend graphique ; données raster ; caméra, animation, atlas, compression ; streaming ; scripts (étape 6) ; clé ou tri de composition (sauf O1 autorisé) ; vocabulaire ou règles de jeu ; modification d'un ADR ; modification de l'étape 4 (sauf O1 autorisé).

## 13. Fichiers (prévisions, à confirmer au document d'exécution)
- **`crates/schedule/src/system.rs` — exception r1.3, strictement limitée à ce fichier** : les trois bornes de durée de vie décrites en en-tête. Aucun autre fichier de `crates/schedule`, aucun fichier de `crates/ecs`, ni `access.rs`, `model.rs`, `order.rs`, `contract.rs`, `data.rs`, `scheduler.rs`, `World`, `ResourceSlot`. Toute autre adaptation exigée par le compilateur est exposée avant d'être faite.
- `crates/assets/src/` : `lib.rs`, `load.rs`, `manifest.rs` (a priori inchangé), `sprite.rs` (adaptation interne seulement : appel du préfixe partagé) ; nouveaux `header.rs`, `tileset.rs`, `map.rs`, `font.rs` ; `Cargo.toml` inchangé (aucune dépendance).
- `crates/forge/src/` : `lib.rs`, `batch.rs`, `sprite.rs` ; nouveaux `tileset.rs`, `map.rs`, `font.rs` ; `tests/pipeline.rs` étendu ou nouveaux tests d'intégration.
- `crates/shell/examples/` et `crates/shell/tests/` : scène d'acceptation ; `crates/shell/Cargo.toml` (dépendances de développement : `verbum-assets`, `verbum-forge`) — **[Non vérifié]**.
- `Cargo.toml` racine : `[workspace.dependencies]` à compléter, à partir de la version courante.
- `docs/` : ce plan, le document d'exécution, le handoff ; `tree.md` à régénérer.

## 14. Critères d'acceptation (clôture)
1. Chaque type (sprite, tileset, map, font) passe la batterie commune (§7.1) et ses tests propres (§7.2).
2. Les tests de l'étape 2 passent sans changement de sémantique.
3. `verbum-assets` : `no_std`, sans `alloc`, `[dependencies]` vide ; aucun parseur de format humain.
4. Le niveau d'acceptation circule jusqu'au Backend headless et correspond au littéral attendu (§7.3) ; `crates/render/**` inchangé.
5. Mutations détectées selon le protocole (§7.4).
6. Aucun invariant ni ADR modifié ; limites du §10 consignées ; seule modification hors `assets`/`forge`/`shell` : `crates/schedule/src/system.rs` (r1.3).
7. Contrôles du §9 verts sur le dépôt réel, par vous.

## 15. Décisions restantes pour valider ce plan
A-1 à A-6 sont retenus (§5). Restent : (1) validation de la révision r1.3 et examen conjoint du résultat de G-LT (U1 ne reprend qu'après preuve positive) ; (2) fourniture du **`Cargo.toml` racine courant** et de `crates/shell/Cargo.toml` avant le document d'exécution (la copie antérieure à l'étape 4 n'est pas utilisée).

## 16. Fin de l'étape
L'étape 5 se clôt quand le §14 est satisfait et que vous l'avez déclaré après exécution des contrôles sur le dépôt réel. Le document d'exécution suit la validation de ce plan ; le handoff suit la validation du document d'exécution.
