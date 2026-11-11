# ADR-N01 — Domaine Réseau

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
Cette ADR fait suite à une page d'axes ayant identifié trois zones de tension (convergence de plusieurs instances de simulation, autorité, déterminisme distribué), soumises à Gemini et GPT. Contrairement à Assets et Sauvegarde, la question posée n'était pas seulement « ce domaine s'instancie-t-il proprement ? » mais explicitement « le corpus actuel suffit-il, ou révèle-t-il une lacune réelle ? ». La rédaction ci-dessous teste, point par point, si chaque zone de tension se résout par composition des invariants existants.

## Décision

### 1. Quel est le contrat du domaine ?
Le domaine Réseau achemine des données entre plusieurs instances de simulation, sans jamais interpréter leur sens métier. Chaque instance de `World` reste une machine locale, entièrement aveugle à l'existence du réseau : son contrat demeure inchangé — état initial + séquence d'Acquisition → nouvel état, indépendamment de la provenance de cette séquence (clavier, fichier, ou socket).

**Résolution testée** : la « convergence » entre plusieurs instances n'est jamais une propriété du `World` — elle n'a donc pas à être définie comme un nouveau concept architectural. C'est une responsabilité d'orchestration : garantir que chaque instance reçoit la bonne séquence d'Acquisition. Cette responsabilité relève entièrement du Shell, déjà mandaté pour traduire des événements externes en données d'Acquisition (ADR-S01, point 2), sans qu'aucune extension de son contrat ne soit nécessaire.

### 2. Qui possède quoi ?
- **Le Shell** possède l'unique accès aux sockets et au transport réseau — déjà explicitement listé parmi les dépendances de plateforme confinées au Shell (ADR-S01, point 4). Rien de nouveau ici.
- **Le Shell** traduit les paquets reçus en données d'Acquisition, et les intentions réseau produites par des systèmes (données transitoires, produites librement — ADR-004) en paquets sortants. Instanciation directe de la traduction sans interprétation déjà actée (ADR-S01, point 2) et du principe de circulation (`02-concepts.md`).
- **Le `World`** reste seul créateur d'`EntityId`, y compris pour une entité dont l'existence est signalée par le réseau — remapping identique à celui déjà acté pour la Sauvegarde (ADR-SV01) : un identifiant stable et local à la source externe est traduit en `EntityId` par le `World`, jamais copié directement.
- **L'autorité sur une donnée** ne requiert aucun nouveau propriétaire architectural. Elle se résout par composition de deux mécanismes déjà existants :
  - une politique du Shell : quelles données réseau il traduit en Acquisition valide, lesquelles il rejette ;
  - un composant métier ordinaire (ex: une capacité comme `LocalPlayer` ou `RemoteReplicated`, cohérent avec la composabilité par capacité déjà actée en `01-runtime-ecs.md`), consulté par le système propriétaire unique en écriture d'une donnée donnée — le principe de propriétaire unique (`02-concepts.md`) reste strictement intra-phase, inchangé.

**Résolution testée** : aucun nouveau propriétaire n'a été nécessaire pour répondre à « qui décide ? ».

### 3. Quelles frontières s'appliquent ?
Les trois frontières déjà nommées (`02-concepts.md`) suffisent :
- **Frontière de connaissance** : le `World` ignore complètement le réseau — aucun système n'est « conscient du réseau » ; l'information d'autorité ou de provenance transite uniquement par des composants ordinaires (cf. point 2).
- **Frontière de strate** : le transport réseau est Plateforme (Shell) ; la traduction en Acquisition et la production d'intentions réseau sont Modèle architectural ; le format de sérialisation sur le fil est Structure interne, opaque aux systèmes.
- **Frontière de compilation** : sans objet — la Forge n'intervient pas dans ce domaine, comme pour la Sauvegarde.

