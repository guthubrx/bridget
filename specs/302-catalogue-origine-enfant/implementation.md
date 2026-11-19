# Catalogue d'origine d'un enfant distant

Signalement Territoire mcp-43375-6aff39e1-25, tâche
4ea82821-6ff2-4f78-a2a6-7e352cd9ce71, enfant
6d6970cd-171f-403e-b306-f13ad4e66111. Lecture de la base maître en mode ro :
résultat conservé, état result_available, error null, placement sysle2.
Le résultat indique que la production métier n'a pas été exécutée.

Trace fournisseur exacte : bridget_capabilities, le 19 novembre 2026 à
17:59:11 UTC, refus origin_worker_unavailable en 76 ms. Le refus n'est donc
pas un délai de réponse de 30 secondes observé. Le wrapper a terminé à
17:59:50 UTC avec exit_code 0 et failure null. Aucun incident de lecture des
fichiers n'est déduit du refus du catalogue. L'alias SSH SYS absent dans
l'environnement enfant est un fait séparé, pas une panne du catalogue.

Défaut identifié : foreign_catalogue177 et parent_operation utilisent le nom
hôte de la présence. Sur SYS, le système et les enfants vivants annoncent
backup ; le registre provisionne sysle2 et ne contient pas de machine backup.
Le placement durable de cet enfant contient bien sysle2. L'identité de worker
est déjà résolue correctement par host_for_owner dans les autres parcours.
La présence arrêtée de l'enfant est une projection synthétique du maître et
ne constitue pas une preuve de son nom hôte historique. Le diagnostic
s'appuie sur le placement, la construction de l'environnement du wrapper,
le nom réel du serveur, les présences vivantes et la reproduction ciblée.

Test : avec une tâche distante placée sur sys302, une présence backup302 et
un worker courant enregistré sur sys302, la lecture du catalogue échoue avec
origin_worker_unavailable avant correction. Aucun vrai fournisseur lancé.
La première compilation du test a signalé une accolade manquante dans sa
fixture JSON ; corrigée avant la reproduction du défaut fonctionnel.

Correction : le catalogue et l'échange d'origine utilisent host_for_owner,
puis revérifient la même identité après réponse. Le worker doit toujours être
provisionné, connecté, récent et capable. Les contrôles de connexion primaire,
d'instance, de forme et d'empreinte des permissions restent en place.
Le refus brut réutilise le guidage d'attestation existant et garde son code.
Aucun alias ajouté, aucune configuration machine renommée, aucun droit élargi.

Activation : blocage natif update_missions_active_or_unconfirmed déjà constaté
au lot 300 ; aucun arrêt ni nouvelle mission de qualification lancé. La mission
métier reste terminée avec son résultat conservé, sans reprise automatique.

Après correction, le test traverse le worker simulé et reçoit le catalogue ;
le retrait de sa capacité OriginProofV1 provoque toujours un refus. Une assertion
de texte a été alignée sur le guidage existant (« mission de remplacement »),
sans modifier les contrôles. Le test dédié passe.

Revue native : première soumission rejetée avant admission parce que le SHA
source était abrégé. Le paramètre est corrigé avec le SHA complet, même clé
bridget-302-origin-catalogue-review-20261119-r1 ; tâche admise
 af5b69fd-b080-4117-b760-b1410fef8faa. Pas de tâche de remplacement ni de repli
vers un autre mécanisme de délégation. Revue en lecture seule du checkout Mac.

Revue reçue et lue par le port natif : aucune anomalie bloquante constatée dans
le chemin lu, mais une lacune d'identité signalée. Le relecteur n'a pas exécuté
les tests ni vérifié le diff. Correction ciblée : l'échange vérifie sous son
verrou l'identifiant et la révision du worker figés par le placement ; après
réponse, le placement, le worker courant et la liaison sont revérifiés.
Le placement durable est volontairement prioritaire sur le nom affiché par la
présence. Il ne dispense pas de l'identité provisionnée, de la liaison primaire
ni du contrôle des permissions. Aucun élargissement de droits.

Six scénarios du test 302 passent (7,14 s) : nominal, autre identifiant worker,
autre révision, changement d'instance pendant l'échange, changement du placement
pendant l'échange, empreinte de permissions incorrecte. Le cas nominal vérifie
aussi le refus après retrait de capacité. La compilation a identifié sept
appels de test à adapter au nouveau paramètre attendu du worker ; ils ont été
adaptés sans modifier leurs scénarios. Aucune mission fournisseur réelle lancée.

Nouvelle vérification de maintenance du 19 novembre : updates prepare sur le
maître refuse toujours update_missions_active_or_unconfirmed. Aucun bail acquis
ni service arrêté. Le statut fédéré indique waiting_idle sur Cartae et SYS,
et client_check_unavailable sur Mac pour le candidat antérieur 299. Ce contrôle
ne qualifie pas le lot 302, encore source seulement. La livraison active reste
incomplète ; les états natifs concernés doivent être clôturés par leur parcours
propre avant activation. Aucun rejeu de la mission métier n'a été demandé.

Non-régression après la garde d'identité : 15 tests origin_proof passent, avec
un test vivant sur deux hôtes ignoré par défaut car sa fixture privée est
requise ; 3 tests origin_busy passent également. Le nettoyage automatique a
retiré 10,03 Gio de caches après les tests. Aucun fichier métier ni preuve
n'a été supprimé, aucune branche ou worktree créé.

Clippy strict bibliothèque et tests terminé avec succès après correction de
revue (1 min 22 s). Diff complet relu, contrôle des espaces réussi. Cache final
0,84 Gio. Pas de preuve visuelle requise : correction du routage et du contrat
de refus, sans changement d'interface graphique. Sources prêtes à publier ;
aucune image 302 annoncée compilée, installée ou active.
