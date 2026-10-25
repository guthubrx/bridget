# Archives techniques de l’adaptateur T3

Document d’origine : [audits/2026-10-15/session-2026-10-15-spec-168-v15-02/scoring.md](../../../../../../../audits/2026-10-15/session-2026-10-15-spec-168-v15-02/scoring.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [fixed] LOW : Branche hors Git devenue inaccessible après le retour anticipé
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/src/lifecycle.rs:476 ; 2be7164bdf838f5bf0a9
  À463–465 outside=true retourne déjà Ok(None). La variable locale ne change plus avant476. Le second test outside && ... ne peut donc jamais rendre Ok(None). Défaut de lisibilité prouvé par lecture, sans défaut runtime ni permissivité Git démontrés. Correction r1 et validation exécutée r1/r2 : Oracle Git local exécuté PASS r1 : SHA exact, base absente, mismatch, Worktree et hors Git. Le helper source_base_commit est réellement appelé ; suppression de branche morte confirmée par revue indépendante. Aucune faute runtime nouvelle n’était alléguée pour ce LOW.
  Action : Remplacer uniquement le return if outside...else de476–481 par return Err("workspace_base_preflight_failed".into());. Réutiliser les oracles Git existants ; ne pas modifier les gardes du préflight ni créer un nouveau banc. Réduction manuelle estimée :5lignes.
- [fixed] MEDIUM : Le budget du petit décideur est renouvelé après sa préparation
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/src/daemon/native_delegation.rs:1795 ; ca1693b4332559b91339
  Source r0 : remaining est capturé avant routing_lightweight_options, qui prend le mutex d’état, relit le registre et exécute launch_preflight. Le thread démarre ensuite le transport qui calcule sa deadline par Instant::now()+options.timeout. Le délai de préparation et le délai de scheduling sont donc ajoutés à l’échéance physique. Si l’échéance globale expire pendant ces étapes, le garde préspawn du transport vérifie une nouvelle échéance et peut encore lancer un enfant. Le receiver reste borné par global_deadline et le worker ne possède pas le store : aucun CAS tardif n’est démontré. Le permis demeure détenu jusqu’au drain. Aucun dépassement runtime n’a été mesuré. La partie transport est du propre auteur de cet audit ; la conclusion de chaîne repose aussi sur le code appelant C. Correction r1 et validation exécutée r1/r2 : 7 oracles transport et l’oracle permis passent r1. Le premier vérifie deadline déjà expirée sans started/seen avant l’invocation valable ; stdin bloqué et drain sont aussi exercés. La même échéance absolue est confirmée statiquement de l’appelant au transport. Aucun RED deadline n’a été exécuté ; aucune borne universelle des préflights synchrones n’est revendiquée.
  Action : Transporter l’Instant global absolu dans ClaudeIsolatedDecisionOptions au lieu d’une Duration préparée. Le vérifier à l’entrée, juste avant spawn et pendant les I/O. Conserver le receiver sur la même échéance et le permis dans le thread jusqu’à arrêt physique. Étendre un oracle existant avec deadline déjà expirée (zéro processus) et une invocation valable/drain. Une Duration recalculée avant spawn du thread ne couvre pas son scheduling.
- [fixed] MEDIUM : La seconde sonde d’effort n’est pas comparée aux faits figés
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/src/daemon/native_delegation.rs:1251 ; ccd70c502f5da262bae1
  La première sonde compare définition, droits, époque et mapping au snapshot1239–1244. Un effort différent du défaut provoque une seconde sonde1251. Son observation est retournée sans comparaison puis remplace chosen_plan.observations[index]1532 et devient la nouvelle référence. Une modification de définition, de session ou de mapping entre les deux sondes peut donc sembler adoptée au choix. Les gardes route/Prepare vérifient ensuite la nouvelle référence. Les droits peuvent légitimement varier avec effort. Lecture de chaîne confirmée aussi par l’owner C : les gardes ultérieures ne restaurent pas la référence initiale. Définition, mapping et époque peuvent changer entre deux sondes fraîches puis être stockés comme référence de choix. Aucun scénario runtime ni exploitation établi. Correction r1 et validation exécutée r1/r2 : Oracle natif exécuté PASS r1 : permission modifiée par effort acceptée, dérives définition/mapping refusées sans Task. La comparaison d’époque est vérifiée dans le code ; aucun cas dynamique d’époque distinct n’est revendiqué.
  Action : Après la seconde sonde, comparer définition brute, mapping et époque à la première observation fraîche déjà validée. Ne pas comparer permission_digest entre deux efforts différents. Même si allow_epoch_change autorise une reprise sur une nouvelle session, l’époque doit rester fixe entre ces deux sondes fraîches. Étendre la fixture existante avec dérive entre sondes et cas stable d’effort final.
- [fixed] MEDIUM : Le premier Prepare distant refuse un Worktree dont la base diffère du HEAD cible
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/src/daemon/native_delegation.rs:4161 ; 5e4326675a7054f0ef50
  Pour un Worktree épinglé sur S présent au worker dont HEAD vaut T≠S, le probe et le Worktree final visent S. Le premier WorkerPrepare conserve base_commit=S mais impose workspace=None. prepare_for prend alors la branche Shared et compare HEAD=T à S, puis refuse avant admission. Le second Prepare Worktree ne peut être atteint. Preuve de chaîne source, sans exécution par cet auditeur. Correction r1 et validation exécutée r1/r2 : Contrat PrepareV2 et inspection/token/refus Start passent r1. Le vrai maître Worktree S≠HEAD échoue d’abord r1 sur le seul corps attendu, qui omettait la note de workspace désormais produite. Seule la fixture est corrigée en r2 : assertions de métadonnées/Git et corps exact, aucune tolérance de suffixe libre. Le rejeu exact r2 passe 1/1 ; une émission et rejeu stable conservés. Aucune source produit n’a changé entre ces deux campagnes.
  Action : Préserver la sémantique Worktree/base_ref/SHA lors du contrôle préalable, sans effet ni état lançable avant Task durable ; revalider normalement avant matérialisation et Start. Étendre un oracle maître+worker au cas S≠HEADworker et aux refus/droits inchangés.
- [fixed] HIGH : Le probe groupe le réseau mais répète les mêmes processus Git pour chaque modèle
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/src/remote_worker.rs:2233 ; 0de45c352ed0e35dc790
  C modèles du même projet/base/workspace appellent chacun mapping_for puis probe_workspace_preconditions. SharedGit :4 processus Git par modèle ; Worktree :13 ou17. Borne128 mais aucune agrégation malgré plan et R14 explicites. Sévérité Article XVIII, sans panne runtime alléguée. Contre-audit indépendant deciders168_finish confirmé ; reçu joint dans counter_audit. Correction r1 et validation exécutée r1/r2 : RED exécuté et conservé : 521 processus Git pour 128 candidats et 2 tuples, borne 40 dépassée. Le même oracle passe r1 après cache par lot ; son assertion prouve ≤40 appels pour ce cas. Aucun compte exact de 17, gain de vitesse, ni benchmark global n’est déduit. Contrôles individuels et Prepare/Start frais restent vérifiés.
  Action : Mémoriser les seuls faits Git positifs/négatifs par clé complète dans le lot de probe ; conserver droits/définition/modèle/effort par candidat et tous les recontrôles frais Prepare/Start. Pas de cache global.
- [refuted] INFO : Alerte de secret réfutée au site1 (idempotency)
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/tests/fixtures/artifacts/generated-provenance-v1.json:2 ; 63322e0650203470d7d1
  Le JSON figé porte idempotency_key dans une publication de fixture. Le consommateur artifact_types.rs563–573 inclut cette fixture et valide ArtifactPublicationV1 ; ce champ déduplique une publication, sans rôle d’authentification.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site2 (test-secret)
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/tests/core_089_service_test.rs:71 ; 67ca12eb2b6169b125e4
  Le littéral est défini par write_greffe_policy dans un test d’intégration. Le site appelant356–363 utilise unique_home ; les helpers26–50 effacent lenvironnement et créent le namespace0700, la politique59–65 est créée0600. Refus de l’hypothèse d’une fuite de configuration active : constante de fixture locale, sans validation fournisseur.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site3 (idempotency)
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/src/cli.rs:11976 ; 9c05ef555289c1e9ef7d
  Le champ est IdempotencyResult.idempotency_key émis par un faux serveur Unix. guidance156_tests est sous cfg(test)11588 ; namespace temporaire privé11829–11860 et enfant env_clear12009–12010. Un identifiant de corrélation ne donne pas un droit de connexion.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site4 (idempotency)
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/tests/held159_e2e.rs:1198 ; 09f91debdf3075439d6a
  La ligne est une requête SELECT sur send_deliveries, dans le scénario de crash ; le message est fabriqué puis transmis par SendIdempotent1124–1147. idempotency_key sélectionne le message de fixture et nest pas utilisé pour authentifier.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site5 (idempotency)
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/tests/held159_e2e.rs:1273 ; b8ed518dfac30a5faa81
  Même scénario : SELECT phase après reprise dun message de fixture SendIdempotent. Le champ idempotency_key est une clé de recherche SQL ; aucune connexion fournisseur fondée sur cette valeur.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site6 (idempotency)
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/tests/held159_e2e.rs:1778 ; 69db0237cf25014a5135
  Le test1727–1744 envoie deux messages synthétiques et lit sa base privée. SELECT COUNT de1778 vérifie la file retenue après relance, pas une authentification.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site7 (placeholder)
  /private/tmp/bridget-audit168-source-20261015/specs/026-operations-greffe-central/contracts/greffe-authorization.example.json:4 ; 5f6f7342950d2e326efc
  Valeur contractuelle non hexadécimale qui demande explicitement un remplacement. Le test greffe_authorization.rs1007–1019 remplace attestation_key avant utilisation dans une politique de fixture.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site8 (placeholder)
  /private/tmp/bridget-audit168-source-20261015/specs/038-regeneration-politique/contracts/greffe-authorization-refresh.example.json:4 ; 12c6683b2d568d341256
  Valeur contractuelle non hexadécimale qui demande explicitement un remplacement. Le test greffe_policy_refresh.rs824–843 remplace attestation_key avant écriture de la politique de fixture.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site9 (test-secret)
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/src/wrapper.rs:8509 ; b01a5c2f10ca45b66912
  Constante dans reconnect_tests gardé cfg(test)8423. Le test8507–8541 linjecte dans OutputRedactionLease et dans une sortie simulée, puis assert!(!line.contains(secret)) prouve lintention de sentinelle de masquage. Aucun chargement de credential actif observé.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site10 (idempotency)
  /private/tmp/bridget-audit168-source-20261015/skills/bridget/references/commandes.md:1413 ; 4038ceeaa07d58c9c456
  Exemple JSON sous la section publication dartefact. arguments.idempotency_key, données tabulaires, sources et justification décrivent une déduplication métier. Aucun usage comme jeton dauthentification dans ce bloc.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site11 (public-protocol-nonce)
  /private/tmp/bridget-audit168-source-20261015/specs/090-codex-interactif/probe_shared_session.py:78 ; ac6b0c22f9aa231ecf44
  Le champ est Sec-WebSocket-Key dans une poignée de main HTTP Upgrade vers socket.AF_UNIX. Décodage local du nonce égal au texte public de lexemple WebSocket ; nonce de protocole, sans preuve ou rôle de secret dauthentification.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site12 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/148-identite-delegation/validation/glm-recette-reelle-r2.md:86 ; 68f603c7881968726c07
  La ligne est une affectation nommée AUTH_COMMAND_SHA256 et sa valeur a exactement la forme64hex. Le contexte de recette92 distingue la véritable variableAPI héritée sans en afficher la valeur ; ce site conserve lempreinte dune commande autorisée.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site13 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:887 ; 4ba02cf0d926387dfde0
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site14 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:37 ; b5f87a0ef8125d1706c4
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site15 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:52 ; fb3c2626f6ddad5163a5
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site16 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:71 ; 75cd00c594e57dbe2a18
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site17 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:226 ; 2d2d106ce3c662bb0100
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site18 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:336 ; a0947a6ffffdf9ebbfb3
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site19 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:870 ; 9d1a881901b54c8caba6
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site20 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:948 ; 8bf7114e09f924bb2fef
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site21 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:1401 ; 7f7a28e8e5f7392420ac
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site22 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:1415 ; 6fc4310ff094835fd217
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site23 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:1429 ; 712fd5ceb22300cb173c
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site24 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1222 ; a891de7a0d6c74468233
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site25 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1223 ; d70f3233f999971aa84d
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site26 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1224 ; fb6ff82b740834a1d2e0
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site27 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1225 ; 2d5b838799282d76a028
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site28 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1311 ; 51bf27525859e7158393
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site29 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:2187 ; e87b54e6494448d231b6
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site30 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1778 ; 8d6ae82ab7506b12ec1f
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site31 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1866 ; 0089ae732d2127fd7be0
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site32 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:20 ; de0d785d2f7bb1664040
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site33 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:150 ; 7d1de31a74ab66cae700
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site34 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:152 ; 4e34bdc46c9175e5e825
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site35 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:153 ; 3b25f98e52b604d689f4
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site36 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:157 ; 0189feb3737c03c4ca2d
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site37 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:187 ; 0845cf1ef966a9fce8cc
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site38 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:188 ; 437302964b2f33f1f855
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site39 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:207 ; 0e1e3f8e981814a76f93
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site40 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2239 ; 3fe3b00bd3958b4b9d06
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site41 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2255 ; dc94bd9a227b8eeaf905
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site42 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2259 ; 61a20f57a11178aa57c3
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site43 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2267 ; ca60cb79ca78ff986df3
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site44 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2291 ; 3bd164ff2587fc5d27df
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site45 (idempotency)
  /private/tmp/bridget-audit168-source-20261015/specs/156-guidage-file/validation/diff-156-after-T032.patch:245 ; adf006c7aa89f3093060
  Le champ est IdempotencyResult.idempotency_key émis par un faux serveur Unix. guidance156_tests est sous cfg(test)11588 ; namespace temporaire privé11829–11860 et enfant env_clear12009–12010. Un identifiant de corrélation ne donne pas un droit de connexion. La ligne historique du patch correspond octet pour octet au siteCLI11976, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site46 (idempotency)
  /private/tmp/bridget-audit168-source-20261015/specs/156-guidage-file/validation/diff-156.patch:245 ; c0cb170201701f1fd0e2
  Le champ est IdempotencyResult.idempotency_key émis par un faux serveur Unix. guidance156_tests est sous cfg(test)11588 ; namespace temporaire privé11829–11860 et enfant env_clear12009–12010. Un identifiant de corrélation ne donne pas un droit de connexion. La ligne historique du patch correspond octet pour octet au siteCLI11976, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site47 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25109 ; d8e6d269fd48fee315d6
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits. Ligne du patch strictement identique à la ligne du reçuJSON associé, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site48 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25239 ; 844261318b9fd71fb5ce
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits. Ligne du patch strictement identique à la ligne du reçuJSON associé, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site49 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25241 ; 744e00c8e24b274be918
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits. Ligne du patch strictement identique à la ligne du reçuJSON associé, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site50 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25242 ; 4432f659291fd2695ca2
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits. Ligne du patch strictement identique à la ligne du reçuJSON associé, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site51 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25246 ; 08ca956e4cdd2536fe61
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits. Ligne du patch strictement identique à la ligne du reçuJSON associé, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site52 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25276 ; e383056a147219d4954d
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits. Ligne du patch strictement identique à la ligne du reçuJSON associé, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site53 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25277 ; 27064ca47096e45b2594
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits. Ligne du patch strictement identique à la ligne du reçuJSON associé, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] INFO : Alerte de secret réfutée au site54 (checksum)
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25296 ; 2f72a42d94e930974fa2
  Entrée64hex dun dictionnaire complet chemin de fichier→SHA-256, vérifié par parsing structurel. Le mot auth/key/token dans le nom du fichier a déclenché le détecteur ; la valeur est une empreinte de fichier et ne sert pas dautorisation. Les artefacts historiques ne sont pas reconstruits. Ligne du patch strictement identique à la ligne du reçuJSON associé, hors préfixe+.
  Action : Aucune rotation ni suppression fondée sur cette alerte. Conserver le contexte et ce verdict ; toute modification de cause demande une nouvelle qualification.
