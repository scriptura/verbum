# ADR-S04 — Politique temporelle du Shell

## Statut
**Adopté — 11 novembre 2026.**

Historique : projet rédigé le 11 novembre 2026 (statut Proposé), après adoption d'ADR-S02 et validation d'une page d'axes préparatoire (non normative, révision 4 du 11 novembre 2026), corrigé après audit le même jour, puis adopté. Cette ADR ne modifie aucune ADR adoptée. Elle instancie trois délégations explicites : ADR-S01 point 5 (la traduction du temps réel en temps logique est une politique du Shell, réservée à ADR-S04) ; ADR-S02 point 12 (quantum et provenance, nombre d'itérations par progression, rattrapage, abandon ou resynchronisation après retard, affectation temporelle des lots d'entrées, détermination « une itération est-elle due » sans attente) ; ADR-S03 point 11 (répartition des événements entre lots).

Restent explicitement délégués : la transmission et la répétition du Frame et le cycle de vie du Backend (ADR-R03 ou ADR-S05 éventuelle) ; le cycle de vie de la plateforme (suspension, reprise, éventuel contrôle de pause) ; toute modification de la Forge et du format des artefacts (étape 5 et cadre d'implémentation associé).

## Contexte
ADR-S01 fait du Shell le propriétaire des sources de temps réel (point 5) et renvoie la politique de traduction temps réel → temps logique à une future ADR-S04. ADR-S02 définit une progression non bloquante qui exécute 0..N itérations, rend compte du nombre d'itérations, du besoin de réveil et de l'état terminal, et précise que l'adaptateur ne fournit ni politique temporelle ni instant décisionnel. Elle laisse ouverts le quantum, le nombre d'itérations par progression et le traitement du retard. ADR-S03 pose le déterminisme pour une même séquence de lots publiés et renvoie à S04 ce qui concerne la répartition des événements entre lots.

Il manque donc : la relation entre le temps logique des ticks et le temps réel ; la règle qui détermine quelles itérations sont dues ; le traitement du retard et de l'interruption ; la provenance du quantum. Aucun texte adopté ne fixe ces points. Le comportement du cadencement provisoire actuel (`FixedStepPacing`) est explicitement provisoire et ne constitue pas une décision.

```
Temps réel (horloge monotone, propriété du Shell)
        │  observation à l'entrée et à la fin d'une progression
        ▼
Échéancier (état du Shell) : échéances espacées de q
        │  K itérations au plus ; résiduel comparé à M
        ▼
0..K itérations du cœur d'itération (ADR-S02) ; compte rendu de progression
```

## Décision

### 1. Temps logique et temps réel : deux notions distinctes
- Le **temps logique** est compté en **ticks**. Un tick correspond à une itération exécutée. Le `World`, les systèmes et l'interpréteur de scripts ne manipulent que des ticks, jamais une durée ni un instant réels (ADR-S01 point 5 ; `00-principes.md` invariant 3).
- Le **temps réel** est lu par le Shell sur une horloge monotone dont il est propriétaire (ADR-S01 point 5 ; ADR-S02 point 5).
- La **politique temporelle** est la règle par laquelle le Shell met en correspondance les deux : elle décide, à partir d'observations du temps réel, combien d'itérations sont exécutées et quand. Elle est propriété du Shell. Elle n'est jamais fournie par l'adaptateur ni passée à chaque demande de progression (ADR-S02 points 5 et 13).
- L'exécution d'une itération fait avancer le temps logique d'un tick. **Une échéance abandonnée ne devient jamais une itération** : aucun tick logique n'est sauté ; le temps logique n'avance simplement pas pour le temps réel correspondant.

### 2. Quantum logique : un par configuration de jeu, cohérence garantie avant exécution
- Il existe un **quantum logique unique par configuration de jeu**, noté q. Il n'existe pas de quantum par script, par système ou par artefact.
- q est une **donnée de build-time connue de la Forge**, qui l'utilise pour convertir les unités humaines en ticks (`00-principes.md` invariant 1).
- **q est une durée strictement positive.** C'est une propriété structurelle du contrat, non une valeur numérique à fixer : elle garantit que l'échéancier est bien défini et que l'ensemble des échéances dues à un instant donné est fini (point 3). Cette ADR ne fixe aucune valeur numérique de q.
- **La cohérence est garantie sur toute la chaîne Forge → artefacts → Shell avant l'exécution.** Le runtime obtient q d'une donnée de build, ou vérifie que sa configuration correspond à celui des artefacts avant leur exécution. Le Shell ne peut pas exécuter silencieusement des artefacts compilés pour un autre quantum, ni choisir indépendamment un quantum incompatible. Le mécanisme de transmission ou de vérification est laissé à l'implémentation.
- Le Shell reste propriétaire de l'horloge réelle et de la politique d'échéance (point 1). Il ne possède pas le choix de q.
- **Règle des durées non entières en ticks.** Une durée humaine qui ne correspond pas à un nombre entier de ticks exige une règle déterministe explicite (rejet, quantification déterministe ou autre). Cette ADR n'en choisit aucune et n'en prescrit aucun mécanisme ; la règle relève probablement de la Forge.
- **Dépendance vers l'étape 5 de Forge.** Cette ADR définit le contrat de cohérence et identifie la dépendance. Elle n'autorise ni ne réalise implicitement une modification du format des artefacts ou du manifeste `VMAN`, protégés par la porte G-CM de l'étape 5. Cette évolution se traite dans le cadre documentaire et d'implémentation approprié.
- **Cadence nominale et progression effective.** L'échéancier a une **cadence nominale** : deux échéances successives (point 3) sont espacées de q en temps réel. Ce n'est pas une garantie qu'un tick logique est exécuté pour chaque intervalle réel q écoulé : le retard conservé, le rattrapage et l'abandon (points 4 à 7) font que le nombre de ticks exécutés peut diverger du temps réel écoulé. La **progression effective** du temps logique est d'un tick par itération exécutée (point 1). Cette ADR n'introduit aucun facteur d'échelle entre temps logique et temps réel (ralenti, accéléré) ; une telle notion ne serait pas couverte par cette ADR.

*Note non normative sur l'état actuel.* Aucun artefact ne porte de quantum ; la durée de pas du cadencement provisoire est fournie par l'appelant ; aucune conversion secondes → ticks n'existe. Le principe 1 de `00-principes.md` n'est pas encore opérationnel faute de cette donnée.

### 3. Échéancier : définition, initialisation, avancement

**3.1 Définition.** Le Shell tient un **échéancier** : suite d'**échéances**, instants réels planifiés, espacées de q, avec un repère sur la prochaine échéance ni exécutée ni abandonnée. Une échéance est **due à l'instant t** si elle est planifiée à un instant inférieur ou égal à t et n'a été ni exécutée ni abandonnée. Comme q est strictement positif (point 2), l'ensemble des échéances dues à un instant donné est fini. Entre deux réancrages (point 7), l'échéance suivant une échéance exécutée est cette échéance augmentée de q : elle ne dépend pas de l'instant réel de l'exécution. Il n'y a donc pas de dérive due à la durée des itérations. L'échéancier est un état du Shell, de la strate Plateforme ; il n'est jamais lu par le `World`.

**3.2 Initialisation de l'échéancier.** Le Shell dispose d'un échéancier initial **établi avant de déterminer les itérations dues** : c'est une condition préalable à toute progression qui a un effet. Cette initialisation relève de la composition et de l'initialisation du Shell (ADR-S01 points 1 et 7). Aucun texte adopté n'en fixe la mécanique (instant d'origine, première échéance) : elle est laissée à l'implémentation. Elle est **distincte du réancrage** du point 7, qui n'a lieu qu'après un abandon, et ne s'en déduit pas. Le comportement du cadencement provisoire actuel au démarrage, notamment le délai avant la première itération, ne constitue pas une règle.

