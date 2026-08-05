# Roadmap — Verbum, ordre d'implémentation

## Statut
Document de planification, non normatif — ne fixe aucun invariant architectural, seulement un ordre de construction. Révisable sans procédure d'ADR. Établi après audit contradictoire du corpus (`HANDOFF-AUDIT.md` et historique Git associé).

## Principe directeur
Chaque étape ne commence que si la précédente est validée par un **test qui exprime son invariant**, pas seulement par du code qui compile. L'ordre suit une contrainte de dépendance réelle (ce qui a besoin de quoi pour exister), jamais une préférence esthétique. Aucune étape ne produit de rendu avant l'étape 4 — le socle se prouve sans pixels.

Ce document ne répond pas à « dans quel ordre développe-t-on les fonctionnalités ? » mais à « dans quel ordre peut-on démontrer que le modèle décrit par les ADR existe réellement dans une implémentation ? » — c'est un protocole de validation du corpus, pas un plan de fonctionnalités ; à préserver comme telle lecture pour tout futur lecteur.

Un jalon n'est pas validé lorsqu'il fonctionne. Il est validé lorsqu'aucun invariant architectural n'a dû être modifié pour le faire fonctionner.

---

## Étape 0 — `verbum-ecs` : le World nu
**Contenu** : `EntityId`, Sparse Set, composants, Ressources, Command Buffer, application des mutations, Query. Aucune dépendance graphique, système, ou Forge.

**Dépendance** : aucune — fondation.

**Test d'invariant à faire passer avant de continuer** :
> spawn → ajout composant → query → command buffer → resolve → query
Doit démontrer, en test unitaire seul : « produire est libre, résoudre est unique » (`01-runtime-ecs.md`), et qu'un handle invalide échoue toujours via `(index, génération)` — cas canonique : despawn puis réutilisation de l'index libéré, où l'ancien handle doit rester invalide après réattribution.

**Explicitement hors de cette étape** : capacités fixées par Forge — ADR-001 point 5 autorise nommément un mécanisme de test/benchmark à leur place.

**Artefact produit** : crate `verbum-ecs` publiable, testable sans aucune dépendance externe.

---

## Étape 1 — Scheduler
**Contenu** : contrat de système (interface déclarative), construction du graphe de dépendances à partir des contrats, phases, point de validation, exécution.

**Dépendance** : Étape 0 (le Scheduler opère sur des contrats qui lisent/écrivent des données du World).

**Tests d'invariant** :
> Deux systèmes d'une même phase, l'un lisant ce que l'autre a écrit plus tôt dans la même phase, observent la même valeur — et un cycle de dépendance provoque un échec de construction, jamais une erreur runtime (ADR-004, point 4).
> Plusieurs systèmes produisant librement vers une même donnée transitoire, un seul système en aval en assure la résolution vers la donnée persistante — le graphe construit doit respecter « produire est libre, résoudre est unique » (`01-runtime-ecs.md`) au niveau du Scheduler, pas seulement au niveau du Command Buffer déjà testé en étape 0.

**Résultat à ce stade** : un moteur complet qui ne produit rien à l'écran — c'est le comportement attendu, pas un manque.

**Artefact produit** : premier exécutable headless exécutant plusieurs ticks sans sortie visible.

---

## Étape 2 — Forge minimale (un seul type d'asset)
**Contenu** : un pipeline complet mais étroit — `sprite.toml → Forge → sprite.bin → lecture binaire → Ressource`. Pas de validation exhaustive, pas de second type d'asset. Le chargement runtime peut être simulé par un appel direct en test, sans Shell réel — cohérent avec le mécanisme de test déjà autorisé pour les capacités de stockage (ADR-001, point 5).

