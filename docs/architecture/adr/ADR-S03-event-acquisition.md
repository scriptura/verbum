# ADR-S03 — Acquisition des événements

## Statut
Adopté. Date : 19 octobre 2026.

## Contexte
ADR-S01 fait du Shell le traducteur des signaux externes en données d'Acquisition, sans interprétation (point 2). Il interdit toute influence sur une itération en cours (point 6), fait invoquer les infrastructures comme des boîtes noires (point 3) et réserve l'assemblage au Shell (point 7). ADR-000, point 3, fixe le point de passage — l'invocation de l'infrastructure `World` — mais laisse la forme de l'interface ouverte. Cette ADR fixe la responsabilité de traduction, le contrat des données d'Acquisition et leur remise au modèle.

## Décision

**1. Responsabilité de traduction.** Le Shell traduit les signaux externes en données d'Acquisition standardisées, sans interprétation métier (ADR-S01, point 2) et sans référence à un type de la Plateforme ni propriété partagée (ADR-000, point 3). Cette traduction s'exerce côté Shell, avant tout dépôt dans la boîte de réception.

**2. Mécanismes substituables.** Le choix d'une bibliothèque de fenêtrage, l'encodage bas niveau des événements et le mode d'écoute des sources sont des décisions d'implémentation. Justification : la méta-règle de `00-principes.md` ne fige que des invariants structurels, et ces mécanismes restent substituables sans modifier un autre document. Ils sont substituables à condition de préserver le contrat observable : le contrat du point 1, les règles de frontière, de publication complète, de capacité et de saturation. Les systèmes n'ont alors ni leur contrat ni leurs types à changer pour accueillir une autre implémentation conforme. Le remplacement n'oblige pas à reproduire les mêmes lots en situation d'acquisition concurrente (point 11). La table de classification de `00-principes.md` traite de même le layout de `EntityId` et le mécanisme de reset transitoire.

**3. Collecte, lot et publication.** Le Shell collecte les données dans une boîte de réception dont le stockage relève de la Plateforme, jamais du `World`. Le stockage de la boîte et la responsabilité fonctionnelle sont distincts : le Shell assume la collecte, la traduction et la publication (point 5). Les dépôts sont possibles à tout moment, y compris pendant une itération, et ne touchent jamais le `World`. Le lot est l'ensemble ordonné, dans l'ordre d'acceptation par la boîte de réception, des données publiées pour une itération. Sa capacité est fixée avant la simulation et il peut être vide. La publication est l'unique acte par lequel le lot devient visible du modèle.

**4. Porteur.** Le lot courant est porté par une Ressource typée dédiée. Ce choix est nouveau dans cette ADR et se fonde sur le modèle général des Ressources (ADR-005). La Ressource est transitoire (ADR-005, point 4). Son stockage physique appartient au `World`. Clé et stockage sont des décisions d'implémentation.

**5. Rôles à la frontière.**
- **Destinataire de l'entrée :** l'infrastructure `World` invoquée (ADR-000, point 3). Ce n'est pas un consommateur unique du lot.
- **Autorité unique de publication :** le Shell, au travers du code concret d'assemblage. La boucle générique du Shell ignore les types du noyau. L'assemblage relie le porteur à l'exécution sans acquérir aucun droit général de mutation du `World`. Son droit se limite au remplacement intégral de cette Ressource.
- **Lecteurs :** les systèmes dont le contrat déclaratif prévoit la lecture.
- **Propriétaires de résolution :** ceux des éventuelles productions transitoires distinctes dérivées du lot. Le lot lui-même n'a aucune résolution.

Le rôle de propriétaire de durée de vie du lot (`01-runtime-ecs.md` §4) revient à la responsabilité du Shell relative à la collecte et à la publication du lot. La publication est l'acte par lequel cette responsabilité s'exerce : elle n'est pas un propriétaire.

