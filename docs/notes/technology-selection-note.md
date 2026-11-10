# Verbum — Note de sélection technologique
## Plateforme native et exécution graphique : winit + wgpu

### Statut

**Sélection provisoire pour expérimentation, sous réserve de validation de l'intégration — 9 novembre 2026**

Orientation initiale établie le 8 novembre 2026.

Cette note enregistre le choix de la première pile technologique que Verbum expérimentera pour son intégration graphique native.

Elle ne constitue pas un ADR et ne modifie aucun invariant architectural du moteur.

La sélection constitue une décision de travail réversible : `winit + wgpu` est retenu pour la première intégration expérimentale, mais son adéquation au Shell réel reste à démontrer.

**Pile retenue : `winit + wgpu`.**

**Alternative examinée mais écartée pour l'instant : `SDL3 + wgpu`.**

---

## 1. Objet et contexte

Après l'achèvement du Stage 6 (Scripts), Verbum doit franchir une nouvelle étape : rendre possible une première exécution graphique visible dans une fenêtre native.

Le moteur dispose déjà de plusieurs fondations :

- un ECS et un Scheduler indépendants de la plateforme ;
- un Shell responsable de l'exécution de la simulation ;
- un mécanisme de pacing pour la simulation à pas fixe ;
- un Renderer qui produit un `Frame` immuable ;
- un contrat Backend qui reçoit ce Frame pour assurer sa traduction et sa présentation graphique.

L'objectif n'est pas de sélectionner un framework de jeu, mais les bibliothèques de bas niveau nécessaires à l'intégration d'une plateforme native et d'une API graphique.

Deux responsabilités doivent être distinguées :

| Domaine | Responsabilité |
|---|---|
| Plateforme native | Création et cycle de vie des fenêtres, réception des événements OS |
| Exécution graphique et présentation | Surface, ressources GPU, encodage des commandes graphiques et présentation de l'image |

La sélection retenue est composée de deux bibliothèques complémentaires :

- **winit** : fenêtrage et gestion de la boucle événementielle ;
- **wgpu** : API d'exécution graphique GPU multiplateforme.

Verbum conserve la responsabilité de son propre modèle de rendu 2D, de sa composition et de son contrat `Frame` / Backend.

Aucun framework de rendu ou de jeu de niveau supérieur n'est recherché.

---

## 2. Sélection retenue

### 2.1. winit — plateforme native

`winit` est retenu pour :

- la création et le cycle de vie des fenêtres ;
- la réception des événements de fenêtre et de périphériques ;
- les événements clavier et souris ;
- l'intégration aux mécanismes natifs de redessin ;
- la fourniture des handles nécessaires à la création de la surface graphique.

L'intégration s'appuiera sur le modèle actuel de `winit` :

- `EventLoop` ;
- `ApplicationHandler` ;
- `run_app()` ;
- `WindowEvent::RedrawRequested` ;
- `Window::request_redraw()` ;
- `ControlFlow::Wait` / `ControlFlow::WaitUntil`.

L'application construit l'EventLoop et l'adaptateur implémentant `ApplicationHandler`, puis transfère le pilotage de la boucle événementielle à `run_app()`.

`winit` gouverne donc la distribution des événements de plateforme et le cycle de vie de sa boucle événementielle.

Cela ne lui confère pas la propriété de la simulation.

L'application hôte reste responsable de l'orchestration du Shell, notamment de la politique de simulation à pas fixe, du nombre de ticks à exécuter et du moment où une présentation doit être demandée.

L'adaptation au modèle piloté par callbacks est acceptée. Elle devra toutefois être vérifiée contre le fonctionnement réel du Shell, sans modifier prématurément son modèle d'exécution.

L'attente entre deux événements devra exploiter les mécanismes de contrôle de flux de `winit`. Une échéance de simulation pourra notamment être utilisée pour programmer le prochain réveil avec `ControlFlow::WaitUntil`.

Le rendu sera déclenché en réponse à `WindowEvent::RedrawRequested`. `about_to_wait` servira notamment à déterminer le prochain réveil et à demander un redessin si nécessaire ; il ne sera pas utilisé comme une boucle de rendu permanente.

### 2.2. wgpu — exécution graphique et présentation

