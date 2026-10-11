# Preuve RED des intégrations T004/T048 — session152

Relecture en lecture seule le 2026-10-11 à 14:22:54 UTC. Exécutions Cargo effectuées par le principal. Aucun Cargo, Git ou service lancé par l'owner de ce reçu. Les cinq scénarios ont compilé : **0 PASS, 5 FAIL, 0 ignoré, 5 filtrés ; 5,27 s**.

Journal du principal :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-r2-rouge-e2e.log

SHA256 : cfeb37636dc7dbededd68d4f34ed72c3c47b865912489503079f7f0258b8c11e ; 4373 octets.

Source E2E inchangée depuis la remise du lot :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/native_delegation_e2e.rs

SHA256 : 7368afc3da2aeeb48bb1c7bf084e2fcdf56031051308f89aa65bcaff2527cfe5 ; 55978 octets.

Les cinq fixtures ont une observation initiale started : fournisseur closed_fixture, modèle glm5.3, effort high, t3_present=false. Leur bootstrap contient l'instance de l'enfant et l'identifiant de mission attendu. Chacune a ensuite enregistré le prompt bloqué et blocked152. Le stimulus de panne suit donc la préparation native réelle.

| Scénario exact | Racine privée | Résultat observé |
|---|---|---|
| native152_rouge_identite_frontieres_lien_absent | /private/tmp/n148-c74da859b8 | Refus de démarrage : native_restart_identity_mismatch. |
| native152_rouge_identite_frontieres_flotte_neuve_tache_ancienne | /private/tmp/n148-42765de1c4 | Refus de démarrage : native_restart_identity_mismatch. |
| native152_rouge_identite_frontieres_marqueur_lisible_different | /private/tmp/n148-d1e4146693 | Refus de démarrage au second contrôle du marqueur. |
| native152_rouge_identite_frontieres_marqueur_deja_reconcilie | /private/tmp/n148-9c352110f7 | Le daemon redémarre ; la tâche devient failed/unreachable. |
| native152_rouge_panne_mission_working | /private/tmp/n148-b2a9cbf6a2 | Le daemon redémarre et réconcilie l'ancien groupe ; la tâche devient failed/unreachable. |

## Trois refus de démarrage

Les trois journaux initiaux daemon-1.log attestent une écoute du socket, une connexion du parent et l'enregistrement de l'enfant avant le crash. Les journaux suivants sont copiés intégralement. L'erreur figure chaque fois en ligne 3.

/private/tmp/n148-c74da859b8/daemon-2.log

~~~text
[2026-10-11T14:20:07Z INFO  bridget_daemon::service_limits] service RLIMIT_NOFILE: soft=1048575, hard=9223372036854775807, budget=4096
[BRIDGET] PID file écrit avec verrou exclusif: /private/tmp/n148-c74da859b8/state/bridget.pid (PID 34837)
daemon error: native_restart_identity_mismatch
~~~

SHA256 : c9dfb8a424a82d08846ef3d8a442b822e8db4fa09281d347e82ac0f0f7890a4b ; 289 octets.

/private/tmp/n148-42765de1c4/daemon-2.log

~~~text
[2026-10-11T14:20:07Z INFO  bridget_daemon::service_limits] service RLIMIT_NOFILE: soft=1048575, hard=9223372036854775807, budget=4096
[BRIDGET] PID file écrit avec verrou exclusif: /private/tmp/n148-42765de1c4/state/bridget.pid (PID 34842)
daemon error: native_restart_identity_mismatch
~~~

SHA256 : 8f4b70bfacdba95f3d05d43f174de4dae91a5fe0aeba857d1f77172d43a79113 ; 289 octets.

La tâche conserve l'instance 9e8a8ea9-ca19-46fb-b7cd-f182e6616770. Le marqueur lisible contient l'instance neuve a4fe725c-3d14-4bcc-a99f-174d7a5f595b, la commande boundary152-ac5ee1b9-ff80-4ece-9333-74d0efe459db et la génération 2. Cela correspond à la frontière « flotte neuve, tâche ancienne » préparée par les API publiques.

