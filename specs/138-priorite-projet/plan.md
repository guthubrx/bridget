# Plan 138 — Priorité au projet dans les échanges

Date: 2026-10-06
Branche: session-138-priorite-projet
Statut: Implemented ; 20/20 tâches terminées, contrôles globaux PASS, Converge pass1 CONVERGED et audit A du diff validé. Cinq MED de maintenance non bloquants restent ouverts.

## Contexte technique

Rust 2024, serde, SQLite/rusqlite et SHA-256 existent déjà. Agent Loop est en
Python et possède ses tests unittest. Aucune nouvelle dépendance ni aucun service
n'est requis. Les essais utilisent un BRIDGET_HOME, une socket et un target Cargo
distincts. Le travail non fusionné des sessions 135 et 136 reste préservé.

Le registre ProjectReference/runtime a été retiré. Le refus de lancement est
visible dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs:12331.
La CLI produit encore project: None. Cette fonction ne réactive pas ce registre.

## Décision et architecture

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/138-priorite-projet/plan.md#extrait-1).

## Points d'extension et responsabilité

Les chemins sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet.
Chaque changement doit préserver les éditions simultanées de ses voisins.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/138-priorite-projet/plan.md#extrait-2).

Le script de skill est modifié dans le worktree dotfiles isolé, base 5fc64e37.
La copie Claude du même dépôt est vérifiée sans écraser ses différences propres.
La skill vivante reste inchangée. Aucune installation globale ou modification
des lanceurs de production n'entre dans cette session.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/138-priorite-projet/plan.md#extrait-3).

## Constitution Check

PASS de conception: session autorisée, worktree dédié, aucun fournisseur payant,
aucun restart production, aucun nouveau registre ou package. ADR 046 documente
le choix. Les accès membre, preuves auxiliaires, contrats fermés et reçus durables
restent les autorités existantes. Les standards de tests sont adaptés à Rust
et unittest ; les commandes frontend ne s'appliquent pas à ce périmètre.

Article XVIII: comparaison O(1), filtrage O(A), fil O(M) borné. Article XIX:
un champ de motif partagé et une variante d'annuaire ; une seule colonne de
warnings durable pour le contrôle existant. Aucun stockage de consentement de fil.
Article XX: une règle lisible et une garde commune réduisent les décisions
répétées. Le mainteneur peut lire le contrat, observer le verdict et exécuter
la recette sans contexte caché. Aucun moteur de politique général n'est ajouté.

Les ponts constitution et standards manquent dans le worktree. Ceux du dépôt
principal ont été lus. La constitution utilisateur et ses références ont été
appliquées. La synchronisation éventuelle relève du principal ; cet artefact
ne copie pas les fichiers générés.

## Séquence de vérification

Créer d'abord le Gherkin 138. Exercer les comportements avant modification.
Étendre le fait de connexion et le contrat, puis la garde directe, les fils,
les façades et enfin Agent Loop. Chaque tranche reçoit des tests ciblés.
La matrice couvre local/global/inconnu, identité forgée, worktree/symlink,
cross connu avec et sans motif, réponses corrélées, fil mixte silencieux,
ancien client, ancien reçu et opération acceptée en cours de remise.
Un fil A/B/U conserve la garde A/B malgré U inconnu. Une reprise de notification
après échec transporte le motif figé, pas le mandat courant modifié.

Les essais utilisent un vrai daemon temporaire et des transports de test locaux.
Les tests ne lancent aucun fournisseur. Réutiliser les assertions SQLite et
les échanges CLI/MCP de 102/136, plutôt que créer un second harnais.
Tests Agent Loop: recrutement local, busy, ROOT extérieur sans mandat, mandat
worker/coordinator/ROOT et plusieurs ticks sans nouvelle confirmation humaine.

Après les contrôles ciblés: cargo test --workspace, fmt --check, clippy avec
-D warnings, build release isolé et unittest de la skill. Puis lecture adverse,
Analyze, Converge et audit demandés par le pipeline principal. Les commandes
et résultats réels seront consignés ; ce plan ne déclare aucun test passé.

## Divergences volontaires et limites

ListAgents reste global pour préserver les diagnostics. Un nouvel annuaire scoped
répond à l'intention de communication sans casser cette observation existante.
Les inconnus ne sont pas proposés automatiquement, mais les anciens envois
restent possibles avec avertissement. Cette compatibilité limite la portée
de la règle ; elle ne constitue pas un contrôle d'accès de sécurité.

L'avertissement est établi avant l'effet et retourné au caller. Il n'impose pas
un dialogue humain en deux temps. La comparaison est un fait de communication,
pas une permission sur les fichiers ni une preuve de confidentialité fournisseur.

Livraison de session: code et preuves isolés, sans commit, installation globale,
fusion, API fournisseur ou redémarrage production.