`wgpu` est retenu comme API graphique de bas niveau.

Il fournit notamment :

- la création et l'utilisation des surfaces de présentation ;
- l'accès aux périphériques graphiques ;
- les buffers et textures GPU ;
- les pipelines graphiques ;
- l'encodage des commandes ;
- la soumission des commandes au GPU ;
- la présentation de l'image produite sur la surface.

`wgpu` ne constitue pas le Renderer 2D de Verbum.

Il fournit l'API d'exécution graphique utilisée par le Backend pour traduire le contenu du `Frame` en opérations graphiques, jusqu'à la présentation de l'image produite.

La représentation, la composition et les décisions relevant du modèle de rendu demeurent sous la responsabilité de Verbum.

La combinaison winit/wgpu utilise `raw-window-handle` 0.6 pour l'interopérabilité entre la fenêtre et la surface graphique.

### 2.3. Schéma d'intégration visé

Le schéma distingue explicitement la construction de l'application, le pilotage événementiel et le chemin de présentation.

```text
Entrée de l'application
       │
       ├── construit EventLoop
       │
       ├── construit ApplicationAdapter
       │
       ▼
EventLoop::run_app(&mut adapter)
       │
       ▼
ApplicationAdapter : ApplicationHandler
       │
       ├── Événements de plateforme
       │       │
       │       ▼
       │   Acquisition / Input
       │
       ├── Échéances de simulation
       │       │
       │       ▼
       │   FixedStepPacing
       │       │
       │       ▼
       │     Shell
       │       │
       │       ▼
       │   Scheduler / ECS
       │       │
       │       ▼
       │    Renderer
       │       │
       │       ▼
       │      Frame
       │
       ├── Demande de redessin
       │
       └── Attente jusqu'au prochain événement
```

Le chemin de présentation est distinct du chemin de production du Frame :

```text
                     Frame
                       │
                       ▼
              Backend graphique
                       │
          ┌────────────┴────────────┐
          │                        │
          ▼                        ▼
 Données de rendu issues      État GPU interne
     du moteur                - Device
                              - Queue
                              - Surface
                              - Configuration
                              - Pipelines
                              - Buffers
          │                        │
          └────────────┬───────────┘
                       │
                       ▼
              Encodage des commandes
                       │
                       ▼
                 Soumission GPU
                       │
                       ▼
             Texture de présentation
                       │
                       ▼
                    Present
```

`RedrawRequested` est l'occasion événementielle de déclencher ce chemin de présentation. Il ne constitue pas un élément intermédiaire entre le Frame et le Backend.

Ces schémas décrivent les responsabilités visées. Le chemin d'exécution exact, la récupération du Frame et l'articulation avec `Shell::run_loop` restent à vérifier dans le code réel.

---

## 3. Motifs de sélection

### 3.1. Alignement avec l'écosystème Rust

`winit` est une bibliothèque Rust de bas niveau largement intégrée à l'écosystème Rust graphique.

Cette intégration présente plusieurs avantages pour un projet dont les composants sont principalement construits en Rust :

- documentation et exemples directement pertinents ;
- retours d'expérience nombreux ;
- intégrations existantes avec les bibliothèques graphiques ;
- absence d'une bibliothèque SDL3 native supplémentaire à provisionner ;
- dépendances et cycle de build directement intégrés à Cargo.

Cet avantage concerne l'environnement de développement et de maintenance autant que le programme final.

Il ne signifie pas que `winit` dispense de toute dépendance système ou graphique native.

`winit` s'appuie sur les interfaces de fenêtrage propres aux systèmes d'exploitation. Selon la plateforme, son fonctionnement implique notamment les infrastructures Win32, Cocoa, X11 ou Wayland. Certaines bibliothèques système peuvent également être chargées dynamiquement.

De son côté, `wgpu` dépend des infrastructures graphiques disponibles sur la machine et des pilotes correspondants.

La distinction recherchée n'est donc pas l'absence absolue de dépendances natives, mais l'absence d'une bibliothèque multimédia native tierce supplémentaire à gérer dans le projet.

### 3.2. Réduction du périmètre des dépendances

Verbum ne cherche pas à adopter un ensemble de services de plateforme déjà constitué.