/private/tmp/n148-d1e4146693/daemon-2.log

~~~text
[2026-10-11T14:20:07Z INFO  bridget_daemon::service_limits] service RLIMIT_NOFILE: soft=1048575, hard=9223372036854775807, budget=4096
[BRIDGET] PID file écrit avec verrou exclusif: /private/tmp/n148-d1e4146693/state/bridget.pid (PID 34836)
daemon error: statut bootstrap invalide: identité native du marqueur incohérente pour 9ca530ec-fbcf-4a9d-85ad-0f7dc5d33a10
~~~

SHA256 : 792d2853119847e626ed299070aca3dba340ad4dfbef097cfd942bf198070d69 ; 367 octets.

Ce troisième journal ne porte pas littéralement native_restart_identity_mismatch. Il prouve le second refus fatal prévu par T048/T043 dans le contrôle du marqueur. La tâche et le fournisseur initial portent l'instance 24127139-12e8-4cac-9449-a8cf01dac26d. Le marqueur lisible porte l'instance différente 1de0c631-a494-40ef-93da-9be49f87b9a8. Sa commande et sa génération restent celles de la tâche. Ce refus vient du moteur ; il ne s'agit pas seulement d'une absence de socket constatée par le harnais.

## Deux échecs de tâche

Lecture des payloads SQLite avec mode=ro, sans mutation :

| Fixture | Tâche | État | Erreur | restart_resumes |
|---|---|---|---|---|
| /private/tmp/n148-9c352110f7 | e4ceb963-4ff5-480c-928d-e38ef66c1f20 | failed | unreachable | 0 |
| /private/tmp/n148-b2a9cbf6a2 | cde850d3-54f2-45d9-88a0-22b2eea886a9 | failed | unreachable | 0 |

Le premier daemon repris écoute son socket puis s'arrête proprement sur le SIGTERM du harnais, PID 34838.

/private/tmp/n148-9c352110f7/daemon-2.log

SHA256 : fd07d303100e97139b5eec261ed4e422a11cf0c7fc59c61948cf375d94d94bf7 ; 1093 octets.

Le second daemon repris, PID 34922, confirme la réconciliation avant l'écoute :

~~~text
[2026-10-11T14:20:09Z INFO  bridget_daemon::daemon] réconciliation: 1 ancien(s) groupe(s) terminé(s): eaa00d49-70ec-4039-9e0d-73bd4eb83949
~~~

/private/tmp/n148-b2a9cbf6a2/daemon-2.log

SHA256 : 952fc9356e3256710a6de87cbc95756fecb98b39a29ed62ae3023f840760de80 ; 1234 octets.

Les deux erreurs unreachable correspondent aussi aux réponses de statut copiées dans le journal Cargo. Les lignes du helper d'arrêt identifient l'exécutable privé /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/bridget, avec le parent de test 34481.

## Processus et limites

Observation /bin/ps -ww -axo pid=,ppid=,pgid=,command= à **14:22:54 UTC**. Filtrage sur les cinq chemins privés, les cinq noms d'enfants, les PID fournisseurs et les PGID connus : **0 ligne correspondante**. Contrôle complémentaire de l'exécutable /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/bridget : **0 ligne**. Aucun processus de ces cinq fixtures n'était encore présent. Aucun signal manuel n'a été envoyé.

Les trois fixtures dont le daemon refuse le démarrage gardent leurs marqueurs et une tâche working sur disque. La présence du marqueur ne prouve pas la présence du processus. Les deux autres tâches gardent cleanup_done=false dans le payload observé. Ce champ ne change pas le constat de processus absent.

Le scénario principal échoue au premier redémarrage. Son second redémarrage, l'admission du petit-enfant après reprise et sa propre reprise restent à exécuter lors du GREEN. Aucun succès de ces étapes n'est déduit. T004(a) Starting, les refus avant reprise et T048 « instance de tâche neuve, flotte ancienne » appartiennent aux tests internes de facade151. Les marqueurs illisibles restent hors FR-029.

