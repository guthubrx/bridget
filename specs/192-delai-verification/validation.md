# Qualification192 — Délai de preuve distante

## Incident et faits
Demande Politique opus166-raccord-optimisations-ksstore-20261027-01, signalement opus166-ksstore-raccord-resource-limit-20261027-01 : refus candidate/resource_limit, not_recorded, sans tâche. Source worker KS chargée b6c045cd1787f465e9727a6a1b7f7bd6d42820e0, PID2399518. Ce motif a un seul producteur dans cette source : try_send Full vers le canal technique borné à8. Il ne mesure ni RAM/CPU ni quota fournisseur. Aucune nouvelle sonde ni tentative métier faite pour le diagnostic.
Le worker est vivant. Une lecture ciblée des enfants /proc a observé des Git status sur /home/moi/.local/share/bridget-projects/opus2d/repository/.worktrees/bef1a3bd. Le journal par PID, et non le filtre unité qui était vide, contient des sondes terminées entre0,16s et9,72s ; la fenêtre du refus ne conserve pas de réponse corrélée. L’occupation actuelle observée et ces durées ne reconstituent pas la composition historique de la file.
Le tick de supervision du worker ne lance pas Git : le rapport et sa seconde validation viennent des Reconcile explicites. Le maître peut cesser d’attendre après5s, retirer son attente et redemander une preuve ; le worker peut encore traiter la précédente. Chaque lecture Git admet10s, plusieurs lectures peuvent s’enchaîner. Ce décalage est confirmé par la source et la contre-épreuve, sans prétendre être l’unique cause du refus historique.

## Changement et limites
Seul le Reconcile de remote_dispatch_proof passe de5à30s, constante nommée. ParentProof, Release, permis, TTL et réservations restent inchangés. Créneau asynchrone et rotation existants conservés. La réponse continue de passer par le contrôle de connexion/époque/worker/commande et l’empreinte des droits ; aucune preuve ancienne mise en cache pour accepter une remise.
30s reste une borne, pas une garantie de succès pour toute durée ni un traitement de toute saturation. Les demandes métier, leurs clés et leurs contraintes restent conservées ; aucun rejeu, remplacement, changement de capacité, annulation de bef ou redémarrage.

## Contrôles
Contre-épreuve sur le même scénario de réponse valide à6s, seul délai remis temporairement à5s dans le worktree isolé : échec attendu101, remote_permissions:WorkerUnavailable. Restauration30s dans finally puis réussite des tests. Logs : /home/moi/.cache/bridget192/baseline-five-seconds.log et spec192-final.log. Aucun effet sur le maître actif.
15contrôles ciblés PASS :2spec192,1rotation/asynchronisme174 (vrai délai borné30s),1scénario172 à9états,8diagnostics190,3confirmation184. Le scénario lent traverse le vrai handler de réponse worker, une seule requête, tâche/slots/mission inchangés ; une empreinte différente reste refusée. Guides32pages/522liens/13paires et git diff --check PASS. Aucune FULL. Revue indépendante à effectuer.

## Runtime et conservation
L’urgence de rangement Cartae/SYS a été retirée par le propriétaire après son relevé18Gio/18Gio ; aucun nettoyage d’urgence de caches déclenché. Les nettoyages avant/après Cargo restent ceux du script obligatoire, cache partagé7,12Gio. Les anciennes revues190/191 sont déjà traitées et ne sont pas recréées.
Le maître actif reste183 ; ce changement ne sera pas annoncé chargé tant que la maintenance native ne l’admet pas. Les travaux métier et les reçus restent conservés. Pas de nouveau relevé périodique du même refus de maintenance.