Le moteur construira ses propres sous-systèmes selon ses besoins et ses contrats.

Il est donc préférable, à ce stade, de sélectionner des bibliothèques spécialisées :

| Besoin | Direction retenue |
|---|---|
| Fenêtrage | winit |
| Exécution graphique GPU | wgpu |
| Rendu 2D | Verbum |
| Acquisition clavier/souris | Adaptateur de plateforme |
| Manettes | À évaluer indépendamment si nécessaire |
| Audio | Sous-système indépendant, à sélectionner ultérieurement |
| Textures et atlas | Pipeline Assets / Forge |
| Simulation | Shell / Scheduler |

Les services supplémentaires fournis par SDL3 ne constituent pas un problème en eux-mêmes. Verbum ne dispose simplement d'aucun besoin établi qui justifierait actuellement de préférer cette solution intégrée à une bibliothèque de fenêtrage plus ciblée.

Ce choix évite d'adopter prématurément des services dont le moteur n'a pas encore établi le besoin.

### 3.3. Réduction des contraintes de liaison et de distribution

Le binding Rust SDL3 requiert la bibliothèque native SDL3.

Dans sa configuration dynamique par défaut, il faut rendre cette bibliothèque accessible au chargeur de bibliothèques de la plateforme.

SDL3 peut aussi être compilé depuis ses sources ou lié statiquement. Ces possibilités sont valides, mais introduisent des choix supplémentaires de build et de distribution.

Avec `winit`, Verbum évite l'introduction de cette bibliothèque native tierce supplémentaire.

Ce point est intéressant pour :

- la reproductibilité des builds ;
- le cross-compiling ;
- la distribution du jeu ;
- la maintenance des dépendances ;
- la réduction des opérations spécifiques à SDL dans les environnements de développement et de déploiement.

Il ne faut toutefois pas en déduire que `winit + wgpu` produit systématiquement un exécutable autonome. Les dépendances aux infrastructures de plateforme et aux pilotes graphiques demeurent.

Le bénéfice recherché est une réduction du nombre de composants natifs supplémentaires dont Verbum doit organiser la compilation, la liaison et la distribution.

Les conséquences de distribution propres à SDL3 sont détaillées en annexe A.

### 3.4. Compatibilité avec les frontières de Verbum

Le modèle de `winit` ne nécessite pas de transférer la responsabilité de la simulation à la bibliothèque de fenêtre.

L'application hôte peut conserver la politique de simulation à pas fixe et invoquer le Shell au moment approprié.

La distinction reste donc :

- `winit` gouverne la distribution des événements et le cycle de vie de la plateforme ;
- l'application gouverne l'orchestration ;
- le Shell exécute la simulation ;
- le Renderer produit le Frame ;
- le Backend concret exécute le rendu graphique avec wgpu.

Cette répartition est cohérente avec l'architecture existante.

---

## 4. Alternative étudiée : SDL3 + wgpu

SDL3 a été examiné comme alternative sérieuse à `winit`.

Il présente plusieurs qualités :

- bibliothèque de plateforme établie ;
- gestion des fenêtres et des événements ;
- modèle de polling compatible avec une boucle principale explicitement pilotée par l'application ;
- gestion intégrée du clavier, de la souris et des manettes ;
- services complémentaires, notamment audio et clipboard ;
- possibilité d'interopérabilité avec wgpu via `raw-window-handle`.

Son principal avantage architectural pour Verbum est son modèle de boucle événementielle.

Une application SDL3 peut naturellement exprimer un cycle de cette forme :

```text
while running:
    poll_events()
    execute_due_ticks()
    present()
```

Ce modèle est agréable pour un moteur dont l'application veut conserver explicitement la maîtrise de sa boucle principale.

Cependant, cette propriété n'établit pas une supériorité fonctionnelle de SDL3 pour la simulation : `winit` permet également de préserver la politique de ticks du moteur, même si son cycle de vie est piloté par callbacks.

### 4.1. Motifs de report de SDL3

La sélection de SDL3 impliquerait l'introduction d'une bibliothèque native C supplémentaire dans le projet.

Le binding Rust `sdl3` présente un niveau de maturité et de consolidation inférieur à celui de `winit` dans l'écosystème Rust.