**Résolution testée** : aucune quatrième frontière (« frontière d'autorité ») n'a été nécessaire. Ce qui semblait, dans la page d'axes, requérir une nouvelle catégorie de séparation se résout entièrement par composition d'une politique de Shell et d'un composant métier ordinaire.

### 4. Quelles garanties sont offertes ?
- Chaque instance de `World` reste localement déterministe, sans exception : même contrat qu'en `00-principes.md`, quelle que soit la source de la séquence d'Acquisition.
- Aucune donnée reçue du réseau ne modifie une itération en cours — instanciation directe du principe déjà généralisé « aucune infrastructure ne peut modifier une itération logique en cours » (ADR-S01, point 6). Toute donnée réseau reçue devient une nouvelle entrée d'Acquisition d'une itération future.
- Un mécanisme de réconciliation ou de rollback, s'il est utilisé, n'instancie pas ADR-SV01 elle-même — il partage avec la Sauvegarde un même motif architectural plus abstrait : **extraction d'un état logique, puis reconstruction via le `World`**. La Sauvegarde et le rollback réseau sont deux instanciations distinctes de ce motif commun, avec des paramètres différents (médium durable vs éphémère, fréquence rare vs continue), pas une relation de dépendance de l'un vers l'autre. Cette distinction a nécessité une clarification croisée d'ADR-SV01 (médium et fréquence explicitement reclassés comme paramètres d'implémentation du motif, non comme propriétés intrinsèques de la Sauvegarde) pour que le motif reste correctement partageable sans laisser croire que le rollback est une forme de sauvegarde.

**Résolution testée** : le déterminisme distribué ne s'est pas révélé être une propriété distincte à définir. La question s'est reformulée, comme le proposait GPT, en « le déterminisme est-il une propriété locale de chaque `World` ou une propriété globale d'un ensemble de `World` ? » — et la réponse, dans le cadre de ce contrat, reste locale : le Shell garantit uniquement que chaque instance reçoit la bonne séquence d'Acquisition, jamais une coordination qui échapperait à ce canal.

### 5. Qu'est-ce qui est explicitement exclu ?
- Toute interprétation métier des données transmises par le domaine Réseau lui-même (frontière de connaissance) ;
- toute connaissance du layout mémoire interne d'une instance distante (frontière modèle/structure interne) ;
- toute mutation directe d'une instance de simulation par une autre sans passer par le cycle Acquisition local ;
- toute notion d'autorité codée comme un mécanisme du `World` plutôt que comme composition Shell + composant métier (cf. point 2).

### 6. Quelles questions sont volontairement laissées à l'implémentation ?
Topologie (autoritaire centralisée, pair-à-pair...), protocole de transport (TCP, UDP, QUIC, WebSocket...), format de sérialisation sur le fil, stratégie de compensation de latence (prédiction, interpolation), fréquence et médium concrets d'un éventuel mécanisme de rollback, granularité de la réplication (état complet vs delta).

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Le `World` reste local, déterministe, aveugle au réseau | Invariant (inchangé depuis `00-principes.md`) |
| Le Shell possède le transport réseau et traduit sans interpréter | Invariant (instanciation d'ADR-S01) |
| L'autorité se résout par politique Shell + composant métier, sans nouveau propriétaire | Invariant (instanciation de `01-runtime-ecs.md` et `02-concepts.md`) |
| Aucune donnée réseau ne modifie une itération en cours | Invariant (instanciation d'ADR-S01, point 6) |
| Rollback/réconciliation et Sauvegarde sont deux instanciations d'un même motif (extraction → reconstruction via le `World`), pas une dépendance de l'un vers l'autre | Invariant (motif commun, ADR-R01/02 et ADR-SV01 clarifiée en conséquence) |
| Topologie, protocole, sérialisation sur le fil, compensation de latence | Hors périmètre — implémentation |

## Bilan
Les trois zones de tension identifiées par la page d'axes se résolvent entièrement par composition des invariants existants — confirmant la prédiction de GPT plutôt que mon hypothèse initiale d'un invariant manquant. Le seul ajustement nécessaire n'était pas une extension du langage architectural, mais une clarification croisée d'ADR-SV01 (généraliser le médium et la fréquence de l'extraction/composition), effectuée en même temps que cette ADR. Le domaine Réseau s'intègre donc au corpus sans inflation conceptuelle, au prix d'une seule correction rétroactive mineure sur un document déjà adopté.
