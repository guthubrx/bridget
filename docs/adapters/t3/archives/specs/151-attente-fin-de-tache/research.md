# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/research.md](../../../../../../specs/151-attente-fin-de-tache/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11 (révisée après la contre-revue Codex, voir
`adversarial-review-codex.md`). Source : lecture du dépôt principal non modifié
(`/Users/moi/Nextcloud/10.Scripts/64.bridget`, base `1f2c661d`) et de
`specs/150-socle-delegation/` (acquis supposé livré). Les lignes citées sont
celles de cette base. Mécanismes T3 : `specs/150-socle-delegation/research-t3-bridget.md`
(A1, B1, B3, B4). Aucun test ni build lancé.

<a id="extrait-2"></a>

## Extrait 2

- **Décision** : `NativeDelegationRequest::Wait` est traité dans `handle_with_fact`
  (`native_delegation.rs:280-320`) avant la prise du verrou d'état. La boucle prend le
  verrou le temps d'une lecture, puis le relâche.
- **Preuve** : un thread par connexion (`daemon.rs:4965-4970`, `handle_connection`
  `:4988`). `handle_with_fact` relâche déjà le verrou avant `tick`
  (`native_delegation.rs:313-319`).
- **Alternatives** : `bridget_events` avec `once` (`mcp.rs:2001`) observe des fins de
  tour, pas des résultats de tâche ; abonnement Lineage (`native_lineage.rs`) est la
  voie humaine et exige `--t3-thread`.
- **À vérifier (T001)** : aucune coupure d'inactivité sur une connexion client du
  daemon. La lecture de trame a un délai de 10 s seulement après le premier octet
  (`daemon.rs:5040`).

<a id="extrait-3"></a>

## Extrait 3

- **Décision** : à l'expiration, `timed_out: true` et les tâches telles qu'elles sont.
- **Raison** : T3 ne tue pas l'enfant à l'expiration (A1). Le délai d'attente est
  indépendant de `mission_deadline_at` (150 : `reply_timeout`).
- **Réveil garanti** : l'attente n'écrit rien (R6) et `tick` ne la connaît pas. Le message
  de résultat part donc exactement comme sans attente. Le test
  `native151_boite_message_apres_attente_expiree` le prouve.