- [refuted] MEDIUM : `Debug` impls for client session values expose TLS secrets
  /private/tmp/bridget-audit168-source-20261015/Cargo.lock:686 ; af32dd76bfb22dbce765
  Version rustls0.21.12 dans la plage affectée, mais fuite de secrets réfutée sur le chemin Bridget lu : minreq construit une configuration privée par défaut, Resumption::default utilise ClientSessionMemoryCache, aucun ClientSessionStore personnalisé ni Debug de valeurs de sessions dans les sources applicatives. L’avis exclut le cache intégré. L’héritage n’est pas le motif de réfutation ; aucun secret réel observé.
  Action : Conserver l’absence de store personnalisé/debug de sessions ; requalifier si ce comportement change. La mise à niveau de la version affectée reste un sujet séparé.
- [refuted] LOW : Unauthenticated ServerHello can downgrade later connections to classical key exchange
  /private/tmp/bridget-audit168-source-20261015/Cargo.lock:686 ; e51780f0350206f381c0
  Version rustls0.21.12 dans la plage affectée de l’avis. L’impact décrit est une perte de confidentialité post-quantique par déclassement de X25519MLKEM768. Le chemin compilable minreq utilise les groupes de rustls0.21.12 : X25519, secp256r1 et secp384r1 uniquement. Aucun groupe hybride/post-quantique n’est supporté par cette configuration. Ce symptôme est donc réfuté sur la configuration source lue, sans déclarer la version corrigée ni exclure le comportement de mémorisation de kx_hint.
  Action : Réexaminer lors d’une mise à niveau ajoutant un groupe hybride ; viser une chaîne rustls corrigée>=0.23.46. Ne pas présenter l’actuel TLS classique comme post-quantique.
