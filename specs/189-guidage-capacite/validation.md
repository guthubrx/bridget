# Validation189
Constats réels autorisés en lecture seule : reçu opus166-correction-saut-planificateur-cartae-20261027-01 refusé durablement resource_saturated sans tâche. Le snapshot initial enregistré est non saturé (CPU35%, mémoire disponible11%, charge28%, seuils90/10/100). Aucune preuve conservée du contrôle final : impossible de dire si la saturation à cet instant était réelle ou incorrecte ; ne pas utiliser la charge actuelle pour reconstruire cette histoire.
La trace native d605c969 montre le refus transitoire au tour200 puis une lecture réussie au tour201, résultat disponible et ACK par le propriétaire. Cause précise du refus antérieur non établie ; aucune action sur cette tâche.
Six tests189 PASS : preuve fermée/roundtrip, inconnus et données invalides, refus durable atomique/rejeu immuable/sans tâche, ancien refus sans preuve, dépassement de borne sans mutation, précontrôle MCP refusé sans seconde connexion ni opération de tâche et sans donnée privée. Le premier build de tests a trouvé une erreur de portée du module de fixture MCP, corrigée avant ces contrôles. Aucun runtime métier ou service modifié.

Le contrôle de régression du service168 a révélé une assertion qui exigeait l’ancien transport interne nu ; l’assertion vérifie maintenant le code public inchangé et la preuve CPU réellement reçue (95%). Aucun critère de saturation assoupli. La régression du score168 passe.

Source001a97aaf814ac7fe0c7fbd0f9604665604a0e7b : 23 contrôles distincts PASS (6spec189,14store163,score168,service168,prétransport138). Documentation32pages/522liens/13pairesPASS. Aucun test ignoré par ces filtres ; aucune FULL. Contre-revue indépendante demandée avec cette source exacte.

Contre-revue b45d760a-4c89-4982-93a7-194de2270bf2 favorable sur001a97aa, résultat lu/ACK et garde fermée. Revue statique, normes absentes de son checkout signalées ; aucun build ni test délégué. Build release PASS (1min11s), cache partagé7,11Gio. Statut natif final : maître183-admission-7478e9e7 ; tâche historique e5959986 toujours cancelling/native_cancel_unverifiable. Aucun Prepare identique répété, aucun arrêt forcé ni modification manuelle du store.

Complément du signalement opus166-ksstore-resource-limit-vue-20261027-01 : binaire KS réellement chargé183-b6c045cd, PID2399518 ; source b6c045cd résolue. En production, CandidateProbeRefusal::ResourceLimit provient uniquement de worker_inbound_busy, appelé par try_send Full. File sync_channel bornée à8 demandes techniques. Aucun journal de cette branche au moment du refus ; profondeur et état actuels non observés, cause amont du remplissage inconnue. Aucune sonde manuelle additionnelle ni demande métier rejouée pour ce diagnostic.

Complément file worker : 8 tests189 et3tests186PASS ; contrôles de saturation de la file172 et guides en cours. Tests ajoutés : cause limitée au motif candidate/resource_limit fermé ; refus worker/ResourceLimit et payloads invalides ne permettent aucune conclusion de file pleine ni fuite de données. Aucune règle de capacité modifiée.

Source849c88993ae04c9c30fa59eb4c7d751ad2532f92 : les13contrôles du complément sont PASS (8spec189,3spec186,2worker172) ; guides32pages/522liens/13pairesPASS. Aucun changement des limites, aucun test FULL, aucune nouvelle admission métier.

Contre-revue4a94a7a0 défavorable sur849c8899 : chemin initial Ok(routing_probe_failure) non couvert par le guidage encodé ; test worker/ResourceLimit invalide car sérialisation snake_case. Corrections : projection commune appelée après validation/construction des motifs sur les deux chemins ; motif worker/resource_limit valide conservé sans conclusion de file pleine ; test natif complet du probe initial jusqu’au refus not_recorded, sans tâche ni réservation et worker connecté. Le candidat849 rejeté n’a pas été installé ni activé.

Complément corrigé :14contrôles PASS (9spec189,3spec186,2worker172), dont refus initial not_recorded via le vrai handler natif simulé et motif worker/resource_limit valide ; guides32pages/522liens/13pairesPASS. Aucune sonde manuelle additionnelle sur KS ni demande métier rejouée ; aucun changement de limite ni activation.

Contre-revue finale dd494bef-5204-48a1-a5ed-519c700f716b favorable sur550b096f ; les deux objections4a94a7a0 sont corrigées. Résultat lu/ACK et garde fermée. Candidat189-queue-550b096f9b32 vérifié horsstaging, releasePASS, installedPASS, cache7,11Gio ; non actif.

Les nouvelles missions sont uniquement les contre-revues natives autorisées ; leur admission peut utiliser les sondes habituelles de flotte. Aucun appel manuel supplémentaire au probe KS, aucun rejeu ou nouvelle mission Opus2D. Les contre-revues ne sont pas présentées comme une absence de trafic natif.
