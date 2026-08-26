# Plan 020 — Vérification transactionnelle avant envoi

## Résumé

Ajouter un hook `pre-push` Bash autonome et un banc d'intégration créant des
dépôts Git temporaires. Le hook observe le distant, vérifie la cohérence de
l'annonce, calcule l'union des commits réellement nouveaux et applique un
filtre générique de co-autorat. Toute indétermination ferme la transaction.

## Contexte technique

- Langage : Bash portable Linux/macOS.
- Dépendances : Git et outils POSIX déjà requis par le projet.
- Entrée : lignes transactionnelles fournies par `pre-push`.
- Sortie : silence en succès ; diagnostic actionnable sur stderr en refus.
- Activation : hors lot jusqu'au jury et au merge, puis gouvernée par SPEC-018.
- Base historique d'implémentation :
  `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`.
- Base de jury gelée pour reproduire les charges :
  `7024df31de5b23bfeca27eb5588a5465a872a8b8`.
- Base de livraison : tête de `origin/main` relue lors du rebase final tardif.

## Décisions de conception

### D-2001 — Observer le distant, ne pas deviner l'amont

Le hook interroge toutes les références du distant fourni par Git. Une branche
neuve n'a pas besoin d'amont local et les références de suivi potentiellement
périmées ne servent pas de vérité.

### D-2002 — Comparer deux ensembles d'atteignabilité

Les têtes locales non supprimées forment l'ensemble positif. Les commits
atteignables depuis les références distantes observées forment l'ensemble
négatif. Un seul calcul produit l'union positive moins l'union négative. Cette
définition conserve l'héritage déjà distant sans masquer un commit réellement
nouveau.

### D-2003 — Refuser les états non démontrables

Une interrogation distante en échec, une divergence entre annonce et
observation, un objet manquant ou un calcul Git impossible rendent la preuve
incomplète. Le hook refuse alors l'intégralité de l'envoi.

### D-2004 — Filtre générique et directement testable

Le filtre porte sur la structure `Co-authored-by:`, sans nom propre. Il est
défini dans le hook sous forme de fonction sourçable : le banc vérifie donc le
motif réel, sans recopier sa logique.

Le code retour du filtre est un contrat fermé à trois états : `0` signifie
« interdit trouvé », `1` signifie « message propre », toute autre valeur
signifie « inspection impossible » et refuse la transaction.

### D-2005 — Activation seulement après admission

Le lot produit un artefact versionné et testé. Il ne crée pas
`/home/moi/.git-hooks/pre-push`. Après merge, l'installateur gouverné par
SPEC-018 pourra projeter le fichier admis vers le répertoire actif.

### D-2006 — Indexer l'observation distante une seule fois

Une passe `awk` charge les R références observées puis compare les U mises à
jour. Une recherche linéaire dans les R références pour chaque mise à jour est
interdite : elle transformerait la borne annoncée en O(U×R).

## Algorithme

1. Lire et valider toutes les lignes de mise à jour.
2. Observer les références du distant avec leur identifiant exact.
3. Indexer une fois l'observation distante, puis vérifier en une passe que
   chaque ancien identifiant fourni par Git correspond ; détecter aussi une
   création devenue concurrente.
4. Vérifier localement tous les objets nécessaires et éplucher les références
   menant à un commit.
5. Calculer en une passe les commits des nouvelles têtes absents de toutes les
   références distantes observées.
6. Lire chaque message de commit introduit et refuser dès la première trace
   interdite, en affichant l'identifiant fautif.

Complexité : O(U + R + C), avec U mises à jour, R références distantes et C
commits introduits. La lecture des messages est O(C) ; aucun produit cartésien
entre références et commits n'est créé.

## Structure cible

```text
scripts/
├── git-pre-push-authorship.sh
└── test-git-pre-push-authorship.sh
specs/020-verification-pre-push/
├── checklists/requirements.md
├── implementation.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

## Stratégie de test

Le banc construit des dépôts locaux et distants temporaires sans écrire dans
l'historique du projet. Les commits hostiles sont créés par plomberie dans ces
fixtures uniquement. Les scénarios exécutent directement le hook avec le
contrat stdin officiel.

Les quinze témoins sont : branche neuve, force-push, multi-références, nom
inventé, erreur du filtre, héritage déjà distant et flux propre, les quatre
causes fail-closed séparées, indexation distante linéaire, échec de `mktemp`
avant dérivation de chemin, échec fermé de l'indexeur, arrêt du runner à la
première erreur de fixture et sensibilité à une mutation syntaxique du filtre.

## Constitution check

- Périmètre minimal : deux scripts de production/test et la documentation de
  session, sans nouvelle dépendance.
- Test-first : le banc est exécuté rouge sur un stub permissif avant le code.
- Isolation : branche dédiée dans le clone de travail isolé
  `/home/moi/revue/jc1/.worktrees/session-20-amend`, validée par le coordinateur.
- Anonymat : aucun nom d'outil ni trailer d'assistance n'est ajouté aux
  artefacts ou aux commits.
- Activation : interdite avant jury/merge.
- Complexité : calcul ensembliste linéaire, sans boucle références × commits.

## Risques et garde-fous

- Distant très riche dont certains objets ne sont pas locaux : refus explicite
  et demande d'actualisation, conformément au fail-closed.
- Course entre l'annonce de push et l'observation secondaire : divergence
  détectée et refusée.
- Contournement volontaire du hook local : hors portée de cette couche et à
  traiter par une future barrière de réception.
- Suppression prématurée du checkout source : SPEC-018 doit garantir que
  l'activation pointe vers un commit admis et durable.