- [confirmed] LOW : Name constraints for URI names were incorrectly accepted
  /private/tmp/bridget-audit168-source-20261015/Cargo.lock:698 ; 3561c2c3ea7e25c47dd5
  rustls-webpki0.101.7 est dans la plage affectée. Le client HTTPS minreq passe par WebPkiVerifier puis verify_for_usage : validation des contraintes de noms disponible sur une connexion HTTPS autorisée par decision_service. L’avis primaire mainteneur exige une chaîne correctement signée mais mal émise comportant les contraintes URI concernées ; aucun exploit ni certificat de production observé. Il s’agit d’un risque de dépendance applicable et non d’une faille introduite par168. Cargo.lock identique au HEAD/base ne réfute pas cette version.
  Action : Faire évoluer la chaîne de dépendances TLS vers webpki>=0.103.12 (et hors plage alpha affectée), idéalement>=0.103.13 pour couvrir aussi0104, avec rustls corrigé. minreq2.14.1 contraint webpki0.101 et rustls0.21 : un simple cargo update dans ces contraintes ne suffit pas ; choisir un correctif amont ou une évolution explicitement validée et tester les refus TLS.
- [confirmed] LOW : Name constraints were accepted for certificates asserting a wildcard name
  /private/tmp/bridget-audit168-source-20261015/Cargo.lock:698 ; 04cadddd5ed45eeb4b87
  rustls-webpki0.101.7 est affecté par le contrôle de contraintes DNS pour certificats wildcard. Le chemin HTTPS minreq/WebPkiVerifier peut atteindre la validation de chaîne concernée. L’avis mainteneur exige signature valide et mauvaise émission du certificat ; aucune exploitation réelle alléguée. Risque de dépendance applicable malgré son héritage à HEAD/base. Pas de contournement des contrôles TLS affirmé sans ces prérequis.
  Action : Même évolution TLS que0098 : webpki>=0.103.12, de préférence>=0.103.13, sur une chaîne compatible et corrigée ; couverture des certificats wildcard/contraintes en validation ultérieure. Aucun changement source effectué.
