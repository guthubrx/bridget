# Validation257 — actif0.8.24

Source exécutable :8a3dc84553fc3091f03e04a7d67ff80cbe02d0ca, intégrée à main et poussée. Build257-ack-diagnostic-r1.

## Défaut et correction
Un ACK groupé contenant une tâche déjà continuée refusait tout le lot avec task_not_ready sans nommer la tâche. Le handler nomme désormais les seules tâches non prêtes dans detail.tasks : task_id, round, state, workspace_report_ready. Il vérifie la propriété de tout le lot avant de révéler ces états. Le refus laisse toutes les tâches et les ACK inchangés. Aucun format durable, droit, mission ou seuil modifié.

## Contrôles utiles
- Mac et Linux : spec257 du groupe mixte PASS ;11régressions native151_ack PASS sur chaque plateforme. Douze contrôles distincts, recouvrement Mac/Linux non additionné.
- Le test du groupe mixte vérifie ronde4/mission_pending, ACK vide, absence de mutation de payload/epoch/projection et succès du groupe corrigé limité aux résultats prêts.
- Le mock MCP réel sur socket vérifie que le texte du refus conserve task_id, ronde, état et récupération. Un tools/list du binaire installé confirme la description actualisée sans lancer de fournisseur.
- Revue indépendante c0e70b5f-dbc6-46b9-8649-9e507ac130bd terminée, lue et ACK natif acquis. HypothèseP2 levée par le verrou réel du handler, couvrant lecture/ACK et Continue ; P3 disposés dans adversarial-review.md. Pas de seconde revue inutile ni de verdict indépendant inventé.
- Couverture spécifique workspace_report_ready=false non ajoutée ; prédicat et garde existants inchangés. Aucun essai de mission métier ni de modèle supplémentaire.

## Activation et retour
Sept composants contrôlés par chemin/PID/SHA sur quatre hôtes : worker et adaptateur Mac, maître/mobile/worker Cartae, workers SYS et KS. Mac SHA2fb8ba69d9ff66d7944ab7f8f8367477f703cc173281ab5a1ea663dd1a6cef23 ; Linux SHAb578e052bd3344ccd0d0a726a5119d7e6453891f6f8d7bab1d61c9475c9eccfb.
Retour256-reference-schema-r1 qualifié en lecture/écriture/inverse et pointeurs rollback écrits atomiquement sous verrou sur les quatre hôtes. Ancien retour245 refusé par le validateur existant. Aucun retour réel exécuté.
Coupure maître1,65s. Deux wrappers Mac, deux Cartae et trois SYS conservés avec leur identité de démarrage. Les sept payloads protégés et le registre Mac SHA2b4f409e464066f099a6541009332e567ae6a0fb772596421b6172c93f9d32c9 sont inchangés.
Application T3 non arrêtée/remplacée. Onze processus/birth initiaux restent identiques ; un renderer52941 a disparu et46921 est apparu à19:56:55, avant la maintenance Bridget Mac de19:57:33. Cause non attribuée, aucun faux bilan « tous les PID inchangés ».
Session nativeV2 attestée. Lectures juste après coupure temporairement primary_absent, puis annuaire natif rétabli ; Parix busy/projet attesté. Quatre rapports0.8.24/up_to_date/fresh vérifiés. Anciennes façades et wrappers en cours conservés ; aucune mise à jour globale de leurs catalogues affirmée.

## Publication et conservation
Release publique v0.8.24 : six actifs relus anonymement et vérifiés SHA. APK8024, certificat et lignée8023 vérifiés ; pas de reupload supplémentaire. Sources publiques : https://github.com/guthubrx/bridget/releases/tag/v0.8.24 .
Parix prévenu avant passage (reçu en vol, lecture non affirmée). Résultat déposé avec intent=information, reçu mcp-15009-6af2192e-2/stored_without_wakeup : aucune réponse ni tour demandé. Aucune mission métier rejouée.
Dix fichiers/copies vérifiés retirés,354832958octets logiques, dont l’ancienne image255 Mac sans usage et hors retour. Image active257 et retour256 conservés. Caches Mac7,22Gio/SYS0,61Gio/Cartae4,46Gio après entretien natif. Worktrees SYS/Cartae propres et intégrés conservés faute d’observation complète de5/45processus ; aucun processus d’un autre travail terminé pour nettoyer. Vue native de revue, preuves et images utilisées conservées.
Le worktree/branche257 Mac est retiré après intégration finale et vérification des usages/fichiers ignorés ; détail dans le reçu privé de clôture. Aucun changement visible d’interface : diaporama non requis.

Preuves privées : /Users/moi/.cache/bridget257-ack-diagnostic/ .

## Nettoyage complémentaire demandé le 9 novembre 2026

La demande de commit, intégration à main, push et nettoyage reprend la livraison
déjà intégrée en 44f3e51e. Aucun nouveau changement exécutable ni nouveau build.
Les reçus du nettoyage ont été relus après l'interruption du fil.

- Seize worktrees intégrés retirés : huit de compilation sur SYS et huit Android
  sur Cartae. Leurs seize branches ont aussi été retirées. Les fichiers ignorés
  étaient des sorties de génération identifiées ; les usages ont été vérifiés
  avant suppression. Taille logique des worktrees : 2 931 961 150 octets.
- Cinquante-neuf autres branches intégrées retirées de SYS après vérification des
  références de tâches et admissions. Total : soixante-quinze branches locales
  retirées. Le distant GitHub ne porte plus que main ; aucune référence distante
  supplémentaire à supprimer au contrôle final.
- Intermédiaires retirés : environ 7,22 Gio sur Mac, 655 557 731 octets sur SYS et
  4 784 487 547 octets sur Cartae. Sorties anciennes à la racine des profils
  retirées aussi ; reçus Mac et Cartae : 261 934 882 et 179 640 400 octets.
  Le total documenté dépasse 15 Gio logiques, worktrees compris. Ce total ne
  mesure pas directement l'espace physique récupéré.
- Un seul checkout subsiste sur chacun des quatre hôtes. Mac et KS ne gardent
  que main. SYS conserve trente-deux branches encore référencées par des tâches
  non closes. Cartae conserve une branche d'archive contenant du travail non
  intégré. Aucune clôture de tâche ni suppression de ce travail pour nettoyer.
  Images actives, retour256, anciennes images utilisées et preuves conservés.

Reprise : main propre au même commit sur les quatre hôtes et distant GitHub
vérifié. Les sept composants chargent toujours les SHA257 attendus. Le binaire
de retour et le champ rollback restent conformes sur chaque hôte ; aucun passage
d'activation en attente. Aucune sortie obsolète sélectionnée ne reste dans les
profils contrôlés. Les processus Mac ont changé depuis le reçu précédent ; leur
image chargée a été vérifiée de nouveau, sans attribuer la cause de ce changement.
Aucun arrêt ni redémarrage effectué pendant cette reprise.

Cette clôture modifie seulement le bilan documentaire. Vérifications : reçus,
état Git, fichiers présents et images chargées ; pas de répétition des tests
applicatifs déjà acquis. Reçus détaillés conservés dans
/Users/moi/.cache/bridget257-ack-diagnostic/cleanup-requested/ .
