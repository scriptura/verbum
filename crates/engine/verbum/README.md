# verbum

> **Crate de façade.** Ce crate ré-exporte les briques publiques du moteur Verbum
> (`verbum-ecs`, `verbum-schedule`) derrière une API unique. Le projet complet,
> sa documentation architecturale et ses ADR vivent dans le dépôt :
> <https://github.com/scriptura/verbum>.

**Verbum** est un moteur de jeu 2D, écrit en Rust, construit selon les principes
**ECS** (*Entity–Component–System*), **DOD** (*Data-Oriented Design*) et
**Ahead-Of-Time** (AOT).

Plus qu'un moteur, Verbum est une recherche sur la manière de construire un runtime
dont le comportement découle d'un petit nombre d'invariants architecturaux, plutôt
que d'une accumulation de mécanismes particuliers.

Son nom — *Verbum*, « le mot » en latin — reflète cette intention : chaque terme du
vocabulaire architectural possède une définition unique, chaque responsabilité est
nommée une seule fois, et chaque nouveau domaine est d'abord confronté aux concepts
existants avant que de nouveaux ne soient introduits.

## Crates du workspace

| Crate | Rôle |
|---|---|
| [`verbum`](https://crates.io/crates/verbum) | Façade (ce crate) |
| [`verbum-ecs`](https://crates.io/crates/verbum-ecs) | Cœur ECS strict, `no_std`, sans allocation |
| [`verbum-schedule`](https://crates.io/crates/verbum-schedule) | Ordonnanceur construit à partir des contrats de systèmes |
| [`verbum-assets`](https://crates.io/crates/verbum-assets) | Layout d'artefacts et lecteurs runtime |
| [`verbum-forge`](https://crates.io/crates/verbum-forge) | Compilateur AOT des formats humains vers artefacts runtime |
| [`verbum-render`](https://crates.io/crates/verbum-render) | Modèle de données de rendu |
| [`verbum-script`](https://crates.io/crates/verbum-script) | Runtime de scripts, étape d'interprétation pure |
| [`verbum-shell`](https://crates.io/crates/verbum-shell) | Boucle à pas fixe, acquisition d'événements, orchestration |

## Principes

L'architecture de Verbum repose sur quelques invariants simples.

### Les données sont premières

L'état de la simulation est exclusivement porté par des **Composants** et des
**Ressources**. Les **Systèmes** ne possèdent aucun état propre : ils transforment
des données selon un contrat déclaratif connu à l'avance.

### La complexité appartient à la Forge

Le runtime n'interprète pas des formats humains. Les assets, scripts, dialogues,
cartes sont transformés, avant consommation, en artefacts adaptés à l'exécution par
un outil AOT appelé **la Forge**. Le runtime ne découvre pas les données : il les
exécute.

### Le World ignore la plateforme

Le **World** contient exclusivement l'état logique de la simulation. Le **Shell**
possède le temps réel, les périphériques, l'OS, le rendu, l'audio et les
infrastructures. Cette séparation garantit que la simulation reste indépendante de
toute plateforme d'exécution.

### Le modèle d'exécution est unique

Les systèmes ne décident jamais quand ils s'exécutent. Les scripts ne constituent
pas un second modèle d'exécution. Le réseau n'en introduit pas davantage. Toutes
les transformations de la simulation s'inscrivent dans un unique modèle
d'exécution déterministe, construit une fois à partir des contrats déclarés.

## Documentation

L'ensemble des décisions architecturales est documenté sous forme d'Architecture
Decision Records (ADR) :

→ <https://github.com/scriptura/verbum/blob/main/docs/architecture/README.md>

La documentation n'est pas un commentaire du code : elle constitue la spécification
normative de l'architecture.

## Licence

MIT OR Apache-2.0

---

> *« Chercher le plus petit langage architectural capable d'absorber un domaine
> entier sans devoir s'étendre. »*