### 4. Progression : observations d'entrée et de fin
Une progression (ADR-S02 point 4) applique la règle suivante.

1. **Observation d'entrée.** Le Shell observe l'heure t_entrée et détermine l'ensemble D des échéances dues à t_entrée.
2. **Cas D vide.** Si D est vide : aucune itération, aucune coupure, aucune seconde observation, aucun abandon (ADR-S02 point 4 : sans effet si rien n'est dû).
3. **Exécution.** Le Shell exécute au plus K itérations parmi D, dans l'ordre des échéances. Chaque itération applique le cœur d'itération d'ADR-S02 (point 2) avec sa propre coupure et sa propre publication. **Les échéances devenues dues pendant l'exécution ne sont pas ajoutées à D** : l'ensemble d'itérations d'une progression est celui déterminé à l'entrée ; la progression n'est pas une boucle de réévaluation continue des échéances.
4. **Observation de fin.** Le Shell observe l'heure t_fin et calcule le **résiduel** R : le nombre d'échéances dues à t_fin, ni exécutées ni abandonnées. R comprend le reliquat de D non exécuté et les échéances devenues dues pendant l'exécution.
5. **Décision.** R est comparé à M (point 5) : si R ≤ M, les échéances résiduelles sont **conservées** ; si R > M, **toutes** les échéances résiduelles sont **abandonnées** et l'échéancier est réancré (point 7).

Le nombre d'itérations normalement exécutées est donc min(K, |D|), sous réserve des conditions d'arrêt, d'erreur et d'échec de publication prévues par ADR-S02 et ADR-S03 (point 8).

**Seconde observation de l'heure.** Cette ADR autorise explicitement le Shell à lire l'heure plus d'une fois par progression. C'est une précision de portée d'ADR-S02 : la formule « à un instant que le Shell observe lui-même » (point 4) qualifie l'ensemble des itérations dues, non le nombre de lectures de l'horloge, que ADR-S02 ne limite pas (point 5).

**Progression terminale.** Si la progression se termine par un arrêt observé ou par une erreur (y compris un échec de publication), le traitement terminal d'ADR-S02 prime : aucune décision de conservation ni d'abandon n'est prise pour poursuivre un échéancier devenu sans objet.

**Progression suivante.** Chaque progression détermine son propre ensemble D à sa propre observation d'entrée. Les échéances conservées en font partie si elles sont dues.

### 5. Paramètres K et M : structure du contrat, valeurs hors contrat
- **K** : nombre maximal d'itérations exécutées dans une progression.
- **M** : seuil du résiduel, en nombre d'échéances, au-delà duquel les échéances résiduelles sont abandonnées.
- K et M sont des **entiers strictement positifs, finis et explicites**. Ils sont **indépendants** : aucune relation implicite n'existe entre eux, en particulier pas K = M / q.
- **Règle de comparaison** : un résiduel inférieur ou égal à M est conservé ; un résiduel strictement supérieur à M est abandonné.
- Leurs valeurs numériques sont des **paramètres de composition** de l'instance de Shell : fixées à sa composition (ADR-S01 point 7), **stables pendant son fonctionnement**, jamais fournies librement à chaque appel de progression. Cette ADR ne fixe aucune valeur numérique de K, de M ni de q.
- **Ce que bornent K et M, et ce qu'ils ne bornent pas.** K borne le nombre d'itérations d'**un appel**, pas son temps réel d'exécution ni le travail total nécessaire pour résorber un retard. M borne le résiduel **à l'instant de mesure** (t_fin), pas le retard entrant dans une progression ni le retard à tout instant. M n'est ni une garantie de retard maximal ni une garantie de latence.

### 6. Résiduel conservé et résiduel abandonné
- **Résiduel conservé (R ≤ M)** : les échéances restantes demeurent dues. La prochaine progression les verra et pourra les exécuter. Aucun tick n'est perdu.
- **Résiduel abandonné (R > M)** : toutes les échéances résiduelles sont supprimées de l'échéancier. Elles ne deviennent pas des itérations (point 1).
- **Conséquence après une interruption importante.** Lorsque le retard à l'entrée est grand, la progression exécute d'abord min(K, |D|) itérations tardives (point 4), puis observe t_fin et décide. L'abandon n'est décidé **qu'après** ces itérations. La politique n'« saute » donc pas une interruption : elle exécute une première tranche d'itérations tardives avant d'abandonner le reste, si R dépasse M. Si R reste ≤ M, tout est conservé et rattrapé par les progressions suivantes.
- Le fait que M soit strictement positif implique qu'un résiduel d'une seule échéance est toujours conservé.

### 7. Réancrage R1 [règle normative]
Lorsqu'un abandon est décidé à t_fin :
- l'**instant de réancrage** est t_fin ;
- la **prochaine échéance** est fixée à t_fin + q ;
- la **phase de l'ancien échéancier n'est pas conservée** ;
- la prochaine itération n'est donc **pas immédiatement due**.

L'instant de réancrage et la prochaine échéance sont deux notions distinctes : réancrer à t_fin signifie que la prochaine échéance vaut t_fin + q, non qu'une itération est immédiatement due.

Cette règle appartient au contrat architectural, non à une liberté d'implémentation : le réancrage détermine les échéances futures et modifie donc la suite observable des instants d'échéance, et par suite la trace des itérations (point 9). C'est une sémantique temporelle du Shell, non un détail de représentation ou de calcul. L'implémentation reste libre de la représentation de l'échéancier et du calcul.

**Alternative écartée.** Conserver la phase par rapport à l'origine de l'échéancier (prochaine position de la grille initiale). Aucune exigence adoptée ne la justifie : ADR-AU01 ne pose aucune exigence de synchronisation avec l'horloge du périphérique audio et ADR-N01 ne pose ni phase, ni tick commun, ni alignement d'échéanciers entre instances. Conserver une phase par rapport à l'origine locale n'aligne par ailleurs rien sur une référence externe. Ce choix est révisable par une ADR future qui introduirait une véritable exigence de synchronisation temporelle ou d'alignement de phase.

### 8. Compte rendu de progression : obligations de S04
Le contenu du compte rendu est celui d'ADR-S02 point 6 (itérations exécutées, besoin de réveil, état terminal). S04 en précise le besoin de réveil, sans fixer de représentation ni de signature :

| Situation en fin de progression | Besoin de réveil restitué |
|---|---|
| D vide à l'entrée | prochaine échéance planifiée, postérieure à t_entrée ; aucune garantie sur sa position par rapport à l'instant de réception du compte rendu |
| R = 0 | prochaine échéance planifiée, postérieure à t_fin |
| 0 < R ≤ M | la prochaine échéance non exécutée, **déjà due** |
| R > M | t_fin + q (point 7) |

- Les garanties de position portent sur l'instant d'observation (t_entrée ou t_fin), non sur l'instant où le compte rendu est reçu ou exploité : une échéance restituée peut avoir été dépassée entre-temps.
- Le nombre d'itérations exécutées vaut au plus K.
- **Une échéance restituée peut être antérieure à l'instant de restitution.** Elle signifie alors « demander une progression dès que possible », et non une attente jusqu'à un instant futur. Cette lecture est compatible avec ADR-S02 point 6 : l'échéance y est « souhaitée » et un réveil superflu est sans conséquence.
- En dehors de l'état terminal, la politique de S04 fournit toujours une prochaine échéance. **Aucune règle de S04 ne produit « aucune échéance »** : ce cas d'ADR-S02 point 6 reste disponible, mais n'est pas utilisé ; il ne sert pas de mécanisme de pause (point 11).
- Le compte rendu d'ADR-S02 ne porte aucune information sur les échéances abandonnées ; S04 n'en ajoute pas.
- Dans une progression terminale, le compte rendu porte l'état terminal ; aucun besoin de réveil ni aucune décision de conservation ou d'abandon ne sont requis (point 4).

### 9. Lots d'acquisition et déterminisme
**9.1 Aucune règle supplémentaire d'affectation des lots.** S04 ne crée aucune règle d'affectation temporelle des lots à l'échelle d'une progression. Cet énoncé résout le renvoi d'ADR-S02 point 12 (affectation temporelle des lots d'entrées) et d'ADR-S03 point 11 : la règle est celle qui découle du cœur d'itération adopté.
- Chaque itération réalise sa propre coupure de l'`Inbox` et sa propre publication (ADR-S02 point 2 ; ADR-S03 points 6 et 7). Les lots vides comptent. Aucun lot n'est rejoué.
- Les événements déjà déposés dans l'`Inbox` avant la première coupure d'une progression sont consommés par la première itération. Cela ne s'étend pas aux événements encore en attente dans la file native de la plateforme, qui peuvent ne pas avoir été distribués à l'adaptateur donc ne pas avoir été déposés.
- Dans le modèle monothread initial, aucun signal de plateforme n'est distribué pendant une progression (ADR-S02 points 8 et 9) : en l'absence de nouveaux dépôts, les coupures suivantes d'une même progression reçoivent des lots vides. Ce n'est **pas** une garantie absolue : d'autres fils peuvent déposer à tout moment (ADR-S03 point 3), et un tel dépôt peut figurer dans le lot d'une itération ultérieure de la même progression.
- **Échec de publication.** Un lot valide, éventuellement vide, est publié avant chaque itération effectivement exécutée. Si la publication échoue, l'itération correspondante n'est pas lancée (aucune invocation) et n'est pas comptabilisée comme exécutée (ADR-S02 point 2 ; ADR-S03 point 6).

