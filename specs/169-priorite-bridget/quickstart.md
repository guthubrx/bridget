# Vérification et activation SPEC169

Ce guide décrit les contrôles à faire après implémentation. Aucune commande ci-dessous n'a été exécutée par le responsable documentaire.

Plan et procédure de retour arrière : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/plan.md

Contrat du mode et de la continuité : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/contracts/delegation-preference.md

## Vérifier le comportement ciblé

Depuis le worktree T3 confirmé par ROOT, utiliser les bancs existants et compléter leurs cas utiles. Exemple pour la cible de développement prévue :

```sh
cd /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget
vp test run /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/serverSettings.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-core/src/server/orchestrationInstructions.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/ProviderSessionManager.test.ts
vp test run /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/CodexDeveloperInstructions.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/Drivers/CodexMcp.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/bridget/BridgetLineage149.test.ts
```

Les tests d'adaptateurs ACP/OpenCode/Cursor/Pi complètent ce socle. Prouver les transitions réelles, le champ par tour sans MCP et vers legacy, la stabilité d'identité et la conservation byte-identique d'un fichier invalide après refus d'écriture. Claude doit transmettre `snapshot: false` et attendre son travail de fond sans employer un mode périmé. Pi exige reçu privé du bon mode corrélé à la tentative ET ACK avant le premier vrai prompt, même OFF ; l'ACK seul ou un ancien reçu ne suffit pas. Aucun fournisseur supplémentaire n'est lancé pour obtenir une preuve.

Pi est limité aux contrats et fixtures : son binaire est hors PATH. Ne pas annoncer de recette réelle Pi ni installer ce fournisseur pour ce lot.

Faire le lint des seuls fichiers changés et les types des paquets affectés selon /Users/moi/11.Repositories/t3code-local/AGENTS.md. Ne pas lancer de contrôle global du dépôt. Conserver les sorties exactes et les limites, sans annoncer de succès global.

## Régler la préférence

Relever le chemin absolu réellement résolu par `ServerSettingsService` dans l'environnement cible. Modifier ce fichier existant en conservant ses autres valeurs. Le champ ajouté est :

```json
{ "preferBridgetDelegation": true }
```

Cet extrait n'est pas un fichier complet à substituer aux réglages. `false`, ou une clé absente dans un fichier valide, donne OFF. Un document invalide ou une lecture en erreur donne un diagnostic et `blocked`, jamais un OFF prétendument lu. Le changement prend effet dans les nouvelles consignes avant le tour suivant.

Avec ON, Codex et Claude demandent l'outil automatique Bridget s'il est réellement exposé. ACP, OpenCode, Cursor et Pi annoncent son absence de montage. Les droits existants restent applicables et un refus ne lance pas T3 en secours. OFF restitue les consignes habituelles. Les tâches déjà lancées gardent leur moteur.

## Installer puis activer

ROOT relève l'état réel et prépare la sauvegarde de l'application et des réglages. Il construit le paquet signé `0.0.45-local.169` depuis la source intégrée et en vérifie la provenance. Il installe puis active T3 par sa procédure normale. L'utilisateur a déjà autorisé ces actions ; aucune nouvelle confirmation n'est attendue.

Ne pas utiliser `launchctl submit` ou créer une relance persistante. Préserver les bases et les conversations. Ne pas confondre paquet installé et code chargé. La procédure complète et le retour arrière figurent dans le plan.

## Relever le résultat réel

Le reçu de livraison doit contenir les chemins absolus observés, les sources et empreintes utiles, les identités de processus avant/après, la preuve du code chargé et du réglage ON. Il doit aussi relever les nouveaux contextes, une reprise de conversation et le suivi des tâches des deux moteurs.

Une sous-tâche depuis un raccord existant vérifié doit obtenir une tâche Bridget unique et son résultat corrélé. Une recette isolée doit porter cette mention ; elle ne remplace pas la preuve d'activation de la cible. Aucun secret de configuration ou de credential ne figure dans les reçus. Le pipeline reste ouvert tant que ces preuves manquent.

## Limites après contre-revue R2

OFF conserve la politique habituelle. Sans MCP, une enveloppe de reset est ajoutée même au premier tour inconnu ; elle n'est plus identique à l'historique. Ordinal1 ne garantit pas une conversation native neuve. Codex reçoit t3_code_orchestration et t3_code_runtime, puis déduplique le contexte identique. Seule la constante OFF avec MCP,6 986 octets, est prouvée byte-identique. Coût de contexte restant non mesuré.

Une query Claude blocked peut le rester jusqu'à réouverture ou changement de clé/processus. Les deux qualifications SDK/CLI couvrent des sorties normales sans prompt ; ni processus figé ni reprise existante positive ne sont qualifiés. C1 n'est pas confirmé dans le SDK installé ; la lecture ne prouve pas une absence physique absolue de processus figé. Visibilité UI du message de divergence non prouvée. P2 et J1 restent clos ; audit source pass, J2 partiel et FR017/SC003 réel/SC009/J3–J6 ouverts. Cette note ne demande aucune nouvelle campagne.

## Checkpoint documentaire après clôture J1

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.