**6. Durée de vie, initialisation, renouvellement.** Le lot courant dure une itération logique complète, éventuellement composée de plusieurs phases. Il est renouvelé avant chaque itération, la première comprise, même s'il est vide, et aucune donnée d'une itération précédente ne survit. La publication est un remplacement intégral. Le porteur est initialisé, à l'assemblage, avec un lot vide déterministe. Aucune itération ne commence tant qu'un lot valide n'a pas été publié pour elle. Aucun état « absent » n'est observable par les systèmes. La publication et l'invocation de l'itération forment une seule séquence du code d'assemblage. Si la publication échoue, l'itération n'est pas lancée.

**7. Frontière et prélèvement.** La publication n'a lieu qu'à la frontière unique située après la fin d'une itération et avant le début de la suivante. Elle prélève le contenu de la boîte à une coupure :
- les événements acceptés avant la coupure appartiennent au lot publié ;
- ceux qui arrivent après restent en attente pour une publication ultérieure ;
- chaque dépôt est entièrement avant ou entièrement après la coupure.

Le lot publié est complet, sans troncature ni publication partielle. La capacité de la boîte est inférieure ou égale à celle du lot. Une violation de cette relation est un bug de configuration (ADR-000), jamais une dégradation silencieuse.

**8. Lecture.** Un système lit le lot en lecture seule, uniquement si son contrat le déclare (ADR-003, point 5 ; ADR-005, point 2). Plusieurs lecteurs sont autorisés, sans restriction par type de phase. Aucun système ne déclare d'écriture ni de production du lot. Cette interdiction est une règle de composition : les types Rust actuels ne la garantissent pas statiquement. Elle est vérifiée par un contrôle des contrats des systèmes assemblés et par la revue. Les systèmes ignorent l'origine du lot et l'instant de sa publication.

**9. Entrée de frontière et données internes.** Le lot est une entrée de frontière, non un accumulateur de contributions à résoudre. La règle de consommation unique s'applique aux productions qui exigent une résolution, pas à la simple lecture d'une entrée. Toute donnée dérivée du lot est une donnée transitoire distincte, produite par des systèmes (ADR-004), avec son propre propriétaire de durée de vie et sa propre résolution. Cette ADR n'impose ni leur existence ni leur forme.

**10. Saturation.** Les capacités sont fixes. Si la boîte est saturée, le nouvel événement est refusé par une erreur explicite retournée au déposant. Les événements déjà acceptés sont conservés, dans leur ordre d'acceptation par la boîte de réception. Il n'y a ni perte silencieuse, ni écrasement, ni croissance dynamique. Le refus n'est jamais transmis à une itération en cours (ADR-S01, point 6).

**11. Déterminisme.** Pour une même séquence de lots effectivement publiés, la séquence d'états est la même (`00-principes.md`, invariant 3). Cette ADR ne prétend pas que l'ordre d'acceptation d'événements concurrents, ni leur répartition entre deux lots, soit reproductible d'une exécution à l'autre. Au cours d'une exécution donnée, l'ordre d'acceptation par la boîte de réception est un ordre concret et celui du lot. Cela relève de la politique temporelle (ADR-S04).

**12. Interdictions explicites.**
- Aucune publication pendant une itération.
- Aucun droit général du Shell de modifier les Ressources du `World`.
- Aucune allocation ni croissance dynamique sur le chemin de l'itération.
- Aucun accès d'un système au lot hors de son contrat.
- Aucun retour d'information du `World` vers la boîte de réception pendant une itération. Les dépôts externes restent permis à tout moment ; après la coupure, les événements reçus restent en attente pour une publication ultérieure.
- Aucune interprétation métier dans les données d'Acquisition.

