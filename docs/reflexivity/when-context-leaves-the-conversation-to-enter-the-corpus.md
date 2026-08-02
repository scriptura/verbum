# Note — Quand le contexte quitte la conversation pour entrer dans le corpus

L'un des enseignements les plus intéressants de la construction de Verbum n'est pas une décision architecturale particulière, mais une observation sur le processus lui-même.

Au cours de ce travail, la rédaction du corpus s'est poursuivie malgré une circonstance inhabituelle : les échanges avec GPT se sont répartis sur deux sessions distinctes, ouvertes pour des raisons purement pratiques, sans continuité conversationnelle complète.

À première vue, cette rupture aurait dû fragiliser la cohérence du raisonnement. Elle ne l'a pourtant pratiquement pas fait.

La raison apparaît rétrospectivement simple : à mesure que le corpus grandissait, la connaissance cessait progressivement d'être portée par la conversation pour être transférée dans les documents eux-mêmes.

Au début d'un projet, une grande partie de la compréhension est implicite. Les hypothèses vivent encore dans les échanges, les intuitions n'ont pas reçu de vocabulaire stable, et une interruption de la discussion entraîne facilement une perte de contexte.

À l'inverse, lorsqu'un corpus atteint une certaine maturité, les découvertes importantes sont systématiquement réinjectées dans la documentation :

* les intuitions deviennent des invariants ;
* les termes reçoivent une définition unique ;
* les responsabilités sont explicitement localisées ;
* les frontières sont nommées ;
* les décisions sont justifiées plutôt que simplement retenues.

Autrement dit, le contexte quitte progressivement la mémoire des interlocuteurs pour devenir une propriété du corpus.

La conséquence est importante : la cohérence du projet dépend de moins en moins de ceux qui l'ont élaboré, et de plus en plus de la qualité des documents eux-mêmes.

Cette observation explique probablement pourquoi la coexistence de plusieurs modèles de langage (Claude, Gemini, GPT), ainsi que l'utilisation de plusieurs sessions indépendantes, n'ont pas conduit à une fragmentation du résultat. Les modèles ne partageaient pas une mémoire commune ; ils partageaient un référentiel documentaire devenu suffisamment précis pour servir lui-même de source d'autorité.

Cette évolution rappelle un phénomène déjà rencontré lors de la conception du calendrier liturgique sous Rust. Les principes qui y avaient émergé — Forge AOT, représentation binaire orientée données, runtime volontairement minimal, accès déterministes en O(1) — avaient déjà conduit à transférer progressivement les décisions depuis le code vers une spécification explicite.

Verbum généralise cette démarche. Ce qui était auparavant une méthode de production logicielle devient une méthode de production architecturale.

Une autre évolution apparaît au fil du corpus.

Au début, la question dominante était :

> « Comment concevoir ce domaine ? »

Peu à peu, cette question a été remplacée par une autre :

> « Les invariants existants suffisent-ils déjà à exprimer ce domaine ? »

Ce déplacement est probablement le changement méthodologique le plus profond de toute la démarche.

À partir d'ADR-A01 (Assets), les nouvelles ADR n'ont plus été abordées comme des exercices de conception, mais comme des tentatives de réfutation du langage architectural existant. Chaque domaine réputé difficile (Sauvegarde, Réseau, Audio, Scripts) était traité comme un test : était-il capable d'obliger le corpus à introduire une nouvelle primitive ?

Dans la plupart des cas, la réponse fut négative.

Le travail ne consistait donc plus à inventer des concepts, mais à démontrer qu'ils n'étaient pas nécessaires.

Cette inversion est révélatrice d'un corpus qui commence à atteindre une forme de fermeture conceptuelle.

Enfin, cette expérience suggère un critère pratique de maturité documentaire.

Un corpus devient réellement autonome lorsque sa cohérence ne dépend plus de la mémoire de ses auteurs. À partir de ce moment, une nouvelle session d'IA, un nouveau collaborateur, voire une implémentation dans un autre langage devraient pouvoir retrouver les mêmes conclusions en s'appuyant uniquement sur les documents.

Ce n'est plus la conversation qui garantit la continuité.

C'est le corpus lui-même.

---

_2 août 2026_