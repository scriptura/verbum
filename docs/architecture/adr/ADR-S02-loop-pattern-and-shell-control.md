# ADR-S02 — Modèle de boucle et pilotage du Shell

## Statut
**Adopté — 10 novembre 2026.**

Historique : projet rédigé le 10 novembre 2026 (statut Proposé), révisé après audit externe le même jour, puis adopté. La décision adoptée concerne exclusivement le modèle de boucle et le pilotage du Shell. Restent explicitement délégués : la politique temporelle et le rattrapage (ADR-S04) ; la transmission et la répétition du Frame et le cycle de vie graphique (clarification d'ADR-R03 ou ADR-S05 éventuelle) ; le découpage des crates et l'intégration concrète (spécification ultérieure).

Cette ADR ne modifie aucune ADR adoptée. Il instancie la délégation explicite d'ADR-S01 (« Ce qui n'est pas tranché ici » : choix entre boucle bloquante et boucle pilotée par callbacks de plateforme → ADR-S02). Il s'appuie sur `page-axes-shell-boucle-presentation.md` (non normative) pour le contexte des questions ouvertes, et sur le code de `verbum-shell` pour les comportements implémentés. Toute orientation de la page d'axes reprise ici est justifiée dans cette ADR ; aucune n'est reprise par autorité.

## Contexte
ADR-S01 fait du Shell le propriétaire du cycle de vie du processus et de la « boucle principale » (point 1), des sources de temps réelles (point 5) et de l'assemblage des infrastructures (point 7). Il impose que le Shell invoque les infrastructures comme des boîtes noires (point 3) et qu'aucune infrastructure ne modifie une itération en cours (point 6).

L'implémentation actuelle (`run_loop`, `FixedStepPacing`, `Inbox`) réalise une boucle **bloquante** : `run_loop` possède le thread, attend par une fermeture de cadencement (`cadence()`), observe l'arrêt (`stop()`), prélève l'`Inbox` (`cut()`), invoque l'infrastructure une fois, et compte l'itération.

Une intégration native par fenêtre inverse la possession du thread : la bibliothèque de fenêtrage possède la boucle de distribution des signaux de l'OS et rappelle l'application par callbacks. `run_loop`, bloquante, n'y est pas directement utilisable. Cette ADR détermine comment le Shell conserve la propriété de la sémantique d'exécution du moteur lorsque la boucle de distribution lui est extérieure.

```
Boucle d'événements de la Plateforme (distribution des signaux OS)
        │  dépôt des données d'Acquisition ; demande de progression
        ▼
Shell : modèle temporel, coupure, publication, invocation, arrêt, erreur
        │
        ▼
0..N itérations de World::execute()  (boîte noire, ADR-S01 point 3)
```

## Décision

### 1. Deux boucles distinctes, à ne jamais confondre
- La **distribution des événements de plateforme** est assurée par le mécanisme de plateforme lorsqu'il existe (par exemple une bibliothèque de fenêtrage qui possède sa boucle et rappelle l'application). Elle appartient à la Plateforme. En mode headless, aucune boucle de distribution OS n'est nécessaire.
- Le **pilotage de l'exécution du moteur** est la conduite de la suite des itérations de `World::execute()` et des coupures qui les précèdent. Il est assuré soit par `run_loop` (pilote bloquant), soit par un pilote événementiel. `run_loop` n'est donc pas une boucle de distribution des événements OS : c'est un pilote bloquant de la boucle d'exécution du moteur. La **sémantique** de cette exécution appartient **toujours** au Shell, quel que soit le pilote.

La « boucle principale » d'ADR-S01, point 1, désigne la boucle d'exécution du moteur : le Shell possède la sémantique d'exécution, pas la mécanique de distribution des signaux OS. Il s'agit d'une précision de portée, non d'un amendement d'ADR-S01 : ADR-S01 renvoie lui-même à cette ADR le choix entre boucle bloquante et boucle pilotée par callbacks.