La gestion de la liaison et de la distribution de SDL3 ajoute par ailleurs des opérations propres à cette dépendance native, selon la plateforme et le mode de liaison retenu.

SDL3 apporte également des services intégrés dont Verbum n'a pas encore établi le besoin.

Nous ne considérons pas que l'avantage de son modèle de boucle principale soit, à lui seul, suffisant pour compenser ces coûts.

### 4.2. Portée de la décision

SDL3 n'est pas considéré comme une technologie inadaptée ni définitivement rejeté.

Il est simplement **écarté de la première intégration expérimentale**, car son adoption ne présente pas actuellement un avantage suffisamment important pour justifier son coût supplémentaire.

Il pourra être réexaminé si des besoins concrets de plateforme, de contrôleurs, d'audio ou d'intégration multiplateforme le justifient.

Aucune seconde PoC SDL3 n'est planifiée à ce stade.

---

## 5. Politique Rust et versions des dépendances

La version Rust 1.85 précédemment utilisée par Verbum ne constitue pas une contrainte forte.

Le projet accepte de relever sa toolchain lorsque cela apporte un bénéfice technique ou facilite la maintenance des dépendances.

La sélection technologique ne doit donc pas être artificiellement limitée à des versions anciennes de wgpu ou de winit.

À la date de cette note, les références examinées sont notamment :

| Bibliothèque | Version de référence | Statut et MSRV déclaré |
|---|---|---|
| `winit` | 0.30.13 | Version stable ; Rust 1.70 |
| `wgpu` | 30.0.1 | Version stable ; Rust 1.87 |
| `SDL3 Rust` | 0.22.0 | Version publiée ; alternative non retenue |

Une distinction est nécessaire concernant `winit` :

- la version stable 0.30.13 déclare un MSRV de 1.70 dans son manifeste publié ;
- la branche 0.31 est en préversion, avec notamment `0.31.0-beta.4` publiée le 24 octobre 2026 ;
- cette branche préliminaire est à surveiller, mais ne constitue pas la référence retenue pour la première intégration.

La référence reste donc `winit` 0.30.13, sauf raison technique justifiant l'adoption d'une version ultérieure.

La documentation de wgpu 30 indique un MSRV de 1.87 pour les utilisateurs de la bibliothèque. Son propre dépôt de tests et d'exemples peut utiliser une version supérieure, ce qui ne constitue pas automatiquement une exigence pour les projets consommateurs.

La toolchain effective de Verbum et sa politique de CI seront alignées sur les versions finalement retenues au moment de l'intégration.

Aucun MSRV historique ne doit être conservé sans justification propre au projet.

Les versions indiquées ici constituent des références d'évaluation, et non un verrouillage définitif des dépendances.

---

## 6. Contraintes architecturales à préserver

La sélection de `winit + wgpu` ne doit pas entraîner de modification des responsabilités déjà établies.

Les règles suivantes guident l'intégration :

1. Aucun type `winit` ou `wgpu` dans les contrats indépendants de plateforme, notamment ceux de l'ECS, du Scheduler, des systèmes de simulation, du Renderer et du `Frame`.
2. Les types `winit` restent confinés à l'adaptateur de plateforme et à l'application hôte.
3. Les types `wgpu` sont permis dans l'initialisation graphique et dans l'implémentation concrète du Backend ; ils ne doivent pas pénétrer les contrats indépendants de la plateforme.
4. Le Shell conserve ses responsabilités d'exécution et de pacing.
5. L'application hôte assure l'adaptation entre les événements de plateforme et les mécanismes d'acquisition de Verbum.
6. Le Renderer conserve son contrat de production d'un `Frame` immuable.
7. Les ressources de fenêtre et de GPU restent hors du World.
8. La composition du Frame reste indépendante de l'exécution graphique et de la présentation.

### 6.1. Frontière Frame / Backend

Le `Frame` constitue l'unique source de données de rendu produite par le moteur et consommée par le Backend.

Cela ne signifie pas que le Backend est une fonction sans état.

Un Backend graphique concret peut posséder son propre état, notamment :

- `Device` ;
- `Queue` ;
- `Surface` ;
- configuration de surface ;
- pipelines ;
- buffers persistants ;
- caches de textures ;
- dimensions de la cible de présentation ;
- état nécessaire au traitement des erreurs graphiques.

