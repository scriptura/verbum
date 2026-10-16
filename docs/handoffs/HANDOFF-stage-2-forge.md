# HANDOFF — Verbum : passation vers l'étape 2 (Forge minimale)

Date : 2026-10-16. Document non commité. Aucun code, aucune ADR, aucune modification de la Roadmap n'accompagne ce handoff. Point d'arrêt : relecture par l'utilisateur, puis validation du plan de construction de l'étape 2.

## 0. Avertissement de lecture (à lire en premier)

Ce handoff repose sur les sources réellement lues dans la session qui l'écrit. Trois niveaux sont distingués partout :

- **[Lu]** : texte vérifié dans un document que la session a eu sous les yeux.
- **[Déduit]** : conséquence que je tire d'un texte lu ; à contester si la source complète dit autre chose.
- **[Non vérifié]** : dépend d'un document que la session n'a pas pu lire.

Documents lus : `ROADMAP.md` (version fournie pendant S5, texte intégral), `01-verbum-scheduler.md` r5 et `01-verbum-scheduler-sessions.md` (extraits ciblés par recherche, non relus en entier), `HANDOFF-verbum-schedule-phase8.md`, l'arbre du dépôt du 15 octobre (`tree.md`), le code de `verbum-schedule` et le `lib.rs` réel de `verbum`.

