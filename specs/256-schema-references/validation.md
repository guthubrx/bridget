# Validation 256 — services et catalogue livrés en 0.8.23

9 novembre 2026. Source exécutable :
8ed970a7f74dcb27e9a1894ad51c1b46f9e21319.
Build actif : 256-reference-schema-r1.

## Constat et correction

Signalement Parix mcp-58833-6af1da92-40 : une continuation a été refusée avant
enregistrement pour references[0].label absent. Le propriétaire a déjà soumis
sa correction avec une nouvelle clé ; aucune demande métier n’est rejouée ici.

La lecture du catalogue brut 0.8.22 confirme que kind/label étaient déjà requis
et que file imposait host/path par un allOf conditionnel. La déclaration de la
façade chargée affichait pourtant Array<unknown>. La conversion interne de cette
façade n’est pas prétendue diagnostiquée dans son code.

Le schema partagé du brief expose désormais six objets complets dans oneOf.
Chaque branche déclare son kind, label et tous ses champs propres obligatoires,
avec additionalProperties:false :
- file : host, path ;
- url : url ;
- message : id, target, source_label ;
- journal : agent, from_seq, to_seq, source_label ;
- thread : thread_id, from_seq, to_seq, source_label ;
- artifact : artifact_id, version_id, source_label.

Ces champs correspondent au validateur natif existant. Les bornes de séquence
sont0 pour journal et1 pour thread. Les limites de artifact_id/version_id suivent
maintenant MAX_ITEM_BYTES=2048, comme le natif ; l’ancien schema les sous-décrivait
à256. Le schema conserve les extensions de limites UTF-8 et de taille sérialisée.
La description du brief et celle de task_continue rappellent kind/label et,
pour file, host/path absolu. Aucun changement du validateur, de l’enveloppe de
mission, des clés, des droits, des tâches ou des formats de stockage.

## Contrôles utiles

Trois contrôles distincts passent sur Mac et Linux, sans addition des plateformes :
spec256_reference_variants_expose_native_required_fields,
spec252_brief_schema_exposes_native_utf8_limits,
native153_mcp_continue_catalogue_et_champs_fermes.
Le nouveau test couvre six formes valides, chaque champ requis retiré, puis un
champ supplémentaire. Il compare aux refus du validateur natif existant.
Les régressions vérifient limites, schema partagé et fermeture de continuation.

Un processus MCP neuf 0.8.23 répond réellement à initialize/tools/list :
six branches complètes dans delegate, auto_delegate, task_continue et handoff.
Contrôle indépendant avec Draft202012Validator sur ce schema effectivement
publié : six exemples valides acceptés, 35 formes incomplètes ou avec champ
supplémentaire refusées. Aucun appel de modèle pour ces qualifications.

Premier script d’édition arrêté par une assertion sur une phrase présente deux
fois ; la modification de description a ensuite ciblé la seule ligne
task_continue. Aucun doublon ajouté ni validateur modifié.
Builds Mac/Linux via scripts/build.py, caches partagés entretenus. APK8023
construit en46s, signé, certificat et lignée8022 vérifiés. Contrat API technique
et versionAndroid seuls : pas de correction d’écran nécessitant un diaporama.

## Revue et limites

Contre-revue native 8dc3bc36-3df1-4668-bf1a-c8a45153163f, ClaudeKS/Sonnet,
source exacte8ed970a7 : résultat lu et acquitté, aucunP1/P2 confirmé.
Les quatre P3 sont disposés dans adversarial-review.md. Aucun build ou test
indépendant n’a été prétendu exécuté par le relecteur.
Analyze PASS, Converge CONVERGED, aucun manque confirmé du périmètre annoncé.

Limite client : le rendu TypeScript de chaque façade après chargement n’est
pas mesuré. Le schema standard et ses descriptions sont qualifiés ; aucune
garantie d’élimination de Array<unknown> dans un client particulier n’est
inventée. Les anciennes façades gardent leur catalogue jusqu’au rechargement
naturel. T3 reste ouvert. Les contraintes lexicales (chemin absolu, http(s),
UUID, texte non blanc, ordre des séquences) restent contrôlées par le natif ;
le schema ne prétend pas remplacer ces contrôles.

## Activation réelle et préservation

Sept composants 0.8.23 chargés et SHA vérifiés sur quatre hôtes :
core/mobile/worker Cartae, workersSYS/KS, worker/connecteurMac.
Outils de mise à jour relancés nativement après écriture atomique du rollback
sous verrou, sans activation concurrente.
Mac SHA378bb021eb78f641d78285cc7c9c296c27b619b24acd89ea450d55bf3bebb13a.
Linux SHA22e05c5f39c1124fe9e832485bcabac9e69158e5f69cf0db25ba2f2ed7250fb3.

Retour255R1, source0fc30045a675b21e53f14b5333bb522d2cd4cbba :
forward/return/inverse et SHA qualifiés, ancien lecteur245 refusé.
Versions des clients installés vérifiées sans authentification ni modèle.
Coupure du maître1,60s. Un wrapperCartae et deuxSYS aux passages gardent
leurs PID et identités de démarrage ; aucun wrapperMac/KS actif à ce contrôle.
Onze processusT3 et leurs dates de démarrage, septpayloads protégés inchangés.
RegistreMac inchangé pendant256, empreinte capturée avant et après :
2b4f409e464066f099a6541009332e567ae6a0fb772596421b6172c93f9d32c9.
Ce repère actuel remplace60acca pour les prochains passages ; sa variation est
antérieure au passage256, pas attribuée ici à une cause non lue. Ne pas restaurer
l’ancienne copie à l’aveugle.

Session nativeV2 réussie ; parentParix busy/projetattesté après passage.
T3 n’a pas été arrêté ni remplacé. Les anciens MCP et les wrappers de modèles
encore vivants restent conservés, sans activation globale de ces processus
annoncée. Aucune demande métier, ancienne clé ou tâche n’a été rejouée.

## Publication, diagnostic, Git et nettoyage

Release publique v0.8.23 : six actifs téléchargés anonymement, SHA conformes.
APK8023 signé, 3967548octets,
SHA29b1407cd89f6e5e03981b73b1231ce0ff4ba9b4bae65f17879c31c4d6e64424.
Source intégrée et poussée ; main final propre et aligné sur les quatre hôtes.

Parix prévenu avant passage et informé du contrat actif avec ses champs requis.
Reçu final mcp-91015-6af1ded8-3,
remise4f60e0f1-4aad-4c60-9cd6-1f93ad8d1bb3, initialementin_flight.
Aucune lecture métier affirmée, aucun broadcast ni demande de relance.

Propre worktreeMac et branche locale/distante retirés après intégration,
contrôle des ignorés et absence d’usage. Treize copies de transfert et
téléchargements de vérification retirés : 307279259octets logiques.
WorktreesbuildSYS/AndroidCartae intégrés et propres conservés : cinq processus
SYS et trente-neufCartae illisibles lors du contrôle des usages. Aucun processus
étranger arrêté pour le ménage. Vue native de revue, reçus, sauvegardes, preuves,
image activeR1, retour255R1 et anciennes images utilisées ou sous observation
incomplète conservés. Derniers caches reconnus : Mac5,09Gio, SYS8,59Gio,
Cartae4,46Gio, KS0Gio.

Preuves et recettes privées :
/Users/moi/.cache/bridget256-schema-references/.
