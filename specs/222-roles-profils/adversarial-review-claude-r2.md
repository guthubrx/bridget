# Contre-revue indépendante222 — ronde2
Je lis les artefacts de revue en lecture seule (Linux ; le Mac n'est pas accessible depuis ce checkout).Le fichier `adversarial-review-claude.md` n'est pas au chemin indiqué. Je cherche où il se trouve dans le cache.Les deux documents clés sont lus. Je vérifie maintenant les points d'ancrage cités dans le code, sans rien modifier.Les ancres `daemon.rs:4958-4959` et `protocol.rs:200/224` sont confirmées. Il reste à vérifier que la table `work_submissions` ignore bien les lignes `prepared` dans la reprise existante.**Verdict : APPROVE_WITH_CHANGES** (ronde 2, lecture seule ; aucun fichier modifié, aucun build, test ni envoi).

**Levée des P1 de la ronde 1 (au niveau contrat)**
- **Grant non lié au lancement** : levé. `closure-after-review.md` définit `MandateGrant` versionné avec `approved_by`/`approval_ref` humain. La réservation, la relecture de la révision et le recontrôle avant RELEASE passent sous le même `DaemonState`. `request_recovery` et `recovering=true` sont interdits pour un réveil métier.
- **Attribution / coupure ambiguës** : levé. Les quatre stades A/B/C/D de reprise sont explicites, avec contre-épreuves obligatoires.
- **Identité indéfinie** : levé. `Specialist.id = agent_profiles.agent_id`, `profile_id` distinct.
- **Oubli / capture / rappel vagues** : levé. Tombstone par `source_key`, hooks Task/ObservedActivity, rappel lexical sans FTS.
- **RuntimeBinding redondant** : retiré dans `data-model.md`, présence dérivée du roster.

**Vérifications de code (lecture seule)**
- `daemon.rs:4958-4959` : `ExecutionStore::open` et `DelegationStore::open` reçoivent bien le même `config.db_path`. Le « même fichier SQLite » est confirmé.
- `protocol.rs:200` et `:224` : `MobileAdministrationV1` et `ControlStateV1` sont bien des variantes de `ClientCapability`. La correction de R1 est juste.
- `execution_store.rs:1768` : la contrainte `work_submissions.state` accepte déjà `prepared`. Les insertions de file (`:392`, `:451`) n'écrivent que `accepted`. Le principe « prepared n'est pas un début d'exécution » est donc compatible avec le schéma actuel.

**Peut-on produire des tasks sans autorisation magique ni duplication ?** Oui, sous conditions. L'admission exige la relecture du grant sous mutex, la réservation avec canon, une promotion `prepared→accepted` unique, et une clé idempotente. La contrainte à ajouter est listée ci-dessous.

**Corrections minimales à faire avant code**
1. **`plan.md` l.65-69 contredit le contrat** : il parle de « session nommée persistante » et de « sa file existante ». Le contrat impose `SpawnOrder.persistent=false` et une reprise corrélée. Réécrire ces lignes ou déclarer explicitement que `closure-after-review.md` les remplace.
2. **Kinds de définition incohérents** : `plan.md` l.40 liste profil / compétence-procédure / composition, `data-model.md` l.2 liste profile/team. Aligner sur un seul vocabulaire.
3. **Capture automatique ambiguë** : `plan.md` l.98-99 dit « les sorties de missions sont indexées automatiquement ». Le contrat ne retient que le résultat Task corrélé et l'ObservedActivity. Préciser que seuls ces deux hooks créent une entrée, en « résumé rapporté ».
4. **Dérivation de `native_submission_id` non spécifiée** : elle doit être une fonction déterministe de `(issuer_scope, request_id, digest du canon)`. Sans cela, le stade B « reprendre la soumission déterministe » n'est pas implémentable.
5. **Garde sur les lignes `prepared`** : ajouter une exigence explicite et un test de contre-épreuve. Aucun drain, recovery ou lecture existant ne doit admettre une ligne `prepared` sans entrée d'exécution ou de file. Je n'ai pas vérifié exhaustivement toutes les requêtes `work_submissions`.

**Non vérifié**
- Le Mac (`/Users/moi/.cache/bridget222-review`) : non accessible depuis ce checkout, donc non relu.
- Le fait que `DaemonState` couvre effectivement la réservation `FleetSupervisor` : cité par le contrat, pas relu dans le code.
- Le signalement d'incidents à `bridge1016` : non envoyé. C'est une action sortante, interdite en mode plan. À faire par vous ou à ma demande explicite.

Ce travail est une revue, pas un plan d'implémentation. Je n'ai donc écrit aucun fichier de plan et je n'ai pas appelé ExitPlanMode.

[Bridget] espace de travail : branche 222-revue-plan-roles-r2, base e88d1db, 0 commit(s), modifications non commitées : non, tête sur la branche : oui, worktree conservé : /home/moi/.local/share/bridget-projects/bridget/repository/.worktrees/b47a61ea. Consultez le statut de la tâche avant tout retrait. Clôturez la tâche, préservez le travail et vérifiez les autres usages du dossier.

## Décision du responsable
Les trois P1 sont levés au contrat, pas déclarés implémentés. Les cinq précisions sont retenues et corrigées dans plan/contrat et tâches T009/T010/T011/T016/T027. Aucune réduction de périmètre. Le code et les recettes restent à réaliser.