### 2. Une définition unique de l'itération : le cœur d'itération
Une **itération** est la séquence indivisible : *observation de l'arrêt → coupure de l'`Inbox` → publication et invocation de l'infrastructure (une seule fois) → comptage*. Cette sémantique est définie **une seule fois**, dans le Shell, et appliquée à l'identique par tous les pilotes. Elle préserve, sans les modifier :
- coupure unique à la frontière entre deux itérations (ADR-S03, point 7) ;
- publication et invocation en une seule séquence ; si la publication échoue, l'itération n'est pas lancée (ADR-S03, point 6) ;
- aucune publication pendant une itération (ADR-S03, points 7 et 12) ;
- arrêt observé uniquement entre deux itérations, une exécution engagée n'étant jamais interrompue ;
- première erreur de l'infrastructure rendue **inchangée**, sans réexécution.

Le périmètre de ce que la publication fait (écriture de la Ressource porteuse du lot) reste celui d'ADR-S03, point 5 : le code d'assemblage la réalise ; le cœur d'itération ignore les types du noyau.

### 3. Deux pilotes, un seul cœur
- **Pilote bloquant** : attend selon la politique de cadencement, puis applique le cœur ; conservé pour le headless, les tests et toute application qui choisit une boucle bloquante. Son contrat observable actuel (suites de comportement AQ, LC, PC, RA du code de référence) est préservé.
- **Pilote événementiel** : s'insère dans une boucle de distribution extérieure. Il dépose les données d'Acquisition dans l'`Inbox`, demande une progression au Shell (point 4) et programme son attente d'après le compte rendu.

> Pour une même séquence d'observations (échéances atteintes, demandes d'arrêt, dépôts acceptés), les deux pilotes produisent la même séquence de lots, la même séquence d'invocations de l'infrastructure et la même issue.

Cette équivalence est une **exigence de conception** : elle découle de l'unicité du cœur et se vérifie par des tests, non par construction du compilateur. Que le pilote bloquant soit réécrit au-dessus du cœur ou aux côtés de celui-ci est une décision d'implémentation, tant que la sémantique n'existe qu'une fois. Une duplication de la sémantique d'itération entre pilotes est exclue.

### 4. Définition de la progression
Une **progression** est la demande faite au Shell, à un instant qu'il observe lui-même, d'exécuter les itérations alors dues, et de rendre compte. Elle ne reproduit pas une itération : elle en **déclenche 0..N** par application du cœur (point 2). N est fixé par la politique temporelle (ADR-S04), non ici.

Garanties sémantiques :
- **Non bloquante.** Ce terme signifie ici exactement : aucune attente interne jusqu'à la prochaine échéance ; exécution synchrone des itérations éventuellement dues ; restitution du contrôle à l'appelant après cette progression. Une itération engagée n'est jamais interrompue. Ce n'est **pas** une garantie de faible latence : N itérations successives peuvent retarder la distribution des événements de plateforme sur le même thread (point 9). La progression se termine en temps fini ; ADR-S04 devra définir une politique temporelle compatible avec cette garantie. Aucun plafond de rattrapage ni quantum n'est fixé ici.
- **Sans effet si rien n'est dû.** Une progression qui n'exécute aucune itération n'effectue **aucune coupure** : la coupure fait partie de l'itération, non de la progression. Un contenu déposé reste donc en attente pour la première itération effective. Conséquence : une progression demandée trop tôt ou de façon redondante est inoffensive.
- **Idempotente après état terminal.** Une fois l'état terminal atteint (arrêt observé ou erreur), toute progression ultérieure n'exécute **aucune nouvelle itération** et **ne change pas l'état terminal**. Cette idempotence ne suppose pas que la valeur d'erreur puisse être dupliquée ou déplacée plusieurs fois : elle n'impose aucune contrainte de clonabilité au type d'erreur (point 7).
- **Non réentrante.** Aucune progression n'est demandée depuis l'intérieur d'une itération (ADR-S01, point 6).

