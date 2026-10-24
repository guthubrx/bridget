# Routage : changement d'effort et empreinte du profil

## Incident et cause

Les deux demandes de revue mobile du 24 octobre sont enregistrées en refus
durable `routing_connection_changed`, sans tâche et sans choix durable.
Les journaux du worker montrent, pour chacune, un premier lot de dix candidats,
puis deux sondes d'un candidat. Ce second contrôle unitaire correspond à la
branche où l'effort choisi diffère de l'effort par défaut.

`WorkerRuntime::inspect_candidate` résout le profil avec les arguments du
modèle et de l'effort. Son empreinte change donc lorsque l'effort change.
`refresh_routing_selected` exigeait pourtant la même empreinte à cette étape.
Le contrôle confondait une variation autorisée de l'effort avec une dérive
de la définition. L'effort exact choisi lors des incidents n'est pas conservé
dans ces reçus refusés ; il n'est pas inventé ici.

## Correction

La première relecture du candidat reste inchangée. Si l'effort change, la
seconde requête contient deux candidats : l'effort choisi et l'ancien effort
comme témoin. Aucun autre modèle, compte ou hôte n'est essayé.

Le worker V2 charge déjà un registre unique pour le lot. Il clôt aussi une
opération commune de lecture des droits ; une source instable invalide les
verdicts du lot. Aucun champ ni version de protocole n'est ajouté.

Le témoin doit conserver définition, droits, époque, mapping et projet.
La sélection doit rester sur la même époque, le même mapping et le même projet.
Son empreinte de définition peut différer : elle décrit ses nouveaux arguments.
Ses droits restent attestés, puis contrôlés à Prepare. La disparition d'un
des deux verdicts ferme le contrôle ; le tri ne transforme pas le témoin en choix.

Coût : un candidat supplémentaire dans un lot borné à deux, sans aller-retour
réseau supplémentaire. Aucun changement de schéma, d'idempotence ou de permission.

Les refus réels `routing_connection_changed` et `routing_candidate_unavailable` gardent leurs codes. Son détail explique maintenant la vérification
du catalogue et du lien, le suivi de l'admission existante et la fermeture d'un
refus durable. Le texte n'invite pas à contourner les contraintes. Les motifs
fermés reçus restent présents dans `detail.reasons` ; sans motif, aucune cause
n’est inventée. Le sort de la demande reste distinct de l’éligibilité du candidat.

## Référence T3

Dans T3, `ProviderCommandReactor::ensureSessionForThread` vérifie l'identité
de l'instance. `CodexAdapter::sendTurn` transmet séparément l'option
`reasoningEffort` pour cette instance. Ce mécanisme confirme la distinction
entre identité du fournisseur et option du tour. T3 n'a pas la sonde distante
de Bridget ; il ne justifie aucune suppression de nos preuves de droits.

Pour le guidage, `ClaudeModelCatalog::formatClaudeVersionUpgradeMessage` relie
une version constatée à une action précise. `applyAntigravityAcpModelSelection`
refuse le modèle absent des options et conseille un modèle disponible. Bridget
conserve aussi le motif observé, mais ne conseille aucun remplacement des axes
imposés et doit en plus expliquer le sort de sa demande idempotente.

## Validation

Le test de routage avec sockets Unix utilise maintenant les vrais digests de
`AgentRegistry::pin_delegation_model` et `resolved_definition` pour high/low.
Avant correction : échec `routing_connection_changed` sur ce seul changement.
Après correction : même test réussi.

La fixture couvre aussi une dérive de définition, mapping ou droits, puis
la disparition du témoin et celle du choix. Toutes restent refusées sans tâche
ni remplacement. Les réponses périmées et le rejeu du choix durable restent testés.

Lot initial `spec168_` et `native174_projet` : 40 réussis, 0 échec, 0 ignoré.
Il inclut le routage, les choix HTTP, les quotas, l'idempotence et le précédent
correctif de projet. Les fournisseurs sont simulés ; aucune mission métier
mobile n'est rejouée. Les journaux RED/GREEN restent privés hors Git.

