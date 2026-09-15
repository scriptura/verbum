# Choix du langage pour Verbum : L'instanciation de l'architecture

**Date :** 25 août 2026
**Objet :** Validation de Rust comme langage d'implémentation de Verbum. Le choix ne repose pas sur la richesse de son écosystème standard, mais sur sa capacité à encoder statiquement les frontières de connaissance et à contraindre l'agencement mémoire (Data Layout).

## 1. Contexte

Verbum est un moteur de jeu 2D reposant sur un pipeline déterministe. Son architecture exige une stricte séparation entre les données et la logique (DOD), une résolution des ressources à la compilation (AOT) et la garantie de zéro allocation sur le hot path (la boucle de simulation par frame).

L'analyse de l'alternative Zig démontre que Rust est le candidat naturel, à condition d'imposer des restrictions drastiques sur l'usage de son écosystème.

## 2. L'adéquation structurelle au modèle DOD

### 2.1 Le typage comme preuve de frontière

L'architecture de Verbum repose sur des domaines qui ignorent mutuellement leur sémantique. Rust permet de garantir ces frontières de connaissance à la compilation, là où Zig exigerait une simple convention.
L'isolation des systèmes, l'opacité du `World` et la passivité du Backend sont encodées via le système de visibilité (`pub(crate)`, types opaques, traits scellés). Le compilateur prouve les invariants.

### 2.2 L'esquive du Borrow Checker

Le modèle ECS de Verbum (composants purs `Copy`/`Clone`, Sparse Sets, relations exprimées par des indices `EntityId` et non par des pointeurs) est parfaitement aligné avec les contraintes de Rust.
L'absence de graphes d'objets auto-référencés et de partage mutable non structuré signifie que le *borrow checker* n'est jamais un adversaire. Les données sont contiguës et plates, optimisant la prédiction de branchement et minimisant les *cache misses*.

## 3. Le noyau logique : `#![no_std]`, `#![no_alloc]` et accès mémoire direct

L'argument classique en faveur de Rust (la richesse de son écosystème) est ici un anti-pattern. Le cœur d'exécution de Verbum rejette l'usage de bibliothèques génériques lourdes.

### 3.1 Refus de la désérialisation dynamique

L'introduction de crates comme `serde` ou `bincode` au sein du runtime est proscrite. Le runtime ne doit procéder à aucun parsing ni aucune instanciation d'objets.
La Forge (AOT) est la seule entité autorisée à traiter des formats humains. Elle génère des structures binaires figées, alignées selon une disposition C stricte (`#[repr(C)]`).

### 3.2 Zero-copy memory mapping

Au démarrage ou lors d'un streaming, le moteur n'alloue pas de mémoire pour copier les assets. Il utilise le *memory mapping* (mappage direct d'un fichier en mémoire vive par l'OS). Le noyau logique se contente de caster ces segments de mémoire bruts vers les structures `#[repr(C)]` correspondantes. Ce mécanisme garantit l'accès immédiat aux données sans cycle CPU gaspillé en allocation ou en copie.

### 3.3 Isolation `#![no_std]`

Pour s'assurer qu'aucune allocation implicite n'infecte le pipeline logique, le noyau (ECS, Systèmes, Command Buffer) doit être conçu dans un environnement `#![no_std]` (dépourvu de la bibliothèque standard Rust). Le hot path devient structurellement incapable d'allouer dynamiquement de la mémoire.

## 4. La frontière C-ABI comme contrainte de design salvatrice

L'interface entre le noyau logique (`World`) et le Shell (orchestrateur des entrées/sorties et du fenêtrage) ne doit pas exploiter les types de haut niveau de Rust. Elle doit se conformer à la C-ABI.

*Définition : La C-ABI (Application Binary Interface pour le langage C) est la norme de représentation binaire la plus basse et universelle, forçant l'utilisation de primitives simples et de pointeurs bruts au détriment des structures complexes spécifiques à un langage.*

Plutôt que d'être un fardeau d'interopérabilité, imposer cette frontière binaire entre les strates produit deux garanties :

1. **Densité des données :** Le modèle de données qui transite entre le Shell et le noyau reste primitif, plat et contigu.
2. **Étanchéité sémantique :** Elle interdit physiquement les fuites d'abstractions haut niveau (objets intelligents, pointeurs partagés `Arc`/`Rc`) entre l'infrastructure et la logique métier.

## 5. Maîtrise des temps de compilation et effacement de type

La composition par traits et l'usage intensif de fonctions génériques (typique des implémentations ECS naïves) provoquent une monomorphisation massive.

*Définition : La monomorphisation est le processus par lequel le compilateur génère une copie distincte du code machine pour chaque type utilisé avec une fonction générique. Cela augmente drastiquement la taille du binaire et les temps de compilation.*

Pour conserver un cycle d'itération AOT performant, le runtime privilégiera l'*effacement de type* (type erasure) aux frontières des systèmes : la manipulation de pointeurs bruts typés dynamiquement sous le capot, masqués par une API sûre. Les requêtes (`Query`) résolvent l'accès mémoire dynamiquement via les métadonnées des composants, plutôt que de s'appuyer sur la génération de code au *compile-time*.

## 6. Décision

**Verbum est implémenté en Rust.**

Ce choix ne valide pas l'utilisation de l'écosystème généraliste du langage, mais exploite sa capacité à imposer des contraintes strictes. Le duo `#![no_std]` / C-ABI agit comme un corset architectural qui garantit l'exécution déterministe, zéro allocation, et orientée données exigée par le moteur.

---

__Synthèse rédigée le 20 août 2026__
__Révisée le 25 août 2026__