Il reçoit également les informations de cycle de vie de la plateforme qui sont nécessaires à son fonctionnement, notamment les changements de dimensions ou les transitions affectant la disponibilité de la surface.

Ces informations ne constituent pas des données de rendu issues du moteur et ne doivent pas être introduites dans le `Frame`.

L'organisation précise de ces opérations entre l'application hôte, l'adaptateur de plateforme et l'implémentation concrète du Backend reste à déterminer à partir du code existant.

Cette note ne fixe aucune nouvelle signature d'API et ne décide pas d'introduire un trait Backend public.

### 6.2. Durée de vie des ressources

Une attention particulière sera portée aux relations de durée de vie entre la fenêtre, les handles de plateforme et la `wgpu::Surface`.

Le type `Surface<'window>` et les mécanismes sûrs de `wgpu` permettent de représenter ces relations dans le modèle de durée de vie de Rust. Selon la forme du handle transmis à `Instance::create_surface`, la surface peut conserver une source de handle ou être liée par emprunt à une fenêtre extérieure.

Le propriétaire effectif de ces ressources et leur ordre de destruction devront donc être explicites dans l'intégration.

La note ne présuppose pas une stratégie unique de possession : celle-ci devra respecter l'API sûre de wgpu et la durée de vie réelle des fenêtres.

---

## 7. Prochaine étape : validation par une micro-PoC

La sélection de `winit + wgpu` constitue une décision de première intégration expérimentale. Elle n'est pas encore une validation de l'intégration au moteur.

La prochaine étape consiste à confronter cette pile au Shell, au Renderer et au contrat Backend réellement présents dans le dépôt.

La PoC doit rester minimale et expérimentale.

### 7.1. Objectifs

1. Créer une fenêtre native avec winit.
2. Créer une surface wgpu à partir de cette fenêtre.
3. Présenter une première image simple.
4. Faire fonctionner le véritable chemin de simulation Verbum.
5. Récupérer le véritable Frame produit par le Renderer.
6. Présenter une contribution visible issue de ce Frame.
7. Traiter la fermeture de la fenêtre et son redimensionnement.
8. Vérifier les dimensions nulles ou non présentables, notamment lors de la minimisation.
9. Vérifier la classification et le traitement des résultats de surface, notamment la reconfiguration lorsque celle-ci est possible et l'arrêt contrôlé lorsqu'elle ne l'est pas.
10. Observer les transitions `resumed` / `suspended` et leurs conséquences sur les ressources de fenêtre et de surface, sans étendre la PoC au support complet des plateformes mobiles.

### 7.2. Cadence et attente événementielle

La PoC devra vérifier explicitement l'intégration entre `FixedStepPacing` et le modèle événementiel de winit.

Les critères suivants sont requis :

| Critère | Résultat recherché |
|---|---|
| Simulation | Indépendante de la fréquence de présentation |
| Rattrapage | Possibilité d'exécuter 0..N ticks selon le pacing |
| Redessin | Présentation déclenchée par `RedrawRequested` |
| Attente inactive | Pas de boucle active inutile lorsque ni tick ni redessin ne sont dus |
| Réveil | Le prochain tick dû ou événement de plateforme réveille l'application au moment approprié |

Lorsque rien n'est dû, l'EventLoop doit pouvoir attendre sans consommation CPU inutile.

L'application doit programmer son prochain réveil à partir de sa propre politique de pacing, notamment au moyen de `ControlFlow::WaitUntil`.

Les événements de plateforme peuvent naturellement provoquer des réveils supplémentaires, sans que cela implique d'exécuter systématiquement un tick ou un rendu.

Le rendu ne doit pas être exécuté en continu simplement parce que `about_to_wait` est appelé.

### 7.3. Suspension et reprise

La PoC observera le comportement des ressources lors des callbacks `resumed` et `suspended`.

Elle devra déterminer :

- où la fenêtre est créée ;
- quand la surface est créée ;
- quelles ressources sont conservées ou recréées ;
- ce que fait le Backend lorsque la présentation est suspendue ;
- comment le Shell et le pacing se comportent pendant cette période ;
- si la simulation continue, ralentit ou se met en pause selon une politique relevant de l'application et non de winit.

