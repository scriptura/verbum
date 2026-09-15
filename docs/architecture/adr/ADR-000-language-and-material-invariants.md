# ADR-000 — Langage et invariants matériels

## Statut
Non adopté. Date : 25 août 2026.

## 1. Quel est le contrat du domaine ?

Fournir un environnement d'exécution (langage et contraintes de compilation) garantissant statiquement l'absence d'allocation dynamique sur le hot path, le déterminisme absolu de la simulation, et l'étanchéité stricte des frontières de connaissance entre les strates du moteur.

## 2. Qui possède quoi ?

* **Le compilateur (via le système de types Rust) :** Possède l'autorité exclusive pour valider le respect des frontières de connaissance et l'isolation des domaines à la compilation.
* **La Forge (hors runtime) :** Possède la responsabilité exclusive de produire des artefacts binaires aux layouts stricts.
* **La Plateforme (Shell) :** Possède l'allocation initiale de la mémoire (via *memory mapping* ou buffers bruts), qu'elle transmet au noyau sans en posséder la sémantique métier.

## 3. Quelles frontières s'appliquent ?

* **Frontière de compilation :** Le runtime ne possède aucun parseur. Toute la complexité de lecture de formats humains (YAML, JSON) est reléguée à la Forge. Le runtime consomme exclusivement des structures binaires denses.
* **Frontière de strate :** La C-ABI est l'unique interface binaire autorisée entre la Plateforme (Shell) et le Modèle architectural (`World`). Elle interdit formellement la fuite de types dynamiques, de pointeurs intelligents ou d'abstractions haut niveau d'une strate à l'autre.
* **Frontière de connaissance :** Les interdits architecturaux (ex: le `World` ignorant la sémantique métier) sont encodés par la visibilité stricte (`pub(crate)`, types opaques, traits scellés) de sorte qu'une violation ne puisse pas compiler.

## 4. Quelles garanties sont offertes au reste du moteur ?

* **Contiguïté et prévisibilité :** Les données manipulées respectent un agencement (*Data Layout*) plat et contigu, maximisant l'efficacité des lignes de cache CPU.
* **Impossibilité matérielle d'allouer sur le hot path :** Le noyau logique est confiné dans un périmètre `#![no_std]` et `#![no_alloc]`, interdisant l'utilisation de types à allocations cachées (`Vec`, `String`, `Box`).
* **Zéro-copie :** Les flux de données traversant les frontières utilisent le cast direct de buffers ou le mappage mémoire sans *marshalling* intermédiaire.

## 5. Qu'est-ce qui est explicitement exclu du périmètre ?

* L'utilisation de la bibliothèque standard (`std`) et des allocateurs dynamiques au sein du Modèle architectural et de la Structure interne.
* La gestion de la mémoire par ramasse-miettes (*Garbage Collector*).
* L'utilisation de structures polymorphes dynamiques (vtable, objets dynamiques) sur le hot path.

## 6. Quelles questions sont volontairement laissées à l'implémentation ?

* Les mécanismes concrets d'effacement de type (*type erasure*) ou de manipulation de pointeurs bruts en coulisses pour résoudre les requêtes de l'ECS.
* L'autorisation pour la strate Plateforme (Shell) d'utiliser la bibliothèque standard (`std`) et des crates de haut niveau pour dialoguer avec l'OS, tant que la frontière C-ABI vers le noyau est strictement respectée.