## Test méthodologique (`00-principes.md`)
1. **Que possède ce domaine ?** Le Shell possède les signaux bruts, leur traduction et les sources de temps (ADR-S01, points 2 et 5). Il assume la collecte, la traduction et la publication du lot (points 1, 3 et 5). Le stockage de la boîte de réception relève de la Plateforme, jamais du `World` (point 3).
2. **Que produit-il ?** Des données d'Acquisition standardisées (point 1), remises en un lot par itération logique, éventuellement vide (points 3 et 6).
3. **Qui est autorisé à consommer ?** Après la publication, seuls les systèmes dont le contrat déclare la lecture, en lecture seule et sans restriction de nombre (point 8). Le corpus pose la consommation unique pour les productions à résoudre (`00-principes.md`). Cette ADR clarifie que la règle ne s'étend pas à la lecture d'une entrée de frontière.
4. **Que lui est-il interdit de connaître ?** La boucle générique ignore les types du noyau, les systèmes, les phases et les contrats (ADR-S01, point 3). Le Shell n'interprète pas les signaux (point 1). Les systèmes ignorent l'origine du lot et l'instant de publication (point 8). Le lot ne contient aucun type de Plateforme.
5. **Quelle frontière protège cette ignorance ?** Le point de passage d'ADR-000, point 3, l'invariant d'ADR-S01, point 6, et le contrat déclaratif d'ADR-003. La coupure unique entre deux itérations (point 7) les matérialise.

## Ce qui n'est pas tranché ici
- Bibliothèque de fenêtrage et encodage bas niveau (point 2).
- Catalogue des sortes d'événements standardisés. Il n'est pas substituable comme le reste, car le changer modifie les systèmes.
- Présence éventuelle d'un instant temps réel dans une donnée (ADR-S04).
- Boucle (ADR-S02), politique temporelle (ADR-S04), orchestration (ADR-S05) et moitié « Extraction » d'ADR-000.
- Clé et stockage du porteur, remise à vide, valeurs de capacité.
- Remontée d'une saturation vers la logique de jeu.

## Classification

| Décision | Statut |
|---|---|
| Le Shell traduit sans interpréter | Exigence du corpus (ADR-S01, point 2) |
| Aucun type de Plateforme ni propriété partagée | Exigence (ADR-000, point 3) |
| Capacités fixées, pas de croissance dynamique | Exigence (ADR-000) |
| Accès par contrat déclaratif uniquement | Exigence (ADR-003, point 5 ; ADR-005, point 2) |
| Aucun retour vers une itération en cours | Exigence (ADR-S01, point 6) |
| Donnée transitoire : durée de vie bornée, état initial déterministe avant toute lecture, propriétaire de durée de vie | Exigence (`01-runtime-ecs.md` §4), sans désigner le propriétaire du lot |
| Déterminisme pour une même séquence de lots publiés | Exigence (invariant 3), avec limite explicitée |
| Mécanismes de traduction substituables | Clarification de portée (méta-règle) |
| Plusieurs lecteurs ; consommation unique non applicable à la lecture d'une entrée | Clarification de vocabulaire et de portée (le corpus n'en détaille pas toutes les conséquences) |
| Quatre rôles distincts à la frontière | Clarification de portée |
| Responsabilité du Shell tenant lieu de propriétaire de durée de vie | Décision nouvelle (précédent : ADR-A01 §4) |
| Ressource dédiée au lot courant, transitoire | Décision nouvelle (fondée sur le modèle général des Ressources, ADR-005) |
| Porteur initialisé à un lot vide déterministe ; publication obligatoire avant chaque itération, première comprise ; renouvellement avant chaque itération ; durée d'une itération | Décision nouvelle |
| Coupure unique entre deux itérations, sémantique du prélèvement | Décision nouvelle |
| Refus explicite à saturation, existant conservé dans l'ordre | Décision nouvelle (instancie ADR-000) |
| Pas de publication partielle | Décision nouvelle |
| Interdiction d'écriture et de production du lot : règle de composition | Décision nouvelle, vérifiée par contrôle et revue (non imposée par les types) |
