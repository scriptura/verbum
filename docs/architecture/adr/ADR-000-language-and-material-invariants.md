# ADR-000 — Langage d'implémentation et vérification statique des invariants

## Statut
Adopté le 10 octobre 2026. Rédigée le 25 août 2026. Révision de cohérence : 10 octobre 2026 (alignement sur `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md` et ADR-001 à ADR-Sc01 ; prise en compte de deux audits externes).

Points ouverts connus, non bloquants pour l'adoption : la forme de l'interface de franchissement de `world.execute()` et la relation entre `renderer.compose()` et la composition interne au modèle (point 3) restent à arbitrer avant l'intégration du Shell et du Renderer ; aucune alternative à Zig n'a été évaluée (Contexte).

## Contexte
Cette ADR est issue du draft `rust-vs-zig.md`. Elle est numérotée 000 parce qu'elle précède logiquement le corpus, mais elle a été rédigée après lui. Conformément au statut de `00-principes.md` (en cas de conflit apparent, c'est la nouvelle décision qui est reformulée), elle s'aligne sur les ADR existantes et n'en modifie aucune.

Le corpus pose des invariants qui ne sont pas, par eux-mêmes, vérifiés par quoi que ce soit :
- zéro allocation sur le hot path, et non au sens absolu (`00-principes.md`, point 4) ;
- déterminisme de la simulation (`00-principes.md`, point 3) ;
- absence de parsing de formats humains au runtime (`00-principes.md`, point 1) ;
- frontières de connaissance entre domaines (`00-principes.md`, principe des frontières de connaissance).

Cette ADR répond à une seule question : ces invariants sont-ils tenus par convention, ou vérifiés par la chaîne de build ? Elle retient la seconde réponse partout où elle est atteignable, et enregistre Rust comme instanciation actuelle. L'engagement de vérifier statiquement ces invariants est propre à cette ADR : aucune ADR antérieure ne le formule. Il est limité aux propriétés effectivement vérifiables.

Conformément à la règle méta-architecturale, l'invariant est la vérification statique ; le langage, `no_std` et le système de visibilité sont des mécanismes. Le choix de Rust est néanmoins consigné ici comme décision structurante, car il conditionne tout le code à venir.

Limite héritée du draft : la seule alternative évaluée est Zig, et la comparaison tient en une phrase. Aucune autre alternative n'a été évaluée.

## Décision

### 1. Quel est le contrat du domaine ?
Fournir une chaîne de build (langage et organisation du code) dans laquelle les invariants structurels du corpus sont vérifiés statiquement plutôt que tenus par convention, dans les limites précisées au point 4.

Le **noyau** désigne, dans cette ADR, l'ensemble du code atteignable depuis `world.execute()` — le point d'invocation que le Shell traite comme une boîte noire (ADR-S01, point 3) : `World`, stockages, Queries, systèmes, Command Buffer, modèle d'exécution construit — à l'exception de l'implémentation de la capacité d'exécution décrite à la garantie 2, qui est du code de la Plateforme. Le noyau est défini par son graphe d'appel, non par une strate : le `World` en tant qu'exécutant relève de la Plateforme (`02-concepts.md`), son contenu relève du Modèle architectural.

L'atteignabilité statique est une approximation du hot path (`00-principes.md`, point 4) : elle est plus large que l'exécution effective, puisqu'elle inclut les branches rarement empruntées. Cette rigueur supplémentaire est assumée, car seule l'atteignabilité est vérifiable statiquement.

Deux ensembles sont distingués :
- **Le périmètre du noyau** : le graphe d'appel défini ci-dessus. C'est le seul ensemble sur lequel la garantie 1 est vérifiée statiquement.
- **Le périmètre de l'invariant d'allocation** : tout code du hot path (`00-principes.md`, point 4), y compris, le cas échéant, du code situé hors du graphe d'appel de `world.execute()`. Hors du noyau, l'invariant s'applique mais n'est exigé que par contrat et vérifié par revue (point 4, limite de preuve).

