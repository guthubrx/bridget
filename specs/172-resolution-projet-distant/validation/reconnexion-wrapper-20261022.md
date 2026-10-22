# Reconnexion du wrapper distant — 22 octobre 2026

Le worker Cartae exécute `172-50b0673f` depuis 2026-10-22T10:42:32.469598+02:00, PID 2037495.
Source de production revue et testée : `50b0673f9a26ce28e0f3a8dd35c4d9dd93834e74`.
SHA-256 du binaire : `548527c81da79b5c27423db41b21e930faf9a6c738bcb2faab1b141a59cec69f`. Paquet0.4.1, build distinct.
Les commentaires et deux assertions inertes sont corrigés dans66da72f8 ; la décision
exécutée reste identique au code source du paquet. Main est intégré et poussé.

## Remise façades et incident

Le fournisseur a reçu la Task1ff9a4d5-88c6-46f8-b6e1-f251869ee92f malgré le refus
initial remote_release:WorkerUnavailable. Le premier résultat a été accepté ; la
ronde2 a été acquittée. Ce refus ne prouvait donc pas une absence d'exécution.
La cause réseau précise de l'acquittement Release manquant n'est pas établie ici.

La ronde2 a ensuite été interrompue à09:36:55 CEST lors du redémarrage du maître
réalisé pour la livraison précédente. Le journal porte turn_end/cancelled à cet
instant. Le wrapper natif sortait sur EOF avec child_requests=deny, même avec un
contexte distant. Le worker restait vivant, mais fermait le slot sans résultat2 ;
le maître conservait un working devenu faux. Les identités persistées n'étaient
pas une preuve de survie du fournisseur. Le reçu précédent est corrigé.

Son propriétaire a appelé une seule annulation officielle, puis relu cancelled.
cleanup_done=true, guard_released=true. Aucun résultat2 final, aucune nouvelle mission
métier. Six commits propres sont conservés, tête e7d6d67b557715254095cfd5f949646653e12908 :
/home/moi/10.Scripts/69.opus2D/.worktrees/1ff9a4d5

## Correction et validation

Un WorkerResultContext validé utilise maintenant la reconnexion existante, avec le
même transport, sur EOF ou erreur de lecture. Les droits fournisseur sont inchangés.
Les natifs locaux gardent leur comportement. Disconnect explicite et signal d'arrêt
conservent leur effet. La grâce distante est300s par coupure ; un échange déjà engagé
peut dépasser d'environ20s. Chaque coupure réinitialise la grâce. Après expiration,
le fournisseur est arrêté. Cette valeur est retenue par le responsable de correction
avec environ4s de redémarrage observé ; ce n'est pas un engagement de disponibilité.

Référence T3 lue avant correction :
/Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderRuntimeRecoveryService.ts
T3 prépare et réconcilie la reprise sur tout le cycle. Bridget réutilise son worker
qui demeure présent pour garder le même fournisseur, sans rejouer le travail.

Sept tests distincts passent sur la source du paquet. Vrais daemon, worker et wrapper,
fournisseur simulé sans réseau : coupure/restart du maître en cours de tour, résultat
terminé pendant la coupure, grâce de reconnexion, panne native locale, expiration
avant release, annulation avec descendantsetsid et redémarrage worker sans second
fournisseur. RED/GREEN lié par empreintes sur le fichier final pour le tour actif.
Le cas de fin pendant la coupure est GREEN, sans RED séparé. Aucun test FULL.
Revue r2 9d063b81-c2e1-412a-b2d7-f5cf897bd78e : APPROVE, réserves mineures levées dans66da72f8.

## Activation et recette réelle

Seul le worker est remplacé, KillMode=process et SendSIGKILL=no. Le maître82087,
le pont898 et T3(28528/28591) gardent PID et date de naissance ; ASAR inchangé.
Le processus distant déjà actif conserve PID2027384 et naissance217334127.
Son journal avant/après livraison conserve la sessionclaude-stream-json-2027660.
Les wrappers antérieurs gardent leur ancien binaire jusqu'à leur fin ; le maître
n'est pas redémarré pour cette livraison. Les nouveaux wrappers utilisent le build corrigé.

Recette MCP officielle réelle : Task508c947f-038a-42fe-836f-363d1fad6fda, même enfant
e0c802bc-0b5b-41bf-945c-b303c949205b, deux rondes de lecture seule. Marqueurs
BRIDGET172_WRAPPER_REEL_R1_OK et BRIDGET172_WRAPPER_REEL_R2_OK, résultat intégral
lu et ACK à chaque ronde. Même PID et naissance du wrapper2038650 et du fournisseur2038724,
exécutable wrapper du nouveau paquet. Rejeu strict de la seconde demande : même Task,
même enfant, même résultat, aucune troisième ronde ni nouveau lancement.
Garde ensuite close par MCPcancel : résultat conservé, cleanup_done=true,
guard_released=true, idle_deadline_at=null. Aucun calcul métier lancé par cette recette.
Le maître n'est pas coupé pendant cette recette réelle ; les coupures restent prouvées en isolé.

Preuves et sauvegardes privées : /home/moi/.cache/bridget-remise1ff172 et
/Users/moi/.cache/bridget-remise1ff172. Ancien paquet conservé pour retour arrière :
/home/moi/.local/lib/bridget-communication/172-e5195b7de803/bin/bridget
Les fichiers de fabrication, journaux, bases et travaux étrangers sont conservés.
Le cache de build reste conservé : lsof incomplet, aucune purge forcée.
