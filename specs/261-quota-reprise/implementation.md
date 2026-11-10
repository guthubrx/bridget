# Implémentation 261

Le pilote relit son quota natif au repos, sur son propre processus. Cadence
bornée à trente secondes, délai de lecture de deux secondes, aucune inférence.
La réponse est publiée par le lecteur existant vers la même instance.
Les gardes de quota, de dépense, de propriétaire et de reprise restent intactes.
Le refus de reprise ajoute les seuls faits de quota de l'enfant autorisé.

Contrôles ciblés Mac : deux scénarios de transport réussis, un scénario de
projection et d'absence de fuite à une autre connexion réussi. Un premier
essai de fixture a refusé un chemin temporaire trop long ; TMPDIR=/tmp corrige
l'environnement du test. La régression de qualification160 passe aussi.
Aucune batterie générale n'est lancée pour ce correctif.

Revue indépendante : tâche4c4916be-07e7-4ed0-9259-75827be1bc91,
clé bridget-261-quota-refresh-review-20261110-r1. Le patch soumis est complet ;
la vue de commit est la base245218a3cdd99a12b0eefc45b33ce0da8023d548.

Source exécutable : 75259217a72b3177cc0f50ba58c1fc3f97e5e22e, poussée sur main.
Version0.8.27 /261-quota-refresh-r1 activée sur les sept services, quatre hôtes.
Le core a été indisponible1,73s. Empreintes des exécutables chargés vérifiées,
formats aller/retour et pointeurs rollback qualifiés. Le registre dédié Mac
reste inchangé. Aucun ancien managed-wrapper trouvé lors du contrôle final.

Lecture native réelle, sans fil ni tour : codex-cartae-normal expose2% consommés
sur10080minutes au10novembre2026 à22:14:12UTC, soit98% disponibles. Cette lecture
de qualification ne modifie pas le cache de Bridget. Le changement de fait
dans le pilote conservé est vérifié par les scénarios complets de transport.

Parix confirme dans t3-394bcd7c-8b6a-4f42-9d7b-9f582cac1cc4 : les deux
devantures sont déjà actives et vérifiées sur Mac, version272b376, après la
reprise directe demandée par François. Main Parix propre et poussé selon son
reçu. Aucune nouvelle admission ni effet métier rejoué. L'ancienne mission
137451d0-90b5-4489-8fb4-29c2dec23d55 reste close : sa garde expirée n'a pas été
réouverte. Le résultat, les fichiers préservés et l'autorité métier restent
chez son parent.

Publicationv0.8.27 : six actifs téléchargés anonymement, SHA256 conformes.
APK8027 signé, lignée et certificat de retour vérifiés ; aucune interface
Android modifiée. Pas de diaporama requis pour cette correction technique.

Entretien natif après installation :3,97Gio d'intermédiaires Mac retirés ; les
caches protégés sont conservés par le script, sans suppression forcée. Version
active261 et retour259 conservés sur chaque hôte. Le staging de sources de
compilation est reproductible depuis le commit et retiré après contrôle des
usages. Aucune branche ni aucun worktree créés pour261.

Les preuves privées sont conservées sous :
/Users/moi/.cache/bridget261-quota-reprise
Publication : https://github.com/guthubrx/bridget/releases/tag/v0.8.27
