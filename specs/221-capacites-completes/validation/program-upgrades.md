# Correction 221 — mises à jour des programmes

État : correction intégrée, poussée et activée sur les sept services Bridget.
Base e88d1db46242de93142708fa3396293141ee8e92, aucune modification de profil
privé dans ce lot. La qualification SYS0.166 précédente reste acquise.

## Cause et changement

L’observation historique épinglait le SHA du programme. Remplacer ce programme
rendait le profil indisponible malgré un protocole utilisable. La correction
conserve l’observation historique et ajoute une qualification privée automatique
de l’image nouvelle, dans le collecteur et le créneau existants. Préflight ne
lance aucun processus et n’écrit rien. La preuve contient le SHA de l’image et
l’identité automatique des sources du vérificateur. Les droits et les tâches
admises ne sont jamais réécrits.

## Vérifications utiles

- 13 tests filtrés spec221_program_compat PASS en 6,37 s sur Linux : remplacement
  d’image sans mutation du profil, preuve privée, vocabulaire inconnu obligatoire
  et facultatif, parent réellement illisible, cache atomique, révision du
  vérificateur, intersection des opérations, cause locale au thread, guidage,
  downgrade vers maître ancien et arrêt réellement retenu sans SIGKILL.
- Recette du programme réel Codex0.163 PASS en 8,58 s : sept méthodes témoins
  acceptées par les schémas générés, initialize et account/read, sans compte ni
  modèle. Réutilisée après les deux corrections de classement, qui ne changent
  ni le CLI ni le protocole de la recette.
- 7 native157_preflight PASS : contrôles historiques sans spawn conservés.
- 38 motifs distincts dans 128 raisons : sortie mesurée32157octets, cadre existant
  conservé. Contrôle acquis avant les derniers changements de classement.
Total : 22 contrôles distincts acquis ; aucune campagne métier supplémentaire.

Commandes via scripts/build.py cargo test -p bridget-daemon --lib :
spec221_program_compat, native157_preflight_,
spec221_probe_refusal_all_distinct_codes_fit_existing_frame.
La recette ignorée reçoit BRIDGET_REAL_CODEX_PROGRAM explicitement ; aucun
programme n’est découvert par lancement de modèle.

## Revues et limites

Conception97fe1835, implémentation36a5a0a2 et contre-revuec7a75d01 lues, ACK et
clôture une fois. La R1 exigeait un état non vérifié pour schéma inconnu, une
preuve liée au vérificateur, des temporaires nettoyés et une révision par image :
traités et testés. La R2 a trouvé l’erreur facultative ignorée et EACCES sur la
racine classé pending : corrections ciblées, test réel de permission PASS.
La remarque sur le minimum5 s est réfutée par MIN_REFRESH et son usage dans
collect ; aucune nouvelle promesse ajoutée. Delta final auto-audité, pas de
quatrième revue générale.

Les erreurs avant RELEASE ferment le canal de bootstrap ; wait_ready et
persist_marker attendent l’enfant abandonné. Après RELEASE, l’arrêt incertain
garde Child et marqueur et bloque toute seconde sonde.
Pas de qualification de tous les fournisseurs : automatisation Codex seulement.
Pas de garantie sémantique complète ni d’accès au modèle. Une poignée de main
en échec conserve l’absence de preuve et le contrôle sera repris à la cadence
d’échec. Le test de négociation ancien pair ne vaut pas recette bout en bout
de deux versions actives. Pas de modification visible d’interface graphique :
preuve technique conservée ici, sans diaporama artificiel.

## Cohérence et minimalisme

T183–T186 rattachées au contrat program-upgrades, au plan du lot courant et aux
invariants d’admission existants. Aucun nouveau magasin métier, aucun secret lu
par le contrôle de schéma, aucune nouvelle dépendance résolue ; sha2 existant
est réutilisé pour build.rs. Coût justifié : vérification structurelle fermée
et persistance privée pour supprimer la mise à jour manuelle des profils.
Programme global InProgress : ce correctif ne clôture pas les catégories restantes.

## Livraison constatée

Source fonctionnelle a83186dfb609fcc2cda2e96722f9022022376d33, intégrée à main et
poussée. Les357fichiers du manifeste d’assemblage correspondent au commit ;
14fichiers de sources ont changé. Assemblages via scripts/build.py :
Mac1min26s et Linux2min32s. Le candidat R1 n’a jamais été activé.

Build actif221-program-compat-r2 :
- Linux SHA256 d64028f32c74e76f3fd2f5867ccfa9ce65c5f8480041b6667fd6280bab2c3955.
- Mac SHA256 0c2281a23dc53695b546021375925f7e6cc6d6ebdfaaf92c63c9823490ef7ad5.
- Cartae maître906655, mobile906717, worker907557.
- SYS worker1275565 ; KS worker1644442.
- Mac worker7188 et pont7623.
Les images réellement chargées ont été hachées ; ce n’est pas seulement le
lien symbolique d’installation. La capacité worker_program_compatibility_v1
est présente dans l’identité du maître chargé.

Retour : image précédente221-worker-probe-lane-r2, formats contrôlés dans les
deux sens et clients check/validate-report PASS avant chaque installation.
Pointeurs natifs rollback Cartae/Mac mis à jour par l’écrivain atomique sous
verrou updater, sauvegardes conservées et outils de mise à jour redémarrés.
Linux retourSHA7cd9c1a8957dd350a9ec9d41d02119bfe104b933c8656328cb725183f6d49396 ;
Mac retourSHAd9a6b0987034496a4ff2d14a68a57da74477ba0bf40a77189d57569d7d5852aa.

Coupure maître2,20s ; wrapper Cartae2500061/naissance270585460 conservé.
Aucun processus métier, ancienne mission ou compte relancé.
Une réattestation du fil puis capabilities a rencontré primary_absent pendant
la republication. Après l’exposition réelle de bridge1016 dans le log du pont,
la lecture a réussi :45options, delegations_halted=false. Aucun nouveau
redémarrage ni boucle de réattestation. Les anciens clients MCP ouverts restent
conservés ; ce lot n’ajoute aucun outil/schéma et le guidage vient du maître actif.

Préavis et retour envoyés aux quatre propriétaires concernés. Résultats de remise
in_flight conservés : absence d’ACK ne devient pas une preuve de lecture.
Sources main synchronisées sur le Mac et Cartae. SYS/KS : main des deux magasins
mirror.git/repository synchronisée par bundle ; HEAD des checkouts de travail
détachés conservés, aucune réécriture d’origin.

## Entretien de livraison

Trois copies Linux de transfert retirées après vérification des images installées :
125189688octets. Actif et retour vérifié conservés sur chaque hôte.
installed a été appelé sur chaque staging : Mac refuse un staging non enregistré
pour ce dépôt ; SYS conserve les intermédiaires faute d’observation complète.
Aucun effacement forcé. Inventaire SYS borné à30s : processus inconnus,
deadline/process_access_unavailable ; les worktrees de revue propres et intégrés
restent donc conservés. Le checkout221 reste le responsable des lots suivants.
Ces conservations ne sont pas présentées comme un nettoyage complet.
Programme56/186InProgress après T183–T186 ; aucune parité globale annoncée.
