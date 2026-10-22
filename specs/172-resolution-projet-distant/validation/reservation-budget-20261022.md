# Réservation du résultat distant — livraison du 22 octobre 2026

Le build 172-cbc034bd est actif dans le maître Bridget. La réservation du résultat dispose de 15 secondes au lieu de 3. Le délai RPC et l’échéance envoyée au worker restent issus du même délai. Les contrôles d’expiration et d’annulation sont conservés.

## Incident et portée

Tâche signalée : e6283066-7755-4d6b-be16-54f3f167d6a8, demande opus158-notredame-partiel-integre-revue-cartae-20261022-01.

La lecture durable donne failed, cleanup_done=true et aucun résultat. Le worker conserve sa commande en phase terminale, sans permis ni release_epoch. Le wrapper 2318765 est absent. Le travail de revue n’a pas été remis au fournisseur. Aucune annulation, reprise ou nouvelle mission métier de remplacement n’a été envoyée par ce correctif.

Une sonde du même worker a pris 4,318 secondes dans la fenêtre du refus. Cette trace ne suffit pas à prouver la cause du délai historique. Le défaut de budget est reproduit séparément par le test ci-dessous.

## Correction et preuve discriminante

Source binaire : cbc034bd8fa5f8d8699f2f621955df8c831426e2, intégrée et poussée dans main.

Le nouveau test utilise un vrai maître, un vrai worker et leurs transports Unix. Un relais de fixture retarde de 4 secondes la première requête ReserveResultSlotUntil avant son arrivée au worker. Le fournisseur est simulé en Python, hors réseau.

Sur l’ancienne production, ce test échoue avec remote_start_failed:remote_result_slot:WorkerUnavailable. Avec la correction, il vérifie une seule réservation, un seul tour fournisseur, un résultat unique et le même résultat au rejeu. Le relais préserve la négociation du délai dans ce cas. Les anciennes fixtures gardent leur masque.

Les contrôles ciblés donnent 17 PASS :

- 1 test de bout en bout avec retard de réservation.
- 2 tests de négociation du délai et de message d’erreur.
- 14 tests worker172_inbound_, dont expiration après file et verrou, annulation et saturation sans effet tardif.

Aucune suite complète. La fixture a la même empreinte dans le RED et le GREEN : 8b8ef1b8547239768819bf67e25f0a3a2440882a024d2378351add061dfee09f. Empreinte de native_delegation.rs final : dbe18130e9b14dd79ce23e0aa6c4169d97f96a109b28a77e085c1147e2470962.

Référence T3 lue : /Users/moi/11.Repositories/t3code/apps/server/src/provider/acp/AcpSessionRuntime.ts, lignes 65–67, 734–749 et 935–950 sur ef86c2bbc3388be8fc593db23a323e3d80b58509. T3 applique des budgets par opération. Il n’a pas de réservation équivalente pour un résultat de worker distant. Bridget conserve son échéance commune et ses preuves de nettoyage.

## Revue et limites

La revue indépendante 74f5eb18-7220-46c5-b124-d5a11df5a54c donne APPROVE. Résultat complet lu, ACK confirmé, tâche clôturée. Le code auteur et le code relu sont au même commit ; leurs empreintes et états Git propres ont été comparés par l’opérateur.

Réserves conservées :

- Une saturation prolongée peut dépasser 15 secondes. Le test couvre un retard de 4 secondes, sans reproduire une véritable charge de sonde.
- La réservation peut être appelée dans la boucle de rappel déjà existante. Un worker silencieux peut maintenant retenir cette boucle 15 secondes au lieu de 3. Le correctif n’ajoute pas de nouvelle tentative ; les tentatives de réservation préexistantes pour mission_pending restent inchangées.
- Les 60 secondes du permis concernent ReleaseLeaseV2. Le permis des workers anciens reste à 5 secondes. Ce parcours ancien n’est pas une preuve de continuité qualifiée par cette livraison.

## Compilation et activation

Binaire macOS signé et vérifié : SHA-256 5d09a527e3a93484d1c843211275608f8f8eeb3beb28da26f0b2eaf056c38089. Compilation réussie en 116,987 secondes.

Le seul maître passe du PID 76289 au PID 13340. Le contrôle officiel bridget_status confirme running=true et build_id=172-cbc034bd.

T3 50681/50799 et le pont 898 sont préservés. Le worker Cartae 2219043 reste actif, sans redémarrage, sur son build compatible 172-76b682d6. Aucun changement de protocole, de droits ou de configuration fournisseur.

Les commandes Notre-Dame 28da9844-0695-4f99-9159-b9beeab9bd6d et revue e8df00b4-e3b4-482c-83bc-4f305ea2ac28 conservent leurs identités, générations, marqueurs, wrappers et fournisseurs pendant le remplacement. Leurs résultats sont ensuite disponibles sans erreur. La tâche historique e6283066 reste en échec nettoyé ; elle n’a pas été relancée.

## Recette réelle après activation

Task 9d13c023-eed1-4522-8e73-d7962132766c, demande recette172-slot-budget-live-cartae-20261022-r1.

La lecture réelle par l’enfant Bridget sur Cartae retourne hostname cartae, version 0.4.1 et Duration::from_secs(15). Marqueur BRIDGET172_SLOT_BUDGET_LIVE_OK reçu. Chemin fourni : /home/moi/10.Scripts/64.bridget. L’enfant signale qu’il ne peut pas vérifier le chemin canonique sans shell ; l’opérateur confirme séparément ce chemin canonique par lecture Python sur Cartae.

Résultat complet lu, ACK confirmé. Un seul rejeu strict donne la même Task et le même résultat textuel. La recette est clôturée, continuable=false ; le store donne cleanup_done=true et guarded=false. Aucune remise fournisseur métier.

Cette recette prouve le parcours distant après activation. Elle n’injecte aucun retard dans le worker de production. La preuve du retard vient du test isolé.

## Transmission et conservation

Résultat envoyé à l’intégrateur Opus2D par mcp-59331-6ada1310-1, et à Monuments par mcp-59331-6ada1310-2. Reçus initiaux in_flight, sans confirmation de lecture.

L’archive source de compilation macOS est retirée après lsof complet sans fichier ouvert : 96 056 322 octets logiques. Le binaire final, le rollback et les preuves privées sont conservés dans /Users/moi/.cache/bridget-slot-budget172. Le cache Cargo partagé est conservé.

Les worktrees Cartae auteur et revue, propres et fusionnés, restent conservés : le contrôle lsof signale un inventaire incomplet sur des montages système. Aucune suppression forcée. Cette correction technique ne nécessite pas de diaporama d’interface.
