# Livraison 170 — clôture des demandes distantes

Statut : Implemented. Pipeline : Livré avec conservations justifiées.
Huit tâches terminées le 16 octobre 2026.

## Résultat

Bridget 0.4.1 est installé et actif sur le Mac et Cartae. Une réponse distante acceptée clôt la demande corrélée sans fabriquer d’accusé de transport. La reprise traite aussi une tâche déjà nettoyée. Une vraie absence de réponse conserve son expiration.

Source des deux binaires et du tag : `57f2fff31b1c23d015a483a5a97754130b1e44bc`.
Ce commit est intégré et poussé sur main. Les documents finaux sont un commit descendant distinct ; ils ne changent pas ces binaires.

Release : https://github.com/guthubrx/bridget/releases/tag/v0.4.1 .
Tag annoté : `3cfff6be14496a0a8c97e92ce039c2c52b817869`.
Les deux archives et leurs fichiers SHA-256 ont été retéléchargés et vérifiés.

## Validation ciblée

- 46 tests `remote162_` réussis.
- 3 tests `remote170_` réussis.
- Après le dernier ajustement du trigger SQL sélectif et du journal d’erreur, seul le test SQL affecté a été rejoué : 1 réussite. Ce replay n’est pas un quatrième test distinct.
- Formatage ciblé et contrôle d’espacement réussis.
- Aucune suite complète relancée. Les limites globales antérieures restent celles de la version 0.4.0.

Le test SQL bloque seulement la transition vers `answered`. Pendant ce blocage, le résultat accepté reste durable et le collecteur ne produit aucun rappel ni timeout. Après retrait du blocage, le vrai entretien clôt une tâche déjà nettoyée.
Le reçu accepté doit correspondre à la mission et à la ronde courantes. Les identités du parent et de l’enfant sont vérifiées avant écriture. Les résultats vides restent refusés par le contrat existant, sans nouvelle capacité dans cette correction.

Les erreurs de commande et de compilation intermédiaires sont conservées dans les journaux privés. Le premier lancement R1 sans cache réemployé a été arrêté ; les commandes suivantes ont utilisé le cache existant et deux jobs. Aucun succès de filtre vide n’est compté.

Rapport des tests :
/home/moi/10.Scripts/64.bridget/specs/170-delais-reponses-distantes/validation/tests.md

Journaux privés :
/home/moi/.local/state/bridget-delivery170/r2/

## Paquets

Les constructions verrouillées et hors ligne utilisent le même export Git propre. Les manifestes des paquets source sont identiques :
`453b8e4bc6d7488e1d31c8dbc1eafe6200b2edfa8a8eeda0309c06dc46b1161e`.

- Binaire macOS signé : SHA-256 `aa7ebbde371bf0a92894c67dcf1d4fe9797ce3fe440ae4dbadbaaabb4d7e9664`.
- Binaire Linux : SHA-256 `845d7bad512ae4dc7e92eaf8ebdc33522bc2823f13c4371670f08290447880bd`.
- Archive macOS : SHA-256 `9cd2f7621260b24b4ec996b82739cbc998d6520f6a50f184beebaceaaded36cd`.
- Archive Linux : SHA-256 `c26fab1fb0cfceb2eaddc14adba17078c614382056594aa07e09da4a6251108b`.

Reçu macOS :
/Users/moi/.cache/bridget-release170/0.4.1-57f2fff31b1c-mac/build-receipt.json

Reçu Linux :
/home/moi/.local/state/bridget-delivery170/build/runs/0.4.1-57f2fff31b1c-linux/build-receipt.json

Publication et vérification :
/home/moi/.local/state/bridget-delivery170/release-0.4.1-57f2fff31b1c/publication170.json
/home/moi/.local/state/bridget-delivery170/release-0.4.1-57f2fff31b1c/verification170.json

## Activation et conservation de T3

Maître Bridget : PID 55610, né à 06:27:39 CEST.
Pont Bridget : PID 55688, né à 06:27:42 CEST.
Worker Cartae : PID 560830, naissance Linux 164002582, binaire 0.4.1 exact.