Allocation : l'invariant existant d'ADR-S03, point 12, s'applique au **chemin de l'itération** et reste inchangé. Pour le reste de la progression et du compte rendu, la maîtrise des allocations du code propre au Shell relève de l'implémentation ; cette ADR ne crée aucun invariant général d'allocation.

### 5. Source de temps et échéance
Le Shell lit le temps réel et détermine seul si une itération est due (ADR-S01, point 5). L'adaptateur de plateforme ne fournit ni politique temporelle, ni instant servant à décider de l'exécution, ni durée de tick.

Substituer la source de temps du Shell (tests sans horloge murale) est une décision d'assemblage, prise par l'assembleur du Shell (ADR-S01, point 7) ; ce n'est pas un paramètre de chaque demande de progression.

L'échéance est une donnée **sortante** du Shell vers l'adaptateur (point 6). Elle est distincte du mécanisme de réveil de la boucle de la Plateforme : ce mécanisme n'est qu'un moyen, pour l'adaptateur, d'être rappelé à temps.

### 6. Compte rendu de progression
Chaque progression rend un compte rendu portant, sémantiquement :
1. le nombre d'itérations exécutées par cette progression ;
2. le **besoin de prochain réveil** : soit une échéance en temps réel, soit « aucune échéance » ;
3. l'**état terminal** éventuel : arrêt observé, ou première erreur.

« Aucune échéance » signifie qu'aucune itération n'est due tant qu'une condition extérieure ne change pas ; ce n'est pas un état terminal, et les deux ne se confondent jamais. L'adaptateur attend alors sans échéance. Avec une échéance, il programme une attente **jusqu'à** cet instant ou jusqu'à l'arrivée d'un événement. L'échéance est donc une échéance **souhaitée** : le compte rendu ne fournit **aucune garantie temps réel** sur le moment effectif du réveil, qui peut survenir après l'instant demandé selon l'ordonnancement de l'OS. Un réveil prématuré ou superflu est sans conséquence (point 4).

Le **type** du compte rendu, ses noms et ses signatures ne sont pas fixés ici.

### 7. Arrêt et erreur
- Une demande d'arrêt, formulée par la Plateforme (par exemple la fermeture de la fenêtre) via un moyen fourni par le Shell et indépendant de tout type de fenêtrage, est observée **avant la coupure** de la prochaine itération : si elle est observée, aucune coupure n'a lieu, et le contenu de l'`Inbox` reste intact (comportement actuel de `run_loop`).
- Trois choses sont distinguées :
  1. la **conservation** de la première erreur rendue par l'infrastructure : elle est conservée **inchangée**, sans réexécution, ni transformation, ni remplacement par une erreur ultérieure ;
  2. sa **restitution** : la première erreur est rendue observable à l'adaptateur, qui est informé de l'état terminal par le compte rendu (point 6). Aucune clonabilité de l'erreur n'est imposée et aucune restitution répétée de sa valeur n'est exigée ;
  3. les **progressions ultérieures** à l'état terminal : elles n'exécutent plus rien et préservent l'état terminal (point 4).

  Aucun mécanisme concret de transfert ou de stockage de l'erreur n'est fixé. Ce mécanisme (transfert de la valeur, référence, autre) et la manière dont l'adaptateur termine la boucle de la Plateforme et rend l'erreur à son propre appelant sont des décisions d'implémentation.
- L'arrêt et l'erreur ne sont jamais transmis à une itération en cours (ADR-S01, point 6).

### 8. Déclencheur côté Plateforme
Exigence sémantique : **le pilote événementiel de plateforme demande la progression du Shell après la distribution et le traitement du lot courant de signaux de plateforme, avant que la boucle de plateforme n'entre en attente.** Il utilise ensuite le besoin de réveil communiqué par le Shell (point 6) pour déterminer son attente : programmation d'un prochain réveil si une échéance est communiquée, attente sans échéance sinon. La progression n'est jamais demandée pendant la distribution d'un signal ni pendant une itération.