Les sources E2E restent inchangées jusqu'au premier GREEN du principal. Ce reçu archive les preuves accessibles au moment de la lecture. Les répertoires temporaires peuvent disparaître après cette phase.

## Première tranche US3 : diagnostic réel puis adaptation de fixture

Cette section suit la tranche du principal du 2026-10-11, **2 PASS, 3 FAIL, 0 ignoré ; 18,56 s**. Elle conserve la preuve RED R2 ci-dessus.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r1-native152_rouge.log

SHA256 : df981e55e2a33eca6cb31e120e3e57bf491951229a382de335f3cdf4ee972adb ; 3309 octets.

Les sources N/D/fleet/E2E ont été figées pendant le diagnostic. Lectures privées effectuées à 15:04:25, 15:05:09 et 15:06:43 UTC. Aucune source modifiée avant le GO du principal. Les lectures SQLite ont utilisé mode=ro.

| Échec R1 | Donnée observée |
|---|---|
| panne_mission_working | /private/tmp/n148-8c139604cf : le payload working/compteur1 porte encore 39c7c34e-0f8c-40b6-9d03-691ec498b64b. L'événement started réel suivant porte 85e3572a-000b-4cc2-aad1-dbbc921af506, avec le même mission_id 1df7979b-abba-441d-b898-1a455a49e962. Le nouveau lien est encore reserved lors de l'arrêt provoqué par la panique. |
| flotte_neuve_tache_ancienne | /private/tmp/n148-4af8c04beb : l'oracle attend started pour la génération synthétique 2, instance 6fb88279-f658-4de0-947d-debf1bad8004. Evidence contient seulement started pour g1=54b1d828-0371-42bd-b9dc-b35f770d8d08 et g3=c6562354-93e7-4cf4-9530-f8420edb8ed8. Le payload final porte g3, working/compteur1. La flotte est Running, commande recovery-ff225152-dafd-4bbb-9715-30bcd596f679, génération3, lien natif open avec le bon delegation_id cee56c5b-4e36-48b9-a289-18547bcb95e7. |
| marqueur_deja_reconcilie | /private/tmp/n148-40107f7ecc : le test échoue sur group_exists(45213) après le retour du helper. Le marqueur possède birth=1791730818361458. Le helper pouvait sauter l'attente quand le leader disparaissait avant ses descendants. À 15:05:09, proc_pidinfo(45213, PROC_PIDTBSDINFO) rend 0 ; le leader est absent et aucun processus de cette fixture n'est observé. |

Les journaux de g3 confirment le daemon démarré pour les deux premiers cas. Le refus de ces assertions ne démontre donc pas un nouveau défaut d'identité du moteur. L'état working étant durable, il ne constitue plus une barrière de reconnexion.

Empreintes des deux journaux repris :

- /private/tmp/n148-8c139604cf/daemon-2.log : 4be44548cc10be5f4461ea866b8a3d8e0af8f2e9c37e74d57ce35af9f937dadb.
- /private/tmp/n148-4af8c04beb/daemon-2.log : 378e748d1dba2b4fa55d6afaa92fea463d6e7b16f0419e4e3b123bb215c640a5.

Contrôle des processus : à 15:05:09, aucune ligne ps liée aux trois racines, aux noms d'enfants, aux PID fournisseurs ou au groupe 45213. À 15:06:43, aucune ligne pour /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/bridget. Aucun signal manuel ni cleanup supplémentaire effectué.

GO explicite du principal reçu après ces données. Adaptation limitée à la fixture E2E existante :

- wait_native_ready152 relit le payload jusqu'à working, une instance différente de la précédente et un événement started réel concordant sur l'instance et le nom d'enfant. La borne de 15 s et le pas de 20 ms reprennent ceux de l'attente de preuve existante. La génération synthétique sans fournisseur ne peut satisfaire la barrière. La même barrière couvre le parent, le petit-enfant et les frontières réparables.
- await_native_group_gone conserve la borne existante de 6 s et le pas de 20 ms. Si le leader est absent, le helper attend seulement la fin du groupe. Il n'envoie aucun signal sans naissance attestée.