**9.2 Déterminisme.** La politique ne modifie pas le déterminisme posé par ADR-S03 point 11 : pour une même séquence de lots effectivement publiés, la séquence d'états est la même. La séquence de lots publiés compte un lot par itération exécutée, lots vides compris. Le nombre d'itérations et les frontières entre lots font donc partie de ce qui caractérise une exécution ; la seule suite des événements non vides ne la détermine pas.
- La politique fait dépendre du temps réel le **nombre** et les **frontières** des itérations, donc la suite des lots publiés. Deux exécutions dont les observations de l'horloge diffèrent peuvent produire des traces d'itérations différentes, et donc des suites d'états différentes. Aucune équivalence d'états n'est affirmée entre deux traces différentes.
- Ce recours au temps réel n'est pas un ordonnancement dynamique au sens de `00-principes.md` invariant 3 : l'ordre des systèmes et des phases reste fixe (ADR-004 ; ADR-S01 point 3) ; seul le nombre d'itérations, déjà reconnu dépendre du Shell par ADR-S01 point 5, varie.
- Rejouer fidèlement une exécution suppose de disposer de la suite complète des lots publiés, lots vides compris. Cette ADR ne spécifie pas leur enregistrement.

### 10. Finitude, garanties et limites
**Garantie.** Chaque progression se termine en temps fini (ADR-S02 point 4) : au plus K itérations sont exécutées, la seconde observation de l'heure n'est ni une boucle ni une attente, et l'ensemble d'itérations est fixé à l'entrée.

