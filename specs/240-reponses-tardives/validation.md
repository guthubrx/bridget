# Validation 240 — correctifs activés, défaut de raccordement encore ouvert

## Résultats du signalement

Signalement mcp-61249-6aeebcc2-1, projet opus2d. Les lectures techniques
ne déclenchent ni annulation ni admission. Aucun rejeu métier.

- 5cd9de55 : result_available, sans erreur, cleanup_done. Le cancel du
  7 novembre à05:47:54UTC a échoué au raccordement, avant transmission.
  Le résultat ultérieur ne prouve aucune annulation.
- A2-vie-opus : reçu chosen/révision2, task_id nul, cible KS. Le canon
  conservé n'impose aucune machine ; la description sysle2 du signalement
  ne désigne pas la décision durable de cette demande.
- C-jonction : reçu admitted/révision3, tâche737767cf. L'appel interrompu et
  l'avis d'échec concernent cette même demande, pas deux missions inconnues.
  Appel interrompu à05:49:30UTC ; tour d'origine67 terminé à05:49:35.366UTC ;
  admission durable à05:50:06UTC ; échec avant remise à05:51:12UTC.
  La preuve du tour terminé ne suffit plus au contrôle final. Aucune remise
  métier n'est attestée par ces faits.
- Le snapshot de5cd9 impose en Claude discovery tools=[Read,Glob,Grep] et
  refuse Bash/Edit/Write/NotebookEdit/WebFetch/WebSearch. L'absence de git/wc
  via shell vient de cette posture, pour tous les modèles ; aucun droit shell
  n'est ajouté et aucune délégation passée sur T3 n'est modifiée.

A2 et C : canons, états, révisions et task_id strictement identiques avant/après.
Payloads7024/77ba/702d strictement inchangés ;2fb et revues métier conservés.

## Causes confirmées et corrections

Journal maître et worker KS, 7 novembre05:48:24UTC :
corrélation62e012c1-463f-4d09-9e73-63ffcd951c69 ; command_id et task_id
b6d51346-bd9e-48b3-86d8-d899398c5bb7 ; génération0, instance vide.
Rejected/PrepareExpired arrive après retrait de l'attente. Le maître rend
command_mismatch ; le worker ferme son canal. Ce défaut est confirmé.

Le cache éphémère conserve au plus4096 corrélations initiales réellement
écrites dont l'attente n'a pas rendu de réponse. Seuls les refus tardifs
exacts sont ignorés, après contrôle de forme, capacités, identité, connexion,
époque et commande. Aucune tâche, garde ou fraîcheur n'est modifiée.
Prepared réussi n'est jamais absorbé : son parcours existant de conservation
reste nécessaire. La fermeture physique ou le remplacement du lien purge
l'histoire ; un contact simplement périmé ne la purge plus. Une entrée évincée
reste soumise aux refus existants. Les répétitions du même tuple sont
volontairement inertes, sans nouvelle autorité.

Les causes de capacité/lien ne deviennent plus automatiquement des refus de
droits avant Release. Une réponse absente reste inconnue ; un refus explicite
garde son enum fermé dans origin_proof_rejected:<motif>. Le worker distingue
un refus natif typé de la preuve/tour d'un défaut de transport. Aucun détail
HTTP libre, secret, nouvelle permission ou assouplissement de preuve.
Le délai remote_prepare:WorkerUnavailable nomme l'étape et la lecture durable
à faire ; il ne déclare pas à lui seul le serveur arrêté.

## Limite encore ouverte : raccordement du parent

La cause précise des délais binding_transport_unavailable et JSONL n'est pas
établie. Ce défaut a aussi touché le premier appel de contre-revue R2, avant
exécution de l'outil ; aucun reçu n'était enregistré pour cette clé. Après
trois sondes de socket réussies et une lecture native authentifiée réussie,
la même demande exacte a été admise une fois. Aucun remplacement.

Une capture de piles du cœur, sans arguments de fonctions, a été faite en
0,216s pendant une lecture réussie : elle ne capture pas le blocage et
n'en fournit pas la cause. Les lectures natives rétablies ne prouvent pas
l'absence définitive de ce défaut. Ni panne de compte ni lien causal avec la
maintenance antérieure ne sont inventés. La prochaine preuve utile reste
la phase et l'attente réelles pendant un raccordement qui dépasse son délai.

Après activation, un avis a échoué sur un délai JSONL sans reçu. Le client
MCP/send est identique à237 : après émission, send_then_wait rend toujours
outcome_unknown avec id/issued_at. L'erreur générique observée précède donc
SendIdempotent ; aucun message correspondant n'est enregistré dans le ledger.
Cinq lectures natives concurrentes réussissent ensuite en2,225–3,426s pendant
une observation système bornée, sans contenu de trame. Elle ne capture pas
le défaut ; aucun appel non inactif observé ne dépasse0,539s. Le même corps
est repris après ce contrôle, sans redoubler les destinataires déjà servis.
Ce retour de liaison ne résout pas la cause intermittente.

## Tests et revue

