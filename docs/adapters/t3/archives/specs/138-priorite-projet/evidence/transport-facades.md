# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/evidence/transport-facades.md](../../../../../../../specs/138-priorite-projet/evidence/transport-facades.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Périmètre : wrapper.rs, t3code.rs et t3code_contract.rs uniquement.
Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet.

<a id="extrait-2"></a>

## Extrait 2

- `connect_and_register_with_domain_at` reste le chemin propriétaire du wrapper.
  Il annonce le cwd constaté avec `source=git` à chaque connexion, y compris
  lorsque le daemon attribue la nouvelle identité. Le domaine reste distinct.
- `connect_and_register_at` n'annonce jamais le cwd du wrapper. T3 réutilise ce
  chemin pour ne pas hériter du projet du processus d'adaptation.
- `parse_snapshot` rattache la racine T3 par `projectId` à un seul projet publié.
  Référence absente ou ambiguë : racine inconnue. Le worktree reste indépendant.
- `LinkWorker::new` et les ticks existants publient les faits T3 via
  `send_wrapper_message`. Chaque reconnexion repart d'une annonce. Un tick
  identique ne republie rien. Un changement est transmis sans décider de sa
  cohérence : seul le daemon détermine si les racines sont contradictoires.
- Les lecteurs traitent `ProjectContextResult` sans injecter son résultat dans
  la conversation, ni consommer une livraison ou un Disconnect en attente.

<a id="extrait-3"></a>

## Extrait 3

- `spec138_snapshot_uses_project_id_not_worktree_or_title`.
- `spec138_snapshot_missing_or_ambiguous_project_stays_unknown`.
- `spec138_owner_announces_git_cwd_on_each_registration_without_consuming_frames` :
  deux échanges avec un véritable pair Unix, fait Git puis Disconnect conservé.
- `spec138_t3_owner_announces_unknown_project_instead_of_using_adapter_cwd` :
  un socketpair réel, annonce T3 inconnue plutôt que le cwd de l'adaptateur.
- L'assertion du test existant `snapshot_reel_0_0_40_est_lu_et_les_champs_manquants_sont_nommes`
  vérifie également la racine publiée.