**Non garanties.** Cette ADR ne garantit pas :
- une **durée réelle maximale** de progression : le coût d'une itération n'est pas borné, et un produit « K fois le coût d'une itération » est une estimation de travail, non une borne de durée. La progression s'exécutant de façon synchrone sur le fil de la boucle de plateforme dans le modèle initial (ADR-S02 point 9), une progression longue retarde la distribution des signaux de plateforme ;
- une **faible latence** de réponse aux entrées ;
- un **retard maximal** : M ne borne que le résiduel mesuré à t_fin ; une interruption peut créer un retard arbitrairement grand avant la progression suivante (point 5) ;
- la **convergence du rattrapage** : le retard n'est résorbé que si le débit d'exécution durable du Shell, cycle complet compris (retour à la boucle de plateforme, traitement des événements, éventuelle présentation bloquante), dépasse strictement le débit d'arrivée des échéances. À égalité, le retard n'est pas résorbé et fluctue avec la gigue. Sous une surcharge durable, le résiduel dépasse M et des abandons se répètent : c'est le comportement prévu de la politique, non une défaillance.

**Fidélité.** Après un abandon, le temps réel écoulé dépasse le temps logique simulé. Le temps logique conserve son sens (ticks) ; ce sont les durées réelles associées, par exemple celle d'un script suspendu par `Wait`, qui peuvent s'étirer. Aucune exigence adoptée ne demande que le temps simulé suive le temps réel.

