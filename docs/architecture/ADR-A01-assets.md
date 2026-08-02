# ADR-A01 — Domaine Assets : invariants

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
Cette ADR fixe le contrat du domaine Assets — ce qu'il est autorisé à faire, ce qui lui est interdit de connaître, et par quelles frontières il communique avec le reste du moteur. Elle instancie le vocabulaire déjà établi (Forge et frontière de compilation — `02-concepts.md` ; Ressource — ADR-005 ; principe de circulation — `02-concepts.md` ; assemblage par le Shell — ADR-S01) plutôt que de le redéfinir. Le format concret des fichiers, l'organisation des répertoires, ou le layout du manifeste restent hors périmètre — objets d'une future ADR d'implémentation, sans conséquence sur les invariants ci-dessous.

## Décision

### 1. Ce qu'est un asset — et ce qui n'en est pas un
Un asset est une donnée statique, produite hors du moteur (image, échantillon audio, disposition de niveau, graphe de dialogue, table d'équilibrage...), qui devient, après passage par la Forge, une Ressource (ADR-005) directement consommable au runtime.

N'est **pas** un asset :
- une donnée d'état dynamique produite pendant la partie (relève du domaine Sauvegarde, non traité ici) ;
- une donnée générée procéduralement au runtime (aucune source produite hors moteur, donc aucun passage par la Forge) ;
- une configuration déterminée par la plateforme ou le Shell (résolution d'écran, chemins système...), qui relève d'ADR-S01, pas du domaine Assets.

### 2. Responsabilités exclusives du domaine Assets
Deux acteurs, jamais confondus, cohérents avec `00-principes.md` :
- **la Forge** (build-time) : seule autorisée à parser des formats de configuration humains (YAML, JSON...), à valider la cohérence des données de design, et à produire des artefacts binaires ou du code généré ;
- **le chargement runtime** (infrastructure de plateforme, assemblée par le Shell — ADR-S01, point 7) : seul autorisé à effectuer l'accès disque réel (mmap, lecture de fichier) et à produire les Ressources correspondantes par cast zéro-copie.

Aucun système métier ne parse un format humain, ni n'effectue d'accès disque directement — ces deux responsabilités restent confinées à ces deux acteurs.

### 3. Artefacts produits à la compilation
La Forge produit des représentations binaires directement interprétables par le runtime, sans étape de parsing — cohérent avec l'invariant AOT déjà posé en `00-principes.md`. Les garanties de layout mémoire concrètes (aujourd'hui obtenues via `#[repr(C)]` en Rust) relèvent de l'implémentation ; l'invariant architectural est l'absence de parsing, pas une convention de langage particulière. Elle produit également le manifeste qui permet au runtime d'adresser chaque Ressource via un **identifiant opaque**, résolu avant le hot path — cohérent avec la préallocation par capacité déjà actée pour les composants (ADR-001, point 5), étendue ici aux Ressources d'asset. La représentation concrète de cet identifiant (entier, hash, couple pack/offset, handle compressé...) est une décision d'implémentation ; l'invariant est que sa forme reste opaque au runtime et qu'aucune chaîne de caractères ni chemin de fichier ne franchit la frontière de compilation.

### 4. Garanties offertes au runtime
- **Absence de propriétaire d'écriture au runtime** : une Ressource d'asset n'a aucun propriétaire d'écriture pendant l'exécution. La Forge produit son contenu ; le chargement runtime (infrastructure assemblée par le Shell) la charge et la libère ; les systèmes ne font que la lire. Aucun acteur runtime — système métier, éditeur intégré, ou autre — ne modifie son contenu une fois chargée. Ceci élimine toute ambiguïté sur un futur outil (éditeur en jeu, hot-reload) qui tenterait de modifier une texture ou une table directement en mémoire runtime plutôt que de repasser par la Forge.
- **Adressage stable** : par clé unique, comme toute Ressource (ADR-005), résolue au chargement, jamais recalculée sur le hot path.
- **Absence de parsing au runtime** : aucune structure de configuration humaine n'est interprétée après la Forge — cohérent avec `00-principes.md`.
- **Absence de découverte dynamique** : l'ensemble des assets utilisés par le jeu est entièrement connu à la compilation par la Forge ; le runtime ne scanne jamais un système de fichiers pour découvrir quels assets existent.

### 5. Interfaces autorisées avec les autres domaines
- **Vers la Forge** : aucune interface runtime — séparée par la frontière de compilation (`02-concepts.md`), elle n'existe pas pendant l'exécution du moteur.
- **Vers les systèmes** : un asset chargé est une Ressource ordinaire, consommée selon le contrat déjà posé en ADR-003/ADR-005 — aucune règle de dépendance ou de contrat distincte.
- **Vers le Shell** : le domaine Assets est une infrastructure de plateforme parmi celles que le Shell assemble (ADR-S01, point 7) — l'accès disque, en tant que dépendance de plateforme, reste confiné à cette infrastructure, jamais accessible directement par un système.
- **Streaming en cours de partie** : instancie directement le principe de circulation (`02-concepts.md`). Un système émet une intention de chargement (donnée transitoire, produite librement — ADR-004) ; l'infrastructure Assets la traite de façon asynchrone, en tant que dépendance de plateforme, hors de toute itération logique en cours (ADR-S01, point 6) ; le résultat (Ressource chargée et prête) ne revient jamais dans l'itération en cours — il devient une nouvelle donnée d'Acquisition d'une itération future. Aucune promesse, aucun callback, aucune corrélation garantie entre l'intention et le fait acquis.

- **Déchargement — dual symétrique du chargement** : le déchargement d'un asset instancie le même principe de circulation, dans l'autre direction du cycle de vie. Un système émet une intention de libération (donnée transitoire, produite librement) ; l'infrastructure Assets la traite de façon asynchrone ; la disparition effective de la Ressource n'est visible qu'à une itération future, jamais pendant l'itération en cours. Ceci ne constitue pas une mécanique supplémentaire : c'est la même preuve que le principe de circulation couvre nativement les deux sens du cycle de vie d'une Ressource d'asset, sans qu'aucun invariant nouveau n'ait dû être introduit pour le second sens.

### 6. Décisions explicitement rejetées
- **Chargement implicite** : aucun mécanisme n'infère automatiquement qu'un asset doit être chargé à partir de la présence d'un composant ou d'une heuristique quelconque — toute demande de chargement est une intention explicite émise par un système.
- **Scan récursif de répertoires au runtime** : rejeté — l'existence de tout asset est déterminée exhaustivement par la Forge à la compilation, jamais découverte en explorant un système de fichiers pendant l'exécution.
- **Résolution par nom (réflexion) sur le hot path** : rejetée — l'adressage runtime passe exclusivement par les clés et index générés par la Forge, jamais par une recherche de chaîne de caractères pendant la simulation. Un mécanisme de résolution par nom pour l'outillage de développement (éditeur, hot-reload) n'est ni actée ni rejetée par cette ADR : s'il existe, il reste hors du hot path et hors périmètre architectural, à documenter séparément.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Un asset devient une Ressource après passage par la Forge | Invariant |
| Seule la Forge parse des formats humains ; seul le chargement runtime fait l'accès disque | Invariant |
| Aucun parsing, aucune recherche par nom sur le hot path | Invariant |
| Adressage runtime par identifiant opaque résolu avant le hot path ; représentation concrète = implémentation | Invariant |
| Absence de propriétaire d'écriture au runtime (aucun acteur runtime ne mute une Ressource d'asset) | Invariant |
| Absence de découverte dynamique (ensemble des assets connu à la compilation) | Invariant |
| Le chargement et le déchargement instancient tous deux le principe de circulation (deux flux indépendants, jamais un dialogue synchrone) | Invariant (instanciation de `02-concepts.md`) |
| Chargement implicite, scan récursif, résolution par nom sur le hot path | Rejetés |
| Format concret des fichiers, organisation des répertoires, layout du manifeste | Hors périmètre — implémentation |
| Outillage de développement (éditeur, hot-reload) | Hors périmètre — non tranché |

## Bilan — test à cinq questions
| Question | Réponse |
|---|---|
| Que possède ce domaine ? | La Forge (build-time) et l'infrastructure de chargement runtime (plateforme, assemblée par le Shell) |
| Que produit-il ? | Des Ressources persistantes (ADR-005), en lecture seule pendant leur durée de vie runtime, adressées par clé |
| Qui consomme cette production ? | Tout système métier, exactement comme pour toute Ressource — aucune règle distincte |
| Que lui est-il interdit de connaître ? | Aucune sémantique métier (cohérent avec l'ignorance déjà actée pour le `World`) ; aucune connaissance directe des systèmes qui consomment ses Ressources |
| Quelle frontière protège cette ignorance ? | Frontière de compilation (Forge ↔ runtime) et frontière de connaissance (Assets ↔ systèmes métier, via le contrat Ressource ordinaire) |

Le domaine Assets s'intègre sans qu'aucun invariant nouveau n'ait dû être ajouté à `00-principes.md`, `01-runtime-ecs.md`, ou `02-concepts.md` — seule une instanciation du vocabulaire déjà établi a été nécessaire, confirmant une nouvelle fois la robustesse du socle.
