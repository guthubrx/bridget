# Livraison212 — catalogue de flotte et profils SYS/KS

## Cause et correction

Le signalement monuments utilisait machine_id=sys, absent du registre. Les identifiants déclarés sont sysle2 et ksstore.
Le catalogue providers est construit sur la machine du parent. Ses refus command_missing et unsupported_capability ne vérifiaient pas les programmes de SYS/KS ; leur présentation laissait croire le contraire.
La version212 conserve ces verdicts locaux et indique providers_scope, execution_scope et machine_id. Elle ajoute fleet.options depuis le catalogue public de routage existant. Ces options sont seulement déclarées et activées : eligibility/auth restent not_checked et entitlement reste unknown. Les politiques, droits et faits dynamiques sont vérifiés à l’admission. Aucun nouveau droit, aucune sonde fournisseur dans la lecture.
Un catalogue absent ou désactivé conserve un code fermé et un conseil cohérent. Aucun chemin privé ni texte d’erreur interne n’est ajouté à la projection.

## Configuration native

Les deux serveurs ont Claude Code2.1.310 et Codex0.164.0, paquets complets, comptes configurés et bac à sable disponible. Cela ne prouve pas l’accès fournisseur.
Nouveaux profils et connexions : claude-sysle2-native et claude-ksstore-native, modèles claude-opus-5-5 et claude-sonnet-5-5.
Les anciennes définitions Claude/Sonnet et Codex restent identiques. Les profils nouveaux utilisent les comptes déjà présents sur leur serveur ; aucune clé, aucun compte et aucun dossier d’authentification copiés.
Autorités : /home/moi/.cache/bridget-core/agents.json sur Cartae ; /home/moi/.config/bridget/agents.json sur SYS et KS ; /Users/moi/.cache/bridget-core/origin-agents177.json sur le Mac.
Sauvegarde privée, verrou project-preparation.lock et remplacement atomique. Les empreintes des anciennes définitions, les mappings de projet, les credentials workers et les révisions de provisionnement sont conservés.
Les modifications privées persistantes ne sont pas un nouveau défaut universel de configuration pour toute installation vierge. La correction de présentation, elle, est dans le binaire et les sources communes.

## Qualifications réelles

Tâches de contrôle distinctes, en worktree et development, sans modification ni opération métier :
- SYS/Opus : 6d4d4036-0603-44eb-b420-b3ec4b4d2aa5, résultat BRIDGET212_SYS_OPUS_OK.
- SYS/Sonnet : 5dbfedf1-cbfc-4790-b30a-90ba29f50419, résultat BRIDGET212_SYS_SONNET_OK.
- KS/Opus : b2f81c9b-ff30-4d3d-9119-c59f5f76c901, résultat BRIDGET212_KS_OPUS_OK.
- KS/Sonnet : c0ef3646-ae2d-4bfc-9119-0b340fafdae8, résultat BRIDGET212_KS_SONNET_OK.

Les résultats ont été lus, acquittés et clôturés nativement. cleanup_done=true et guard_released=true relus côté maître pour ces quatre tâches et les deux revues.

Codex SYS reste configuré et déclaré ; sa définition n’a pas changé. Pas de nouveau résultat Codex SYS revendiqué dans cette session.

## Codex KS : limite restante attestée

Une qualification native ebc9eab4-2dc1-4765-a9b3-0c131351ef63 a réellement démarré, puis échoué avec terminal Codex: failed.
Le reçu worker est terminal. La tâche est cancelled, cleanup_done=true et guard_released=true.
Journal fournisseur corrélé : session01a1889b-362c-7ec2-b50c-8d3c54668327, événement task_complete, erreur « workspace routing discovery unauthorized (401) », codex_error_info=other.
Cette preuve ne dit ni quota épuisé, ni programme absent, ni modèle interdit. La cause plus fine du refus401 n’est pas établie.
La connexion codex-ksstore est désactivée dans les trois autorités pendant la résolution, pour ne pas y router automatiquement des travaux.
Une connexion officielle par appareil a été ouverte dans le navigateur système du Mac ; elle a expiré sans validation (retour124). Aucun succès de connexion ni réparation du compte n’est revendiqué. Aucun code d’autorisation ou token n’est conservé dans ces preuves.

Action humaine concrète, sur le même compte destiné à KS :
ssh -t ksstore '/home/moi/.local/bin/codex login --device-auth'

Après succès attesté : vérifier le compte et le refus fournisseur, remettre la connexion native en service, puis créer une seule qualification distincte. Ne pas rejouer la mission de contrôle échouée ni remplacer un chantier métier d’issue inconnue. Les travaux Claude qualifiés continuent indépendamment.

## Tests et revues

