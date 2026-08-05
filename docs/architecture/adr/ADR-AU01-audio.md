# ADR-AU01 — Domaine Audio

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
La page d'axes a ouvert une question centrale sans la trancher : le domaine Audio est-il une projection d'état (motif du Renderer), un flux de commandes impératives, ou une troisième forme hybride ? Cette ADR met trois modèles concrets en concurrence et les confronte aux invariants déjà posés, plutôt que d'en choisir un par préférence.

## Décision

### 1. Quel est le contrat du domaine ? — confrontation des trois modèles

**Modèle A — Projection complète (motif Renderer strict).**
Chaque itération, les systèmes composent une description complète de l'état sonore désiré (tous les émetteurs actifs, leurs paramètres). Le Shell reçoit cette projection entière et doit lui-même déterminer quelles voix démarrer, mettre à jour, ou arrêter par rapport à l'itération précédente.

*Élimination affinée* : le problème n'est pas que le Shell compare deux états successifs — comparer des valeurs pour éviter un appel API redondant (ex: ne pas réappeler `SetGain` si le gain n'a pas changé) est une optimisation transparente d'application d'un état déjà déterminé, qui ne modifie en rien le résultat logique observable, et reste légitimement du ressort du Shell. Ce qui est interdit, c'est que le Shell **décide d'un nouvel état logique** qui n'était écrit nulle part dans les données reçues — par exemple, décider qu'un son « doit démarrer » sans que cette transition n'ait été explicitement produite par le modèle architectural. C'est cette décision de transition logique, et elle seule, qui viole « le Shell traduit, il n'interprète jamais » (ADR-S01, point 2).

Le Modèle A tel que formulé initialement (« le Shell détermine lui-même quelles voix démarrer, mettre à jour, ou arrêter ») reste éliminé, non pas parce qu'il impliquerait un calcul de différentiel en général, mais parce qu'il fait porter au Shell la décision de transition elle-même, plutôt qu'une simple optimisation d'un état déjà tranché en amont.

**Modèle B — Commandes impératives.**
Les systèmes métier suivent eux-mêmes l'état de continuité d'un son (via un composant persistant sur l'entité émettrice) et émettent des commandes discrètes (`StartLoop`, `UpdateLoop`, `StopLoop`) au moment des transitions.

*Évaluation* : ce modèle ne viole aucun invariant — un composant persistant avec un propriétaire unique en écriture est une instanciation ordinaire déjà couverte (`02-concepts.md`). Il reste valide, mais disperse la logique de suivi de continuité dans chaque système producteur, sans point de centralisation.

**Modèle C — Déclaratif suivi d'une résolution de différentiel, au sein du modèle architectural.**
Les systèmes produisent, chaque itération, une description de l'état sonore désiré (donnée transitoire, produite librement — ADR-004, exactement comme les contributions de rendu). Un système de résolution (par exemple, un `AudioDiffResolutionSystem`, cohérent avec la règle de nommage par transformation) compare cette description à l'état désiré de l'itération précédente (une Ressource persistante dont il serait l'unique propriétaire en écriture) et produit un différentiel sous forme de commandes discrètes — instanciation directe du motif déjà établi « produire est libre ; résoudre est unique ». Le Shell ne reçoit que ces commandes déjà résolues et se contente de les traduire vers l'API audio concrète.

Ce modèle est une instanciation particulièrement naturelle des invariants existants, pas une obligation architecturale en soi : le contrat du domaine n'exige pas la présence d'un système de résolution nommé ; il exige seulement que toute décision de transition logique (démarrer/continuer/arrêter un son) soit prise dans le modèle architectural, jamais dans le Shell. Un moteur simple pourrait tout à fait émettre directement des commandes de transition depuis les systèmes producteurs eux-mêmes (proche du Modèle B), sans système de résolution dédié ; un moteur plus sophistiqué pourrait centraliser cette logique dans un système de résolution comme celui décrit ici. Les deux satisfont le même contrat.

*Évaluation* : ce modèle satisfait tous les invariants existants sans exception. Le calcul de différentiel, qui disqualifiait le Modèle A côté Shell, est ici légitime car il a lieu côté modèle architectural, où « résoudre » est une opération normale (ADR-004) — le Shell reste un pur traducteur.

**Résultat de la confrontation** : le Modèle A, tel que formulé (le Shell décide lui-même des transitions), est éliminé par contradiction directe avec ADR-S01. Le Modèle B reste valide. Le Modèle C, présenté ci-dessus, en est un raffinement naturel plutôt qu'un modèle concurrent :
- un son ponctuel, sans continuité à suivre (une explosion, un impact, un son d'interface), n'a besoin d'aucun suivi d'état : c'est une intention pure, consommée une fois (instanciation triviale d'ADR-004) — aucune décision de transition n'est nécessaire, un simple `PlaySound` transitoire suffit.
- un son continu dont les paramètres évoluent dans le temps (moteur, pluie, ambiance) requiert que la décision de transition (démarrer/continuer/arrêter) soit prise dans le modèle architectural — que ce soit directement par le système producteur (Modèle B) ou via un système de résolution dédié centralisant cette logique (Modèle C, cf. ci-dessus). Le contrat n'impose que la localisation de la décision, jamais le mécanisme qui la produit.

Ceci n'est pas une propriété spécifique aux sons continus, mais l'instanciation, pour l'audio, d'un principe plus général :

> Tout phénomène possédant un cycle de vie voit ses transitions décidées dans le modèle architectural ; le Shell n'en applique que les effets.