Trois garanties constituent le contrat :
1. **Aucune allocation dynamique pendant l'exécution de `world.execute()`.** Elle est vérifiée statiquement sur le noyau ; sa limite de preuve est précisée au point 4.
2. **Aucun accès direct du noyau aux API de plateforme** (horloge, système de fichiers, threads, sockets). Le noyau ne peut pas les nommer. Il peut en revanche recevoir de la Plateforme une capacité d'exécution abstraite pour le parallélisme intra-système (`01-runtime-ecs.md`, section 5), qui n'expose ni thread, ni OS, ni cycle de vie. Le code qui implémente cette capacité appartient à la Plateforme ; il est invoqué pendant `world.execute()` et reste soumis à la garantie 1 (point 4).
3. **Frontières de connaissance non franchissables à la compilation**, pour celles qui s'expriment comme « le domaine X ne peut pas nommer Y ».

Cette ADR ne garantit pas le déterminisme de la simulation. Elle retire des sources de non-déterminisme au noyau (garantie 2). Le déterminisme lui-même reste porté par `00-principes.md` (ticks fixes, ordre d'exécution fixe, ADR-003 point 4, ADR-004). Le déterminisme numérique des flottants n'est traité ni ici ni dans `00-principes.md` au-delà de l'exclusion du `deltaTime`.

### 2. Qui possède quoi ?
- **Le Shell** possède la construction du noyau et la fourniture de sa mémoire initiale (capacités fixées avant la simulation — ADR-001, point 5). Il possède toute dépendance de plateforme (ADR-S01, point 4), y compris les threads, et fournit, s'il y a lieu, la capacité d'exécution abstraite du point 1. Il est autorisé à utiliser `std` et l'allocation.
- **Le noyau** ne possède aucune ressource de plateforme ni aucun allocateur. Il peut utiliser, sans la posséder, la capacité d'exécution que la Plateforme lui fournit.
- **L'infrastructure de chargement** (assemblée par le Shell — ADR-A01, point 2) possède l'accès disque et le cast zéro-copie des artefacts en Ressources.
- **La Forge** (build-time) possède la production des artefacts dont le layout est partagé avec le runtime, ainsi que la génération de code éventuelle (ADR-002, point 2).
- **La chaîne de build** (compilateur, analyse du graphe de dépendances, lints) n'est propriétaire d'aucune donnée. Comme la Forge, elle est extérieure au système qu'elle produit (`02-concepts.md`, note sur la Forge) : elle vérifie, elle ne possède pas.

### 3. Quelles frontières s'appliquent ?
- **Frontière de compilation (Forge ↔ runtime).** Le runtime ne parse aucun format de configuration humain (`00-principes.md`, point 1 ; ADR-A01). Cette règle porte sur les formats humains. La lecture au runtime d'une représentation binaire qui n'est pas produite par la Forge (reconstruction d'une sauvegarde — ADR-SV01 ; données réseau — ADR-N01) n'est pas tranchée ici ; elle relève de ces ADR, et le cast zéro-copie ne lui est pas applicable par défaut. Un artefact de la Forge dont le layout ne correspond pas à celui attendu par le runtime est rejeté au chargement, hors hot path. Il s'agit d'une vérification de conformité de format, non d'une validation de cohérence métier.
- **Frontière de strate — point de passage : l'invocation de l'infrastructure `World` par le Shell** (ADR-S01, point 3). Les flux qui la traversent obéissent au principe de circulation (`02-concepts.md`) : données d'Acquisition en entrée ; données d'Extraction (`01-runtime-ecs.md`, section 5) en sortie. La forme exacte de cette interface n'est pas fixée par le corpus et n'est pas tranchée ici : en particulier, la représentation de frame peut être retournée par `world.execute()`, lue comme Ressource après son retour, ou produite par un appel distinct (`renderer.compose()`, ADR-S01 point 3, à rapprocher de la composition interne au modèle, ADR-R02). Le périmètre du noyau (point 1) reste défini par le seul graphe d'appel de `world.execute()` ; il ne dépend pas de cette question. Seul le périmètre de l'invariant d'allocation (point 1) en dépend : si la composition de la représentation de frame s'exécute hors de `world.execute()`, le code qui la réalise relève du hot path (`00-principes.md`, point 4) et est soumis à l'invariant, par contrat et revue, sans entrer dans le noyau. Cette ADR ne tranche pas. Cette question reste un point ouvert distinct, à arbitrer avant l'intégration du Shell et du Renderer ; elle n'est pas un obstacle à l'adoption de cette ADR. Cette ADR exige seulement que les données qui franchissent la frontière ne contiennent ni propriété partagée ni référence à un type de la Plateforme. Leur représentation concrète (layout, aplatissement) n'est pas prescrite et reste à l'implémentation (cf. ADR-R02 pour la représentation de frame). Cette exigence n'est que partiellement vérifiable par les types ; le résidu relève de la revue (cf. point 4).
- **Frontière de connaissance.** Tout interdit de la forme « X ne peut pas nommer Y » (le noyau ne nomme pas le Shell ; le Backend ne nomme ni entité, ni composant, ni système — ADR-R03, point 1) est encodé dans le graphe de dépendances et la visibilité, de sorte qu'une violation ne compile pas. Les interdits qui relèvent d'un contrat (un système ne lit que ce que son interface déclarative expose — ADR-003 ; un seul propriétaire en écriture par donnée persistante) ne s'encodent pas par la seule visibilité ; leur vérification relève d'ADR-002 à ADR-004 et de l'outillage associé.

### 4. Quelles garanties sont offertes au reste du moteur ?
- **Allocation.**
  - *Invariant exigé* : aucune allocation dynamique pendant l'exécution de `world.execute()`.
  - *Vérification* : sur le noyau, par compilation sans bibliothèque d'allocation, audit de l'arbre de dépendances et examen des blocs `unsafe` (cf. table ci-dessous).
  - *Limite de preuve* : la compilation ne démontre pas l'absence d'allocation dans du code qu'elle ne peut pas vérifier — blocs `unsafe`, appels externes, implémentation de la capacité d'exécution fournie par la Plateforme, code du hot path éventuellement situé hors du noyau (point 3). Pour ces derniers cas, l'absence d'allocation est exigée par contrat et vérifiée par revue.
  - Tout ce qui doit allouer s'exécute hors de `world.execute()` — chargement, streaming, accès au médium de sauvegarde — via le principe de circulation : intention transitoire, traitement par la Plateforme, résultat en Acquisition d'une itération future (ADR-A01, point 5 ; ADR-SV01).
- **Capacités fixées avant simulation** : un dépassement de capacité est un bug de configuration, jamais une croissance dynamique (ADR-001, point 5). Un système de reconstruction ou de génération atteignable depuis `world.execute()` écrit dans des capacités préallouées : ses mutations structurelles passent par le Command Buffer (ADR-006) ; ses écritures non structurelles respectent le contrat déclaré du système et la propriété unique en écriture.
- **Preuve conditionnelle** : ce que le compilateur vérifie l'est sous réserve des blocs `unsafe`. Ceux-ci sont bornés, énumérables et confinés aux infrastructures (stockage, cast d'artefacts) ; le code des systèmes n'en contient pas.
- **Parallélisme intra-système** : s'il existe, il passe par la capacité d'exécution abstraite du point 1, fournie par la Plateforme (ADR-S01, point 4). Le noyau n'acquiert jamais lui-même de threads, et le parallélisme ne modifie pas le résultat déclaré d'un système (ADR-003, points 4 et 5).
- **Interpréteurs de script** : un système interpréteur (ADR-Sc01) est un système ordinaire du noyau. Son état d'exécution est externalisé dans des composants ou Ressources de capacité préallouée ; il est donc soumis à la garantie sans allocation.

**Instanciation actuelle (Rust) — décision d'implémentation.**

| Garantie | Vérifiée par | Résidu non couvert |
|---|---|---|
| Pas d'allocation dans le noyau | Crate noyau en `#![no_std]`, sans `extern crate alloc`, arbre de dépendances audité au build, blocs `unsafe` examinés | `unsafe` ; appels externes ; dépendances mal auditées ; implémentation de la capacité d'exécution (contrat et revue) |
| Pas d'accès direct à l'horloge, aux fichiers, aux threads | Absents de `core` | Comportement de la capacité d'exécution fournie par la Plateforme |
| Frontières « X ne nomme pas Y » | Graphe de dépendances des crates, visibilité (`pub(crate)`, types opaques, traits scellés) | Contrats ADR-003 (hors visibilité) |
| Layout des données franchissant les frontières | Représentation explicite, assertions de taille et d'alignement au build | Valeurs invalides, endianness, version : vérifiées au chargement |
| Sûreté des accès | Systèmes sans `unsafe` | Blocs `unsafe` des infrastructures |

Précisions sur cette instanciation :
- `no_std` seul n'interdit pas l'allocation : le crate `alloc` reste utilisable explicitement dans du code `no_std`. C'est l'absence de `alloc` dans tout l'arbre de dépendances du crate noyau qui retire les API d'allocation usuelles. Rust stable ne fournit aucun attribut `no_alloc`.
- Dans un même processus, la convention d'appel `extern "C"` n'est pas nécessaire. Un layout explicite n'est requis que pour les données dont le format binaire est partagé entre la Forge et le runtime ; `#[repr(C)]` en est une instanciation possible, le choix concret relevant du point 6. Ce n'est pas une exigence de toute frontière du moteur. La C-ABI ne garantit pas l'étanchéité sémantique : un pointeur opaque peut transporter n'importe quoi, c'est le typage et la revue qui l'interdisent.

### 5. Qu'est-ce qui est explicitement exclu du périmètre ?
- **L'allocation dynamique et `std` dans le noyau.** La Plateforme (Shell, infrastructures de chargement, Backend, transports) et tout code ni atteignable depuis `world.execute()` ni situé sur le hot path peuvent utiliser `std` et l'allocation, conformément à `00-principes.md`, point 4.
- **La résolution dynamique d'une Query à partir de métadonnées de composants au runtime** (effacement de type appliqué à la forme d'une requête). Elle contredit ADR-002, point 2 (spécialisation statique, aucune résolution « au vol ») et point 3 (mode d'accès dans le type). La préoccupation des temps de compilation (explosion de monomorphisation) est légitime ; elle relève de la mécanique de spécialisation (monomorphisation ou génération de code par la Forge, ADR-002, point 2), pas d'un effacement de type. Le mécanisme par lequel les stockages sont localisés reste celui laissé ouvert par ADR-005.
- **Toute dépendance du noyau à une bibliothèque généraliste de sérialisation.** Décision d'implémentation, non invariant. Trois raisons distinctes, qui ne se confondent pas : l'interdiction de parser des formats humains au runtime est déjà posée (`00-principes.md`, point 1) et ne concerne pas la sérialisation en général ; toute dépendance du noyau élargit l'arbre à auditer pour la garantie sans allocation ; le noyau consomme des Ressources au layout déjà fixé (ADR-A01, point 3) et n'a pas à les désérialiser.
- **Le choix d'un écosystème de crates comme critère de décision.** Le choix de Rust ne valide pas l'usage de l'écosystème généraliste du langage ; il exploite sa capacité à imposer des contraintes vérifiables.
- **Le déterminisme numérique des flottants** (cf. point 1).

**Alternative écartée — Zig** (seule alternative évaluée). Le rejet porte sur une propriété précise : l'encapsulation, vérifiée par le compilateur, du contenu d'un handle valeur tel qu'`EntityId` (`01-runtime-ecs.md`, section 1), dont les accesseurs sont privés. Zig propose des types `opaque {}`, de taille inconnue, qui servent notamment à la sûreté de types dans les interfaces manipulant des pointeurs ; ils ne remplacent pas un handle valeur de taille connue. Selon le guide de style de la documentation officielle de Zig 0.16.0, le langage ne comporte pas de champs privés (vérification effectuée lors de l'audit externe du 10 octobre 2026, à partir de cette source). Portée de ce constat : ce mécanisme précis — encapsulation imposée par le compilateur du contenu d'un handle valeur — n'est pas disponible en Zig comme il l'est en Rust. Il n'en découle pas qu'un moteur ECS correct soit impossible en Zig, et ce constat ne tranche pas à lui seul l'ensemble de la comparaison. Le rejet ne porte pas sur les performances : l'allocateur explicite et `comptime` de Zig sont des atouts reconnus et ne sont pas écartés pour des raisons de performance.