Source exécutable :188dd0298db7f77488f5483b6d17a272611d82f0.
Le delta70908ef6 ajoute uniquement une branche de test sous cfg(test).
Six tests spec240 et deux régressions utiles passent : huit cas distincts.
Le test de timeout réel couvre réponse tardive, absence de Task, faux tuple,
époque périmée et contact non rafraîchi. Le complément couvre aussi le tick
retirant l'attente avant le retour du roundtrip ; il passe. Les répétitions
ne sont pas recomptées.

Deux premières exécutions de la simulation ont révélé sa capacité manquante,
puis son champ approvals_reviewer manquant. Les fixtures ont été corrigées
contre les validateurs réels ; aucune garde de production n'a été réduite.
Assemblages finaux macOS/Linux réussis par scripts/build.py, entretien avant
et après. La seconde compilation était nécessaire aux corrections de R1.

Revue R1 bd6babea-1308-4be8-bd3a-e54c5299b80e : cache acceptable, diagnostic
explicite bloquant. Objections traitées. Contre-revue R2
30b6c318-86c2-4d9f-9897-22d7ec02a289 : acceptable, aucunP1/P2.
Deux résultats lus et ACK acquis. Relectures statiques, aucun test exécuté
par les relecteurs ; résultats auteur distincts.

Réserves P3 : le journal d'absorption tardive ne contient pas son motif fermé ;
limite conservée. Les répétitions de tuple sont délibérément inertes. Le
retrait anticipé par contact périmé est maintenant testé. Un code
origin_request_invalid préfixé peut rester générique dans le réducteur
d'attestation ; aucun droit n'en découle. Un code synthétique resource_limit
préfixé est parsable, mais le producteur normal le rend en busy, ce que teste
la régression ; ce cas non canonique ne rend jamais une preuve valide.

## Activation vérifiée

Bridget0.8.7 /240-late-prepare-r2, sept composants chargés :

- Cartae : cœur1166311, mobile1166377, worker1168402.
- SYS : worker2523847. KS : worker3811163.
- Mac : worker11132 et pont11233.

SHA macOS :ad49331967df76da0b01f9dc7a1612eeac1e81f9fb77ce838079d1c0ba31b216.
SHA Linux :228663e5f95612745fbabbd4527711526a82e79fd7a3809b1d3288e21b318fe9.
Coupure cœur2,26s. Le superviseur métier Cartae2500061/birth270585460
est conservé. Registre Mac dédié inchangé :
cacf9ba06704faf0699e13a6827f556d799282f96fda02b6e4dfb643e8667374.
Retour239 compatible dans les deux sens ; rollback écrit atomiquement sous
le verrou natif sur les quatre hôtes. Aucun retour ni rejeu pour le tester.

Le port de mise à jour automatique a refusé
update_missions_active_or_unconfirmed. Aucune barrière n'a été forcée.
La procédure manuelle déjà autorisée pour une maintenance en activité a
sauvegardé les états cohérents, vérifié les phases des workers et les naissances
des processus, puis employé les unités natives et le remplacement atomique.
Aucun arrêt des processus métier ni modification de leurs demandes.

T3 n'est pas relancé : sept processus et leurs dates de démarrage inchangés.
Backend82856 conservé. AttestationV2, annuaire authentifié et fiche du rôle
révision3/transversal rétablis. Quatre rapports0.8.7/up_to_date/fresh.
Les anciennes façades MCP encore chargées restent conservées avec leur image.
François a explicitement reporté leur rechargement : ne pas redemander ni
fermer l'application.

## Publication et nettoyage

Release publique v0.8.7/id437391736. Six actifs téléchargés sans authentification,
tailles et SHA vérifiés ; les exécutables des archives correspondent aux images
chargées. APK8007 signé, certificat de production et lignée comparés à8006 ;
aucun changement UI ni test UI/FCM annoncé. Sources exécutables et tests intégrés
àmain et poussés. Le diagnostic des cinq signalements a été envoyé au
propriétaire, reçu mcp-66590-6aeec7ad-33, sans demande de rejeu ni lecture inventée.
Dix-sept préavis et seize avis de reprise généraux plus ce diagnostic ont un
reçu. L'unique échec préparatoire d'avis a été qualifié puis repris après retour
de liaison ; aucun message déjà servi n'est renvoyé.

Entretien installed Mac : cache0,72Gio sous budget. SYS : nettoyage refusé
par le contrôle natif, observation privilégiée des processus indisponible.
Ses sources239/240 et caches sont conservés ; aucun contournement.
Cartae : observation complète et zéro usage des deux sources Android239/240 ;
1928 fichiers/283269445octets logiques retirés. Avec les transferts et doublons
vérifiés, total1955fichiers/550148121octets logiques retirés sur les quatre hôtes.
L'actif240, le retour239, les images encore chargées, les releases originales,
les vues de revue sous rétention et le worktree Cartae1cc44f4d modifié/utilisé
restent conservés. La raison précise de chaque conservation est dans les reçus.
Preuves et journaux conservés.

Preuves privées :
/Users/moi/.cache/bridget240-orchestration-incidents.
