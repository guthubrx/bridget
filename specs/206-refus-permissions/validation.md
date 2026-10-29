# Validation206

## Faits de l’incident

Tâche eba4db89-6703-4dd2-b79d-8e30d76bbc39, clé opus2d-sol-compact-20261029-01. Lecture native de la tâche et du reçu worker en lecture seule : failed, erreur remote_start_failed:permission_changed, mission_issued_at=null, cleanup_done=true, guard_released=false. Worktree propre conservé à sa base2aa616d41573dc2b9a8312789c1134e172552818, aucun commit. Reçu de commande génération1347 terminal sans permis Release. Le refus est antérieur à la remise métier ; le bootstrap a été arrêté.

Le journal natif du parent montre le premier appel auto_delegate interrompu à04:29:30UTC. Un appel ultérieur a relu la même tâche à04:30:19 ; aucune seconde tâche créée. Le run488 est resté actif, run489 a été annulé. Ces faits ne démontrent pas le champ de permissions divergent. Le205 ne conservait ni l’étape précise ni les catégories : la cause détaillée historique est inconnue, sans preuve de panne de compte, de serveur ou de révocation.

## Correction

Les six comparaisons existantes restent strictes. Le diagnostic ajoute une étape et uniquement des noms de catégories fermés ; aucune ancienne/nouvelle valeur, credential ou texte fournisseur. Le refus technique public reste permission_changed. La chaîne interne bornée est conservée dans le champ d’erreur existant de Task ; aucun schéma de stockage ni protocole worker neuf. Les anciens refus et suffixes invalides rendent unobserved avec changes vide.

Le guidage distingue nettoyage/rapport en cours et fin confirmée. Une tâche failed ne reprend pas par Continue ou rejeu. Clôture propriétaire et attestation fraîche restent nécessaires avant une nouvelle demande autorisée. Aucun changement de droits ou remplacement silencieux.

## Contrôles et livraison

Tests ciblés et revue en cours. Aucun chargement206 encore effectué. L’envoi utile à monuments mcp-50239-6ae2cf59-6 est accepté in_flight, lecture non déduite. Il ne demande ni gel global ni attente des quatre chantiers indépendants.

## Revue initiale

Revue b2e1fe8a : égalités strictes, confidentialité et code public conformes. P2 de guidage : workspace_report_ready n’atteste pas un rapport complet. Complément : le conseil exige explicitement inspection en cas de report_unavailable et vérification du dossier avant nouvelle admission ; aucune preuve de rapport inventée. Test ajouté avec rapport terminé mais mesures absentes. Trois tests206 et33tests de reprise198/201/202 initialement PASS ; nouvelle vérification du complément en cours. Le premier filtre de régression native_start_recovery ne sélectionnait aucun test ; filtre corrigé start_recovery::,33 réellement exécutés.

## Contrôles du complément

Trois tests206 PASS après le complément, dont rapport terminé/incomplet et conseil d’inspection. Les33tests198/201/202 réellement sélectionnés PASS. Guides32pages/522liens/13paires PASS et git diff--check propre. Première construction84988ab7 non activée ; une seconde construction est lancée depuis780ea521 avec le complément de guidage. Contre-revue296670ce en cours.

## Contre-revue finale

Contre-revue296670ce favorable sur780ea521 contre85852ca3, P2 levé. Résultat complet lu, ACK et clôture native ; revueinitialeb2e1fe8a également lue/ACK/close. Les deux rapports Git sont propres,0commit ; conservés jusqu’à vérification des usages. Aucun avis runtime déduit de ces revues.

## Activation effective

Source fonctionnelle780ea521520a1b60633a604b83d57e2fda88a9f1, sources intégrées sur main ea109932 et poussées avant le chargement. CI hygiène40820700340 success. Binaire final206-permissions-780ea521 construit enrelease, copié hors target et octets vérifiés ; installedPASS. SHA2567ccae3b2077b1744db4f66907f86748da323b74732dc521429af5e481de384f0.

Le29octobre2026à04:59:32UTC, maîtreCartaePID903680 et mobilePID903820 chargent /home/moi/.local/lib/bridget-communication/206-permissions-780ea521/bin/bridget. UpdaterPID903831 actif. Arrêt propre, SendSIGKILL=no, mise à jour concurrente exclue par le verrou natif ; sauvegardes SQLite quick_checkOK et flotte/config avant/après arrêt : /home/moi/.local/share/bridget-maintenance/206-production-20261029T045928Z. Coupure2,79s. Retour205 conservé. WorkerCartae201PID3750537 et wrapper historique inchangés (PID/naissance/exécutable comparés), aucun fournisseur physique Cartae actif observé au passage. Workers SYS/KS204 et Mac183 non redémarrés : cette correction agit uniquement dans le maître pour toutes les destinations, aucune opération ou API worker nouvelle. Ne pas présenter ces composants comme des binaires206.

La première lecture MCP pendant la réinscription applicative a rendu primary_absent malgré l’attestation réussie ; aucune identité fabriquée ni mission recréée. Reprise vérifiée ensuite par le catalogue et l’annuaire natifs : delegations_halted=false,16fournisseurs, présencebridge1016 busy. La coupure du maître et le délai de retour du pont restent distincts.

## Notifications et issue de la tâche

Notifications de bascule : mcp-50239-6ae2d29f-7(monuments),6ae2d2a4-8(Politique),6ae2d2a6-9(trajets). Bilan livré à monuments mcp-50239-6ae2d30a-a/deliveryf81c8ef0-7d29-4261-a4c3-01fb5dff8e73 ; disponibilité après chargement signalée aux deux autres sous mcp-50239-6ae2d349-b et6ae2d34c-c. Messages acceptésin_flight, lecture non déduite.

Après sa clôture propriétaire, eba4db89 est observée cancelled/cleanup_done=true, mission_issued_at=null, garde non déclarée libérée. Aucun Start/rejeu/Cancel de cette tâche par le mainteneur. Le205 n’avait pas enregistré le champ divergent : cette cause historique n’est pas déclarée résolue ni attribuée à l’interruption. Les nouvelles demandes distinctes autorisées n’ont pas de gel de maintenance à attendre. Le nouveau diagnostic et les conseils sont actifs ; aucune réussite future universelle n’est promise.

## Nettoyage

Caches Cargo entretenus avant/après etinstalled,4,23Gio sous budget10Gio ; aucun ancien candidat206 installé, la première construction84988ab7 a été remplacée dans target. Actif206 et retour205 conservés ; anciens201/204/183 encore sous usage conservés. Les deux revues b2e1fe8a/296670ce sont closes/cleanup_done, worktrees propres y compris ignorés,0commit ; observation SYS des fichiers ouverts indisponible (process_access_unavailable), donc dossiers et branches conservés. Aucun retrait forcé. Correction purement technique : tests usuels, aucun diaporama d’interface requis. Aucun service ou calcul métier Opus2D/NE02 touché.
