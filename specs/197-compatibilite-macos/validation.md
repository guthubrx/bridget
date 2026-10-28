# Qualification197 — Compatibilité et préparation d’activation

## Correction réelle
La compilation macOS de la source196 échouait dans remote_worker_read.rs avec quatre erreurs E0308/E0277. mode est un u32 ; libc::S_IFMT et S_IFREG sont u16 sur macOS, u32 sur Linux. La conversion explicite en u32 conserve exactement les constantes et le refus des fichiers non réguliers. Aucun droit, seuil ou effet supplémentaire. Production : une expression modifiée.

## Contrôles exécutés
- Linux Cartae :10tests188 PASS, y compris lecture verrouillée, type de fichier, stabilité et absence d’inférence Terminal.
- macOS ARM64 : build release PASS après correction ;7tests196 de relecture PASS ; signature locale codesign vérifiée.
- Linux glibc2.39 Cartae/KS : build release PASS.
- Linux glibc2.36 SYS : build release PASS, construit avec Rust1.92 sur le grand volume /mnt/storage, sans remplir /.
- Qualification maintenance :13tests de l’updater et1test existant de reconstruction distante PASS. Contre-épreuve197 supplémentaire : annulation distante expirée, flotte désirée absente, état persistant inchangé et aucun fournisseur local préparé, PASS. La première épreuve comparait un Task avant normalisation au lieu de la relecture persistée ; elle a échoué et la fixture a été corrigée pour comparer deux lectures du même magasin. Aucun code métier de redémarrage modifié.
- Revue indépendante demandée par Bridget : refus delegations_halted, clé bridget-revue197-constantes-macos-20261028-01, not_recorded/task_id=null. Aucun modèle lancé, aucun autre moteur employé. Aucun avis indépendant197 favorable inventé. Auto-revue : conversion de constantes seulement, aucune branche ou dépendance ajoutée ; test de préservation séparé du code de production.

## Candidats non actifs
Source fonctionnelle04214db5f89e4159a25a4061bb8b7dfb5969ce52 ; build197-platform-04214db5f89e, version affichée0.6.3. Les commits ultérieurs de preuve ne reconstruisent pas ces octets.
- Mac : /Users/moi/.local/lib/bridget-communication/197-platform-04214db5f89e/bin/bridget ; SHA2562e83f15013573330f05132ff1f9afc07b4f0b6f5306c58a9cfd96192aa42da9d.
- Cartae/KS : /home/moi/.local/lib/bridget-communication/197-platform-04214db5f89e/bin/bridget ; SHA256fb8d378d09076f265be1c737206fd51fd7ff7aae2410ff711f3f4ba948f19ffe.
- SYS : même chemin, build Debian glibc2.36 distinct ; SHA256e5a2593c5cf87b09843a8bee69438ebe3ff0d1d27cd7c198006f9c245a09f57d.
Copies/empreintes/version vérifiées. Commande installed exécutée après copie sur les trois hôtes compilateurs. Aucun lien actif remplacé, service redémarré ou travail métier rejoué.

## Fenêtre de maintenance
Gel posé par l’humain à06:55UTC, génération7, commande b614af6a-3e89-450e-9529-4ff448e98dbf. Les gardes SYS6eea et Cartae b871 sont closes : cleanup_done=true, guard_released=true, cancel_stopped lié au bon enfant. Tous les reçus de commande des trois workers sont terminal/refused. Les cgroups de core, mobile, updater et des trois workers contiennent seulement leur PID principal ; SendSIGKILL=no partout. Aucun fournisseur local sous ces unités observé. Aucune nouvelle délégation admise. Deux anciennes questions restent pending dans Bridget, alors que les deux listes de questions T3 correspondantes sont vides ; elles restent conservées et aucune réponse ou clôture n’est fabriquée.

Le dernier Prepare natif2e25a19d-70c3-44dd-8bdb-ddc88f346206 refuse encore update_missions_active_or_unconfirmed. L’ancienne tâche e5959986-e426-422d-956f-e035dbcf89be reste cancelling avec native_cancel_unverifiable, sans preuve worker ; deux admissions anciennes restent chosen/decision_started, sans tâche. Aucun de ces faits n’a été effacé ni transformé en Terminal. Recherche bornée complète3414fichiers sans erreur dans les stockages d’origine connus : seul le reçu de préparation instance vide/génération0 subsiste ; il ne prouve pas l’arrêt.

Sauvegarde SQLite cohérente et quick_check OK, flotte, configuration et unités conservées sous /home/moi/.local/share/bridget-maintenance/20261028-activation196. Il s’agit d’un état avant arrêt ; une copie finale après arrêt serait nécessaire pour une restauration. Aucun rollback de données automatique prévu. Le retour de binaire doit conserver les écritures reçues, sans restauration silencieuse de cette sauvegarde.

L’activation n’est pas réalisée. Une intervention exceptionnelle hors Prepare exige une décision humaine distincte après qualification de la procédure, conservation de la tâche inconnue et des admissions, exclusion de l’updater et arrêt propre sans escalade. Ce gel ne vaut pas cette décision. La version publique publiée reste0.6.3 ancienne ; les candidats ne sont pas présentés comme une nouvelle release officielle.

## Signalements de remise
Le pont est actif. Les messages ordinaires de pairs sans droit de remise pendant un tour attendent sa fin ; le refus not_parent est maintenu. Politique restait dans son tour204. Cela ne prouve ni panne générale ni perte de message ; pas de changement de droits ou injection par autre canal. Nouveau refus binding_transport_unavailable remonté par trajets bus : aucun redémarrage effectué, maître183/PID351206 confirmé ; clé revue-niveaux-pietons24 sans reçu ni tâche à la lecture ciblée. La cause du délai reste inconnue et n’est pas attribuée au gel. Réattestation de notre seul fil réussie, pas de preuve transposée à un autre parent.