`winit` ne garantit pas la réception de véritables événements de suspension sur toutes les plateformes desktop. La PoC devra donc distinguer les événements effectivement observables des comportements propres aux plateformes disposant d'un cycle de vie applicatif formel.

Il ne s'agit pas d'implémenter immédiatement une prise en charge mobile.

### 7.4. Traitement des résultats de surface

La PoC ne doit pas présupposer que toute erreur graphique est récupérable.

Elle vérifiera la classification des situations rencontrées, en distinguant notamment :

- un délai d'acquisition expiré (`Timeout`) ;
- une fenêtre masquée ou non présentable (`Occluded`) ;
- une configuration devenue obsolète (`Outdated`) ;
- une surface perdue (`Lost`) ;
- une configuration sous-optimale nécessitant une reconfiguration (`Suboptimal`) ;
- une erreur de validation ;
- une perte du périphérique ou un épuisement mémoire.

Le traitement pourra consister à ignorer une image, reconfigurer une surface, recréer certaines ressources ou arrêter proprement l'application.

La stratégie précise devra respecter la sémantique de wgpu 30 et les capacités de récupération réellement disponibles.

### 7.5. Critères de validation

| Critère | Résultat recherché |
|---|---|
| Intégration au Shell | Aucun changement de son modèle de simulation |
| Pacing | Simulation indépendante de la fréquence de présentation |
| Attente | Pas de boucle active inutile lorsque rien n'est dû |
| Réveil | Échéances correctement programmées |
| Renderer | Contrat Frame inchangé |
| Backend | Le Frame est son unique entrée de rendu issue du moteur ; état GPU et cycle de vie restent hors des contrats du moteur |
| Plateforme | Types winit confinés à l'adaptateur |
| GPU | Ressources et relations de durée de vie explicites |
| Surface | Redimensionnement, minimisation et erreurs traités selon leur nature |
| Cycle de vie | Transitions de suspension/reprise observées |
| Build | Dépendances et toolchain reproductibles |
| Exécution | Première image réellement visible |

La PoC ne doit pas être transformée en Renderer de production ni servir de prétexte à une réarchitecture du Shell.

Elle doit seulement démontrer que les interfaces existantes s'assemblent naturellement et que le modèle de contrôle de Verbum fonctionne correctement dans un environnement événementiel.

---

## 8. Conditions de réévaluation

La sélection pourra être revue si l'un des constats suivants apparaît :

- l'intégration de winit impose des adaptations importantes au Shell ou au modèle de simulation ;
- le cycle de vie de winit entraîne une complexité d'adaptation disproportionnée ;
- l'attente et le réveil programmés sont difficiles à concilier avec le pacing existant ;
- une plateforme cible importante est insuffisamment prise en charge ;
- un besoin réel de services intégrés SDL3 apparaît et justifie sa dépendance native ;
- les contraintes de build ou de distribution de winit deviennent moins favorables que celles de SDL3 ;
- une difficulté concrète de maintenance ou d'intégration remet en cause l'avantage actuel de winit.

Une éventuelle réévaluation devra s'appuyer sur une difficulté observée, et non sur une préférence abstraite pour un modèle de boucle événementielle.

La publication d'une version stable de winit 0.31 pourra également justifier une révision des versions de référence, sans remettre automatiquement en cause le choix technologique.

---

## 9. Conclusion

Verbum retient provisoirement **winit + wgpu comme première pile à intégrer expérimentalement**.

Ce choix privilégie :

- l'alignement avec l'écosystème Rust ;
- la maturité de l'environnement de fenêtrage ;
- la réduction des dépendances natives supplémentaires ;
- la séparation entre plateforme et exécution graphique ;
- la compatibilité attendue avec le modèle Renderer / Frame / Backend déjà construit.

SDL3 présente un modèle de boucle principale séduisant et des services intégrés intéressants. Il demeure une alternative valable, mais son coût supplémentaire en dépendances natives, build et maintenance n'est pas actuellement justifié par les besoins établis.

La priorité n'est donc plus de comparer théoriquement SDL3 et winit.

