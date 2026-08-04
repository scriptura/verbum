<img src="./.github/assets/banner.png" alt="Verbum Engine Banner" width="1440" height="720" style="width: 100%">

# Verbum

<!-- Invariants Architecture & Paradigmes -->
[![Architecture](https://img.shields.io/badge/Architecture-ECS-E85D04?style=for-the-badge)]()
[![Design](https://img.shields.io/badge/Design-Data--Oriented-2D6A4F?style=for-the-badge)]()
[![Pipeline](https://img.shields.io/badge/Pipeline-AOT%20Forge-D90429?style=for-the-badge)]()
[![Boundary](https://img.shields.io/badge/Boundary-World%20%7C%20Shell-5C6BC0?style=for-the-badge)]()
[![Entities](https://img.shields.io/badge/Entities-Opaque%20Handles-89023E?style=for-the-badge)]()
[![Systems](https://img.shields.io/badge/Systems-Stateless-455A64?style=for-the-badge)]()
<!-- Pipeline Mecanique & CPU/Memoire -->
[![Hot Path](https://img.shields.io/badge/Hot%20Path-Zero%20Allocation-F77F00?style=for-the-badge)]()
[![Memory](https://img.shields.io/badge/Memory-Sequential%20Traversal-FCBF49?style=for-the-badge)]()
[![Execution](https://img.shields.io/badge/Execution-Deterministic%20%7C%20Fixed%20Ticks-D62828?style=for-the-badge)]()
[![Storage](https://img.shields.io/badge/Storage-Sparse%20Set-023E8A?style=for-the-badge)]()
[![Mutations](https://img.shields.io/badge/Mutations-Command%20Buffer-0077B6?style=for-the-badge)]()

**Verbum** est un moteur de jeu 2D top-down, écrit en Rust, construit selon les principes **ECS** (*Entity–Component–System*), **DOD** (*Data-Oriented Design*) et **Ahead-Of-Time** (AOT).

Plus qu'un moteur, Verbum est une recherche sur la manière de construire un runtime dont le comportement découle d'un petit nombre d'invariants architecturaux, plutôt que d'une accumulation de mécanismes particuliers.

Son nom — *Verbum*, « le mot » en latin — reflète cette intention : chaque terme du vocabulaire architectural possède une définition unique, chaque responsabilité est nommée une seule fois, et chaque nouveau domaine est d'abord confronté aux concepts existants avant que de nouveaux ne soient introduits.

## Principes

L'architecture de Verbum repose sur quelques invariants simples.

### Les données sont premières

Le runtime est organisé autour des données.

L'état de la simulation est exclusivement porté par des **Composants** et des **Ressources**. Les **Systèmes** ne possèdent aucun état propre : ils transforment des données selon un contrat déclaratif connu à l'avance.

La disposition mémoire, les parcours séquentiels et la prévisibilité de l'exécution priment sur les abstractions orientées objets.

### La complexité appartient à la Forge

Le runtime n'interprète pas des formats humains.

Les assets, scripts, dialogues, cartes et autres descriptions sont transformés, avant leur consommation, en artefacts adaptés à l'exécution par un outil AOT appelé **la Forge**.

Le runtime ne découvre pas les données : il les exécute.

### Le World ignore la plateforme

Le **World** contient exclusivement l'état logique de la simulation.

Le **Shell** possède le temps réel, les périphériques, l'OS, le rendu, l'audio et les infrastructures.

Cette séparation garantit que la simulation reste indépendante de toute plateforme d'exécution.

### Le modèle d'exécution est unique

Les systèmes ne décident jamais quand ils s'exécutent.

Les scripts ne constituent pas un second modèle d'exécution.

Le réseau n'en introduit pas davantage.

Toutes les transformations de la simulation s'inscrivent dans un unique modèle d'exécution déterministe, construit une fois à partir des contrats déclarés.

## Organisation du dépôt

Le dépôt est organisé autour de deux responsabilités complémentaires.

* **Forge** : préparation Ahead-Of-Time des données et génération des artefacts consommés par le runtime.
* **Runtime** : simulation, exécution ECS et infrastructures de plateforme.

La structure exacte des répertoires peut évoluer au fil du projet, mais cette séparation architecturale demeure.

## Documentation

L'ensemble des décisions architecturales est documenté sous forme d'Architecture Decision Records (ADR).

La documentation n'est pas un commentaire du code : elle constitue la spécification normative de l'architecture.

Pour découvrir le projet, commencer par :

→ **[`docs/architecture/README.md`](docs/architecture/README.md)**

Ce document présente le parcours de lecture, les concepts fondamentaux et l'ensemble des décisions qui structurent Verbum.

---

> *« Chercher le plus petit langage architectural capable d'absorber un domaine entier sans devoir s'étendre. »*

---

_le 2 août 2026_