Les cinq scénarios sont conservés. Les oracles bootstrap/instance, lien attesté/role/delegation_id, mission stable, deux crashes et petit-enfant sont conservés. Aucun nouveau scénario ni helper de production.

Source E2E après adaptation :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/native_delegation_e2e.rs

SHA256 : b3a90d92e3d565d5e5cba558c46d123597f49d3d6bd672de58b5a329f9ee16c3 ; 57512 octets. rustfmt --check RC0 ; syntaxe Python du fournisseur adaptée validée par AST. Aucun Cargo lancé par l'owner ; le prochain résultat reste à observer par le principal.

Limite supplémentaire constatée : les trois exécutions originales sont starting dans SQLite, avec agent_instance_id NULL, révision0 et génération1. Le fournisseur fermé bloqué ne prouve pas à lui seul une exécution running. Ce fait doit rester visible si la prochaine tranche atteint la remise de reprise. Aucun succès du second crash ou du petit-enfant n'est déduit de cette première tranche.

## US3 R2 : contrat CLI de la fixture de reprise

Tranche du principal : **4 PASS, 1 FAIL, 0 ignoré ; 20,96 s**. Les quatre frontières passent. Le cas panne_mission_working échoue avant toute admission du petit-enfant.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r2-native152_rouge.log

SHA256 : 92462cfcbe1271c09b95c36d9c57383bf571d4cf9d8dcae5bc02c33f12ca5156 ; 2846 octets.

La fixture conservée se trouve dans /private/tmp/n148-d5b7866569. Le fournisseur initial a reçu le prompt bloqué. Le wrapper repris atteste l'instance 3bc90d3a-56d5-4af4-949d-de015e47e530 et la mission inchangée 66de243e-6885-4bc6-8a95-d90be50ca676. Aucun prompt repris ni événement nested152 ne figure dans l'évidence.

/private/tmp/n148-d5b7866569/state/managed-stderr/e6e979a9-f49c-474a-91ad-8061e6b407f2/3bc90d3a-56d5-4af4-949d-de015e47e530-g2/stderr.log

SHA256 : f49ced3bfbbb9026c801f9cbb31085b6a86ea069a332a19d53d1540437e115e2 ; 180 octets.

