# ADR-002 — Query Model

## Statut
Adopté. Date : 31 juillet 2026.

## Contexte
ADR-001 a établi que chaque type de composant possède un stockage indépendant (Sparse Set), et posé l'invariant : *« le `World` expose des stockages ; les requêtes construisent des vues ; les systèmes consomment ces vues »*. Cette ADR définit ce qu'est une Query et fixe la frontière de responsabilité entre stockage, vue, et calcul — chaîne complète : `EntityId → World → Stockage → Query → System → Commandes / données transitoires`.

## Décision

### 1. Une Query est une vue légère, jamais un objet à état durable
Une Query est un adaptateur, pas un objet possédant une identité ou un état persistant. Elle est construite, consommée, puis disparaît dans le même passage d'itération. Aucune Query ne survit à la portée du système qui l'utilise, aucune Query n'est mise en cache ou instrumentée par une structure durable.

### 2. Une Query est spécialisée statiquement, jamais interprétée au runtime
La forme d'une requête (quels composants, en lecture ou en écriture, quels filtres) est déterminée avant l'exécution — par monomorphisation, génération de code, ou tout mécanisme de spécialisation statique équivalent. Aucun dispatch dynamique, aucun parsing, aucune résolution de requête « au vol ». Le terme *spécialisée statiquement* est délibérément plus large que *compile-time Rust* : il reste valable si une partie de la spécialisation est un jour déportée vers un mécanisme de génération de code, sans changer l'invariant.

### 3. Mode d'accès explicite par le type
> Une requête déclare explicitement, dans son type, le mode d'accès de chaque donnée consultée (lecture, écriture, ou tout autre mode exclusif futur).

Cet invariant rend l'intention d'accès inspectable — statiquement ou par un outil externe (Forge, linter d'architecture) — ce qui permet de vérifier que le principe « propriétaire unique en écriture au sein d'une phase » (`02-concepts.md`) n'est jamais violé. La syntaxe concrète (`&T`/`&mut T` natifs Rust, ou des marqueurs explicites `Read<T>`/`Write<T>`) est une décision d'implémentation, tant que l'invariant est satisfait.

### 4. Source d'itération, résolution des autres données par consultation
> Toute requête possède une source d'itération déterministe, puis résout les autres données à partir de cette source.

La source d'itération est ce qui pilote la boucle. Pour une requête portant uniquement sur des composants stockés en Sparse Set (cf. ADR-001), la source d'itération est un stockage — cas particulier déjà nommé *stockage pilote* dans ADR-001. Le concept est ici généralisé : demain, une source d'itération pourra être une structure spatiale (grille, quadtree, BVH) ou tout autre résultat déterministe d'un calcul antérieur (ex: une liste de visibilité), sans remettre en cause cet invariant. Le critère de choix de la source d'itération reste, dans tous les cas, une décision d'implémentation.

### 5. Les filtres sont une consultation d'index de présence, pas une couche spéciale
`With<T>` / `Without<T>` (ou équivalent) ne matérialisent aucune donnée et ne déclenchent aucune logique distincte de la résolution normale des autres composants non pilotes : un filtre conditionne l'itération, il ne produit rien.

### 6. L'entité est une donnée optionnelle du tuple de requête
`EntityId` peut apparaître dans les données consultées par une Query (`Query<(EntityId, &Position, &Velocity)>`) sans traitement spécial — cohérent avec le principe déjà acté qu'une entité est une donnée comme une autre, jamais un concept privilégié.

### 7. Une Query ne produit jamais de données
> Une Query ne transforme rien, ne filtre que pour conditionner l'itération, ne calcule rien, n'alloue rien, ne met rien en cache.

Toute transformation de donnée appartient exclusivement aux systèmes. La Query est strictement passive : elle expose une vue, jamais un résultat. C'est l'invariant qui empêche la dérive commune où une Query devient un mini-moteur d'exécution — dérive que toute l'architecture précédente (World ignorant, Sparse Set isolé, système porteur de la seule transformation) a été construite pour éviter.

## Invariant central

> Une Query est une vue déterministe et temporaire, spécialisée statiquement, construite à partir d'une ou plusieurs sources d'itération indépendantes — sans stockage interne durable, sans couplage avec le `World`, sans identité propre, distinguant explicitement dans son type les données lues des données écrites, et sans jamais produire ni transformer de donnée.

## Ce qui n'est pas tranché ici
La syntaxe concrète de déclaration d'une Query en Rust (générique sur tuple, trait dédié, macro...) est une décision d'implémentation, à documenter séparément lors de l'écriture du code.

## Classification (grille `00-principes.md`)

| Décision | Statut |
|---|---|
| Query = vue légère, jamais un objet à état durable | Invariant |
| Spécialisation statique, jamais de dispatch runtime | Invariant |
| Mode d'accès (lecture/écriture) explicite dans le type | Invariant — la syntaxe exacte est implémentation |
| Source d'itération déterministe + résolution par consultation | Invariant — le critère de choix de la source est implémentation |
| Filtres = consultation d'index de présence, sans logique spéciale | Invariant |
| `EntityId` optionnel dans le tuple, sans traitement spécial | Invariant |
| Une Query ne produit jamais de donnée | Invariant |
| Syntaxe Rust concrète de déclaration | Hors périmètre — implémentation |
