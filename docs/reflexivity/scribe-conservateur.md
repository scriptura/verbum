# Le rôle du scribe conservateur

## Objet

Ce document ne décrit ni un principe architectural, ni une règle de conception. Il décrit une discipline éditoriale employée pendant la rédaction du corpus architectural de **_Verbum_**. Cette discipline peut être résumée en une idée simple :

> Une fois un invariant adopté, le travail ne consiste plus à l'améliorer, mais à vérifier qu'il résiste.

## Une différence de posture

Les assistants utilisés pendant la construction du corpus étaient capables de proposer des architectures alternatives. Pourtant, à partir d'un certain stade, cette capacité est volontairement passée au second plan. Le rôle recherché n'était plus celui d'un concepteur. Il devenait celui d'un **scribe conservateur**.

Autrement dit :

* conserver le langage déjà construit ;
* préserver les décisions déjà adoptées ;
* n'accepter une modification qu'en présence d'une justification explicite.

L'objectif n'était plus de produire des idées. L'objectif était d'empêcher que de bonnes idées détruisent une cohérence plus importante.

## Le principe de présomption de stabilité

Toute décision adoptée bénéficie d'une présomption de stabilité. La question n'est donc plus :

> « Peut-on écrire mieux ? »

mais :

> « Existe-t-il une raison objective de modifier ce qui existe déjà ? »

Une réécriture stylistique n'est jamais une raison suffisante. Une préférence personnelle ne l'est pas davantage.

## Les seules raisons légitimes de modifier le corpus

Pendant cette phase, une modification n'est acceptée que si elle répond à au moins une des situations suivantes :

* une contradiction interne est démontrée ;
* une ambiguïté de vocabulaire est identifiée ;
* deux concepts jusque-là confondus doivent être distingués ;
* un nouveau domaine révèle réellement une primitive impossible à exprimer avec le langage existant.

En dehors de ces cas, la stabilité est préférée à la nouveauté.

## Réfutation avant extension

Cette discipline rejoint directement la méthode générale du corpus. Avant d'introduire un nouveau concept, toute tentative sérieuse doit être faite pour montrer que les concepts existants suffisent déjà.

Autrement dit :

1. essayer de réduire ;
2. essayer de reformuler ;
3. essayer d'instancier ;
4. seulement ensuite accepter une extension si aucune réduction ne résiste.

Cette règle s'applique autant aux idées qu'aux textes.

## Le rôle des audits

Les audits successifs n'avaient pas pour objectif de réécrire le corpus. Ils cherchaient à le mettre en défaut. Les questions posées étaient systématiquement de la forme :

* existe-t-il une contradiction ?
* existe-t-il une ambiguïté ?
* deux documents disent-ils réellement la même chose ?
* un nouveau domaine oblige-t-il à introduire une primitive ?

Cette différence est importante. Un audit n'est pas une recherche d'amélioration. C'est une tentative de réfutation.

## Une observation

Après plusieurs cycles complets d'audits réalisés par plusieurs modèles indépendants, une seule contradiction véritable a été identifiée. Elle concernait une incohérence interne à un ADR déjà existant. Aucun audit n'a conduit à introduire une nouvelle primitive architecturale. Les modifications retenues ont presque toujours consisté à :

* préciser un invariant ;
* retirer une ambiguïté ;
* distinguer deux notions déjà présentes ;
* reclasser une décision comme instanciation plutôt que comme concept nouveau.

Autrement dit, le corpus s'est davantage **compressé** qu'il ne s'est enrichi.

## Conséquence

Cette discipline produit une propriété intéressante. Le corpus cesse progressivement d'être une succession de documents. Il devient un langage. Chaque terme possède un sens unique. Chaque nouveau texte cherche d'abord à préserver ce langage avant de l'étendre. Les révisions deviennent alors un travail de conservation plutôt que de réécriture.

## Conclusion

Le rôle du scribe conservateur n'est pas de défendre le passé. Il est de protéger la cohérence. Lorsqu'un changement est nécessaire, il est intégré sans hésitation, lorsqu'il ne fait qu'offrir une formulation différente, il est volontairement écarté. Ainsi, le corpus évolue moins par accumulation que par clarification. Cette discipline ne garantit pas que l'architecture soit correcte, elle garantit en revanche que toute évolution du langage architectural est explicite, justifiée et proportionnée.

---

_le 4 août 2026_
