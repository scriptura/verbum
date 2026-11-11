# Page d'axes — Politique temporelle du Shell (préparation d'ADR-S04) — révision 4

## 0. Statut et conventions

**Non normative. Non adoptée.** Soumise à validation avant toute rédaction d'ADR-S04. Date : 11 novembre 2026. Aucun ADR, aucun code, aucun test n'est modifié ni lancé.

Balises : **[Acquis]** posé par un ADR adopté · **[Découle]** conséquence directe de textes adoptés · **[Arbitré]** décision de l'utilisateur · **[Orientation]** piste retenue pour examen, non adoptée · **[Option]** possibilité ouverte · **[Exp.]** expérience à réaliser (aucune n'a été exécutée) · **[Dép.]** dépendance vers une autre décision · **[À confirmer]** précision à écrire dans l'ADR · **[Inconnu]** non vérifié.

Périmètre : traitement du retard et de l'interruption. Le quantum n'apparaît que comme arbitrage enregistré (§ 2). Sont exclus : transmission et répétition du Frame, `PresentMode`, cycle de vie du Backend (R03 / S05), toute signature d'API, toute valeur numérique de quantum, de K ou de M.

**Changements de la révision 3.** Statut de K et M arbitré (§ 5.1) ; instant de mesure de M, vérifié contre S02 (§ 5.2 à 5.5) ; échéances devenues dues pendant la progression (§ 4) ; interruption importante explicitée (§ 5.4) ; réancrage R1 et comparaison à `FixedStepPacing` par suivi du cycle complet (§ 5.6) ; tableau comparatif corrigé (§ 6) ; précisions sur A7 et sur les dépôts concurrents ; présentabilité reformulée (§ 7) ; verdict de suffisance (§ 12). ADR-AU01 et ADR-N01 n'avaient pas pu être lus (§ 5.6).

**Changements de la révision 4.** ADR-AU01 et ADR-N01 ont été lus. Aucune exigence de synchronisation ni de fidélité temporelle justifiant R2 n'y figure : R1 est maintenu comme choix proposé et la raison pour laquelle R2 n'apporte pas de bénéfice établi est explicitée (§ 5.6). Deux dépendances résiduelles sont identifiées, sans exigence nouvelle (§ 5.6, § 10). Verdict de suffisance mis à jour (§ 12). Aucune autre section n'est modifiée.

## 1. Décisions acquises

| # | Acquis | Source |
|---|---|---|
| A1 | Le Shell possède les sources de temps réel et détermine seul si une itération est due ; l'adaptateur ne fournit ni politique, ni instant décisionnel, ni durée de tick. La substitution de la source de temps est une décision d'assemblage | S01 pt 5 ; S02 pts 5 et 13 |
| A2 | Une progression est demandée « à un instant que le Shell observe lui-même » et exécute « les itérations alors dues », 0..N ; N est fixé par S04 | S02 pt 4 |
| A3 | Une progression ne bloque pas (aucune attente interne), se termine en temps fini, n'effectue aucune coupure si rien n'est dû, est idempotente après l'état terminal et non réentrante. Ce n'est pas une garantie de faible latence | S02 pt 4 |
| A4 | Le compte rendu porte : itérations exécutées, besoin de réveil (échéance, ou « aucune échéance »), état terminal. L'échéance est « souhaitée » : aucune garantie temps réel ; un réveil prématuré est sans conséquence. « Aucune échéance » signifie qu'aucune itération n'est due tant qu'une condition extérieure ne change pas : indication de besoin de réveil, ni état du Shell, ni canal de commande. Le compte rendu n'a aucun champ pour des échéances abandonnées | S02 pts 4 et 6 |
| A5 | Chaque itération applique le même cœur : arrêt observé → coupure de l'`Inbox` → publication et invocation → comptage. Les deux pilotes donnent la même séquence de lots et d'invocations pour les mêmes observations | S02 pts 2 et 3 |
| A6 | La coupure est unique entre deux itérations ; aucune publication pendant une itération ; aucune donnée d'une itération précédente ne survit | S03 pts 6, 7, 12 |
| A7 | Un lot valide, éventuellement vide, est publié avant chaque itération effectivement exécutée, la première comprise. Si la publication échoue, l'itération correspondante n'est pas lancée et n'est pas comptabilisée comme exécutée. Le déterminisme est posé pour « une même séquence de lots effectivement publiés » : un lot par itération exécutée, lots vides compris. La répartition d'événements concurrents entre deux lots n'est pas reproductible | S02 pt 2 ; S03 pts 6 et 11 ; `00-principes.md` inv. 3 |
| A8 | Le `World` ne reçoit ni durée ni instant ; pas de `deltaTime` flottant ; ticks fixes | `00-principes.md` inv. 3 et « Modèles écartés » ; S01 pt 5 |
| A9 | Modèle initial monothread : la progression s'exécute sur le fil de la boucle de plateforme ; une itération ou une progression longue retarde la distribution des signaux de plateforme | S02 pt 9 |
| A10 | « 0..N itérations pour 0..1 représentation transmise » ne doit pas être rendu impossible. Traité par R03 / S05 | S01 pt 5 ; S02 pt 12 |
| A11 | La première erreur de l'infrastructure est conservée inchangée et rend l'état terminal ; aucune progression ultérieure n'exécute d'itération | S02 pts 4 et 7 |
| A12 | La politique temporelle réelle n'existe pas : `FixedStepPacing` est provisoire et son comportement n'est pas une décision | `pacing.rs` ; S02 |