### 6. Quelles questions sont volontairement laissées à l'implémentation ?
- Le mécanisme concret de vérification de l'absence d'allocation (audit de dépendances, lints, outillage).
- L'organisation en crates et leur nommage.
- Le layout concret des structures partagées (`#[repr(C)]` ou autre), la bibliothèque de cast (`bytemuck` ou autre), le versionnement du layout entre Forge et runtime.
- L'interface et l'implémentation de la capacité d'exécution abstraite par laquelle la Plateforme fournit du parallélisme intra-système au noyau.
- La forme exacte de l'interface de franchissement de `world.execute()` (cf. point 3).
- Le mécanisme de spécialisation statique des Queries (monomorphisation ou génération de code) et l'outillage de vérification des contrats ADR-002/003.
- La politique de résidence mémoire des fichiers mappés (un défaut de page peut allonger le temps de frame sans affecter le déterminisme logique).
- Le lieu d'exécution de la génération procédurale : le corpus ne le fixe pas ; s'il s'agit d'un système atteignable depuis `world.execute()`, il est soumis à la garantie sans allocation.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Les invariants du corpus sont vérifiés par la chaîne de build partout où c'est atteignable, non tenus par convention | Invariant — engagement propre à cette ADR (aucune ADR antérieure ne le formule), limité aux propriétés vérifiables statiquement |
| Aucune allocation dynamique pendant l'exécution de `world.execute()` (noyau vérifié statiquement ; capacité fournie par la Plateforme vérifiée par contrat et revue) | Invariant (instanciation de `00-principes.md`, point 4) |
| Le noyau n'accède à aucune API de plateforme ; il peut recevoir une capacité d'exécution abstraite | Invariant (instanciation d'ADR-S01, point 4) |
| Les interdits « X ne peut pas nommer Y » ne compilent pas | Invariant (instanciation de la frontière de connaissance) |
| Les données franchissant la frontière du `World` ne contiennent ni propriété partagée ni référence à un type de la Plateforme ; leur représentation n'est pas prescrite | Invariant (instanciation du principe de circulation et des frontières de connaissance), partiellement vérifiable ; représentation hors périmètre |
| Périmètre du noyau (graphe d'appel de `world.execute()`) distinct du périmètre de l'invariant d'allocation (hot path) | Invariant (précision de vocabulaire) ; extension éventuelle hors noyau vérifiée par contrat et revue |
| La chaîne de build n'est propriétaire d'aucune donnée (extérieure au système, comme la Forge) | Invariant (instanciation de `02-concepts.md`) |
| Le code des systèmes n'exige aucun `unsafe` ; `unsafe` est confiné aux infrastructures | Décision révisable, réévaluée par mesure |
| Aucune résolution dynamique de la forme d'une Query | Invariant (instanciation d'ADR-002) |
| Rust comme langage d'implémentation | Décision d'implémentation structurante |
| Exclusion des bibliothèques généralistes de sérialisation dans le noyau | Décision d'implémentation |
| `no_std` sans `alloc`, audit de dépendances, `pub(crate)`, traits scellés | Hors périmètre — implémentation |
| `#[repr(C)]`, `extern "C"`, bibliothèque de cast, versionnement de layout | Hors périmètre — implémentation |
| Interface de la capacité d'exécution, politique de résidence mémoire, lieu de la génération procédurale | Hors périmètre — implémentation |
| Forme exacte de l'interface de franchissement de `world.execute()` | Hors périmètre — non fixée par le corpus |
| Déterminisme numérique des flottants | Hors périmètre — non traité |

## Bilan — test à cinq questions

| Question | Réponse |
|---|---|
| Que possède ce domaine ? | Rien au runtime : il fixe des contraintes de build. La chaîne de build vérifie ; le Shell et la Forge possèdent ce qui leur est déjà attribué. |
| Que produit-il ? | Des garanties vérifiées statiquement, pas de donnée runtime. |
| Qui consomme cette production ? | Tous les domaines, en tant que contraintes de compilation sur leur propre code. |
| Que lui est-il interdit de connaître ? | La sémantique métier et la structure du modèle architectural : il ne définit aucun composant, système ni donnée. |
| Quelle frontière protège cette ignorance ? | La frontière de compilation : comme la Forge, ce domaine est extérieur à la grille des strates. |

Cette ADR n'introduit aucun concept nouveau : ni type d'état, ni modèle d'exécution, ni frontière au-delà des trois déjà nommées. Ses garanties sur l'allocation, l'accès à la plateforme et les frontières instancient des invariants existants (`00-principes.md` points 1, 3 et 4, ADR-002, ADR-S01). L'engagement de les vérifier statiquement est en revanche propre à cette ADR, et limité à ce qui est effectivement vérifiable. Par rapport au draft d'origine, elle réduit la portée de l'interdiction d'allouer au code atteignable depuis `world.execute()`, retire l'effacement de type des questions laissées ouvertes et fait de `repr(C)` une instanciation plutôt qu'un invariant.