La contre-revue officielle initiale est elle-même refusée par l'ancien binaire :
`bridget174-routing-effort-review-20261024-01`, état refused, task_id nul.
Ce refus n'est pas une revue acquise. Aucun repli de moteur n'a été employé.

La contre-revue suivante a été admise après activation du correctif :
`35eb38ee-93c6-4be7-a404-674ff973b916`. Elle valide la comparaison des sondes et
la revalidation à Prepare. Elle a demandé le guidage de
`routing_candidate_unavailable`, ajouté dans le complément, et l’actualisation
du présent reçu. Les tests de disparition des deux observations vérifient aussi
le guidage ; le test des motifs structurés vérifie qu’ils restent visibles.

Complément : même lot et `incident171_routing_probe_refusals_reach_client_without_admission`
réussis, soit 41 tests, 0 échec, 0 ignoré. Le premier essai de compilation du
complément a réutilisé la dépendance transport d’un autre checkout utilisant
le même cache et a échoué sur le champ mobile `guide_now`. Le contenu du
worktree est resté intact ; la dépendance transport a été recompilée depuis
ce worktree avant le passage réussi. Aucun cache n’a été copié.

## Livraison

Correction préparée sur la base commune `fb415ad7`, dans une branche séparée.
La source `6582c20f` a été intégrée et poussée sur main, puis activée sur le
maître Cartae sous `174-routing-effort-6582c20f`. Le mobile reste sur le binaire
commun `177-fb415ad7-opt` ; systemd l’a reconnecté lors du remplacement du maître.
Le worker `177-mobile-r8-opt` reste utilisé et n’a pas été redémarré.
Les trois refus durables précédents sont inchangés ; aucune mission mobile
n’a été rejouée. Le complément `5ebc8a44406c638e36fa073d0b2257c21b652e51` est intégré,
poussé et récupéré sur le Mac. Le binaire optimisé a été construit en 3 min 35 s,
puis activé sur le maître Cartae sous `174-routing-effort-5ebc8a44`.
Son SHA-256 est `062b248e02f5af9b37cc791ce2093a46d04efe13b410e15fc9ee67e00e9cd30d`.
Le statut officiel confirme `running: true` et ce build. Le maître passe de
PID 768307 à 792812 ; le mobile est reconnecté par systemd (PID 793066) sur
son binaire commun inchangé. Le worker PID 645547 reste inchangé. Le binaire
`174-routing-effort-6582c20f` est conservé pour le retour arrière.

Après activation, les empreintes des trois reçus refusés restent identiques,
avec `task_id: null` et aucune tâche correspondante. `quick_check` vaut `ok`.
La revue admise est terminée : résultat remis, `cleanup_done: true` et garde
libérée. Aucun nouveau modèle n’a été lancé pour une recette métier mobile.

## Nettoyage et conservations

Le worktree `174-routing-effort` et sa branche `fix-174-routing-effort` sont
retirés sans force après intégration, contrôle des fichiers ignorés, absence
de référence par une tâche et observation complète des fichiers ouverts.
Aucune branche distante de ce nom n’avait été créée. Mac et Cartae ne gardent
que leur checkout main après ce lot.

L’ancien binaire `177-r8-native-project-d5af84f8` est retiré après vérification
des usages et des références de service : 37 261 384 octets logiques.
Restent le maître actif, son retour arrière, le binaire mobile commun encore
utilisé et le binaire r8 encore utilisé par le worker. Aucun processus d’un
autre travail n’est arrêté pour ce nettoyage.

La commande `installed` a retiré 0,56 Gio d’intermédiaires release du cache
commun. Elle a ensuite refusé le retrait du profil debug, car l’observation
des processus est devenue incomplète. Ce profil est conservé avec cette limite ;
les preuves privées restent hors Git. Aucun cache target n’a été copié.

Main Mac est propre. Main Cartae est aligné sur les commits poussés mais porte
les modifications mobiles en cours du responsable mobile. Ces travaux ne sont
ni committés par ce lot ni supprimés. La compilation et les tests du présent
correctif ont utilisé son worktree propre, pas ce delta mobile concurrent.