Cinq tests ciblés distincts PASS : trois spec212, native148_glm_catalogue et routing168_catalogue_12. Les trois spec212 ont été relancés sur le complément final ; ne pas les compter deux fois.
Guides : 32 pages, 522 liens et13 paires de langues PASS ; git diff --check PASS.
Revue f3df2da3-f455-4904-827d-1b951f9daad0 favorable, sansP1/P2, troisP3 corrigés.
Contre-revue 16ac9a02-6fa5-4eef-b7cd-89a910a02ee0 favorable ; les troisP3 sont levés. Revues statiques uniquement, aucun PASS indépendant inventé.
Source fonctionnelle initiale5143c278d2d09f81fc71e4f3404a70d7d86597d7 puis complément19e0b5c32696eda0c98b624a4b5a848d3544692c, intégrés à main et poussés.

## Activation

Build final : 212-fleet-19e0b5c3. Version commerciale0.6.3 inchangée, aucun nouveau tag public.
Linux : /home/moi/.local/lib/bridget-communication/212-fleet-19e0b5c3/bin/bridget
SHA256 a4f83bf41a16f95d42db4edf80a99555419cc233bd42f693cc9262eee3597ebb.
Mac : /Users/moi/.local/lib/bridget-communication/212-fleet-19e0b5c3/bin/bridget
SHA256 f8bc59154f360b9c2d66f439e67d5936d31bd0c8ab2e8935847b61080580b7e5.

Images vérifiées : maîtreCartae3051178, workerCartae3052216, mobile3051284, workerSYS371254, workerKS656408 ; Mac worker13221 et pont13277. Les updaters sont des processus Python, pas des exécutables Bridget anciens.
Sauvegardes cohérentes puis arrêts natifs et rechargements. Première coupure maître1,93s ; complément2,92s. Les quatre wrappers Cartae observés sont préservés, dont l’ancien2500061. Aucun service/calcul Opus2D/NE02 touché et aucun chantier existant relancé.
Le lien natif revient après réinscription du pont ; bridget_session puis capabilities ont réussi avec providers_scope=origin_machine, fleet.status=declared,41options et delegations_halted=false.

La reprise native unique du fil4ffa692b-bf9f-4dc2-884e-ed57b74e0565 a été exécutée. Elle a conservé le processus fournisseur Codex63300 et son client MCP35474. ps et lsof attestent encore /Users/moi/.local/lib/bridget-communication/211-directory-650e5ef5/bin/bridget ; la description d’outil reste211. Le simple redémarrage du tour ne recharge donc pas ce client. Aucune seconde reprise ni terminaison aveugle n’a été faite.
Après cette reprise, UNE bridget_session a réussi et UNE bridget_capabilities a réussi : providers_scope=origin_machine/monordinateur, fleet.status=declared,41options, delegations_halted=false. Le lien et les données du catalogue212 sont disponibles ; cela n’atteste pas le chargement212 du client MCP.
T3 possède le cycle de vie de ces clients. Aucun port de rechargement MCP ou de fermeture réelle de la session fournisseur n’est exposé dans les outils de ce fil. Il reste à recharger réellement le connecteur dans T3, en fermant/réouvrant sa connexion ou la session fournisseur ; relancer seulement le tour est insuffisant dans le cas observé. Aucun nom de bouton non vérifié n’est prescrit. L’exécutable officiel /Users/moi/.local/bin/bridget pointe déjà vers212 ; la configuration installée est correcte. Les anciens exécutables restent conservés tant qu’ils sont utilisés. La livraison des services est activée, mais toutes les sessions clientes ne sont pas déclarées chargées212.

## Entretien et conservations

installed PASS Mac, Cartae et SYS ; mêmes caches réutilisés, sans target par test. Budget observé :4,49Gio Mac,3,68Gio Cartae,0,57Gio SYS.
Six copies binaires de transfert et l’archive de transfert retirées après vérification. Binaires installés actifs, version de retour et anciens exécutables encore utilisés conservés.
Les sept tâches de contrôle/revue sont clôturées nativement. Leurs worktrees ont été vérifiés propres et intégrés, sans fichier ignoré. L’examen des processus non privilégié a relevé des accès non vérifiables :5surSYS,4surKS. Aucun retrait forcé. Sur KS, une lecture privilégiée a confirmé zéro utilisation et zéro accès inconnu : les quatre worktrees ont été retirés par Git sans force. Reçu : /home/moi/.cache/bridget212-r2/cleanup-admin-verified.json. Les trois worktrees SYS restent conservés : cinq processus du même compte ne sont pas entièrement inspectables et aucun accès administrateur non interactif n’est disponible. Les branches légères KS restent conservées car le main du checkout distant est plus ancien ; aucune suppression forcée.
Les références légères dont le checkout principal distant est ancien peuvent rester conservées. Les travaux d’autres livraisons ne sont pas supprimés sur la seule présence de leur nom dans le registre.