- [refuted] HIGH : Reachable panic in certificate revocation list parsing
  /private/tmp/bridget-audit168-source-20261015/Cargo.lock:698 ; 237e002c90241302276d
  Version rustls-webpki0.101.7 sélectionnée et affectée confirmée. Le corps vulnérable BIT STRING existe. Son atteignabilité applicative est réfutée : minreq construit WebPkiVerifier standard, qui transmet &[] //no CRLs ; aucun chargement CRL ou verifier personnalisé dans Bridget. Le contre-audit indépendant suit aussi la variante bit_string_flags via CrlSign/check_crls, gardée par liste CRL non vide. Aucune réfutation fondée sur héritage, compilation exclue ou HTTP local. Contre-audit indépendant joint.
  Action : Aucun correctif produit indispensable établi pour ce chemin HIGH précis. Maintenir absence de CRL/custom verifier et requalifier si ajout ; migration de dépendance reste nécessaire pour supprimer l’inventaire de versions affectées.
- [refuted] HIGH : rustls-webpki: Denial of service via panic on malformed CRL BIT STRING
  /private/tmp/bridget-audit168-source-20261015/Cargo.lock:698 ; dd16d94946c8ee89b123
  Doublon logique de RUSTSEC-2026-0104/CVE-2026-93599, pas une seconde vulnérabilité. La même version affectée est conservée dans l’inventaire. Atteignabilité Bridget réfutée par liste CRL vide et contre-audit indépendant ; voir0104. Le statut de cet alias résout le signal dupliqué sans déclarer la version corrigée.
  Action : Dédupliquer avec RUSTSEC-2026-0104 et conserver son contre-audit ; réexaminer tout futur chemin CRL.