~~~text
unknown option --resume
usage: /Applications/Xcode.app/Contents/Developer/usr/bin/python3 [option] ... [-c cmd | -m mod | file | -] [arg] ...
Try `python -h' for more information.
~~~

Le registre de cette tranche désigne /usr/bin/python3 avec les chemins du script et de l'évidence comme arguments. Le transport injecte --resume/session avant ces arguments. Cette position soumet donc --resume à Python, avant le script. Les arguments de ce processus déjà terminé n'ont pas été capturés par le système. Leur ordre est reconstruit depuis le registre conservé et le constructeur relu, puis confirmé indépendamment par le principal et facade151. Le reader de reprise attend aussi une première trame avant tout prompt.

GO explicite reçu pour adapter uniquement la fixture152 existante. Elle désigne maintenant son script privé comme programme, avec args=[] avant le premier démarrage. Le script possède le mode0700 et le shebang /usr/bin/python3 en première ligne. Il lit les arguments fournisseur dans sys.argv[1:], trouve evidence.jsonl depuis __file__, émet immédiatement system/init avec la session reçue sur --resume et conserve l'init initial après le premier prompt. started.argv devient une preuve des arguments réellement reçus. Les fixtures148/151 et la production restent inchangées.

Source E2E STABLE après cette adaptation :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/native_delegation_e2e.rs

SHA256 : bcaa2e017945269d98b2f63914e612fc6bed80cedb3b5ca7130d9f56e9af8a6c ; 58891 octets.

Les cinq scénarios et tous leurs oracles sont conservés. Les bornes restent : readiness15s/pas20ms, preuve15s, RPC8s et disparition de groupe6s/pas20ms. Au lot STABLE, rustfmt --check RC0 et syntaxe AST Python validée, avec shebang initial et init avant la lecture stdin. Aucun Cargo lancé par l'owner.

## US3 R3 : fournisseur prêt, mission non remise après quarantaine

Nouvelle lecture factuelle après adaptation, sans reprendre l'erreur Python précédente : **4 PASS, 1 FAIL, 0 ignoré ; 20,85 s**. Les quatre frontières passent.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r3-e2e-native152_rouge.log

SHA256 : 7a707deb2aca0768a05f1c401a9fd3ae336c039d426d1979f073d577e418b72b ; 2841 octets.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r3-e2e-results.json

SHA256 : 3dedfa1e0343fd1d69557c29e7764138c39605c36303896ecf7ee837bd72dee3 ; 1828 octets. Le contrôle du principal confirme les empreintes sources stables.

Racine conservée : /private/tmp/n148-6b23b800e8. Mission da40ea67-66bd-4f9e-96b9-6a9bbf8bd5d7, tâche935ca374-bc80-4893-8871-71e9b2a25e0c, enfant b0ab2c88-4b6a-463f-bda3-767f10d789e6. Le fournisseur g2 PID3810 porte l'instance d0f0af8f-29da-40dc-93d2-b87b7aae8688.

/private/tmp/n148-6b23b800e8/evidence.jsonl

SHA256 : 5fb0b4542f6b57f18a2f2872a87ae929d098e112a2cf6a50e1bfa322bfd89174 ; 1358 octets.

started.argv enregistre réellement --resume, native148-fixture, puis les arguments attendus --restricted, --permission-mode plan, --model glm5.3, --effort high et les flags stream-json. Le bootstrap porte la même instance g2 et la mission inchangée. L'évidence ne contient aucun prompt g2, nested152 ou blocked152 g2.

/private/tmp/n148-6b23b800e8/state/managed-stderr/b0ab2c88-4b6a-463f-bda3-767f10d789e6/d0f0af8f-29da-40dc-93d2-b87b7aae8688-g2/stderr.log

Ce fichier est vide : 0 octet ; SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855.

La session fournisseur est sauvegardée dans /private/tmp/n148-6b23b800e8/state/sessions/b0ab2c88-4b6a-463f-bda3-767f10d789e6/claude_provider_session. Elle contient native148-fixture. Sa création et son écriture atomiques datent de15:21:38.921449UTC, après startedg2 à15:21:38.875204UTC. Le fournisseur privé émet init avant de lire stdin ; le reader du transport sauvegarde cet init. Cette concordance prouve l'init consommé. Aucun flux brut stdin/stdout n'a été archivé.

Le journal /private/tmp/n148-6b23b800e8/state/sessions/b0ab2c88-4b6a-463f-bda3-767f10d789e6/2026-10-11.jsonl contient seulement les quatre événements g1 : turn_start, prompt_dispatched, interrupt_requested et turn_end cancelled. La branche MCP de la fixture exige un prompt NESTED_152 ; sans prompt g2, aucune admission MCP n'a été tentée.

La lecture SQLite mode=ro constate : tâcheworking/restart_resumes1/instanceg2 ; commande recovery-d52b518f-1e9f-4934-8963-eebac76279be connectée/génération2 ; lien084fe3eb-94cb-4ec8-bddf-e4c6a2f95ddb open, native_delegate et delegation_id exact ; remise930dadbe-3b31-40f6-9baa-1634746ddffa réaffectée àg2 et phaseindeterminate ; exécution originale starting/génération1/révision0/agent_instance_idNULL ; zéro continuation, réservation de continuation et descendant.

Le contre-diagnostic indépendant de facade151, accepté par le principal, localise le retour inconditionnel après quarantaine dans schedule_execution_recovery. Il coupe avant la reconstruction gouvernée existante. Le principal autorise son owner production à poursuivre uniquement pour la tâche native dont la reprise a déjà été autorisée, après confirmation de quarantaine. La fixture ne reçoit aucun nouveau changement.

À15:24:21UTC, ps ne montre aucun processus de cette racine ni du binaire /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/bridget. Aucun signal manuel envoyé. Source E2E toujours byte-identique au SHA bcaa2e017945269d98b2f63914e612fc6bed80cedb3b5ca7130d9f56e9af8a6c. Ces preuves R3 ne valident ni le second crash ni le petit-enfant.

## US3 R4 : admission réellement tentée, source de permissions indisponible

Après le correctif production du retour après quarantaine, la tranche du principal passe les quatre frontières. Le scénario panne_mission_working reçoit maintenant sa mission dans la génération2 et tente réellement le MCP descendant. Les deux tranches lib de pause ont aussi passé leurs six et cinq cas ; ces résultats restent distincts de l'intégration.

Racine conservée : /private/tmp/n148-e4a1b202ad. Tâche e34600e0-9c60-42f4-a4d4-f65a1588c4b0 ; enfant 25a2dd3f-8789-47e2-ac3b-7438a95993c4 ; instance reprise22443631-8caa-4360-9fa9-ba31ce677af6 ; mission38bf75e3-de98-46f2-94d0-773835a282ea.

/private/tmp/n148-e4a1b202ad/evidence.jsonl

SHA256 : 3822a670fc50ebc5f9bf6f0c9090a8baf6f8758518912285503bbfb03dc585ea ; 1816 octets.

Ce fichier porte started g2/PID21065 avec argv --resume native148-fixture, bootstrap exact, prompt bloqué avec carte de reprise, puis nested152. La réponse réelle est :

~~~json
{"id":1,"jsonrpc":"2.0","result":{"code":"native_delegation_refused","content":[{"text":"permission_source_unavailable","type":"text"}],"isError":true}}
~~~

La lecture SQLite mode=ro montre la nouvelle continuation execution-recovery-3ef2c581-ab06-457f-83a3-968bb19ad3cb, génération2/running/révision1/provider_accepted, rattachée à l'exécution originale. Le lien c1de6d2a-eb0c-4593-b045-e9f2d9acb8f1 est open avec rôle native_delegate, delegation_id exact et instanceg2. La tâche racine reste working/restart_resumes1 avec permission_snapshot=null. Aucun descendant n'a été admis. L'admission initiale utilise le grant humain legacy de la fixture148 ; elle n'a enregistré aucun chemin ni révision de source dans un snapshot.

Le registre privé désigne alors le script /private/tmp/n148-e4a1b202ad/provider.py, args=[], permissions=deny, pass_env=[], sans claude_config_dir. Le script possède un shebang. Le profil /private/tmp/n148-e4a1b202ad/provider/.claude et ses settings.json/remote-settings.json sont réellement absents.

Le helper isolé efface l'environnement puis fixe HOME=/private/tmp/n148-e4a1b202ad/provider et PATH=/usr/bin:/bin:/usr/sbin:/sbin. Le lancement géré préfixe aussi /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug. Aucun programme claude n'existe dans ces cinq répertoires observés.

La capture normale exige le claude direct du PATH, puis l'égalité avec le programme du registre et un programme sans shebang. Ajouter seulement un profil ou un alias vers le script ne satisfait donc pas ce contrat. Le wrapper publie le refus de capture ; parent_fact, après les contrôles de filiation, retrouve ce refus car le snapshot de la tâche est nul. Ce refus de sécurité est correct pour ce montage non attestable.

Sources relues pour cette distinction :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/native_permissions.rs : capture_context_with_env aux lignes74–94 ; observed_fact.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/wrapper.rs : capture avant transport, boucle NativePermissionFact, managed_adapter_environment.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs : parent_fact.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/lifecycle.rs : build_environment.

Identité MCP : les valeurs d'instance et de nom sont enregistrées dans started ; le fichier /private/tmp/n148-e4a1b202ad/state/agent-names/instance-22443631-8caa-4360-9fa9-ba31ce677af6 et sa preuve privée concordent. Le wrapper construit aussi BRIDGET_AGENT_ID_FILE et le namespace privé ; le processus MCP hérite cet environnement. Aucun dump complet des variables n'a été archivé. Les credentials ne sont pas reproduits dans ce reçu.

Le stderr g2 est vide. À15:30:21UTC, ps ne montre aucun processus de cette racine ni du binaire de test. Aucun signal manuel envoyé. Le principal et facade151 confirment indépendamment le défaut de montage de fixture ; aucune garde de permissions n'est supprimée.

## Montage attestable privé : recherche de réutilisation et décision autorisée

Montage149 relu :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/native_permissions149_tests.rs : .git privé, bin/claude Mach-O, PATH privé et profil privé.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs : même montage et capture normale du contexte.

Ces tests unitaires utilisent une copie de /bin/echo. Ils n'exécutent pas de fournisseur stream-json. Ce programme ne peut donc pas porter le même scénario E2E. La recherche ciblée n'a trouvé aucun binaire fixture stream-json attestable existant. Les lanceurs shell/Python ne satisfont pas le contrôle de capture. Le principal et facade151 ont recontrôlé cette limite. Le compilateur C système est déjà requis par rusqlite bundled.

GO explicite du principal, puis confirmation du plan avant édition : reprendre ce montage149 dans la seule branche152 de la fixture, avec un programme temporaire bin/claude compilé par /usr/bin/cc. Le C incorpore les octets du même fournisseur Python. Il lance /usr/bin/python3 -c avec ce texte, transmet les flags sans déplacement et conserve stdin/stdout/environnement. __file__ est fixé au chemin privé du fournisseur pour l'évidence. L'empreinte capturée du programme couvre donc le texte exécuté ; le script sur disque n'est plus son autorité.

Le registre désigne exactement ce programme privé et son profil. Le PATH du daemon contient ce bin privé. .git borne la remontée des sources. Le profil settings.json réel est privé et conserve defaultMode=plan ainsi que les refus Bash/Edit/Write/NotebookEdit/WebFetch/WebSearch. Les artefacts C, programme, script et profil restent dans la racine du scénario. Aucun programme installé ni source de production changé.

L'init précédent ne portait ni permissionMode ni tools. Le reader ne pouvait donc produire un fait de permissions observé. L'init152 rapporte maintenant le mode lu des arguments, l'inventaire fermé Read/Glob/Grep réduit aux --tools reçus et le cwd réel. Le fournisseur vérifie le profil plan, ses six refus et, pour la forme héritée sans --restricted, les arguments --permission-prompts none et --disallowedTools. init152 archive ce fait, sa session, sa configuration et ses arguments. Le wrapper capture et atteste par son chemin normal ; aucun snapshot n'est écrit par la fixture.

Source STABLE annoncée au principal :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/native_delegation_e2e.rs

SHA256 : 298e0f13699815da7bc12c401590c802a708a4a13a4d069823e9d06c3f469b91 ; 63162 octets.

Contrôles statiques : rustfmt --check RC0 ; AST Python embarqué valide ; lecture des formes initiale restricted et héritée, des refus, du mode et de l'inventaire fermé. L'asset d'origine /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/fixtures/native_delegation_148.py reste au SHA256 fb829aa9b875d4735d828d458e1e38c54bf17ad111a5fe46cefab9cb1ba44c1d.

Les cinq scénarios et les oracles bootstrap/lien/mission/deux redémarrages/petit-enfant sont conservés. Les bornes existantes restent inchangées. L'owner n'a exécuté ni CC, Cargo, service ni signal. La compilation du fournisseur aura lieu dans les prochains runs du principal, avant le démarrage du daemon. L'annonce STABLE ne prouve pas encore son exécution ni l'admission du petit-enfant.