## 2. Arbitrage enregistré : quantum temporel [Arbitré, 11 nov.]

- Un quantum temporel unique par configuration de jeu, non un quantum par script.
- Donnée de build-time connue de la Forge (la conversion d'unités humaines en ticks relève de la Forge : `00-principes.md` inv. 1).
- Cohérence garantie sur toute la chaîne Forge → artefacts → Shell avant exécution : le runtime obtient le quantum d'une donnée de build ou vérifie sa configuration contre celui des artefacts ; aucune exécution silencieuse d'artefacts compilés pour un autre quantum.
- Le Shell reste propriétaire de l'horloge réelle et de la politique d'échéance. Le `World` et l'interpréteur ne manipulent que des ticks.
- Valeur, représentation, mécanisme de transmission ou de vérification : implémentation. Pour les durées non entières en ticks, une règle déterministe explicite doit exister (rejet, quantification, autre) ; elle n'est pas choisie ici.

**État réel.** Aucun artefact ne porte de quantum ; `wait.ticks` est un entier brut ; `FixedStepPacing::new(Duration)` reçoit son intervalle de l'appelant ; aucune conversion secondes → ticks n'existe.

**Dépendance explicite (VMAN, étape 5).** S04 peut définir le contrat de cohérence Forge → artefacts → Shell. Il n'autorise ni ne réalise implicitement une modification du format des artefacts ou du manifeste `VMAN`, protégé par la porte G-CM de l'étape 5. Cette évolution relève du cadre documentaire et d'implémentation approprié.

**Points de lecture.** **R-1** « quantum » est ambigu : *quantum logique* (durée de jeu d'un tick, fixe, utilisée par la Forge) et *intervalle d'échéance* (durée réelle entre deux échéances), qui coïncident à vitesse normale. Seul le premier est visé par l'arbitrage. **R-2** La règle des durées non entières relève probablement d'une décision côté Forge.

## 3. Lots d'acquisition [Arbitré dans son principe]

S04 n'introduit **aucune règle supplémentaire d'affectation temporelle des lots à l'échelle d'une progression.**

- **[Acquis]** Chaque itération exécute sa propre coupure et sa propre publication ; les données acceptées avant cette coupure sont publiées pour cette itération ; les données déposées ensuite peuvent être prises par une coupure suivante ; aucun lot n'est rejoué (S02 pt 2 ; S03 pts 6 et 7).
- **[Découle]** Les événements déjà déposés dans l'`Inbox` avant la première coupure d'une progression sont consommés par la première itération. Cela ne s'étend pas aux événements encore dans la file native de la plateforme : non distribués à l'adaptateur, ils ne sont pas déposés.
- **[Découle]** Dans le modèle monothread initial, aucun signal de plateforme n'est distribué pendant une progression (S02 pts 8 et 9) ; en l'absence de nouveaux dépôts, les coupures suivantes de la progression reçoivent des lots vides. **Ce n'est pas une garantie absolue** : d'autres fils peuvent déposer à tout moment par la poignée de dépôt (S03 pt 3), et un tel dépôt peut figurer dans un lot d'une itération ultérieure de la même progression.
- **[Découle, observation]** La finesse temporelle d'une entrée dépend donc des frontières de progression, non d'une règle de S04. Une datation par instants est hors périmètre (S03 : non tranché).
- **Résolution du renvoi de S02 pt 12** (« l'affectation temporelle des lots d'entrées ») : par l'énoncé que S04 ne crée aucune mécanique d'affectation au-delà du cœur d'itération adopté. À reprendre dans l'ADR.

## 4. Vocabulaire et observations temporelles

- **Échéance** : instant planifié à partir duquel une itération est due. L'échéancier est une suite d'échéances ; un pointeur désigne la prochaine non exécutée et non abandonnée.
- **Échéances dues à l'instant t** : nombre d'échéances planifiées ≤ t, ni exécutées ni abandonnées.
- **Quantum q** : durée entre deux échéances successives à vitesse normale (R-1).
- **K** : nombre maximal d'itérations exécutées par une progression. Borne le nombre d'itérations d'un appel, **pas son temps d'exécution réel** ni le travail total à fournir pour résorber un retard.
- **M** : seuil du résiduel (ci-dessous), en nombre d'échéances.
- **Abandon** : des échéances dues sont supprimées de l'échéancier ; elles ne deviennent jamais des itérations. Aucun tick logique n'est sauté : le temps logique n'avance simplement pas pour ce temps réel.
- **Réancrage** : règle qui fixe la prochaine échéance après un abandon. **L'instant de réancrage et la prochaine échéance sont distincts** : réancrer « à maintenant » signifie que la prochaine échéance vaut now + q, non que la prochaine itération est immédiatement due.

**Trois observations à ne pas confondre dans une progression.**

| | Observation | Instant |
|---|---|---|
| (a) | Échéances dues à l'entrée de la progression (L_e) | t_entrée |
| (b) | Échéances devenues dues pendant l'exécution des itérations (arrivées à raison d'une par q de temps réel) | entre t_entrée et t_fin |
| (c) | **Résiduel** : échéances encore dues après la dernière itération de la progression | t_fin |

Le résiduel (c) comprend le reliquat de (a) non exécuté et toutes les échéances de (b). Une formule fondée sur le seul L_e (par exemple L_e − K) ne le représente pas.

## 5. Politique F : contrat et vérification de compatibilité avec S02

### 5.1 Statut des paramètres [Arbitré, 11 nov.]

- **K** : borne structurelle finie et strictement positive du nombre d'itérations exécutées dans une progression.
- **M** : seuil structurel fini et strictement positif du résiduel toléré. Règle de comparaison : un résiduel **≤ M est conservé** ; un résiduel **> M est abandonné**.
- Leur existence, leur indépendance (aucune relation implicite, en particulier pas K = M/q) et leur rôle relèvent du contrat architectural de S04.
- Les valeurs numériques restent un choix de composition et d'implémentation ; elles sont explicites et stables pour une instance de Shell donnée, non fournies librement à chaque appel de progression. Aucune valeur n'est fixée dans l'ADR.
- Cohérence avec S02 : une valeur fixée à l'assemblage ne constitue ni une politique ni un instant fourni par l'adaptateur (S02 pts 5 et 13).
- M fini et strictement positif exclut les cas limites : la conservation illimitée (B, M infini) et l'abandon systématique (C, M nul) ne sont pas des membres de la famille F ; ils restent dans la comparaison (§ 6) comme points de repère.

### 5.2 Mécanique examinée [Orientation, non adoptée]

1. Le Shell observe l'heure à l'entrée (t_entrée) et détermine les itérations dues (a).
2. Il exécute au plus K itérations.
3. Il observe à nouveau l'heure à la fin (t_fin).
4. Il calcule le nombre d'échéances encore dues à t_fin : le résiduel (c).
5. Si résiduel > M : abandon du reliquat et réancrage. Sinon : conservation.

Cette mécanique mesure M sur le résiduel à la fin de la progression.

### 5.3 Vérification contre le texte de S02

| Disposition de S02 | Exigence | Mécanique 5.2 |
|---|---|---|
| Pt 4 : progression demandée « à un instant qu'il observe lui-même », exécutant « les itérations alors dues » | Un ensemble d'itérations déterminé par un instant | L'ensemble est fixé à t_entrée (≤ K) ; compatible à la lettre. Réévaluer l'ensemble avant chaque itération (reprendre l'heure à chaque fois) ne serait pas décrit par ce texte : non retenu ici |
| Pt 4 : progression finie | Terminaison | ≤ K itérations ; la seconde lecture n'est ni une boucle ni une attente ; compatible |
| Pt 4 : sans effet si rien n'est dû, aucune coupure | Pas d'effet | Si rien n'est dû à t_entrée : aucune itération, aucune seconde lecture requise, aucun abandon ; compatible |
| Pt 5 : le Shell lit le temps réel et détermine seul s'il est dû | Propriété de l'horloge | Aucune limite au nombre de lectures n'est posée. « Un instant » qualifie l'ensemble des itérations dues, pas le nombre de lectures. Lecture à **confirmer par une précision dans l'ADR**, sans divergence démontrée |
| Pt 6 : le compte rendu porte itérations exécutées, besoin de réveil, état terminal | Contenu du compte rendu | Inchangé : § 5.4. Aucun champ pour les échéances abandonnées : l'observabilité des abandons n'est pas fournie par S02 et n'est pas ajoutée par S04 |
| Pt 6 : échéance souhaitée, réveil prématuré sans conséquence | Tolérance | Compatible, y compris pour une échéance restituée déjà passée : [À confirmer] |
| Pt 7 : la première erreur est conservée, état terminal | Erreur | Une itération non lancée (échec de publication) ou en erreur ne se compte pas comme exécutée et clôt la progression ; aucune décision de conservation ni d'abandon n'est prise pour une progression terminale |
| Pt 8 : progression demandée avant l'entrée en attente de la boucle | Moment d'appel | La lecture finale est la base la plus fraîche pour le besoin de réveil restitué |
| Pt 12 : détermination « une itération est-elle due à cet instant » évaluable sans attente | Exigence transmise à S04 | Le décompte des échéances dues à un instant en est la généralisation ; aucune attente |

**Divergence avec S02 : aucune démontrée.** Deux précisions à écrire dans l'ADR : la seconde observation de l'heure est permise ; une échéance restituée peut être déjà passée.

### 5.4 Conséquences sur le compte rendu et le prochain réveil

| Situation en fin de progression | Besoin de réveil restitué |
|---|---|
| Aucun résiduel | prochaine échéance planifiée (future) |
| Résiduel conservé (1 à M) | échéance déjà due : réveil immédiat [À confirmer : échéance restituée antérieure à l'instant de restitution] |
| Résiduel abandonné (> M) | prochaine échéance = t_fin + q (R1, § 5.6) |

Le nombre d'itérations exécutées vaut au plus K. « Aucune échéance » n'est jamais restitué à cause d'un retard : il reste réservé à l'absence de toute échéance (A4).

### 5.5 Interruption importante : conséquence explicite

Après une interruption de l'ensemble des progressions du Shell (veille, gel, autre), L_e peut être très grand :

1. La progression suivante exécute **K itérations** (chacune avec sa coupure et son lot, souvent vide), à partir d'un échéancier très en retard : l'abandon n'est décidé **qu'après** ces K itérations.
2. Elle observe t_fin et calcule le résiduel.
3. Si le résiduel dépasse M, le reliquat est abandonné et l'échéancier est réancré.

Il ne s'agit donc pas d'une interruption « sautée » : K itérations tardives sont exécutées avant l'abandon du reste. Si l'interruption est assez courte pour que le résiduel reste ≤ M, tout est conservé et rattrapé.

**Mesure à l'entrée contre mesure à la fin.**

| | Mesure à l'entrée (L_e comparé à un seuil) | Mesure à la fin (orientation) |
|---|---|---|
| Lectures de l'heure | une | deux |
| Décision d'abandon | avant d'exécuter, possible sans exécuter les K itérations | après les K itérations |
| Retard créé pendant l'exécution (b) | non vu avant la progression suivante | vu et compté |
| Sous surcharge durable | retard créé par l'exécution ignoré jusqu'à l'entrée suivante | pris en compte |
| Conformité à la lettre de S02 pt 4 | la plus directe | compatible, avec la précision § 5.3 |

La mesure à la fin est la seule qui contrôle le retard produit par l'exécution elle-même, qui est la composante dominante en surcharge. Son coût est une seconde lecture et l'exécution de K itérations tardives après une interruption.

### 5.6 Réancrage [Orientation : R1, non définitivement acquis]

**R1.** L'ancrage est l'instant auquel le Shell décide le réancrage (ici t_fin) ; la prochaine échéance vaut cet instant plus un quantum ; la phase de la grille précédente n'est pas conservée. La prochaine itération n'est pas immédiatement due.

**R2.** La prochaine échéance est la prochaine position de la grille initiale (phase conservée). Exige de conserver l'origine de l'échéancier et un calcul d'indice.

**Dépendances audio et réseau [lues en révision 4].** ADR-AU01 et ADR-N01 ont été lus intégralement. Dispositions pertinentes :
- **AU01 (pts 4 et 5)** : aucune intention ni commande audio ne bloque la boucle déterministe ; un échec côté plateforme ne remonte pas à une itération en cours ; le déterminisme ne dépend jamais du résultat d'une opération audio. Aucun instant, aucune période, aucune synchronisation avec l'horloge du périphérique, aucune exigence de latence. API, spatialisation, mixage : implémentation (pt 6).
- **N01 (pts 1, 4 et 6)** : chaque `World` reste local et déterministe ; le Shell garantit que chaque instance reçoit « la bonne séquence d'Acquisition » ; aucune donnée réseau ne modifie une itération en cours ; topologie, protocole, compensation de latence, fréquence d'un éventuel rollback : implémentation. Aucune exigence de phase, de numérotation de tick commune ni d'alignement d'échéanciers entre instances.

**Conclusion : aucune exigence de synchronisation ou de fidélité temporelle justifiant R2 n'est posée. R1 est maintenu comme choix proposé.** R2 n'apporte pas de bénéfice établi, pour quatre raisons :
1. R2 conserve la phase par rapport à l'origine **locale** de l'échéancier. Aligner plusieurs instances exigerait une référence temporelle commune, que ni R1 ni R2 ne fournissent et que N01 ne pose pas.
2. L'audio suit l'horloge de son périphérique, que AU01 ne décrit pas : conserver une phase locale ne l'aligne pas dessus, et AU01 n'exige aucun alignement.
3. Le résultat logique ne dépend pas de la phase : seuls les instants de réveil changent.
4. R2 coûte la conservation de l'origine et un calcul d'indice.

Ce choix reste révisable si un ADR futur pose une exigence de phase.

**Observations tirées de ces deux ADR (aucune n'est une exigence, aucune ne touche R1 ni R2).**
- **[Découle]** « La bonne séquence d'Acquisition » entre plusieurs instances suppose une unité de temps logique commune : cohérent avec le quantum unique de l'arbitrage (§ 2). N01 ne la définit pas ; S04 n'en tire aucune règle de réseau.
- **[Découle]** Un abandon est local et ne saute aucun tick logique ; mais le nombre d'itérations exécutées pour un même temps réel peut varier d'une instance à l'autre. La façon d'aligner les ticks entre instances relève de la topologie et de la compensation de latence (N01 pt 6), hors S04.
- **[Dép.]** AU01 pt 2 dit que le Shell reçoit des commandes déjà résolues, sans dire à quel moment il les applique. Pour une progression de plusieurs itérations, ce moment n'est pas déterminé : même question que A10 pour la Frame, à traiter avec l'extraction (R03 / S05). S04 ne la décide pas.

**Comportement de `FixedStepPacing`, suivi du cycle complet** (`pacing.rs`, `shell.rs`). `run_loop` appelle `cadence()` (soit `wait`) avant chaque itération.

1. Premier appel : pas d'échéance mémorisée ; échéance = instant de l'appel + intervalle ; sommeil jusqu'à cette échéance ; elle est mémorisée. La première itération a lieu après un intervalle complet.
2. Appels suivants, à l'heure : échéance = précédente + intervalle ; sommeil jusqu'à elle ; phase conservée, pas de dérive.
3. Appel en retard (échéance calculée ≤ instant de l'appel, intervalle nul compris) : aucun sommeil ; **l'itération en retard s'exécute immédiatement, une seule** ; l'instant de l'appel est mémorisé comme nouvelle base ; l'appel suivant calcule base + intervalle.

Correspondances avec R1 et différences :

| Point | `FixedStepPacing` | Politique F avec R1 (5.2) |
|---|---|---|
| Instant d'ancrage | instant de l'appel, **avant** l'exécution de l'itération en retard | t_fin, **après** les K itérations |
| Prochaine échéance | instant de l'appel + intervalle, donc plus tôt de la durée de l'itération | t_fin + q |
| Seuil | aucun : tout dépassement, même minime, réancre (M effectif nul) | résiduel > M, avec M ≥ 1 |
| Mesure | retard par rapport à l'échéance courante | nombre d'échéances résiduelles à t_fin |
| Attente | sommeil bloquant dans le pilote bloquant | échéance restituée dans le compte rendu |

Conclusion : `FixedStepPacing` est proche de « abandon systématique avec K = 1 et réancrage à l'instant de décision pris avant l'itération ». Ce n'est pas R1 tel que défini en F. Cette lecture résulte du code ; les tests lus (PC-1 à intervalle nul, PC-3 scripté, AQ-5) n'exercent pas le comportement en retard, et aucun test ne l'établit. Cette équivalence partielle ne confère aucun statut décisionnel à `FixedStepPacing` (A12).

## 6. Comparaison

Évaluation par raisonnement sur les textes, non par mesure. B et C sont des points de repère hors de F. D est une borne théorique. « c » : durée réelle d'une itération, inconnue (E14). Les expressions « K · c » sont des estimations de travail, non des bornes de durée : aucune borne sur c n'est démontrée.

**Débits.** ρ_arr = 1/q : débit d'arrivée des échéances. ρ_exec : débit d'exécution durable, c'est-à-dire le nombre d'itérations par unité de temps réel que le Shell soutient en continu, cycle complet de la boucle compris (retour à la boucle de plateforme, traitement des événements, éventuelle présentation bloquante), et non le seul coût d'une itération. Gigue et variations de charge font fluctuer ρ_exec autour de sa moyenne.

| Critère | B conservation (point de repère) | C abandon systématique (point de repère) | F à seuil (K, M finis, M ≥ 1) |
|---|---|---|---|
| Finitude (S02 pt 4) | oui, ≤ K itérations | oui | oui |
| Travail d'un appel | ≈ K itérations | ≈ K itérations | ≈ K itérations |
| Retard observé à l'entrée (a) | K itérations par progression, le reste conservé | K itérations puis abandon | K itérations, puis conservé ou abandonné selon le résiduel mesuré à la fin (§ 5.5) |
| Échéances devenues dues pendant l'exécution (b) | s'ajoutent au retard conservé | abandonnées avec le reste | comptées dans le résiduel mesuré à la fin |
| Convergence du rattrapage | seulement si ρ_exec > ρ_arr strictement ; la résorption se fait au rythme ρ_exec − ρ_arr | sans objet : rien n'est résorbé | résorbe si ρ_exec > ρ_arr strictement et résiduel ≤ M ; sinon abandons |
| ρ_exec = ρ_arr (égalité) | retard non résorbé, constant en moyenne, fluctuant avec la gigue | abandons à chaque dépassement | retard fluctuant ; abandons quand la gigue pousse le résiduel au-delà de M |
| ρ_exec < ρ_arr (surcharge durable) | retard croissant sans borne ; rattrapage futur d'autant | abandons répétés ; débit simulé = ρ_exec | résiduel monte jusqu'à M puis abandon, en dents de scie ; débit simulé = ρ_exec |
| Ce que borne M | sans objet | sans objet | le résiduel **à l'instant de mesure** (fin de progression) seulement. Pas le retard entrant ni le retard à tout instant : une interruption peut créer un retard arbitrairement grand avant la progression suivante. M n'est pas une garantie générale de retard maximal |
| Réactivité de l'event loop | K borne les itérations d'un appel, pas son temps réel ni la fréquence de retour à la boucle, qui dépendent de la plateforme (E1). Aucune garantie de faible latence. Retard important : longues suites d'itérations consécutives | idem, suites courtes | idem |
| Fidélité du temps simulé | suit le temps réel à terme si ρ_exec > ρ_arr ; sinon retard croissant | ralentit à chaque dépassement non résorbé | suit le temps réel tant que le résiduel mesuré reste ≤ M ; au-delà, le temps abandonné n'est pas simulé |
| Scripts `Wait` | logique inchangée (ticks) ; durée réelle fidèle à terme | durée réelle étirée à chaque abandon | fidèle sous M, étirée à chaque abandon |
| Interruption importante (600 s à q = 1/60 s : 36 000 échéances, à titre d'illustration) | avec K = 1 : 36 000 progressions successives d'une itération chacune, non une progression de 36 000 itérations ; travail total 36 000 itérations (phase accélérée perceptible) | K itérations puis abandon | K itérations tardives, puis abandon si le résiduel mesuré dépasse M (§ 5.5) |

**D (borne théorique).** Nombre d'itérations fini car fixé à l'instant observé (S02 pt 4), mais sans borne pratique sur le travail d'une progression, alors que la progression s'exécute de façon synchrone sur le fil de boucle (A9) : non candidate au premier pilote événementiel.

**Déterminisme.** Une même séquence de lots publiés (un lot par itération exécutée, lots vides compris : A7) donne la même séquence d'états (S03 pt 11). La séquence des événements non vides ne suffit pas à caractériser une exécution : le nombre d'itérations et les frontières entre lots, y compris vides, comptent. Les politiques de retard changent précisément cette trace ; elles ne se comparent donc pas sur l'égalité d'états finaux. Le recours au temps réel est confiné au nombre et aux frontières des itérations ; aucun ordre d'exécution interne n'en dépend (S01 pt 3 ; ADR-004). Un rejeu fidèle suppose d'enregistrer la trace complète des lots publiés, lots vides compris ; cela n'est pas du ressort de S04.

**Fondement.** Aucune exigence lue (corpus, ADR-AU01, ADR-N01) ne demande que le temps simulé suive le temps réel, ni qu'un instant ou une phase soit aligné avec un processus externe. Le choix entre familles repose donc sur des critères d'ingénierie (réactivité, convergence, accumulation du retard), non sur une exigence de fidélité. Une exigence future pourrait le modifier.

## 7. Interruption, suspension, non-présentabilité

Deux sujets distincts :
1. **Disponibilité de la présentation** : hors S04 ; R03 / S05 et cycle de vie de la plateforme.
2. **Progression du temps simulé pendant une interruption** : politique de S04.

**Principe retenu [Arbitré, 11 nov.].** La politique F est indépendante de la cause du retard : elle s'applique à toute situation produisant un retard constaté par le Shell (interruption du processus, veille système, autre). S02 ne prévoit aucune pause explicite : une absence de présentabilité ne déclenche pas, à elle seule, une pause. Si la boucle continue de solliciter le Shell, la simulation continue ; si elle cesse de le solliciter, le retard est traité par la politique temporelle au retour (§ 5.5). Aucune de ces conséquences n'est présentée comme garantie universelle : elles dépendent du comportement observé de la plateforme.

**Trois phénomènes à ne pas confondre.**

| Phénomène | Ce que voit le Shell |
|---|---|
| Suspension ou occlusion de la fenêtre | peut ne rien voir : la boucle continue de réveiller le Shell ; un signal n'est pas garanti sur desktop |
| Veille du système | un saut d'horloge, ou aucun saut selon que l'horloge monotone compte ou non la veille |
| Gel du processus | un saut d'horloge au prochain réveil, sans signal |

[Inconnu] La documentation de `Instant` ne garantit pas qu'il comptabilise la veille système ; à documenter par plateforme (E15), sans en faire une garantie portable.

**« Aucune échéance » n'est pas une pause** (A4). Une pause explicite exigerait un contrôle ou un état supplémentaire, absent de S02 : non établi, non décidé.

**Dépendance.** Qui signale suspension, reprise et non-présentabilité au Shell, par quel canal, et si un contrôle de pause existe : cycle de vie de la plateforme (R03 / S05). L'ADR énonce le fait suivant, sans décision de fond : sans canal de commande, l'effet observé d'une non-présentabilité qui laisse tourner la boucle est une poursuite de la simulation.

## 8. Expériences proposées (non exécutées, exploratoires)

Elles éclairent les choix d'implémentation et leurs conséquences ; elles ne sont pas des prérequis au contrat logique. Résultats à présenter par plateforme et environnement, jamais comme garantie portable.

| # | Objet | Résultat observable | Ce qu'elle éclaire |
|---|---|---|---|
| E1 | Ordre et fréquence des callbacks de la boucle de plateforme ; réveil sur échéance passée | Nombre et ordre des passages entre deux progressions ; traitement des événements entre elles | Réactivité réelle avec un retard conservé ; coût de réveil par progression |
| E2 | Blocage de l'acquisition ou de la présentation en synchronisation verticale sur le fil de boucle | Durée des blocages, effet sur l'échéance suivante | Retard d'origine externe, indépendant du coût de simulation ; composante de ρ_exec |
| E8 | Minimisation, occlusion, perte de focus | Signaux émis, cadence des réveils | Existence d'un signal exploitable ; effet sur l'échéancier |
| E10 | Précision et gigue du réveil programmé | Distribution du retard de réveil | Ordre de grandeur d'un retard « normal » ; calibrage de M |
| E14 | Coût réel d'une itération et débit d'exécution durable, par rapport au quantum | ρ_exec et c sur le démonstrateur | Éclaire les valeurs de K et de M ; dépend de la charge, de la machine et du scénario ; aucune garantie universelle |
| E15 | Horloge monotone à travers veille et gel | Retard mesuré après une interruption connue | Documente les plateformes étudiées ; ne définit pas la portabilité de l'horloge |

## 9. Scénarios de conformité (source de temps substituée, sans `sleep`)

- Avant l'échéance : aucune itération, aucune coupure, échéance restituée.
- À l'échéance exacte ; retard inférieur à un quantum ; retard de plusieurs quanta.
- Jamais plus de K itérations par progression.
- Deux lectures de l'heure par progression scriptées séparément (t_entrée, t_fin) ; échéances devenues dues pendant l'exécution comptées dans le résiduel.
- Frontière du seuil : résiduel = M conservé ; résiduel = M + 1 abandonné.
- Abandon : prochaine échéance = t_fin + q, non immédiatement due.
- Interruption longue : K itérations tardives exécutées avant l'abandon.
- Absence de dérive cumulative sur longue durée à retard nul.
- Arrêt et reprise ; état terminal ; échec de publication : aucune itération comptée, aucune décision d'abandon.
- Plusieurs progressions successives avec plusieurs lots : chaque itération coupe seule, aucun lot rejoué, lots vides compris, y compris avec un dépôt d'un autre fil entre deux coupures d'une même progression.
- Équivalence des pilotes bloquant et événementiel (S02 pt 3).
- Non-présentabilité : politique temporelle inchangée tant que la boucle sollicite le Shell.

## 10. Dépendances

- **R03 / S05** : transmission unique après la dernière itération (A10) ; cadence et répétition de présentation ; cycle de vie du Backend ; effet d'une présentation bloquante sur le temps réel observé.
- **Plateforme / cycle de vie** : signal de suspension, reprise, non-présentabilité ; éventuel contrôle de pause.
- **Forge et étape 5 (hors S04)** : porter le quantum ; règle des durées non entières ; toute évolution de `VMAN` sous G-CM.
- **ADR-AU01, ADR-N01** : lus ; aucune exigence temporelle. Dépendances résiduelles : moment d'application des commandes audio produites par plusieurs itérations d'une progression (extraction, R03 / S05) ; alignement des ticks entre instances (topologie, hors S04).

## 11. Réserves de rédaction

- **Famille.** B et C ne sont pas des membres de F (M fini et strictement positif) ; ils servent de repères de comparaison.
- **K et M ne sont pas des bornes de durée.** K borne des itérations par appel ; M borne le résiduel mesuré en fin de progression ; aucun n'est une garantie de latence ni de retard maximal.
- **Deux lectures de l'heure.** Compatibles avec S02 sans divergence démontrée ; à écrire comme précision dans l'ADR (§ 5.3).
- **`FixedStepPacing`.** Proche d'« abandon systématique, K = 1, réancrage avant l'itération » ; non équivalent à R1 de F ; établi par lecture, sans test (§ 5.6).
- **Neutralité de la présentabilité.** Sans canal de commande, l'absence de règle équivaut à une poursuite (§ 7) ; à énoncer comme fait.
- **Absence d'observabilité des abandons.** Le compte rendu de S02 n'en porte pas ; S04 ne l'ajoute pas.
- **Audio et réseau.** AU01 et N01 ne définissent ni instant, ni phase, ni tick commun ; R1 est maintenu faute d'exigence, et non parce qu'une exigence l'établirait.

## 12. Verdict de suffisance et décisions restantes

**La page est suffisamment déterminée pour rédiger ADR-S04.** Aucune ambiguïté de contrat ne bloque ; deux points sont à écrire explicitement :

1. **Précisions à écrire dans l'ADR, sans modifier S02** : la seconde observation de l'heure en fin de progression est permise (S02 pt 4 qualifie l'ensemble des itérations dues, pas le nombre de lectures) ; une échéance restituée peut être antérieure à l'instant de restitution.
2. **Réancrage** : ADR-AU01 et ADR-N01 lus, aucune exigence justifiant R2. R1 est proposé sans réserve de lecture, révisable si une exigence future apparaît.

Décisions à prendre :
1. Confirmer l'ensemble d'itérations fixé à l'entrée et la mesure de M à la fin (§ 5.2 et 5.3).
2. Confirmer qu'une échéance restituée peut être déjà passée (§ 5.4).
3. Confirmer R1 comme règle de réancrage proposée dans l'ADR (§ 5.6 : aucune exigence audio ou réseau en faveur de R2).
4. Niveau de R1 dans l'ADR : invariant structurel (prochaine échéance = instant de décision + q) ou règle de composition laissée à l'implémentation.
5. Confirmer que S04 ne pose aucune règle d'affectation des lots au-delà du cœur d'itération (§ 3).
