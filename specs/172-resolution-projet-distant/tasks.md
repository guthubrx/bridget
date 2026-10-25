# Tâches 172

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/tasks.md#extrait-1).

- [x] T012 Prouver le parcours natif complet par le parcours natif Bridget dans le banc isolé, y compris Ready retardé et rejeu.
- [x] T013 Recette parent natif réel Cartae → enfant Cartae, par le parcours natif Bridget, résultat et rejeu prouvés (r3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/tasks.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/tasks.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/tasks.md#extrait-4).

- [x] T023 Corriger la continuation distante après renouvellement du tour parent, sans changement de droits.
- [ ] T024 Activer le correctif du maître sans interrompre BUS et vérifier la même continuation VOITURES.

- [x] T025 Corriger la sortie fatale du worker sur EOF de sonde ; preuve RED/GREEN et refus permanents conservés.
- [x] T026 Activer le worker corrigé en l’absence de mission native active (527308bb, PID4164309 ; reçu privé).
- [ ] T027 Reprendre explicitement le travail VOITURES après la perte de son ancien processus, sans présenter une nouvelle tâche comme la continuation ancienne.
- [x] T028 Corriger les deux fixtures de rondes héritées : profil0770 observé, permissions0700 explicites ; 14tests de rondes PASS.

- [x] T029 Reconnaître le code Claude rate_limit malgré api_error et préserver une réponse ultérieure réussie.
- [x] T030 Lier les refus observés à la connexion distante ; premier test réel r4/r5 : classification correcte, exclusion perdue après le TTL, complément T033 ouvert.
- [ ] T031 Complément quota1e5ddd07 construit et actif sur Mac/Cartae ; qualification métier VOITURES en attente du propriétaire.
- [ ] T032 Prouver la continuation réelle sur un enfant encore vivant après renouvellement du tour parent ; l’ancien processus54569150 est perdu.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/tasks.md#extrait-5).

- [x] T037 Corriger la réserve des reçus distants terminés sans supprimer les résultats ni affaiblir le plafond des résultats actifs.
- [x] T038 Rendre le refus de capacité distinct d’une panne de stockage ; conserver l’identifiant après admission durable même si le démarrage échoue.
- [x] T039 Vérifier les régressions du store et des admissions, préparer les paquets sans interrompre BUS.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/tasks.md#extrait-6).

- [x] T041 Reproduire le rejeu d’un incident fournisseur sur l’auxiliaire MCP ; réserver ce flux au parent primaire sans perdre ni acquitter l’avis.
- [x] T042 Vérifier les plafonds et le rejeu primaire, activer le correctif maître hors mission active, puis obtenir la lecture officielle de la Task623 depuis trajets bus.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/tasks.md#extrait-7).

- [x] T045 Reproduire et corriger la fermeture du canal worker à saturation de sa file bornée ; vérifier l’arrêt du lecteur et conserver des diagnostics fermés.
- [x] T046 Rendre lisible un refus de réservation du résultat, sans annoncer un démarrage ni supprimer la réconciliation ; tests et revue indépendante.
