# Reçu durable de continuation — session221

Statut : correctif serveur actif, client MCP déjà ouvert encore ancien.
Livraison client incomplète tant que son image reste ancienne. Programme inchangé :
52/182, In Progress.

## Incident et diagnostic

Signalement `opus201-incident-continue-jsonl-20261101-01`. Tâche
`be04d2d9-fa1a-41a4-82aa-ef2f0db18a04`, clé de continuation
`opus201-patrimoine-modernes8-photos-r2-20261101-01`. Le propriétaire a reçu
« réponse daemon indisponible : budget JSONL dépassé ». Le registre confirme
la ronde2 `working`, erreur nulle, mission
`897ba2e4-53b5-4585-9c52-685d40ee2440`. Aucun rejeu de cette consigne.
Ce libellé JSONL correspond à un délai expiré, pas à un volume trop grand.
La trace historique ne permet pas de répartir son temps entre les étapes.

Deux écarts de code sont établis : Continue utilisait le budget des lectures
(10 s, plus33 s avec attestation) malgré une lecture des sources d’origine
plafonnée à30 s puis une observation worker plafonnée à30 s ; après le commit
de `begin_round`, le handler attendait aussi le tick général et ses éventuels
rapports/nettoyages d’autres tâches avant de retourner le reçu. Ces écarts
expliquent un risque vérifiable, pas une attribution historique certaine.

## Correction bornée

- Retour du reçu après `begin_round`, sans tick général dans cet appel Continue.
  La boucle native existante reprend les tâches persistées `mission_pending`.
  Aucun thread, autorité, store, protocole ou rejeu automatique ajouté.
- Budget spécifique de continuation :70 s sans attestation,103 s avec le budget
  d’attestation actuel. Sources30 s + observation worker30 s restent inchangés ;
  aucun délai métier, droit, compte ou modèle modifié.
- Lors d’une réponse perdue, le client récent nomme les deux identifiants,
  indique que la ronde peut déjà être remise et demande la lecture du statut.
- `mission_pending` est un reçu durable, distinct de la preuve `working`.
  Cadence cible1 s ; les autres opérations et la réservation distante peuvent
  retarder la remise. Aucune garantie de départ en1 s.

## Vérifications ciblées et contre-revue

Trois nouveaux tests bibliothèque Linux PASS : vrai socket avec réponse après
11 s et aucun rejeu ; déconnexion avec identifiants conservés et aucun rejeu ;
vrai handler Continue, même enveloppe deux fois, reçu avant tick, une seule
ronde/mission/enfant puis remise par tick. Régression existante native153
(plafond8rondes et rejeux) PASS.

Le premier appel Cargo sans `--lib` a rencontré cinq fixtures d’intégration
antérieures, dans `tests/fixtures/remote_worker_162.rs`, sans le champ requis
`service_tier`. Ce chemin n’est pas déclaré PASS. Les cibles bibliothèque et
les assemblages exécutables sont distincts ; aucune campagne générale lancée.

Contre-revue6622035b, moteurClaude choisi par Bridget : patch complet lu,
aucun défaut bloquant confirmé, sans compilation de sa part. Résultat lu,
ACK et clôture demandés une fois. Limites retenues : test de reçu local,
preuve distante toujours synchrone avant commit, cadence partagée et sans
garantie de latence. La remarque sur les30 s de sources est réfutée par
`handle_attested`→`reattest`, puis `remote_dispatch_proof`→`remote_parent_proof`
→`origin_proof::sources` (budget30 s) : le paramètre NativePermissionProof
est un accès au mécanisme d’attestation, pas une preuve déjà réutilisable.

Le nouveau budget et le message client exigent une image MCP récente.
Les clients déjà ouverts profitent du découplage serveur mais ne sont pas
déclarés rechargés. Aucune boucle de reprise de l’application hôte.
Aucune mission métier ni qualification fournisseur créée pour ce lot.

## Livraison et constat réel

Source377f647ed0dadf95e55140670b109c8e1d23095f intégrée/poussée sur main.
Cinq tests bibliothèque Linux distincts PASS : trois nouveaux, native153,
`spec172_remote_continuation_distinguishes_execution_state_from_permissions`
(neuf cas de preuve distante, sans nouvelle ronde). Ce dernier réutilise le
contrat distant existant ; ce n’est pas une nouvelle mission réelle. Aucun
nouveau test Mac ; assemblage Mac et contrôles de compatibilité réussis.
Les356fichiers d’assemblage correspondent à la source intégrée.

Build221-continuation-receipt-r1, version0.6.5 :
- Linux41398672octets, SHA256
  `31811c4db7b39bed72cb6bb6c0428e1b20869b088166f156126810217627e467`.
- Mac34180352octets, SHA256
  `a124206c2a67b4e5e2ac9c1582e71b395885a99c2cad231a30e894a9d7def762`.

Sept processus chargés et hachés : maîtreCartae266755, mobile266894,
workerCartae268899, workerSYS1209579, workerKS1592566, workerMac27982,
pontMac28279. Coupure maître2,13s. Wrappers observés conservés :
Cartae2500061/naissance270585460 et SYS1204789/naissance67974065.
Retourjournal-control-r3 vérifié dans les deux sens par le lecteur natif
des formats. Pointeurs rollbackCartae/Mac écrits atomiquement sous verrou
updater, SHA relus ; aucun retour exécuté.

Une session native réussie. Premier catalogue pendant republication refusé
`primary_absent` ; après preuve dans le log que bridge1016 est réexposé,
lecture suivante réussie :45options, `delegations_halted=false`. Une session
réussie seule n’a pas été prise pour preuve de retour du lien.

La tâchebe04d2d9 est désormais `result_available`, ronde2, erreur nulle,
mission897ba2e4 inchangée. Aucun rejeu ni résultat métier lu par ce fil.

Les MCP ouverts observés utilisent encore221-journal-control-r3,
221-thread-budget-r1 ou221-view-sync-r1. Aucun processus MCP récent
221-continuation-receipt-r1 n’est attesté au relevé final. Le serveur retourne
désormais le reçu avant le tick ; le nouveau budget et le guidage de perte
de réponse restent liés au chargement du client récent. Il n’existe pas
d’outil natif de rechargement MCP exposé dans ce fil. Geste humain précis :
finir ce tour, quitter complètement l’application hôte, la rouvrir et reprendre
le même fil, puis vérifier une fois l’image MCP chargée. Aucun marqueur ajouté,
aucun arrêt global de l’hôte ni boucle de reprise lancé par ce lot.

Entretien `installed` tenté sur Mac et SYS après contrôle des images. Mac
refuse le propriétaire du staging ; SYS ne peut pas attester tous les usages.
Caches conservés, sans forcer. Deux copies de transfert Linux et celle du Mac
retirées après vérification des images installées ; actif/retour, preuves,
anciennes images encore ouvertes et worktree de revue restent conservés.
Aucun service métier ou travail indépendant terminé pour le nettoyage.