### 11. Interruption, suspension, présentabilité
- La politique temporelle s'applique **indépendamment de la cause du retard** : interruption du processus, veille système, ou toute autre situation produisant un retard constaté par le Shell.
- ADR-S02 ne prévoit **aucune pause explicite**. Une absence de présentabilité (fenêtre minimisée, surface indisponible) ne constitue donc pas, à elle seule, une pause du Shell. Si la boucle de plateforme continue de solliciter le Shell, les progressions continuent selon la politique ; si elle cesse de le solliciter, le retard est traité au retour par cette même politique (point 6).
- Cette ADR **ne crée ni état de pause ni canal de commande**. « Aucune échéance » n'est pas un mécanisme de pause (point 8).
- La suspension d'une fenêtre, la veille du système et le gel d'un processus ne sont pas nécessairement les mêmes phénomènes temporels : le Shell ne les distingue que par l'effet constaté sur son horloge. Que l'horloge monotone comptabilise ou non la veille système dépend de la plateforme ; aucune garantie portable n'est affirmée, et aucun comportement d'une plateforme particulière n'est présenté comme universel.
- La disponibilité de la présentation, le signal de suspension ou de reprise adressé au Shell et l'existence éventuelle d'un contrôle de pause restent des dépendances du cycle de vie de la plateforme, hors de cette ADR.

