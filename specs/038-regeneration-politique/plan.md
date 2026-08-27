# Plan — Session 038, régénération explicite de la politique

## Contexte technique

- **Langage** : Rust 2024, sans nouvelle dépendance.
- **Paquets concernés** : `bridget-transport` puis ses dépendants
  `maicie` et `bridget-daemon` ; le scanner et le binaire vivent dans
  `bridget-daemon`.
- **Stockage** : JSON privé existant, verrou compagnon privé et remplacement
  atomique via les primitives de `bridget-transport::fsutil`.
- **Plateformes** : Linux et macOS, en réutilisant la mesure `PID + birth`
  existante.
- **Contraintes d'édition** : aucune modification de `cli.rs`, `daemon.rs` ou
  des scripts de déploiement.

## Vérification constitutionnelle

- La session est réservée sous le numéro 038 et part de la base gelée
  `b676afa86174df1def7a57caa73b969567364c42`.
- Le besoin est un incident opérationnel actif : la politique devient périmée
  à chaque redémarrage d'agent.
- La solution réutilise le format, le validateur de politique, la mesure de
  naissance de processus et l'écriture atomique existants.
- Aucun jeton, secret, droit, dépendance ou découverte réseau n'est ajouté.
- La complexité visée est O(n log n), avec des ensembles et cartes ordonnés.
- Une ADR consigne la dissymétrie conservation/remplacement et la complétude
  portée par `marker_source`.

## Architecture

### 1. Source de marqueurs attestée par l'hôte

Chaque principal peut porter une annotation optionnelle `marker_source`,
composée du nom d'hôte observé et du chemin canonique absolu du répertoire de
marqueurs. L'annotation reste facultative pour la garde existante afin de ne
pas casser la politique en service ; elle est obligatoire pour régénérer.

Le scanner ne reçoit pas un nom d'hôte déclaratif. Il mesure le nom de sa
machine, canonicalise le répertoire, refuse un répertoire lien ou illisible,
relit chaque marqueur et compare son `birth` au processus local. Son inventaire
ne peut donc pas être confondu par accident avec celui d'un autre hôte.

Pour un hôte distant, le même binaire est exécuté via SSH sur cet hôte. Seul le
JSON d'inventaire revient vers la machine qui détient la fixture de politique ;
les PID distants ne sont jamais interprétés localement.

### 2. Plan de régénération fermé

La fonction de planification charge la politique privée avec le même parseur
que la garde, puis vérifie :

1. chaque principal possède une source annotée ;
2. chaque source attendue possède exactement un inventaire complet et récent ;
3. chaque inventaire contient au moins un marqueur vivant ;
4. un principal approuvé n'est observé que sur sa source ;
5. aucun principal ne possède deux observations vivantes concurrentes ;
6. les grants existants d'un principal renouvelé ont une sémantique unique.

Un principal vivant remplace toutes ses anciennes instances par une seule
instance nouvelle, en conservant actions, expiration et révocation. Un
principal absent est conservé et signalé. Un principal inconnu est seulement
signalé. Aucune branche n'ajoute un droit.

### 3. Réécriture sérialisée et atomique

L'application ouvre un verrou compagnon `0600` avec `O_NOFOLLOW`, relit la
politique sous verrou, calcule la génération suivante sans saturation, puis
écrit par fichier temporaire synchronisé et renommage atomique. Le répertoire
est synchronisé avant succès.

Une erreur avant le renommage est un échec propre : l'original est encore au
chemin final. Une erreur après le renommage ne permet plus de promettre cet
état. L'outil relit et valide alors le fichier final, expose l'état observé et
rend une issue indéterminée distincte ; il ne simule aucun retour arrière.

Le mode par défaut calcule le rapport sans écrire. `--apply` est nécessaire.
Sans modification utile, le fichier et la génération restent inchangés.

### 4. Point d'entrée isolé

Un binaire dédié `bridget-greffe-policy-refresh` expose deux commandes :

- `scan --markers CHEMIN [--output FICHIER]` ;
- `refresh --policy CHEMIN --inventory FICHIER... [--apply]`.

Le chemin de politique est obligatoire. Aucun défaut ne pointe vers le secret
d'exploitation. L'installation du binaire est explicitement hors session 038 ;
la procédure de compilation et l'usage via SSH sont documentés.

## Oracles

1. Scanner réel : PID vivant + naissance exacte accepté ; PID recyclé ou
   marqueur périmé jamais projeté comme vivant.
2. Source vide, absente, incomplète, dupliquée ou trop ancienne : erreur et
   octets de politique inchangés.
3. Agent redémarré : ancienne instance absente, nouvelle unique, actions et
   grant inchangés, génération `N+1`, mode `0600`.
4. Agent momentanément arrêté : entrée conservée exactement et nommée dans
   `dead_principals`.
5. Principal non approuvé : rapporté mais jamais ajouté.
6. Remplacement atomique : l'observateur avant renommage voit encore la
   politique originale ; après succès il voit seulement la nouvelle. Une erreur
   injectée après renommage relit la politique planifiée mais rend une issue
   indéterminée, tandis qu'une erreur injectée avant conserve l'original.
7. Contrôle positif : le binaire régénère une fixture, puis la garde réelle
   accepte une mutation durable avec la nouvelle instance et refuse l'ancienne.
8. Mutant post-correctif : neutraliser l'incrément de génération fait mourir
   l'oracle de remplacement sur la valeur observée, pas dans la fixture.

## Gates

1. `cargo test --workspace --no-run` avant tout comptage.
2. Univers listé et comptes pour `bridget-transport`, puis
   `bridget-daemon` et les tests d'intégration du binaire.
3. Paquet modifié et dépendants : transport + maicie + daemon.
4. Suite workspace complète une fois avant livraison, dans un créneau daemon
   sérialisé si elle touche les familles de présence.
5. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
   avec différentiel parent/tête si une dette préexistante subsiste, puis
   `git diff --check`.
6. Revue hostile sécurité, minimalisme et responsabilité future.

## Limites

- La session ne rend pas `Register` authentique contre le processus pair.
- Elle ne révoque pas un principal absent et ne décide pas de nouvelles actions.
- Elle ne transporte pas elle-même un inventaire par SSH.
- Elle ne déploie ni n'installe le nouveau binaire.
- Elle ne lit ni ne modifie la politique réelle en service.