Justification :
- les signaux du lot courant ont été traduits et déposés avant la coupure de l'itération qui suit ; autrement, ils attendraient une itération de plus ;
- ce point est celui où le Shell peut, en une seule fois, exécuter les itérations dues et fournir le besoin de réveil dont la boucle de plateforme a besoin pour sa prochaine attente ;
- l'alternative, demander la progression au début du lot de distribution, prélèverait avant les dépôts du lot et introduirait une latence d'une itération pour tout signal arrivé dans ce lot.

Le choix du point d'accroche concret dans une bibliothèque de plateforme est un choix d'implémentation d'adaptateur, substituable tant que l'exigence est respectée. Sa correspondance avec une bibliothèque donnée (points d'accroche, mécanisme de programmation du réveil, version de référence) appartient à la note de sélection technologique et à la future spécification d'implémentation, non à cette ADR. Le rendu graphique n'est pas déclenché par la progression : il demeure hors du périmètre de cette ADR (point 12).

**Conséquence sur la présentation, signalée sans être résolue.** Dans la séquence habituelle d'une boucle de plateforme, le traitement des signaux de fenêtre, notamment une demande de redessin, précède le point où la progression est demandée. Une demande de redessin déjà traitée dans le lot courant peut donc précéder la progression du Shell et utiliser la représentation précédemment retenue. Si la progression produit ensuite un nouveau Frame, l'intégration de présentation devra demander un redessin ultérieur pour l'afficher. La circulation du Frame et le mécanisme de répétition restent hors du périmètre de S02 et dépendent de l'arbitrage R03/S05.

Un dépôt de signal ne déclenche pas à lui seul une itération. Qu'un signal puisse rendre une itération « due » relèverait d'ADR-S04.

### 9. Modèle de fils initial
La première intégration native exécute le Shell **sur le thread de la boucle d'événements de la Plateforme** : le Shell est possédé par l'adaptateur, la progression et le cœur s'exécutent dans ses callbacks, sans fil supplémentaire.

Justification : le plus petit nombre de fils compatible avec le contrat ; aucun transfert de propriété entre fils pour le Shell ni pour l'infrastructure qu'il possède ; aucune synchronisation nouvelle. Les dépôts d'autres sources (autres fils) restent possibles à tout moment par la poignée de dépôt (ADR-S03, point 3).

Conséquences assumées : une itération longue retarde le traitement des signaux de la Plateforme ; une attente bloquante placée par un autre domaine sur ce fil retarde la progression suivante.

**Alternative différée : Shell sur un fil dédié, pilote bloquant inchangé**, la boucle de la Plateforme déposant par la poignée de dépôt. Elle n'est **pas** retenue comme modèle initial. Elle devient candidate si une difficulté technique est **démontrée** : par exemple, retard de distribution des signaux attribuable à la durée des itérations, ou contrainte d'affinité de fil d'une plateforme cible incompatible avec le modèle monofil. Les seuils ne sont pas fixés ici. Le contrat de progression est indépendant du fil : un changement de modèle ne modifierait ni le cœur d'itération ni les garanties du point 4.

**Écartée comme modèle initial : le pilotage par pompage externe de la boucle de la Plateforme**, où le Shell reprendrait la main entre deux passages d'une boucle de plateforme qu'il ferait avancer lui-même. Ce modèle dépend de la disponibilité de ce mode selon les plateformes et de son comportement sur chacune ; il relève de l'évaluation technologique, et cette ADR n'en fait ni un invariant ni une interdiction architecturale générale.

### 10. Possession de l'état
Le pilote événementiel exige un Shell **à état** : cadencement, `Inbox`, infrastructure (et le `World` qu'elle possède), état terminal. Cet état est possédé par le Shell, lui-même possédé par l'adaptateur de la Plateforme ; aucune structure auto-référentielle ni mécanisme `unsafe` n'est requis par le contrat. La représentation de cet état (type nommé, fermeture capturée, structure privée) **n'est pas décidée** ; aucun type public `Shell` n'est introduit par cette ADR. Le dépôt reste exclusivement l'affaire de la poignée de dépôt (ADR-S03, point 3).

### 11. Frontières de connaissance
- Le cœur d'itération et la progression ne nomment aucun type de fenêtrage ni de graphique (ADR-W01, point 3 ; ADR-000, point 3).
- L'adaptateur de la Plateforme ignore la séquence interne d'une itération, le modèle d'exécution, les systèmes et les phases (ADR-S01, point 3), et n'interprète pas les signaux au sens métier (ADR-S01, point 2).
- La bibliothèque de fenêtrage ignore le Shell, le `World` et l'`Inbox`.
- Le noyau (graphe d'appel de `world.execute()`) ne nomme pas le Shell ; le périmètre du noyau n'est pas modifié par cette ADR (ADR-000, point 1).

### 12. Exigences transmises aux décisions voisines
**À ADR-S04 (politique temporelle)** — besoins contractuels, sans en décider le contenu :
- une détermination « une itération est-elle due à cet instant » évaluable **sans attente** ; la politique bloquante actuelle (`FixedStepPacing::wait`) ne l'offre pas, et sa forme non bloquante provisoire ne préjuge pas de S04 ;
- l'expression d'une prochaine échéance, ou de l'absence d'échéance (point 6) ;
- le caractère fini de la progression (point 4) ;
- la décision sur le quantum du tick et sa provenance (lien avec la Forge), le nombre d'itérations par progression, le rattrapage, l'abandon ou la resynchronisation après retard, et l'affectation temporelle des lots d'entrées.

**À ADR-R03 / ADR-S05 éventuelles** — points repérés, non décidés :
- la progression exécute 0..N itérations ; l'invariant d'ADR-S01, point 5 (« 0..1 FrameRepresentation transmise au Backend » pour 0..N itérations) ne doit pas être rendu impossible à satisfaire. Le contrat de progression n'en préjuge pas ; si N peut excéder 1, le moyen d'identifier la dernière itération d'une progression devra être traité là-bas ;
- transmission et répétition de la représentation de frame, cycle de vie du Backend, canaux de contrôle de la présentation, point ouvert d'ADR-000, point 3.

### 13. Interdictions explicites
- Aucune reproduction de la séquence interne d'une itération par l'adaptateur.
- Aucune politique temporelle ni instant décisionnel fourni par l'adaptateur.
- Aucune coupure en dehors d'une itération ; aucune publication pendant une itération.
- Aucune progression depuis l'intérieur d'une itération.
- Aucune seconde implémentation de la sémantique d'itération.
- Aucun type de fenêtrage ou de graphique dans les contrats indépendants de la Plateforme.
- Aucune interprétation métier dans les données déposées.

## Ce qui n'est pas tranché ici
- Quantum du tick, provenance, nombre d'itérations par progression, rattrapage, retard, affectation des lots (ADR-S04).
- Transmission et répétition de la représentation de frame, cycle de vie du Backend, `PresentMode`, canaux de contrôle de la présentation (ADR-R03 / ADR-S05 éventuelle).
- Découpage Cargo de l'adaptateur de fenêtrage et du Backend ; lieu physique du code d'assemblage (ADR-W01 s'applique) ; rôle du crate `verbum-shell` par rapport au rôle Shell.
- Signatures, noms de méthodes, type du compte rendu, représentation de l'état du Shell, moyen concret de demander l'arrêt, réécriture ou non de `run_loop` au-dessus du cœur.
- Seuils de bascule vers un modèle à fil dédié.
- Initialisation du GPU, plancher de version du compilateur, dépendances.

## Alternatives examinées

| Alternative | Statut | Raison |
|---|---|---|
| Boucle bloquante seule | Insuffisante | Incompatible avec une boucle de distribution possédée par la bibliothèque de fenêtrage. Conservée comme pilote. |
| Cœur d'itération partagé (retenu) | **Retenu** | Une seule sémantique ; préserve les contrats d'ADR-S03. |
| Deux implémentations parallèles de l'itération | Rejeté | Duplication de sémantique ; dérive possible entre pilotes. |
| L'adaptateur reproduit coupure, publication, invocation | Rejeté | Viole ADR-S01, point 3 ; éparpille les invariants d'ADR-S03. |
| Temps ou politique fournis par l'adaptateur | Rejeté | Contredit ADR-S01, point 5. |
| Progression qui coupe même sans itération | Rejeté | La coupure vide l'`Inbox` : elle ne peut appartenir qu'à une itération. |
| Progression bloquante (attente interne) | Rejeté | Redonnerait la boucle au Shell, contrairement à l'objet de l'ADR. |
| Une itération déclenchée par chaque signal | Différé | Relève de la politique temporelle (ADR-S04). |
| Shell sur fil dédié | Différé | Voir point 9. |
| Pilotage par pompage externe de la boucle de la Plateforme | Écarté comme modèle initial | Voir point 9 ; relève de l'évaluation technologique. |
| Demande de progression au début du lot de distribution | Non retenu | Latence d'une itération pour les signaux du lot ; point 8. |

## Les six questions de la grammaire des ADR (`00-principes.md`)
1. **Contrat du domaine.** Quel que soit le pilote, la sémantique d'exécution du moteur est celle d'un cœur d'itération unique ; le Shell expose une progression non bloquante qui rend compte du nombre d'itérations, du besoin de réveil et de l'état terminal (points 2 à 7).
2. **Qui possède quoi.** Le Shell : sémantique d'itération, source de temps, `Inbox` et publication, arrêt et erreur. La bibliothèque de fenêtrage : la distribution des signaux, dans le pilote événementiel. L'adaptateur (rôle Shell) : traduction, dépôt, demande de progression, programmation de l'attente (points 1, 5, 8, 10).
3. **Frontières.** Frontière de connaissance : l'adaptateur ignore l'intérieur d'une itération ; le cœur ignore les types de fenêtrage ; la bibliothèque ignore le Shell. Frontière de strate : tout le domaine appartient à la Plateforme. Frontière de compilation : sans objet (point 11).
4. **Garanties.** Équivalence des pilotes, coupure unique, absence de publication pendant une itération, progression sans effet quand rien n'est dû, première erreur inchangée, arrêt jamais intrusif (points 2 à 7).
5. **Exclusions.** Tout ce qui relève de S04, R03, S05 et du découpage des crates (point 12 et section « Ce qui n'est pas tranché »).
6. **Laissé à l'implémentation.** Signatures, types, représentation de l'état, choix du point d'accroche, réécriture de `run_loop`, source de temps substituable.

## Test méthodologique (`00-principes.md`)
1. **Que possède ce domaine ?** La sémantique d'exécution du moteur, la source de temps, l'`Inbox`, l'arrêt et l'erreur.
2. **Que produit-il ?** Des itérations de `World::execute()` et un compte rendu de progression.
3. **Qui consomme ?** L'adaptateur consomme le compte rendu ; l'infrastructure consomme le lot (ADR-S03, point 5).
4. **Que lui est-il interdit de connaître ?** Les types de fenêtrage et de graphique ; l'interprétation métier des signaux ; l'intérieur de `World::execute()`.
5. **Quelle frontière protège cette ignorance ?** Frontière de connaissance (ADR-S01, points 2 et 3 ; ADR-W01, point 3 ; ADR-000, point 3).

## Test de réduction (méthode d'ADR-W01)
1. *Nouveau type d'état ?* Non : l'état du Shell appartient à la Plateforme ; aucune catégorie de donnée du modèle architectural n'est introduite.
2. *Nouveau modèle d'exécution ?* Non : l'ordre interne reste celui d'ADR-004, construit une fois et immuable ; seule varie la fréquence d'appel de `World::execute()`, déjà ouverte par ADR-S01, point 5.
3. *Nouvelle frontière ?* Non : instanciation de la frontière de connaissance.
4. *Concept nouveau ?* Le vocabulaire « pilote », « cœur d'itération » et « progression » est local à cette ADR ; leur éventuelle entrée dans `02-concepts.md` est à examiner après adoption. Le terme `Event` n'est pas utilisé pour les signaux de Plateforme (`02-concepts.md` : non introduit).

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Distinction distribution des événements de plateforme / pilotage de l'exécution du moteur (`run_loop` est un pilote, non une boucle de distribution OS) | Invariant (précision de portée d'ADR-S01, point 1) |
| Sémantique d'itération définie une seule fois, partagée par tous les pilotes | Invariant |
| Équivalence des pilotes pour une même séquence d'observations | Invariant (exigence de conception, vérifiée par tests) |
| Progression non bloquante ; aucune coupure sans itération ; idempotence après état terminal ; non-réentrance | Invariant |
| Le Shell est seul décideur de l'échéance ; l'adaptateur ne fournit ni politique ni instant décisionnel | Invariant (instanciation d'ADR-S01, point 5) |
| Compte rendu : itérations, besoin de réveil, état terminal ; « aucune échéance » distinct de « terminé » | Invariant (contenu sémantique) |
| Arrêt observé avant la coupure ; première erreur inchangée | Invariant (instanciation d'ADR-S01, point 6 ; ADR-S03) |
| Progression demandée après la distribution et le traitement du lot courant, avant l'entrée en attente de la boucle de plateforme ; jamais pendant | Invariant |
| Point d'accroche concret de la demande de progression dans une bibliothèque de plateforme | Décision d'implémentation d'adaptateur (note de sélection technologique) |
| Shell sur le fil de la boucle de la Plateforme comme modèle initial | Décision révisable (critère de bascule : difficulté technique démontrée) |
| Signatures, type du compte rendu, représentation de l'état, source de temps substituable | Hors périmètre — implémentation |
| Quantum, nombre d'itérations par progression, rattrapage, retard, affectation des lots | Hors périmètre — ADR-S04 |
| Transmission du Frame, cycle de vie du Backend | Hors périmètre — ADR-R03 / ADR-S05 éventuelle |

## Maturité, conditions d'adoption, conformité de l'implémentation
Trois niveaux sont distingués.

**1. Maturité du domaine pour une décision architecturale.** Les six questions de la grammaire sont renseignables pour le périmètre de cette ADR : le domaine est suffisamment mûr.

**2. Conditions d'adoption.** Aucune condition préalable ne subsiste. La décision porte sur le modèle de boucle indépendamment d'une bibliothèque de plateforme particulière. Les contraintes d'affinité de fil sur d'autres plateformes que la première plateforme native desktop sont des limites de portée et des points de vigilance, non des conditions bloquantes. Si ADR-S04 s'avérait ne pouvoir offrir la détermination non bloquante exigée au point 12, une page d'axes complémentaire sur la politique temporelle serait à examiner.

**3. Tests de conformité de l'implémentation du cœur commun** (après implémentation, non préalables à l'adoption).
- Équivalence des pilotes (point 3) : suites de progression à source de temps substituée, vérifiant mêmes lots, mêmes arrêts et même première erreur que `run_loop` sur les scénarios existants.
- Propriétés du point 4 : aucune coupure sans itération, idempotence après état terminal, non-réentrance.
- Validation d'intégration sur plateforme cible : comportement du point d'accroche et du réveil sur échéance, articulation entre la terminaison de la boucle de plateforme et le résultat rendu à l'appelant (point 7).
