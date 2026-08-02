# Verbum

**Verbum** est un moteur de jeu (orienté 2D top-down) conçu autour de trois paradigmes non négociables : **ECS** (Entity-Component-System), **DOD** (Data-Oriented Design) et **AOT** (Ahead-Of-Time).

Son nom reflète sa philosophie architecturale : une rigueur sémantique absolue où le code fait exactement et uniquement ce qu'il désigne. Aucune abstraction fuyante, aucune allocation dynamique sur les chemins critiques (hot paths), aucune indirection inutile.

## Principes Fondamentaux

L'architecture de Verbum rejette les modèles traditionnels basés sur l'héritage, les graphes de scènes orientés objets et les environnements d'exécution dynamiques (VM). Le moteur garantit ses performances et son déterminisme via des invariants structurels stricts :

* **Data-Oriented Design (DOD) & ECS Strict :** La disposition en mémoire (data layout) dicte l'architecture. L'état est intégralement contenu dans des Composants plats et des Ressources globales. Les Systèmes sont des pipelines de transformation purs, sans état propre, définis par une interface déclarative stricte.
* **Architecture AOT & "La Forge" :** Le runtime ne fait aucun parsing. Les assets, les configurations, et la logique scriptée (quêtes, dialogues) sont transformés en représentations binaires plates à la compilation par un outil dédié : *La Forge*. Le runtime se contente de consommer ces données.
* **Séparation Shell / World :** 
  * Le **World** contient l'état et la logique. Il est pur, déterministe, et ignore l'existence de l'OS, du réseau ou de l'écran.
  * Le **Shell** est la couche d'acquisition matérielle. Il capte les inputs, gère l'OS, et pilote l'exécution.
  * *Corollaire :* Le moteur ne possède aucune boucle `while true` interne. L'acquisition pilote la logique.
* **Déterminisme Absolu (Scripts & Réseau) :** Aucun système, ni aucun script, ne maintient d'état d'exécution suspendu hors du `World` (pas de coroutines, pas de `yield` natif). Toute attente ou reprise est modélisée par un compteur ordinal (program counter) stocké dans un composant, traité de manière prédictible à chaque frame logicielle.

## Structure du Dépôt

Le projet est divisé entre les outils de préparation des données (AOT) et le runtime (exécution).

* `/forge` : Le pipeline de compilation AOT (ingestion des DSLs, assets, et génération des formats binaires contigus).
* `/engine` : Le cœur du runtime, comprenant le modèle d'exécution ECS.
* `/shell` : Les implémentations d'acquisition matérielle et de projection graphique (Renderer).
* `/docs/architecture` : Le corpus décisionnel du projet.

## Documentation et ADRs

L'architecture de Verbum est intégralement documentée et justifiée par un corpus d'**Architecture Decision Records (ADR)**. 

Si vous souhaitez comprendre les modèles de conception du moteur, l'ordre d'évaluation des composants, ou les contrats d'interface stricts entre le Shell et le World, commencez par le Master Index de l'architecture :

👉 **[Lire la documentation d'architecture (`docs/architecture/README.md`)](docs/architecture/README.md)**

---
*Verbum est conçu avec l'ambition d'exposer les données à l'état pur.*

---

_le 2 août 2026_