# Correction 221 — mises à jour des programmes

État : code vérifié ; activation à consigner après chargement, T186 ouverte.
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
