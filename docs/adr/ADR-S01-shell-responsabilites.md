# ADR-S01 — Shell : responsabilités

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
ADR-R03 a établi que le Backend est une infrastructure de plateforme, hors du modèle architectural. Il manquait un acteur : celui qui possède `main()`, la boucle d'événements, l'horloge réelle, et qui invoque effectivement le `World`, le Renderer et le Backend. Cette ADR fixe les responsabilités du Shell, conformément à la grille de classification à trois strates (`02-concepts.md`) : le Shell appartient entièrement à la strate Plateforme, jamais au modèle architectural.

## Décision

### 1. Le Shell possède le cycle de vie du processus
Pas seulement le point d'entrée (`main()`) : le Shell possède le démarrage, l'initialisation de la plateforme, la création des infrastructures (`World`, Renderer, Backend), la boucle principale, l'arrêt propre, et la libération des ressources. `main()` n'est qu'un détail de cette responsabilité plus large.

### 2. Le Shell traduit, il n'interprète jamais
Le Shell reçoit des signaux bruts de l'OS ou du matériel (scancode clavier, position de souris, évènement fenêtre) et les transforme en données d'Acquisition standardisées (`01-runtime-ecs.md`, section 5), consommées ensuite par le `World`. Il ne qualifie jamais l'action au sens métier : il produit *« pression sur Espace »*, jamais *« saut »* ou *« attaque »* — cette interprétation reste entièrement du ressort des systèmes.

### 3. Le Shell orchestre les infrastructures, jamais les systèmes
Le Shell invoque exclusivement des infrastructures entières comme des boîtes noires : `world.execute()`, `renderer.compose()`, `backend.present()`. Il n'a et n'aura jamais aucune visibilité sur les nœuds internes du graphe de dépendances construit par le modèle d'exécution (ADR-004) — il ne connaît ni les systèmes, ni les phases, ni les contrats, ni l'ordre interne à `World::execute()`. Ceci verrouille l'invariant déjà posé en ADR-004 : le modèle d'exécution est construit une fois, immuable, et n'est jamais influencé de l'extérieur.

### 4. Le Shell confine les dépendances de plateforme
Toute dépendance à l'OS, au matériel, ou au temps réel — fenêtrage, threads, timers, sockets réseau, système de fichiers, signaux, GPU au sens pilote/driver — appartient exclusivement au Shell. Le `World` ne connaît que des données ; le Backend ne connaît que le GPU en tant que cible de présentation (ADR-R03) ; aucun des deux n'a de connaissance directe de ces dépendances de plateforme. Le Shell est le seul endroit du moteur où elles existent.

### 5. Le Shell possède les sources de temps réelles ; la politique temporelle reste à définir
Le Shell possède l'horloge physique, les timers, les réveils, le VSync, les timestamps — toute source de temps réel. La traduction entre ce temps réel et le temps logique consommé par le `World` (fréquence de tick, découplage tick/présentation, gestion du retard, interpolation...) relève entièrement de la politique temporelle, non tranchée ici et réservée à une future ADR-S04. Cette ADR n'impose donc pas silencieusement un modèle à tick fixe ou tout autre schéma particulier — elle fixe seulement que la traduction, quelle qu'elle soit, est une responsabilité du Shell.

Aucun rapport fixe entre une itération de simulation et une présentation de frame n'est présumé par cette ADR :
```
Temps réel
    │
    ▼
Shell (politique temporelle — ADR-S04)
    │
    ▼
0..N itérations de World::execute()
    │
    ▼
0..1 FrameRepresentation transmise au Backend
```

### 6. Aucune infrastructure ne peut modifier une itération logique en cours
> Aucune infrastructure ne peut modifier une itération logique en cours.