### 12. Frontières, dépendances, pilotes
- **Pilotes.** La politique est la même pour le pilote bloquant et le pilote événementiel d'ADR-S02. L'équivalence des pilotes d'ADR-S02 point 3 reste exigée pour une même séquence d'observations ; les lectures de l'heure du Shell en font partie. La forme de l'adaptation du pilote bloquant est laissée à l'implémentation.
- **Source de temps.** Elle est substituable à l'assemblage (ADR-S02 point 5), jamais par un instant fourni à chaque progression. La substitution doit permettre de scripter séparément t_entrée et t_fin.
- **ADR-R03 / ADR-S05 éventuelle.** La transmission d'une représentation de frame, son éventuelle répétition et le cycle de vie du Backend ne sont pas décidés ici. Cette ADR ne rend pas impossible « 0..N itérations pour 0..1 représentation transmise » (ADR-S01 point 5). Elle précise que, pour une progression de plusieurs itérations, le moyen de désigner la dernière itération relève de ces décisions.
- **ADR-AU01.** Elle ne dit pas quand le Shell applique les commandes audio résolues. Pour une progression de plusieurs itérations, ce moment n'est pas déterminé par cette ADR ; il se traite avec l'extraction.
- **ADR-N01.** Chaque instance reste locale et déterministe. L'alignement des ticks entre instances relève de la topologie et de la compensation de latence (ADR-N01 point 6), hors de cette ADR. Le quantum unique (point 2) est cohérent avec la nécessité d'une unité de temps logique commune entre instances.
- **Forge / étape 5.** Voir point 2.
- **Plateforme.** Aucun type de fenêtrage ni de graphique, aucune API de plateforme dans les contrats de la politique.

### 13. Interdictions explicites
- Aucune durée ni aucun instant réel fourni au `World`, aux systèmes ou à l'interpréteur de scripts.
- Aucun instant décisionnel ni aucune politique fournis par l'adaptateur ; aucun K ni M fournis à chaque appel de progression.
- Aucune réévaluation de l'ensemble d'itérations pendant l'exécution d'une progression.
- Aucune règle d'affectation des lots à l'échelle d'une progression ; aucun lot rejoué.
- Aucune décision de conservation ou d'abandon dans une progression terminale.
- Aucun état de pause, aucun canal de commande, aucune dépendance entre présentabilité et chronologie introduits par cette ADR.
- Aucune modification implicite du format des artefacts ou de `VMAN`.
- Aucun nouveau type d'événement d'Acquisition ni aucune datation par instants.
- Aucune valeur numérique de K, de M ou de q dans le contrat.

## Ce qui n'est pas tranché ici
- Valeurs de K, de M et de q ; leur représentation.
- Mécanique d'initialisation de l'échéancier (origine, première échéance).
- Règle de conversion des durées non entières en ticks ; mécanisme de transmission ou de vérification du quantum entre build et runtime ; modification éventuelle du format des artefacts.
- Facteur d'échelle entre temps logique et temps réel.
- Observabilité des abandons ; enregistrement de la suite des lots publiés.
- Signal de suspension, reprise ou non-présentabilité ; contrôle de pause ; comportement de l'horloge monotone pendant la veille, par plateforme.
- Transmission du Frame, désignation de la dernière itération d'une progression, cycle de vie du Backend ; moment d'application des commandes audio ; alignement des ticks entre instances.
- Forme de l'adaptation du pilote bloquant à la politique.

## Alternatives examinées

| Alternative | Statut | Raison |
|---|---|---|
| Une itération par progression, retard conservé sans limite (K = 1, conservation illimitée) | Écartée comme politique | Le retard croît sans borne sous surcharge durable ; correspond à M infini, exclu par l'exigence d'un M fini |
| Abandon systématique de tout résiduel (M nul), notamment K = 1 avec réancrage à l'instant courant | Écartée comme politique | Abandonne le retard au moindre dépassement ; correspond à M nul, exclu par l'exigence d'un M strictement positif. Son comportement n'est pas une décision |
| Rattrapage non borné | Écartée | Finie par l'ensemble fixé à l'entrée, mais sans borne pratique sur le travail d'une progression synchrone sur le fil de boucle |
| Politique à seuil (K, M finis, résiduel mesuré à la fin) | **Retenue** | Borne le travail d'un appel, conserve le retard modéré, contrôle le retard produit par l'exécution elle-même, n'abandonne qu'au-delà de M |
| Mesure de M à l'entrée (retard d'entrée comparé à M) | Écartée | Ne voit pas le retard créé pendant l'exécution avant la progression suivante ; décide d'abandonner avant d'exécuter |
| Réévaluation des échéances dues avant chaque itération | Écartée | S'écarte de la lettre d'ADR-S02 point 4 (ensemble déterminé à un instant) |
| Réancrage sur la grille initiale (R2) | Écartée | Aucune exigence adoptée (point 7) |
| Pause explicite par « aucune échéance » ou par état du Shell | Non retenue | Non établie par ADR-S02 ; dépendance du cycle de vie de la plateforme |
| Règle d'affectation des lots à l'échelle de la progression | Écartée | Contredit le cœur d'itération adopté (chaque itération coupe seule) |