**Dépendance** : Étape 0 seulement (la Ressource produite doit s'intégrer au World existant ; ni le Scheduler ni le Shell ne sont requis pour ce pipeline).

**Test d'invariant** :
> Le runtime ne parse jamais le `.toml` — seule la Forge le fait ; le runtime ne lit qu'un artefact binaire déjà validé (`00-principes.md`, invariant 1).

**Artefact produit** : premier binaire produit par la Forge, relu et casté en Ressource.

---

## Étape 3 — Shell minimal
**Contenu** : boucle `while OS vivant → tick fixe → scheduler.run() → fin`. Console suffit, aucune fenêtre. Le tick fixe n'est ici qu'une politique provisoire de validation — elle ne préjuge pas de la politique temporelle réelle, explicitement réservée à une future ADR-S04 (ADR-S01, point 5).

**Dépendance** : Étape 1 (`scheduler.run()` doit exister comme point d'appel).

**Test d'invariant** :
> Aucune information ne revient vers une itération déjà en cours — même en boucle console, le principe généralisé en ADR-S01 point 6 doit être vérifiable, quitte à le simuler avec une fausse source d'entrée asynchrone.

**Artefact produit** : premier runtime complet piloté par le Shell (World + Scheduler + boucle), toujours sans rendu.

---

## Étape 4 — Renderer minimal
**Contenu** : `Systems → Frame Resource (composition) → Backend → présentation`. Un clear-color ou un carré suffit. Aucune caméra, animation, sprite réel. Un backend headless est préférable à ce stade : le test porte sur la circulation `FrameRepresentation → Backend → aucun retour`, pas sur l'ouverture d'une fenêtre, et le headless isole mieux l'invariant (ADR-R03, point 4) — il démontre que Backend signifie « consommateur passif », pas « API graphique particulière ».

**Dépendance** : Étapes 1 et 3 (le Renderer s'invoque depuis le Shell, en aval du Scheduler).

**Test d'invariant** :
> La représentation de frame est immuable une fois composée, et rien ne remonte du Backend vers l'ECS pendant l'exécution (ADR-R02 point 4 ; ADR-R03 point 5).

**Artefact produit** : première frame produite et présentée (headless ou fenêtrée).

---

## Étape 5 — Assets réels
**Contenu** : sprites, tilesets, maps, fonts — extension de la Forge et du chargement runtime établis en étape 2, sans changement de mécanisme.

**Dépendance** : Étapes 2 et 4.

**Test d'invariant** : aucun nouveau — chaque nouveau type d'asset doit passer le même test qu'en étape 2, sans modification du pipeline Forge → Ressource.

**Artefact produit** : premier niveau chargé et affiché.

---

## Étape 6 — Scripts
**Contenu** : premier système interpréteur, interface déclarative fixe, validation par la Forge de l'ensemble de données accessible au script.

**Dépendance** : Étapes 2 et 5 (un script est un Asset — ADR-A01 — rien à interpréter sans Forge et Assets fonctionnels).

**Test d'invariant** :
> Un script suspendu à mi-exécution ne laisse aucune pile hors du World — l'état repris après un « yield » doit être bit-à-bit reconstructible depuis le seul composant/Ressource qui le porte (ADR-Sc01, point 4). Le test doit couvrir plusieurs ticks entre suspension et reprise (pas un aller-retour immédiat), pour démontrer que l'état reste valide même après que d'autres systèmes ont tourné dans l'intervalle.

**Artefact produit** : premier dialogue ou comportement scripté exécuté de bout en bout.

---

## Repoussé volontairement, sans date fixée
IA, physique, animation, UI, particules, réseau, sauvegarde — domaines déjà classés comme *preuves de fermeture* (`README.md`) plutôt que primitives. Les construire avant que le socle (étapes 0–3) soit stable risquerait de faire évoluer le noyau sous la pression d'un cas particulier plutôt que par instanciation propre.

## Campagne de validation optionnelle, indépendante de l'ordre ci-dessus
Un aller-retour `World → Save → Load → même état` est l'un des tests les plus puissants du corpus : il valide d'un coup identité, composants, Ressources, reconstruction et Command Buffer (ADR-SV01). Rien n'empêche de le mener dès que le socle (étapes 0–3) est stable, en dehors de tout jalon obligatoire — la Sauvegarde reste « repoussée » comme domaine de production, mais peut servir plus tôt de preuve architecturale ponctuelle. Une fois les Scripts (étape 6) disponibles, le même aller-retour appliqué à un script en cours d'exécution (reprise identique après rechargement) devient une preuve croisée d'ADR-Sc01 et ADR-SV01 — un bonus, jamais une condition d'entrée de l'étape 6.

## Prochaine étape de gouvernance
Cette roadmap est destinée à un audit contradictoire par d'autres LLM, au même titre que le corpus architectural — mêmes règles : citation avant jugement, intégration minimale, rejet des préférences déguisées en nécessité.