Ce principe ne protège pas spécifiquement le Shell — il protège l'itération elle-même. Il couvre indifféremment le Backend, l'audio, le réseau, le système de fichiers, l'OS, le GPU, ou tout autre acteur de la strate Plateforme : aucun d'entre eux ne peut renvoyer d'information qui modifierait une itération de `World::execute()` déjà en cours. Toute donnée destinée à revenir vers la simulation (résultat de picking, mesure de performance, retour réseau...) constitue une nouvelle entrée acquise à la phase d'Acquisition d'une itération future — jamais un canal direct vers une itération en cours, quel que soit l'acteur qui tenterait de l'emprunter. C'est une généralisation de la règle de non-inversion déjà posée en ADR-R03 (point 5), qui n'était formulée que pour le Backend.

### 7. Le Shell possède et assemble les infrastructures
> Le Shell possède et assemble les infrastructures du moteur.

Le Shell crée, détruit, choisit les implémentations concrètes, et établit les relations entre les infrastructures (`World`, Renderer, Backend, et tout domaine futur — Audio, Assets...). Le mécanisme concret de cette composition (constructeurs Rust, fonctions `build()`, assemblage statique, génération AOT...) est une décision d'implémentation — l'invariant n'est pas une méthode de composition particulière, encore moins un conteneur d'injection de dépendances, mais le fait que cette composition n'existe qu'à un seul endroit du moteur. Ni le `World`, ni le Renderer, ni aucune infrastructure ne choisit sa propre implémentation ou celle d'une autre : le Shell est le seul endroit du moteur où l'assemblage existe. Ceci rend triviale, par construction, la substitution d'implémentation (Backend Vulkan vs headless, Audio OpenAL vs null, etc.) sans qu'aucune autre partie du moteur n'en soit informée.

## Ce qui n'est pas tranché ici
- La politique temporelle concrète (fréquence de tick, découplage tick/présentation, VSync, interpolation, gestion du retard) : ADR-S04.
- Le mécanisme concret de traduction des événements OS en données d'Acquisition (bibliothèque de fenêtrage, format des événements) : ADR-S03.
- La boucle principale concrète (structure du `loop{}`, choix entre boucle bloquante ou pilotée par callbacks de plateforme) : ADR-S02.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Le Shell possède le cycle de vie complet du processus, pas seulement `main()` | Invariant |
| Le Shell traduit les événements externes, sans jamais les interpréter au sens métier | Invariant |
| Le Shell orchestre des infrastructures entières, jamais les systèmes ou l'ordre interne du modèle d'exécution | Invariant |
| Le Shell confine toutes les dépendances de plateforme (OS, threads, réseau, fichiers, signaux...) | Invariant |
| Le Shell possède les sources de temps réelles ; la traduction temps réel → temps logique est une politique | Invariant — la politique elle-même est hors périmètre (ADR-S04) |
| Aucune infrastructure ne peut modifier une itération logique en cours | Invariant (généralisation d'ADR-R03, point 5) |
| Le Shell possède et assemble les infrastructures (création, destruction, choix d'implémentation, établissement de leurs relations) | Invariant |
| Mécanisme concret de boucle principale et de traduction d'événements | Hors périmètre — ADR-S02, ADR-S03 |

## Suite prévue
- ADR-S02 — Boucle principale
- ADR-S03 — Acquisition des événements
- ADR-S04 — Politique temporelle (tick, frame, VSync, interpolation)
- ADR-S05 — Orchestration des infrastructures

Avec ADR-S01, le Shell rejoint la strate Plateforme de la grille de classification (`02-concepts.md`), mais y occupe une position distincte de celle du Backend : il n'est pas une infrastructure parmi d'autres, il est le **point de composition** de la strate Plateforme — celui qui crée, assemble, et orchestre le `World`, le Renderer, le Backend, et tout domaine futur (Audio, Assets...).

```
Shell (assembleur, propriétaire des infrastructures et du processus)
                   │
     ┌─────────────┼─────────────┐
     │             │             │
   World       Backend       Audio, Assets, ...
     │
     ▼
 Modèle architectural
     │
     ▼
 Structure interne
```

Ce schéma remplace la représentation plate d'ADR-R03 (Shell et Backend au même niveau) : il rend fidèlement le fait que le Shell n'est pas seulement orchestrateur au sens d'appel séquentiel, mais propriétaire de l'assemblage complet des infrastructures qu'il invoque.