**Documents demandés mais NON lus** (absents du contexte et du bac à sable, vérifié par recherche sur le système de fichiers) : `ADR-A01-assets.md`, `ADR-R01`, `ADR-R02`, `ADR-R03`, `ADR-S01`, `ADR-001` (point 5 lu seulement par citation dans la Roadmap et le plan), `ADR-005`, `ADR-000` (citations seulement), `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, `README.md` d'architecture, `docs/drafts/LPC-Paradigm.md` et `docs/drafts/LPC_YAML/*` (dont `build_manifest.yaml`).

Conséquence : l'analyse des responsabilités de Forge, Assets et Renderer (§4 à §6) ne peut pas conclure sur ce que ces ADR imposent. Elle s'arrête à ce que la Roadmap énonce et dresse la liste des questions que seuls ces documents tranchent (§8). La session suivante doit les recevoir avant de cadrer l'étape 2 (§9).

La Roadmap lue est celle fournie pendant S5 ; je n'ai pas vérifié qu'elle est identique à la version courante de `docs/architecture/ROADMAP.md` dans le dépôt.

## 1. Rôle et règles de travail (inchangés)

- Rôle : implémenteur Rust (AOT, DOD, ECS) de Verbum, moteur 2D. Pas d'accès au dépôt : l'utilisateur fournit le contenu exact des fichiers, fait lui-même les commits (messages et hashes : jamais rédigés ni inventés par moi sauf demande) et exécute les contrôles réels (build, test, clippy `-D warnings`, fmt, doc, Miri nightly, MSRV 1.85).
- Cadrage avant code : invariant cité → test → API minimale. Tests d'abord, rouge confirmé. Un concept n'est codé que s'il est porté par un invariant adopté ou rendu nécessaire par le test de la phase.
- Ne jamais affirmer qu'une commande a passé sans l'avoir exécutée ; séparer résultats du bac à sable et résultats réels. MSRV 1.85 et Miri ne se vérifient pas dans le bac à sable.
- Contradiction de directives ou information manquante : s'arrêter et rapporter, ne pas extrapoler. Ne modifier ni ADR, ni docs normatives, ni phases acceptées. `Cargo.lock` seulement si Cargo le modifie réellement ; jamais à la main.
- Livraison : fichiers complets via `SendUserFile`, première ligne = commentaire de chemin puis ligne vide (les `Cargo.toml` sont exemptés). Style : vouvoiement, français concis, structuré, opérationnel ; pas de métaphores ni de pédagogie scolaire ; vocabulaire technique introduit progressivement.
- Trailer `Claude-Session` : propre à chaque session ; ne pas reprendre celui d'une session précédente.

## 2. État à la clôture de l'étape 1

**[Lu]** Étapes 0 et 1 acceptées. Workspace : `crates/verbum-ecs` (noyau, `no_std`, sans `alloc`), `crates/verbum-schedule` (Scheduler, `no_std`, sans `alloc`), `crates/verbum` (façade `no_std` : `pub use verbum_ecs as ecs; pub use verbum_schedule as schedule;`). Édition 2024, `rust-version = 1.85`, `unsafe_code = "deny"`.

Contrôles réels (rapportés par l'utilisateur) : tout vert sur l'état final de 10c, y compris Miri sur `verbum-schedule` et `cargo +1.85.0 test --workspace`. Hashes des commits des phases 8, 9 et S5 : à renseigner par l'utilisateur.

Décomptes du bac à sable (rustc 1.97) : `verbum-ecs` 77 unitaires, 11 intégration, 9 doctests ; `verbum-schedule` 76 unitaires, 6 acceptation, 36 `access`, 24 `scheduler`, 18 `tick`, 52 doctests (19 exécutables, 33 `compile_fail`).

Réserves conservées : la Roadmap n'a pas été relue exhaustivement au cadrage de S5 (acceptation sur la base du plan r5) ; le coût O(N²) du dispatch par récursion n'est pas mesuré (limite connue, pas un défaut).

Pour l'étape 2, seule la dépendance de l'étape 0 compte (§3). Interface utile de `verbum-ecs` : `World<ENTITIES, S, R>` avec `resources()`, `resources_mut()`, `split_mut()`, `apply()` ; `ResourceSlot<T>` ; `CommandBuffer` ; `()` implémente `ComponentSet`. Une Ressource est un champ de la structure `R` définie par l'appelant, clé = position statique (ADR-005, citée par le plan r5).

## 3. Le contrat de l'étape 2 tel que la Roadmap l'énonce

**[Lu]**, citations de la Roadmap :

- Titre : « Étape 2 — Forge minimale (un seul type d'asset) ».
- Contenu : « un pipeline complet mais étroit — `sprite.toml → Forge → sprite.bin → lecture binaire → Ressource`. Pas de validation exhaustive, pas de second type d'asset. Le chargement runtime peut être simulé par un appel direct en test, sans Shell réel — cohérent avec le mécanisme de test déjà autorisé pour les capacités de stockage (ADR-001, point 5). »
- Dépendance : « Étape 0 seulement (la Ressource produite doit s'intégrer au World existant ; ni le Scheduler ni le Shell ne sont requis pour ce pipeline). »
- Test d'invariant : « Le runtime ne parse jamais le `.toml` — seule la Forge le fait ; le runtime ne lit qu'un artefact binaire déjà validé (`00-principes.md`, invariant 1). »
- Artefact : « premier binaire produit par la Forge, relu et casté en Ressource ».
- Étape 0, « explicitement hors de cette étape » : « capacités fixées par Forge — ADR-001 point 5 autorise nommément un mécanisme de test/benchmark à leur place ».

Le principe directeur de la Roadmap s'applique : une étape n'est validée que si aucun invariant architectural n'a dû être modifié pour la faire fonctionner. La Roadmap se déclare « non normative », « révisable sans procédure d'ADR ».

### 3.1 Le pipeline `PNG + YAML → AssetRegistry → FrameSequence` est-il imposé ?

**[Lu]** Non, par la Roadmap. Elle ne mentionne ni PNG, ni YAML, ni `AssetRegistry`, ni `FrameSequence`. Ce qu'elle impose, au niveau contractuel :

1. une source déclarative lisible par la Forge seule (la Roadmap illustre avec `sprite.toml`) ;
2. un artefact binaire produit par la Forge, déjà validé ;
3. une lecture runtime de cet artefact sans parsing de la source ;
4. une Ressource qui s'intègre au `World` de l'étape 0 ;
5. un seul type d'asset, sans validation exhaustive.

Le format source (TOML dans l'illustration), le conteneur binaire, le nom `sprite`, et l'existence d'un registre ne sont pas des exigences de la Roadmap. Le pipeline cité dans la demande est donc **une proposition particulière** au regard de ce texte.

**[Non vérifié]** : ce que `ADR-A01-assets.md` et `00-principes.md` (invariant 1) imposent sur la nature de la source, du binaire et de l'identité des assets. Ils peuvent contraindre davantage que la Roadmap. À lire avant le plan de l'étape 2.

**[Déduit]** Deux écarts du pipeline cité vis-à-vis de la Roadmap à examiner, sans les trancher :
- `PNG` introduit une source binaire d'image, donc un décodage côté Forge ; la Roadmap parle d'une source déclarative (`sprite.toml`) et d'un artefact `sprite.bin`. Que Forge décode des pixels ou ne manipule que des métadonnées est une question de périmètre « minimal » ouverte (§8, Q3).
- `AssetRegistry` et `FrameSequence` sont des noms de concepts runtime. Le plan de l'étape 1 listait `Registry` parmi les noms exclus *de cette étape* ; ce n'est pas une règle pour l'étape 2, mais le statut de ce vocabulaire dans les ADR reste à vérifier avant adoption.

### 3.2 Manifeste ou index d'assets : prévu, nécessaire ?

**[Lu]** La Roadmap ne mentionne aucun manifeste ni index pour l'étape 2. **[Lu, nom de fichier seulement]** `docs/drafts/LPC_YAML/build_manifest.yaml` existe dans l'arbre du 15 octobre ; son contenu n'a pas été lu, et c'est un brouillon associé à LPC : il ne constitue pas une source pour le noyau générique.

Distinction à conserver, deux objets de responsabilités différentes :

| | Index (ou manifeste) de build | Registre runtime |
|---|---|---|
| Vit | côté Forge, au build ; artefact ou état de l'outil | dans le runtime ; Ressource du `World` ou structure chargée |
| Rôle possible | lister sources et sorties, ordre de production, empreintes, résolution des références entre sources, reproductibilité, dimensionnement | associer une identité d'asset à des données lues (identifiant → contenu) |
| Soumis à | invariants de la Forge (validation, déterminisme du build) | invariants du runtime : Ressources (ADR-005), accès déclaratifs (ADR-003), pas de résolution dynamique par métadonnées (point cité par le plan r5, I14), mémoire (ADR-000/001) |
| Peut exister sans l'autre | oui | oui (un artefact unique sans index) |

**[Déduit]** Pour le test d'invariant de l'étape 2 (« le runtime ne parse jamais la source, ne lit qu'un binaire validé »), un asset unique n'exige ni index de build ni registre runtime : aucun invariant lu ne les requiert à ce stade. Ils deviennent des questions réelles si :
- plusieurs assets se référencent (étape 5, tilesets, maps) ;
- la Forge doit valider l'unicité ou la résolution d'identifiants à la construction ;
- la Forge fixe des capacités (ADR-001, point 5) plutôt que le mécanisme de test autorisé ;
- la reproductibilité du binaire doit être démontrée.

Aucun de ces points n'est requis par la Roadmap à l'étape 2. **[Non vérifié]** si `ADR-A01` ou `00-principes.md` en font un invariant. Question Q4, §8. La conclusion actuelle est négative par défaut, au sens « non démontré nécessaire » ; ce n'est pas une décision.

Aucune étape 2bis n'est proposée et la Roadmap n'est pas modifiée.

### 3.3 Autres questions de périmètre de l'étape 2 (sans trancher)

- **Placement** : la Roadmap ne nomme aucun crate. Forge est un outil de build (std probable) ; la lecture runtime relève du noyau `no_std` sans `alloc` (ADR-000, cité). Où vit chaque moitié, et quel crate lit le binaire : ouvert (Q5).
- **« casté en Ressource »** : la Roadmap suppose une lecture quasi sans parsing. Alignement, endianness, version du format, taille fixe : à confronter aux invariants matériels d'ADR-000 (cités, non relus) (Q6).
- **Identité de l'asset** : comment la Ressource est désignée à l'étape 2 (position statique dans `R`, ADR-005) et comment cela évoluera à l'étape 5 sans « changement de mécanisme » (invariant de l'étape 5) (Q7).

## 4. Contrainte d'évolutivité : assets visuels composés de plusieurs couches

Statut : **contrainte d'évolutivité à vérifier** aux étapes pertinentes. Ce n'est ni un invariant déclaré, ni une décision architecturale adoptée. Elle ne modifie pas la Roadmap.

### 4.1 Le besoin générique

Un même ensemble logique affiché (par exemple un personnage) peut devoir être représenté par **plusieurs contributions visuelles indépendantes, avec un ordre d'empilement déterministe**. Formulation neutre : un ensemble logique → N contributions visuelles ordonnées, N non borné par le modèle de représentation (mais borné par une capacité fixée, cf. critères §4.3).

Risque identifié (légitime) : une représentation de rendu construite autour d'« une image par élément rendu » imposerait, pour ajouter des couches, de changer la forme de la représentation de frame, donc ses producteurs (systèmes de composition), son immuabilité (ADR-R02, citée) et ses consommateurs (Backend, ADR-R03). Le coût de correction croît avec le nombre de composants du Renderer construits sur l'hypothèse.

### 4.2 Représentations possibles (non arbitrées)

Familles listées pour permettre la comparaison, aucune adoptée :

1. **Un élément rendu = une image** (état de départ ; fait apparaître le risque).
2. **Plusieurs éléments rendus indépendants par ensemble logique**, l'ordre d'empilement étant une donnée explicite de chaque élément (clé de tri), le regroupement logique restant implicite.
3. **Tableau inline à capacité fixe** de contributions par élément, avec compteur.
4. **Plage (offset, longueur) dans un tampon plat partagé** de contributions, de capacité fixée.
5. **Collection à taille variable** (`Vec<Layer>` ou équivalent) : à confronter aux invariants d'absence d'allocation sur le chemin chaud ; aucune présomption qu'elle soit la bonne forme.

### 4.3 Critères de confrontation

Toute représentation candidate se juge sur : propriété (qui possède les contributions ; immuabilité de la frame une fois composée, ADR-R02 pt 4) ; capacité (fixée avant simulation, dépassement = erreur de configuration, jamais silencieux : principe cité d'ADR-001 pt 5) ; layout mémoire (contigu, sans indirection inutile, DOD) ; absence d'allocation sur le chemin chaud ; déterminisme de l'ordre (relation d'ordre totale et stable : égalité de clés départagée par une règle fixée) ; Backend consommateur passif, rien ne remonte vers l'ECS (ADR-R03 pt 4-5) ; coût de migration depuis la représentation précédente.

### 4.4 Point sur le Scheduler, vérifié en S5

**[Lu, test écrit en S5]** Les producteurs libres d'une même donnée transitoire sont interchangeables (ADR-003 pt 6) et l'ordre entre systèmes sans arête est l'ordre de déclaration, qui n'est pas une sémantique. Le test de S5 ne couvre que l'agrégation additive, donc commutative ; il ne dit rien d'une opération non commutative.

**[Déduit]** Si plusieurs systèmes produisent des contributions visuelles pour un même ensemble, l'ordre d'empilement ne peut pas se déduire de l'ordre d'émission : il doit être porté par la donnée (clé explicite) ou par une résolution unique en aval. À vérifier à l'étape 4, pas avant. Aucune modification du Scheduler n'est demandée ni suggérée.

### 4.5 Ce qui reste hors du noyau générique

Les particularités LPC ne doivent entrer ni dans le noyau ni dans les tests génériques : identifiants de couches et de pièces, noms de parties du corps ou d'équipement, conventions de grille et de feuille de sprites, animations canoniques, règles de résolution d'action, formats de contrats par arme. Le noyau ne connaît que : un ensemble logique, des contributions visuelles, un ordre déterminé par une donnée générique. L'adaptation LPC relève d'une couche d'assets et d'outillage hors noyau, à définir plus tard.

**[Lu, noms de fichiers seulement]** `docs/drafts/` contient des brouillons LPC (`LPC-Paradigm.md`, `LPC_YAML/*`, `grid_64.svg`). Leur contenu n'a pas été lu ; ils servent au plus de cas d'usage pour valider la généricité, jamais de source de vocabulaire.

## 5. Où vérifier la contrainte dans la Roadmap

| Étape | Vérification | Nature |
|---|---|---|
| 1 (Scheduler) | Rien à faire ; close. Observation §4.4 à reprendre à l'étape 4. | Aucune |
| **2 (Forge)** | Vérifier que le format de l'artefact et la Ressource du seul type d'asset ne figent pas, **au niveau de la relation ensemble logique → contributions**, l'hypothèse « une image par ensemble ». **[Déduit]** le besoin multi-couches se joue surtout dans cette relation et dans la désignation des assets, moins dans le format d'un sprite isolé. Pas d'implémentation de couches, pas de second type d'asset. | Vérification de non-fermeture, par examen du plan |
| 3 (Shell) | Sans objet. | — |
| **4 (Renderer)** | **À signaler explicitement avant le cadrage** : la représentation de frame doit pouvoir exprimer plusieurs éléments visuels ordonnés pour un même ensemble logique. Piste de test, si elle relève des responsabilités du Renderer minimal (**[Non vérifié]**, ADR-R01/R02) : un test synthétique de deux éléments visuels indépendants avec un ordre déterminé, par exemple deux carrés distincts, sans vocabulaire, règle ni asset LPC. Compatible avec « un clear-color ou un carré suffit » et avec le test d'invariant de l'étape (frame immuable, rien ne remonte du Backend). Ne pas présumer `FrameResource { layers: Vec<Layer> }` : la confronter aux critères §4.3. | Test synthétique candidat, à décider au cadrage |
| **5 (Assets réels)** | L'invariant de l'étape (« aucun changement de mécanisme du pipeline Forge → Ressource ») peut entrer en tension si l'ensemble logique référence plusieurs assets : vérifier à ce moment que la référence entre assets ne force pas un changement de mécanisme. | Vérification de non-régression du mécanisme |
| 6 (Scripts) | Sans objet à ce stade. | — |

## 6. Ce que ce handoff ne décide pas

Aucune représentation de rendu, aucun format de binaire, aucun format source, aucun registre ou index, aucune étape 2bis, aucune modification de Roadmap ou d'ADR, aucun invariant nouveau. Toute conclusion plus ferme dépend des documents du §0.

## 7. Risques identifiés

1. **Fermeture prématurée** : une première représentation (Ressource d'asset à l'étape 2, frame à l'étape 4) bâtie sur « une image » rendrait la correction coûteuse. Mitigation : vérification de non-fermeture aux étapes 2 et 4, sans préimplémenter.
2. **Sur-anticipation** : introduire couches, registre ou manifeste sans invariant qui les requiert violerait la règle « un concept n'est codé que s'il est porté par un invariant adopté ou rendu nécessaire par le test ». Mitigation : tout ajout doit citer son invariant ou être écarté.
3. **Contamination LPC** : conventions d'un cas cible dans le noyau. Mitigation : tests synthétiques sans vocabulaire de domaine ; revue de nommage.
4. **Ordre porté par l'émission** (§4.4) : dépendance implicite à l'ordre de déclaration. Mitigation : ordre porté par la donnée.
5. **Format source inféré** : adopter `PNG + YAML` ou `sprite.toml` sans lire ADR-A01. Mitigation : §8, Q1.
6. **Lacune documentaire** : conclure sur Forge/Assets/Renderer sans les ADR concernés (§0).

## 8. Questions ouvertes (avec le document qui les tranche)

- **Q1.** `ADR-A01` et `00-principes.md` (inv. 1) imposent-ils la nature de la source, du binaire, de l'identité des assets ? *(ADR-A01, 00-principes)*
- **Q2.** Responsabilités exactes de Forge, Assets, Renderer, et frontière avec le Shell. *(ADR-A01, ADR-R01 à R03, ADR-S01, 02-concepts)*
- **Q3.** « Minimal » inclut-il le décodage d'image côté Forge, ou seulement des métadonnées ? *(Roadmap + ADR-A01)*
- **Q4.** Un invariant exige-t-il un index de build, un registre runtime, ou les deux, avant l'étape 5 ? Si oui, pour quelle responsabilité ? *(ADR-A01, ADR-005, ADR-001 pt 5)*
- **Q5.** Placement des crates (outil Forge vs lecture runtime `no_std`) et dépendances autorisées. *(ADR-000, README)*
- **Q6.** Contraintes de layout du binaire « casté » (alignement, endianness, version, tailles fixes). *(ADR-000, ADR-001)*
- **Q7.** Désignation des Ressources d'assets (position statique, ADR-005) et trajectoire vers l'étape 5 sans changement de mécanisme. *(ADR-005, Roadmap étape 5)*
- **Q8.** Qui porte l'ordre d'empilement : donnée ECS, asset, composition du Renderer ? Où naît la notion d'« ensemble logique affiché » ? *(ADR-R01, ADR-R02, 02-concepts)*
- **Q9.** Le test synthétique à deux éléments ordonnés relève-t-il des responsabilités du Renderer minimal ? *(ADR-R01 à R03, au cadrage de l'étape 4)*
- **Q10.** Capacités de stockage : l'étape 2 fixe-t-elle quelque chose par la Forge ou garde-t-elle le mécanisme de test d'ADR-001 pt 5 ? *(ADR-001, Roadmap)*

## 9. Marche à suivre pour la session suivante

1. L'utilisateur relit ce handoff, puis fournit les documents du §0 non lus : `ADR-A01`, `ADR-R01`, `ADR-R02`, `ADR-R03`, `ADR-S01`, `ADR-001`, `ADR-005`, `ADR-000`, `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, `README.md` d'architecture, `ROADMAP.md` courant du dépôt, `HANDOFF-Stage-1.md`. Les brouillons LPC (`LPC-Paradigm.md`, `LPC_YAML/*`) uniquement comme cas d'usage de généricité, après les autres.
2. La session lit ces sources, cite avant de juger, et répond aux questions Q1 à Q10 en séparant [Lu], [Déduit], [Non vérifié].
3. Elle rédige un plan de construction de l'étape 2 (invariants cités, tests, API minimale, fichiers, risques, découpage en sessions), **sans code**, y inclut la vérification de non-fermeture du §5, et s'arrête pour validation.
4. Aucune implémentation, aucune modification de Roadmap ou d'ADR avant validation du plan.
5. Avant le cadrage de l'étape 4, relire §4 et §5 et décider du test synthétique.

## 10. Fichiers de l'état à rattacher à la prochaine session

Racine : `Cargo.toml`, `Cargo.lock`. `crates/verbum` : `Cargo.toml`, `src/lib.rs`. `crates/verbum-ecs` : `Cargo.toml` et sources utiles à l'étape 2 (`resource.rs`, `world.rs`, `lib.rs`, `component_set.rs`), ou un `tree`. `crates/verbum-schedule` : seulement si l'étape 2 en dépend, ce que la Roadmap exclut. Plans : `01-verbum-scheduler.md` r5 et `01-verbum-scheduler-sessions.md` pour la méthode.