Ce principe dépasse le seul domaine Audio — il s'appliquera de la même manière à tout futur phénomène à cycle de vie que le Shell devra restituer sans jamais le décider (retour haptique, streaming vocal, ou tout autre effet continu de plateforme).

Le contrat du domaine Audio couvre donc deux catégories de phénomènes — instantanés et à cycle de vie — qui satisfont les mêmes invariants par des mécanismes différents, sans que le contrat lui-même ne se scinde en deux. Cette distinction découle de la nature du phénomène sonore (ponctuel vs continu), pas d'une préférence pour une API audio particulière — exactement la réserve posée par GPT.

### 2. Qui possède quoi ?
- Le Shell possède l'unique accès au périphérique audio (dépendance de plateforme, ADR-S01, point 4).
- Les systèmes métier produisent des intentions ponctuelles (`PlaySound`, transitoire) ou une description d'état sonore désiré (transitoire, pour les sons continus).
- L'entité responsable de la décision de transition (qu'elle prenne la forme d'un système de résolution dédié ou d'une logique portée directement par les systèmes producteurs, cf. point 1) est l'unique propriétaire en écriture de tout état persistant nécessaire au suivi de continuité, et l'unique source des commandes transmises au Shell.
- Le Shell ne possède ni ne décide jamais rien côté logique — il traduit des commandes déjà résolues vers l'API audio concrète.

### 3. Quelles frontières s'appliquent ?
Les trois frontières déjà nommées suffisent — confirmé, pas seulement anticipé :
- **Frontière de connaissance** : le Backend audio ignore l'ECS ; le modèle architectural ignore le périphérique audio concret.
- **Frontière de strate** : la production d'intentions et la résolution de différentiel sont Modèle architectural ; la traduction et l'appel à l'API audio sont Plateforme (Shell) ; le format interne de la Ressource d'état sonore désiré est Structure interne.
- **Identité — point désormais quasi démontré** : un identifiant d'émetteur sonore, s'il existe (nécessaire pour tout suivi de continuité, indépendamment du mécanisme retenu), est strictement un identifiant **logique**, jamais un handle de l'API audio sous-jacente (voix OpenAL, canal SDL, handle FMOD/Wwise). C'est une instanciation directe du confinement des dépendances de plateforme déjà posé (ADR-S01, point 4) : le modèle architectural ne manipule jamais un handle de plateforme. Le Shell traduit cet identifiant logique vers le handle concret au moment d'appliquer une commande.

### 4. Quelles garanties sont offertes ?
- Aucune intention ou commande audio ne bloque la boucle déterministe.
- Un échec d'allocation de voix audio par la plateforme (limite matérielle) ne remonte jamais comme information bloquante à une itération en cours (instanciation d'ADR-S01, point 6) ; le déterminisme de la simulation ne dépend jamais du succès ou de l'échec d'une lecture audio.
- Le Shell ne décide jamais qu'un son doit démarrer, continuer, ou s'arrêter — cette décision est toujours résolue en amont, dans le modèle architectural.

### 5. Qu'est-ce qui est explicitement exclu ?
- Toute décision de transition logique (démarrer/continuer/arrêter un son) prise par le Shell plutôt que reçue du modèle architectural (élimine le Modèle A tel que formulé — l'optimisation transparente d'application d'un état déjà déterminé reste, elle, autorisée) ;
- tout identifiant d'émetteur exposé sous forme de handle de plateforme au modèle architectural ;
- toute connaissance de la simulation par le Backend audio ;
- toute dépendance du déterminisme vis-à-vis du résultat d'une opération audio.

### 6. Quelles questions sont volontairement laissées à l'implémentation ?
API audio concrète (backend), spatialisation, mixage, nombre maximal de voix simultanées, format audio, mécanisme concret de représentation de la Ressource d'état sonore désiré.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Le Shell ne décide jamais une transition logique — il peut optimiser l'application d'un état déjà déterminé (élimine le Modèle A tel que formulé) | Invariant (instanciation d'ADR-S01, point 2) |
| Sons ponctuels = intentions pures, transitoires, sans suivi d'état | Invariant (instanciation d'ADR-004) |
| Sons continus = décision de transition prise dans le modèle architectural, mécanisme (producteur direct ou système de résolution dédié) laissé ouvert | Invariant — le mécanisme concret est implémentation |
| Identifiant d'émetteur, s'il existe, strictement logique — jamais un handle de plateforme | Invariant (instanciation d'ADR-S01, point 4) |
| Hybridation événement/état déterminée par la nature du phénomène sonore, jamais par une préférence d'API | Invariant |
| API audio, spatialisation, mixage, nombre de voix, format | Hors périmètre — implémentation |

## Bilan
La confrontation explicite des trois modèles a permis d'éliminer le Modèle A tel que formulé initialement — non par interdiction générale de tout calcul de différentiel, mais par l'invariant plus précis que seule une décision de transition logique, jamais une simple optimisation transparente d'application, est interdite au Shell. Le résultat final est un contrat unique couvrant deux catégories de phénomènes (instantanés et à cycle de vie), délibérément moins prescriptif qu'une première version l'aurait laissé penser : il exige que toute décision de transition soit localisée dans le modèle architectural, sans imposer la présence d'un système de résolution particulier comme mécanisme obligatoire. Le `AudioDiffResolutionSystem` reste une instanciation naturelle et recommandée, pas une clause du contrat. Aucun concept architectural nouveau n'a été nécessaire, conformément à la prédiction initiale ; la méthode de mise en concurrence a surtout permis d'éviter que le contrat ne fige un mécanisme d'implémentation particulier au rang d'invariant.