- [refuted] LOW : webpki: Name constraints for URI names were incorrectly accepted
  /private/tmp/bridget-audit168-source-20261015/Cargo.lock:698 ; 8a79cd8ce9831b541a71
  Alias primaire de RUSTSEC-2026-0098/CVE-2026-93600. Signal dupliqué résolu, tandis que0098 reste confirmé LOW pour la version affectée. Il ne s’agit ni d’une version hors plage ni d’une réfutation du risque URI. Compter un seul finding logique pour cet avis.
  Action : Rattacher ce signal au finding0098 confirmé et à sa remédiation TLS ; ne pas compter deux pénalités pour un avis.
- [refuted] LOW : webpki: Name constraints were accepted for certificates asserting a wildcard name
  /private/tmp/bridget-audit168-source-20261015/Cargo.lock:698 ; c295e6ed359067bba4a0
  Alias primaire de RUSTSEC-2026-0099/CVE-2026-93601. Signal dupliqué résolu, tandis que0099 reste confirmé LOW. La version affectée et le risque sous prérequis de mauvaise émission ne sont pas réfutés. Compter un seul finding logique pour cet avis.
  Action : Rattacher ce signal au finding0099 confirmé et à sa remédiation ; ne pas compter deux fois le même avis.
- [confirmed] INFO : unsafe-libyaml is unmaintained
  /private/tmp/bridget-audit168-source-20261015/Cargo.lock:945 ; 2dc3b75545c66871246c
  unsafe-libyaml0.2.11 est sélectionné via serde_yaml0.9.34+deprecated, utilisé par reprise::load_pin sur un fichier YAML local. L’avis primaire est explicitement informational=unmaintained, sans vulnérabilité précise ni version corrigée. Le risque de maintenance est confirmé, non transformé en exploitation ou HIGH. Présence héritée à HEAD/base sans réfutation.
  Action : Documenter puis planifier le remplacement de serde_yaml/unsafe-libyaml par une solution maintenue après vérification de compatibilité du format des pins. Aucun remplacement opportuniste ni nouvelle dépendance dans168.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/src/wrapper.rs:5972 ; 8dda6574f67a6c089c04
  Constante locale passée à OutputRedactionLease puis ManagedEvent sur UnixStream::pair ; assertion de son absence de la sortie. Fonction test sous cfg(test), sans authentification fournisseur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/tests/core_089_service_test.rs:71 ; 0d276219f5a1c1e7d384
  Clé déterministe construite dans write_greffe_policy et écrite dans le HOME temporaire privé de unique_home ; daemon enfant reçoit ce HOME. Credential de fixture isolée, pas credential opérationnel divulgué.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/plugins/maicie/tests/integration/guichet_gate.rs:134 ; 13a00d7bf55067826d37
  Lease claim_token créée ou remplacée par un pair Unix simulé ou un constructeur GuichetClaim. Consommée par parse_claim/process_guichet_claim ou GuichetClient dans fixture/base temporaire, avec comparaison de réponse/rejeu/échéance. Aucun compte fournisseur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/plugins/maicie/tests/integration/guichet_greffe.rs:76 ; c805a11241ea9a1b9ea5
  Lease claim_token créée ou remplacée par un pair Unix simulé ou un constructeur GuichetClaim. Consommée par parse_claim/process_guichet_claim ou GuichetClient dans fixture/base temporaire, avec comparaison de réponse/rejeu/échéance. Aucun compte fournisseur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/plugins/maicie/tests/integration/guichet_greffe.rs:145 ; f81c5ad51b964335756d
  Lease claim_token créée ou remplacée par un pair Unix simulé ou un constructeur GuichetClaim. Consommée par parse_claim/process_guichet_claim ou GuichetClient dans fixture/base temporaire, avec comparaison de réponse/rejeu/échéance. Aucun compte fournisseur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/tests/guichet_integration_test.rs:69 ; 75b7b2835ef9cc6023a9
  Clé déterministe construite dans write_greffe_policy et écrite dans le HOME temporaire privé de unique_home ; daemon enfant reçoit ce HOME. Credential de fixture isolée, pas credential opérationnel divulgué.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25109 ; 2aa5d9c7647272f6109f
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25239 ; 6416fd10512c699e3cc8
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25241 ; b265b7ec7d141f17c0dd
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25242 ; 4006c072d13aa333d3d2
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25246 ; 40bdd5d3e7283acb9c54
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25276 ; 9b873cd29f3df841d7f7
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25277 ; 8360196db2b3fcd45c1b
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/diff-155.patch:25296 ; ca41a0a84212b31b1689
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/plugins/maicie/tests/integration/guichet_greffe.rs:240 ; 19163c3a5570c582d632
  Lease claim_token créée ou remplacée par un pair Unix simulé ou un constructeur GuichetClaim. Consommée par parse_claim/process_guichet_claim ou GuichetClient dans fixture/base temporaire, avec comparaison de réponse/rejeu/échéance. Aucun compte fournisseur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/tests/held159_e2e.rs:1195 ; 46c3ecccd1595d7fa454
  Champ idempotency_key ou filtre SQL de déduplication, ou exemple artifact_publish soumis au schéma fermé. Sert au rejeu/identité de publication ; aucune capacité d’authentification attachée à cette valeur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/skills/bridget/references/commandes.md:1350 ; 4d6a50098fb3e4250748
  Champ idempotency_key ou filtre SQL de déduplication, ou exemple artifact_publish soumis au schéma fermé. Sert au rejeu/identité de publication ; aucune capacité d’authentification attachée à cette valeur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/plugins/maicie/tests/guichet_forbidden_operations_contract.rs:43 ; 78059e1abe661c044f79
  Lease claim_token créée ou remplacée par un pair Unix simulé ou un constructeur GuichetClaim. Consommée par parse_claim/process_guichet_claim ou GuichetClient dans fixture/base temporaire, avec comparaison de réponse/rejeu/échéance. Aucun compte fournisseur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/src/cli.rs:10609 ; 70cbe8e7c681cff0518e
  Champ idempotency_key ou filtre SQL de déduplication, ou exemple artifact_publish soumis au schéma fermé. Sert au rejeu/identité de publication ; aucune capacité d’authentification attachée à cette valeur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:37 ; 37e61b0d94999347abd7
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:52 ; edb41b155bd8dc4ad87f
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:71 ; 9a1cabfdd15e16f3fbde
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:226 ; 5044c735dc4f1de311a4
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:336 ; f29de119d9e480587130
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:870 ; 89369ff6758c21978845
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:948 ; e0b0cf5b43123a755dd8
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1222 ; eb88501da19e4538e0e4
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1223 ; bfe47a4617eaaa71a573
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1224 ; 3c42e5b2b5b387eabf5f
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1225 ; de07d16bf8ec594f62b1
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1311 ; 87d970a7f2dba7c30da4
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1778 ; 3a42aab3188a9d1ed869
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1866 ; 8d903bf5ddc42904d5c6
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2239 ; 01a3c0416bb5c7f3fd07
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2255 ; 28201ded2244667fa69b
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2259 ; 245a6c39c6df86bd4cd9
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2267 ; 49ef05f0457f6561f4f9
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2291 ; 5a11554af5442ae146a1
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/fixtures/greffe-authorization.example.json:4 ; 4cf87b30e735593e0a1a
  Valeur explicitement à remplacer. Le consommateur historique lit l’exemple, remplace attestation_key par KEY puis écrit une politique privée de fixture avant dépôt/rafraîchissement. Pas une clé réelle exposée.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/fixtures/greffe-authorization-refresh.example.json:4 ; 88746ad6e05a7ed3f8e5
  Valeur explicitement à remplacer. Le consommateur historique lit l’exemple, remplace attestation_key par KEY puis écrit une politique privée de fixture avant dépôt/rafraîchissement. Pas une clé réelle exposée.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/plugins/maicie/tests/contract/guichet_domain.rs:21 ; 6f3452e4f021efdc5122
  Lease claim_token créée ou remplacée par un pair Unix simulé ou un constructeur GuichetClaim. Consommée par parse_claim/process_guichet_claim ou GuichetClient dans fixture/base temporaire, avec comparaison de réponse/rejeu/échéance. Aucun compte fournisseur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/crates/bridget-daemon/tests/fixtures/artifacts/generated-provenance-v1.json:2 ; 328f93476c514572adde
  Champ idempotency_key ou filtre SQL de déduplication, ou exemple artifact_publish soumis au schéma fermé. Sert au rejeu/identité de publication ; aucune capacité d’authentification attachée à cette valeur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/plugins/maicie/tests/contract/guichet_client.rs:55 ; 1de52e02f0a020c90bf3
  Lease claim_token créée ou remplacée par un pair Unix simulé ou un constructeur GuichetClaim. Consommée par parse_claim/process_guichet_claim ou GuichetClient dans fixture/base temporaire, avec comparaison de réponse/rejeu/échéance. Aucun compte fournisseur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/plugins/maicie/tests/contract/guichet_client.rs:108 ; e6d882158ca622567242
  Lease claim_token créée ou remplacée par un pair Unix simulé ou un constructeur GuichetClaim. Consommée par parse_claim/process_guichet_claim ou GuichetClient dans fixture/base temporaire, avec comparaison de réponse/rejeu/échéance. Aucun compte fournisseur.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/fixtures/probe_shared_session.py:78 ; 62d0e6b4ea117d67c2db
  Valeur publique de poignée de main WebSocket Sec-WebSocket-Key, utilisée sur AF_UNIX avec contrôle HTTP101 et fournisseur fixture loopback sans authentification. Ce nonce public n’est pas un bearer token.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:887 ; a9154f37709bac0215ca
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:1401 ; 6c4eae073ab9eb8a2d99
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:1415 ; 5bb2f98141f61f4e82d8
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:1429 ; 4705fe6722a20ce3fc82
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:2187 ; c8e099ff46633274d08a
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/148-identite-delegation/validation/glm-recette-reelle-r2.md:86 ; 6c3a18795cb0f10f32a1
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:20 ; 007f48b8b34630483d48
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:150 ; 146508816270d87c4c0b
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:152 ; 877771808c51f8266417
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:153 ; 87738a9e5cbed0caa712
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:157 ; 0e97604d171430447877
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:187 ; b2b0ecb3240a79d713b1
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:188 ; bb216097d902986904c5
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.
- [refuted] INFO : Alerte historique de secret réfutée par son usage au commit d’origine
  /private/tmp/bridget-audit168-source-20261015/specs/155-demandes-enfant/validation/schema155-generation.json:207 ; 2113a3af1edfbb501cdc
  Empreinte SHA-256 d’un fichier, type de schéma ou commande attestée. Vérification de provenance/intégrité ; aucune valeur de token fournisseur. Les hunks patch155/156 sont reliés au reçu ou au code réel du même commit.
  Action : Aucune rotation ou suppression fondée sur ces sites réfutés. Requalifier si le rôle ou la source change.