Elle est de démontrer que **winit + wgpu s'intègre au code réel de Verbum sans altérer les responsabilités du moteur**, tout en conservant une attente événementielle efficace et un cycle de vie graphique maîtrisé.

La décision de sélection provisoire sera confirmée, ajustée ou remise en question à la lumière de cette première intégration expérimentale.

---

## Annexe A — Conséquences de distribution de SDL3

Cette annexe rassemble les conséquences pratiques qui motivent une partie du report de SDL3. Elle ne constitue pas un inventaire exhaustif des possibilités de packaging.

Le binding Rust SDL3 utilise par défaut une bibliothèque native dynamique.

Selon la plateforme :

| Plateforme | Conséquence d'une liaison dynamique |
|---|---|
| Windows | `SDL3.dll` doit être accessible au chargeur, généralement à côté de l'exécutable |
| Linux | `libSDL3.so.0` doit être accessible via les chemins du chargeur dynamique ; une distribution privée peut nécessiter un `RPATH` / `RUNPATH` adapté |
| macOS | La dylib ou le framework SDL3 doit être accessible via les chemins de liaison et, pour un bundle applicatif, intégré correctement au paquet |

La compilation depuis les sources introduit notamment CMake, un compilateur C et les dépendances nécessaires à SDL pour la cible.

La liaison statique est également possible, mais la documentation de `sdl3-sys` précise que SDL ne recommande pas cette approche.

Ces mécanismes rendent SDL3 distribuable, mais ajoutent des décisions et opérations propres à une bibliothèque native tierce.

Ces coûts sont acceptables lorsqu'un besoin concret les justifie. Aucun besoin de cette nature n'a été établi pour la première intégration graphique de Verbum.

---

## Références techniques

**État documentaire consulté le 9 novembre 2026.**

### Sources déterminantes — APIs et versions

**winit**

- [winit 0.30.13 — Documentation](https://docs.rs/winit/0.30.13/winit/)
- [winit 0.30.13 — Manifeste Cargo et MSRV](https://docs.rs/crate/winit/0.30.13/source/Cargo.toml)
- [winit 0.30.13 — ApplicationHandler](https://docs.rs/winit/0.30.13/winit/application/trait.ApplicationHandler.html)
- [winit 0.30.13 — EventLoop](https://docs.rs/winit/0.30.13/winit/event_loop/struct.EventLoop.html)
- [winit 0.30.13 — Window](https://docs.rs/winit/0.30.13/winit/window/struct.Window.html)
- [winit — Releases](https://github.com/rust-windowing/winit/releases)

**wgpu**

- [wgpu 30.0.1 — Documentation](https://docs.rs/wgpu/30.0.1/wgpu/)
- [wgpu 30.0.1 — Manifeste Cargo et MSRV](https://docs.rs/crate/wgpu/30.0.1/source/Cargo.toml)
- [wgpu 30.0.1 — Instance et création de Surface](https://docs.rs/wgpu/30.0.1/wgpu/struct.Instance.html)
- [wgpu 30.0.1 — Surface](https://docs.rs/wgpu/30.0.1/wgpu/struct.Surface.html)
- [wgpu — Résultats d'acquisition de surface](https://docs.rs/wgpu/30.0.1/wgpu/enum.CurrentSurfaceTexture.html)

**SDL3**

- [SDL3 Rust 0.22.0 — Documentation](https://docs.rs/sdl3/0.22.0/sdl3/)
- [sdl3-sys — Liaison et compilation de SDL3](https://docs.rs/sdl3-sys/latest/sdl3_sys/)

### Sources complémentaires — Build et distribution SDL3

- [SDL3 — Déploiement Windows](https://wiki.libsdl.org/SDL3/README-windows)
- [SDL3 — Documentation CMake](https://github.com/libsdl-org/SDL/blob/main/docs/README-cmake.md)
- [Linux — ld.so et chemins de recherche des bibliothèques partagées](https://www.man7.org/linux/man-pages/man8/ld.so.8.html)
- [Apple — Run-Path Dependent Libraries](https://developer.apple.com/library/archive/documentation/DeveloperTools/Conceptual/DynamicLibraries/100-Articles/RunpathDependentLibraries.html)
- [SDL3 — Documentation macOS](https://wiki.libsdl.org/SDL3/README-macos)