## Les six questions de la grammaire des ADR (`00-principes.md`)
1. **Contrat du domaine.** Le Shell décide, par une politique à échéancier, quelles itérations sont dues à chaque progression : au plus K parmi les échéances dues à l'entrée, résiduel mesuré à la fin et comparé à M, réancrage R1 en cas d'abandon (points 3 à 8). Le quantum est unique par configuration de jeu et cohérent de la Forge au Shell (point 2).
2. **Qui possède quoi.** Le Shell : horloge, échéancier, application de K et de M, décision d'abandon, initialisation de l'échéancier. La Forge : conversion en ticks et connaissance de q à la construction. L'adaptateur : consommation du compte rendu, aucune politique. Le `World` : ticks seulement.
3. **Frontières.** Frontière de connaissance : le `World` et les systèmes ignorent le temps réel ; l'adaptateur ne décide pas de l'échéance. Frontière de strate : tout le domaine appartient à la Plateforme, hors l'obligation sur q. Frontière de compilation : q est connu de la Forge ; la cohérence artefacts/Shell est un contrat.
4. **Garanties.** Progression finie ; échéancier sans dérive entre deux réancrages ; aucun lot rejoué ; aucune décision dans une progression terminale ; déterminisme pour une même séquence de lots publiés (points 4, 9 et 10).
5. **Exclusions.** Durée réelle maximale, latence, retard maximal, convergence ; pause ; transmission du Frame ; modification de la Forge ; affectation des lots par progression (points 10 et 13).
6. **Laissé à l'implémentation.** Valeurs, représentation de l'échéancier, initialisation, mécanisme de cohérence du quantum, règle des durées non entières, pilote bloquant (section « Ce qui n'est pas tranché »).

## Test méthodologique (`00-principes.md`)
1. **Que possède ce domaine ?** L'échéancier, la lecture de l'horloge, les paramètres K et M, la décision d'abandon.
2. **Que produit-il ?** Pour chaque progression, l'ensemble des itérations exécutées, la décision de conservation ou d'abandon et le besoin de réveil du compte rendu.
3. **Qui consomme ?** Le cœur d'itération consomme l'ensemble des itérations dues ; l'adaptateur consomme le compte rendu (ADR-S02).
4. **Que lui est-il interdit de connaître ?** Le contenu et le sens des lots ; les types de fenêtrage et de graphique ; l'état de présentation ; l'intérieur d'une itération.
5. **Quelle frontière protège cette ignorance ?** Frontière de connaissance (ADR-S01 points 2, 3 et 5 ; ADR-S02 points 11 et 13) et frontière de compilation pour q.

## Test de réduction (méthode d'ADR-W01 et d'ADR-S02)
1. *Nouveau type d'état ?* Non : l'échéancier appartient à la Plateforme (Shell) ; aucune catégorie de donnée du modèle architectural n'est introduite.
2. *Nouveau modèle d'exécution ?* Non : l'ordre interne reste celui d'ADR-004, construit une fois et immuable ; seul varie le nombre d'itérations par progression, déjà ouvert par ADR-S01 point 5.
3. *Nouvelle frontière ?* Non : instanciation des frontières de connaissance et de compilation.
4. *Concept nouveau ?* Le vocabulaire « échéancier », « échéance due », « résiduel », « abandon », « réancrage », « quantum logique », « seuil M » et « borne K » est local à cette ADR ; leur éventuelle entrée dans `02-concepts.md` est à examiner après adoption. Le terme `Event` n'est pas utilisé.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Temps logique en ticks, temps réel lu par le Shell ; le `World` ne voit que des ticks | Invariant (instanciation d'ADR-S01 point 5 et de `00-principes.md` inv. 3) |
| Une échéance abandonnée ne devient jamais une itération ; aucun tick sauté | Invariant |
| Quantum logique unique par configuration de jeu, connu de la Forge | Invariant (instanciation de `00-principes.md` inv. 1) |
| Quantum q strictement positif (valeur non fixée) | Invariant |
| Cohérence Forge → artefacts → Shell garantie avant exécution | Invariant (contrat) ; mécanisme hors périmètre |
| Règle explicite pour les durées non entières | Exigence ; règle hors périmètre |
| Échéancier espacé de q, sans dérive entre deux réancrages | Invariant |
| Échéancier initial établi avant de déterminer les itérations dues ; mécanique laissée à l'implémentation | Invariant (existence) ; mécanique hors périmètre |
| Ensemble d'itérations déterminé à l'entrée ; au plus K itérations ; échéances apparues pendant l'exécution exclues | Invariant |
| Seconde observation de l'heure autorisée | Précision de portée (ADR-S02 points 4 et 5) |
| Résiduel mesuré à t_fin ; ≤ M conservé, > M abandonné | Invariant |
| K et M entiers strictement positifs, finis, indépendants, stables par instance, non fournis par appel | Invariant (existence et rôle) ; valeurs : paramètres de composition |
| K ne borne pas la durée ; M borne le résiduel à l'instant de mesure seulement | Invariant (limites) |
| Réancrage R1 : prochaine échéance = t_fin + q, phase non conservée | Invariant (règle normative) |
| Échéance restituée possiblement déjà passée | Précision de portée (ADR-S02 point 6) |
| Aucune règle de S04 ne produit « aucune échéance » ; pas de pause | Invariant |
| Aucune règle d'affectation des lots par progression ; chaque itération coupe seule ; échec de publication : itération non lancée ni comptée | Invariant (conséquence d'ADR-S02 et ADR-S03) |
| Déterminisme pour une même séquence de lots publiés ; trace d'itérations dépendante du temps réel | Invariant (instanciation d'ADR-S03 point 11) |
| Politique indépendante de la cause du retard ; présentabilité sans effet propre sur la chronologie | Invariant |
| Valeurs de K, M et q ; représentation de l'échéancier ; initialisation ; mécanisme de cohérence du quantum | Hors périmètre — composition et implémentation |
| Format des artefacts et de `VMAN` ; Forge | Hors périmètre — étape 5 et cadre d'implémentation |
| Transmission du Frame, cycle de vie du Backend, application des commandes audio, alignement multi-instances, contrôle de pause | Hors périmètre — autres décisions |