<a id="extrait-2"></a>

## Extrait 2

- security.static : pass ; completed, 0 candidat(s).
- security.secrets : pass ; Scan réel r2 terminé,54sites qualifiés/refutés indépendamment ; équivalence exacte de sites et de SHA sources avec les54réfutations antérieures. Aucun secret réel établi dans ces alertes.
- security.history : pass ; Scan exhaustif terminé :12lots,9851blobs,9883profils ;280sites qualifiés et contre-audités indépendamment,0secret confirmé/0non résolu. Rapprochement exact aux trois triages, limites et méthode conservées.
- security.dependencies : pass ; Qualification du périmètre168 terminée, sans défaut HIGH/CRITICAL applicatif établi. Deux LOW TLS et une dette INFO restent actifs et pénalisés ; aucune exigence zéro LOW. Pass signifie triage achevé, pas absence de vulnérabilités. Voir arbitrage ROOT et limites statiques.
- security.authorization : pass ; Gardes locales, définition, mapping et droits conservés dans inspection et vrai Prepare ; token inerte refusé au Start côté maître et worker. Lecture indépendante r1 et oracles contrat/probe128 exécutés, sans validation de credentials réels.
- correctness.logic : pass ; Les causes deadline, mode Worktree et préflight sont corrigées dans le delta lu. Validation affectée r1 et rejeu Worktree r2 exécutés ; aucun retour au budget neuf ni adoption implicite du HEAD distant.
- correctness.concurrency : pass ; Comparaison des deux sondes fraîches corrigée ; permission finale peut varier avec effort. CAS/admission et garde de token conservés. Oracle natif et oracle permis exécutés ; cas dynamique distinct de dérive époque non exécuté.
- performance.complexity : pass ; Défaut N+1 corrigé par faits Git privés au lot. RED 521 appels pour 2 tuples ; PASS de l’oracle 128 atteste sa borne ≤40. Readiness/droits/modèle/effort restent individuels, Prepare/Start frais. Aucun temps gagné ni coût global mesuré.
- tests.regression : pass ; Campagne affectée r1 : 12 PASS et 1 FAIL conservé. La seule fixture du cas Worktree a été corrigée en r2 ; son rejeu exact passe 1/1. Sources produit byte-identiques r1/r2. Il s’agit de 13 oracles distincts finalement prouvés, pas d’une suite complète GREEN.
- architecture.dependencies : pass ; Réemploi des couches et primitives existantes dans le delta confirmé par revue indépendante ; lectures r0 des couches non modifiées réutilisées après égalité des hashes. Aucun service ni couplage T3 nouveau dans ce delta.
- quality.minimalism : pass ; Branche inaccessible supprimée ; cache borné par lot, inspection avant Task et préparation après Task ont des responsabilités distinctes. Pas de framework ni cache global ; lignes résiduelles et duplication globale non mesurées.
- quality.llm : pass ; Budget absolu, autorité locale, parseur fermé et observations figées restent séparés des conseils du décideur. Faux exécutable/flags/environnement et refus de sorties invalides exécutés ; aucun modèle réel exécuté par cette validation.
- reliability.observability : pass ; Délai demandeur et drain physique distincts, permis détenu jusqu’au drain, sorties et JSON bornés : oracles transport/permis PASS. Inspection Worktree sans effets et refus de marqueur Start attestés. Aucune disponibilité ou stabilité de production revendiquée.
- spec.traceability : pass ; Matrice r0 FR/SC/US et exigences inchangées conservées par hash ; les cinq corrections sont reliées à leurs oracles nommés et à la revue indépendante. Audit limité au code T002–T035/T050–T056 ; FULL, T040–T049, recettes réelles et activation ne sont pas prouvées ici.