T3 conserve ses PID 4717 et 4792 et leurs naissances du 15 octobre à 22:09:38/40 CEST. Le tunnel conserve le PID 60206. Aucun signal ni relance T3 ou tunnel.
Les configurations des moteurs, plists et credentials sont conservées. Le catalogue reste à 27 choix : 17 locaux et 10 sur Cartae. Le contrôle d’arrêt des délégations reste inchangé. Les bases passent quick_check et foreign_key_check ; leurs schémas et versions restent identiques. Aucune restauration de base.

Une ancienne admission Revue était `chosen`, sans Task, depuis plus de vingt minutes. Son empreinte exacte a été vérifiée avant les arrêts et conservée dans la sauvegarde froide. Aucune migration n’est faite en170. Après démarrage, le mécanisme natif de reprise l’a refusée avec `permission_not_inherited`, sans création de Task. Aucune ligne n’a été supprimée ou corrigée manuellement.

Le premier contrôle de fichiers du worker a refusé la maintenance. Le worker 0.4.0 a été remis en service après vérification de ses fichiers inchangés. Une nouvelle observation, avec diagnostic conservé et délai de trois secondes après arrêt, a validé la même garde complète avant installation. Aucun avertissement de fichiers ouverts n’a été filtré.
Au démarrage final, le premier worker a rendu `worker_capability_unavailable` ; systemd l’a relancé une fois. Le worker suivant est stable sur le binaire 0.4.1 et a exécuté la recette. Le premier contrôle CLI était trop précoce ; sa lecture suivante a réussi sans nouvel arrêt.

Reçu d’activation :
/Users/moi/.cache/bridget-delivery170/activation/backup-57f2fff31b1c-r3/activation.json

Sauvegarde worker et contrôle stable :
/home/moi/.local/state/bridget-delivery170/activation/backup-57f2fff31b1c-r3/
/home/moi/.local/state/bridget-delivery170/activation/backup-57f2fff31b1c-r3/stabilized-worker-health170.json

## Recette réelle

Demande `recette170-reponse-distante-20261016-r1`, tâche `3f83d63d-44f8-4940-b188-0bd20cab41cc`, mission `95fe4edc-b8af-444f-a03e-f5005b62164d`.

Le résultat contient `RECETTE170_TIMEOUT_OK`, `cartae` et le commit source exact. La réponse finale suit une courte phrase de progression dans le résultat public ; le résultat public entier n’est donc pas présenté comme un marqueur seul.
La demande, créée à 1792124932, est `answered` à 1792124942, avant son échéance 1792124992. `delivery_acked=false` reste un fait de transport. `result_sent=true` et `cleanup_done=true`.
Le rejeu strict renvoie la même Task, le même résultat et le même lancement durable. Après l’échéance, `answered` est conservé et aucun événement de rappel ou timeout n’existe pour cette mission. Les anciennes fausses alertes déjà historisées ne sont pas réécrites.

Preuves :
/Users/moi/.cache/bridget-delivery170/recipe170-before-deadline.json
/Users/moi/.cache/bridget-delivery170/recipe170-after-deadline.json

## Nettoyage

Le cache de construction macOS170 a été retiré après contrôle complet des fichiers ouverts et verrou exclusif. 1123 fichiers, 904709935 octets logiques. Le gain physique observé au contrôle est 159252480 octets ; aucun gain physique égal à la taille logique n’est revendiqué. Le binaire installé, les paquets, les reçus, les sources exportées et les sauvegardes sont conservés.

Les worktrees et branches Cartae R1/R2 sont conservés :
/home/moi/10.Scripts/64.bridget/.worktrees/8a8f5934
/home/moi/10.Scripts/64.bridget/.worktrees/f0950059

R1 contient le premier correctif et ses validations non committés, remplacés par R2. R2 est propre et fusionné au contrôle, mais lsof signale `fuse.portal` et un inventaire potentiellement incomplet. Aucun retrait forcé. Les caches partagés, bases et preuves étrangères sont protégés.

Preuves de nettoyage :
/Users/moi/.cache/bridget-delivery170/cleanup/mac-build-cache170.json
/home/moi/.local/state/bridget-delivery170/cleanup/worktree-guards170.json

Le SHA documentaire final et l’alignement des deux machines sont enregistrés hors de ce document :
/Users/moi/.cache/bridget-delivery170/final170.json