## Maturité, conditions d'adoption, conformité de l'implémentation
**1. Maturité du domaine.** Les six questions de la grammaire sont renseignables pour le périmètre de cette ADR : le domaine est suffisamment mûr. Les arbitrages sur K, M, la mesure du résiduel, R1, l'échéance restituée, les lots et la présentabilité sont rendus.

**2. Conditions d'adoption.** Aucune condition préalable ne subsiste. Les expériences E1, E2, E8, E10, E14 et E15 sont **exploratoires et n'ont pas été exécutées** ; aucune n'est prérequis à l'adoption et le contrat logique ne dépend d'aucun de leurs résultats. Elles éclairent les choix d'implémentation : réveil et réactivité (E1), blocage de présentation (E2), non-présentabilité (E8), précision de réveil (E10), calibrage de K et de M à partir du débit d'exécution durable (E14, sans garantie universelle), horloge pendant veille et gel par plateforme (E15, sans définir la portabilité de l'horloge).

**3. Tests de conformité de l'implémentation** (après implémentation, avec une source de temps substituée, sans `sleep` ni horloge murale).
- Avant l'échéance : aucune itération, aucune coupure, besoin de réveil restitué.
- À l'échéance exacte ; retard inférieur à un quantum ; plusieurs échéances dues.
- Jamais plus de K itérations par progression ; nombre d'itérations égal à min(K, |D|).
- Deux observations de l'heure scriptées séparément ; échéances devenues dues pendant l'exécution comptées dans le résiduel mais absentes de l'ensemble exécuté.
- Frontière du seuil : R = M conservé, R = M + 1 abandonné.
- Abandon : prochaine échéance = t_fin + q, non immédiatement due ; aucune échéance abandonnée n'est exécutée ultérieurement.
- Interruption longue : min(K, |D|) itérations tardives exécutées avant l'abandon.
- Échéance restituée déjà passée lorsque le résiduel est conservé.
- Absence de dérive à retard nul sur longue durée.
- Arrêt ou erreur en cours de progression : aucune décision de conservation ni d'abandon ; échec de publication : aucune itération comptée.
- Plusieurs progressions successives : lots vides comptés, aucun lot rejoué, y compris avec un dépôt d'un autre fil entre deux coupures d'une même progression.
- Équivalence des pilotes bloquant et événementiel pour une même séquence d'observations.
- Initialisation de l'échéancier avant la première progression, indépendamment de R1.
