# Correctif durable de l’identité — complément 149

Date : 2026-10-11. Statut : sources validées, intégrées et poussées ; activation en préparation.

Cause observée : Codex et Claude excluaient toute configuration Bridget existante. Le MCP Codex de ce fil ne recevait donc aucune preuve privée de session.
Le montage officiel reçoit maintenant une preuve fraîche à chaque création/reprise. Les configurations désactivées, étrangères et les identités natives explicites restent protégées.
Le chemin réel du binaire et son environnement canonique sont vérifiés. Claude exige un catalogue connecté de source `dynamic`.
Cause distincte : le pont rejetait les `%` présents dans les identifiants opaques des sous-agents T3. Chaque identifiant est maintenant encodé en un segment HTTP, sans prédécodage.

La revue Sonnet R1 a relevé deux problèmes de montage, corrigés. La revue R2 approuve le code final sans finding bloquant. Les derniers changements sont uniquement de format.
Contrôle final de format et `git diff --check` réussis après ces changements.
Tests ciblés : T3 124 réussis / 1 ignoré ; Rust 25 réussis / 1 ignoré. Les nouveaux tests échouent sur les anciennes sources.
Le contrôle TypeScript global reste en échec sur les diagnostics préexistants consignés dans le rapport ; aucun succès global n’est déclaré.
Sources Bridget : `90e1fab0f4fa7cbc2eb1628018f7e149159393f5`, sur `github/main`.
Sources T3 : `d833387203`, sur `fork/local/main-20261009`.
Les résultats des tests et leurs limites sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/identite-durable-tests149.md.
Le paquet privé et ses preuves sont préparés dans /Users/moi/.cache/t3-identite149-final.OBXQrQ.
Sauvegarde de l’application et du binaire avant installation : /Users/moi/.cache/bridget-identite149-rollback.af6k7sa_. Aucune base modifiée.

Activation : le serveur T3 chargé ne recharge pas son JavaScript après remplacement des fichiers. Une relance T3 reste nécessaire.
La commande native `provider-session.detach` ne suffit pas à actualiser le code du serveur.
Aucune preuve de réussite depuis le MCP de ce fil n’est déclarée avant activation et délégation native réelle.

Après activation du pont `90e1fab0f4fa`, une troisième cause a été observée : des identifiants de 341 et 343 octets étaient utilisés comme noms de fichiers d’état, puis échouaient avec `File name too long`.
Cela bloquait la sauvegarde avant remise et réponse. Le correctif utilise un nom de 41 octets dans un sous-dossier privé et reprend l’ancien état sans le supprimer.
Sources du complément : `9f2ccbf2`, intégrées et poussées `github/main`. Tests du pont : 85 réussis, dont 5 nouveaux tests de reprise ; revue APPROVE.
Rapport : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/identite-fichiers149-tests.md.
Le dernier binaire `9f2ccbf23195` est construit, signé, installé et chargé : daemon PID30562 né le 2026-10-11 à 02:17:41, pont PID30974 né à 02:17:48 (Europe/Paris). Le serveur T3 PID60228 reste ancien.
SHA256 installé : `68c3117ad806a1988eaa49bb39936cc09121156cd39e31e1de24dbaa6157de87`. Reçu : /Users/moi/.cache/bridget-identite149-etats.ABbWhO/package/install-receipt.json.
Observation après relance : 121 fichiers d’état compacts, noms de 41 octets, fichiers privés 0600 et dossier 0700. Aucun `File name too long` dans les nouveaux segments des journaux capturés. Les anciens états sont conservés.
Le code T3 reste inchangé depuis `d833387203`. D’autres tours et sous-agents sont encore actifs ; leur fin est attendue conformément au choix utilisateur.
Un retour à l’ancien binaire Bridget ignorerait les états compacts et pourrait rejouer une remise. Conserver ce nouveau binaire même en cas de retour à l’ancienne application T3.
Le paquet T3 final signé correspond à `d833387203e113ec2a24f947ba8b8064a1c2542b` : ASAR SHA256 `9d9c451143c739ad8cf114927cedce6ab7cbbd751e6ff5efc8d7f12f5709575c`.
La relance reste soumise à l’absence d’autres travaux actifs, ou à leur interruption explicitement autorisée.
