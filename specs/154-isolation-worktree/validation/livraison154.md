# Livraison 154 — 12 octobre 2026

Statut : Livré et activé. 28/28 tâches, CONVERGED passage 2, 29 FR/11 SC/5 US.
Contre-revue Claude Sonnet 5.5, ronde 2 : APPROVE.

## Code et activation

Source intégrée et poussée vers github/main :
`806d0984fe45eea9fcc885770285bc00a85d005e`.
Intégration ff-only, 41 fichiers vérifiés. Build 806d0984fe45 : 58,36 s, RC0.
Binaire signé ad hoc, vérification stricte :
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget
SHA256 : `39834b7d36bd184842ea8cf99994a08e92af583178a97b0dba537643e52ffaa8`.
Le fichier installé a été rehashé et correspond au reçu d’activation.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/154-isolation-worktree/validation/livraison154.md#extrait-1).

## Vérification et limites

Audit final A 99,9 : 0 critique, 0 élevé, 0 moyen, 1 faible. Couverture 84/122
segments du diff, 68,85 %. Validateur RC0, 0 erreur, 0 warning. Le point faible de
performance reste accepté ; la couverture n’est pas une inspection intégrale.

Une seule suite workspace complète : 2194 PASS/9 FAIL/72 ignorés, 499,95 s, RC101,
AVANT les derniers correctifs. Huit échecs sont les références historiques 152.
Le neuvième était une fixture de date locale face au journal UTC. Son correctif
UTC est prouvé par 1 PASS ciblé. Autres passages ciblés : 5 PASS pour le CAS, 3 PASS pour le contrat
null et 2 PASS pour les deux constats retenus de revue. Ces passages éprouvent
des états successifs distincts. Aucun second passage workspace ni GREEN global.
Les 194 cas T001 sont retrouvés sans régression des 193 PASS ; 48 cas154 PASS dans la
suite antérieure. L’activation ne vaut pas nouvelle recette avec un vrai modèle.

## Nettoyage et conservation

Branche session-154-isolation-worktree et worktree 154 retirés à 01:26 CEST.
Aucun reste de worktree créé par les tests. Les chemins de worktree cités dans
les anciennes preuves sont désormais historiques.

Nettoyage du build : 0,29 Gio logiques d’intermédiaires retirés. Cache debug partagé
≈18,30 Gio conservé pour 155–160 ; budget 20 Gio dans l’environnement des prochaines
commandes. Le seuil de nettoyage de ce build était dépassé ; les caches protégés
ont été conservés. Aucun réglage global n’a été changé.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/154-isolation-worktree/validation/livraison154.md#extrait-2).

## Preuves

- Git : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/154-isolation-worktree/validation/git154.json
- Build : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/154-isolation-worktree/validation/build154.json
- Installation : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/154-isolation-worktree/validation/install154.json
- Activation : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/154-isolation-worktree/validation/activation154.json
- Nettoyage : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/154-isolation-worktree/validation/cleanup154.json
- Fixtures privées : /Users/moi/.cache/bridget-delivery154.l50umk90/cleanup154-fixtures.json
- Tests : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/154-isolation-worktree/validation/tests-154.md
- Convergence : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/154-isolation-worktree/validation/converge154-pass2.json
- Audit : /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-10-12/session-2026-10-12-spec-154-01
- Archive : /Users/moi/.cache/bridget-delivery154.l50umk90

Le commit de code installé est distinct des métadonnées documentaires ajoutées
après livraison. Aucun SHA de commit futur n’est annoncé.
