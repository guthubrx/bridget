# Archives techniques de l’adaptateur T3

Document d’origine : [specs/098-t3code-adapter/adversarial-review-bridget-revue.md](../../../../../../specs/098-t3code-adapter/adversarial-review-bridget-revue.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Date : 2026-09-14 08:14 (CEST) · Agent : `bridget-revue`, UUID `a005530d-9168-4887-9aee-0ff3dbfc1a39`, Codex gpt-5.6-sol · Canal : `bridget_send` MCP, réponse liée · Question : défaut logique, sécurité (jeton, réglages, injection dans un tour), trous FR-09801..09812, hypothèses t3code réfutables, abstraction inutile.
- Verdict reçu : **BLOCKED** (8 objections, toutes avec fichier:ligne).

<a id="extrait-2"></a>

## Extrait 2

| objection | verifiee comment | retenue oui/non | raison |
|---|---|---|---|
| 1. `t3 auth session issue` impose les portées administratives ; lecture exige `orchestration:read` | rapport de recherche (`cli/auth.ts:162-193`, `contracts/auth.ts:81-115`) | oui | FR-09803 : portées administratives assumées et documentées |
| 2. `--token-only` cache le sessionId, révocation impossible ; renouvellements non bornés | `cliAuthFormat.ts:107-116`, `cli/auth.ts:216-236` | oui | sortie JSON, stockage session_id, un renouvellement par incident, révocation au retrait |
| 3. Identité d'émission par fil non prouvée ; Codex par instance contredit FR-09804 | contrat public sans identifiant de session fournisseur | oui | option A : v1 sans émission depuis l'agent ; FR-09804 réécrite |
| 4. Variable d'environnement = preuve usurpable | raisonnement vérifié : toute variable est reproductible localement | oui | aucune identité déduite d'une variable d'environnement (limites de la spec) |
| 5. Remise pendant un tour actif contredit US3 | `ProviderCommandReactor.ts:1537-1576` | oui | attente bornée de l'absence de `activeTurnId`, indéterminé ensuite, course résiduelle testée |
| 6. Pas de flux d'événements en HTTP ; journal partiel ; pas de curseur | `environmentHttp.ts:509-538` | oui | projection par différence de snapshots, curseur durable, déclenchement sur toute activité |
| 7. `settings.json` : deux écrivains, pas de rollback | `cli/theme.ts:222-282` | oui, par suppression du besoin | v1 n'écrit plus dans t3code ; rollback par étape pour jeton et service |
| 8. `server-runtime.json.version` versionne le fichier, pas l'API | `serverRuntimeState.ts:11-27` | oui | validation route par route |

<a id="extrait-3"></a>

## Extrait 3

Points confirmés par le relecteur : déduplication par `commandId` cohérente (`OrchestrationEngine.ts:144-171`) ; `t3code_contract.rs` utile.

<a id="extrait-4"></a>

## Extrait 4

| objection | verifiee comment | retenue oui/non | raison |
|---|---|---|---|
| 1. Réponse assistant possiblement incomplète (`streaming = true`) | `orchestration.ts:494-504, 601-609` | oui | attendre `streaming = false`, confirmer par `latestTurn.assistantMessageId` quand exposé |
| 2. Corrélation en attente non durable après redémarrage | `wrapper.rs:572-646` (état `pending` en mémoire) | oui | enregistrement durable `t3code-pending.json` écrit avant le `dispatch`, repris au démarrage |
| 3. Confusion `subject`/`label` | `cli/auth.ts:162-180`, `cliAuthFormat.ts:146-164` | oui | `--subject bridget --label bridget-<installation>`, recherche par `client.label` |
| 4. Détail complet du fil à chaque sondage : coût proportionnel à l'historique | `orchestration/http.ts:64-79`, `orchestration.ts:954-985` | oui | pagination `turnLimit`/`beforeCursor`, relecture seulement si `threadSequence` change |

<a id="extrait-5"></a>

## Extrait 5

Agent : `bridget-revue` (Codex, gpt-5.6-sol). Question : ce pont peut-il perdre, dupliquer ou mal attribuer une réponse, ou modifier t3code, hors des cas testés ?

<a id="extrait-6"></a>

## Extrait 6

| Objection | Vérifiée comment | Retenue | Suite |
|---|---|---|---|
| 1. Appariement ordinal faux dès qu'un tour ne produit aucun message assistant (tour interrompu, fan-out) : la réponse de l'humain serait attribuée à Bridget | Relu `correlate` : rank(B)=1 et un seul tour visible → renvoi du tour de l'humain. Défaut réel | oui | Corrélation prouvée : le rang n'est utilisé que si, après l'ancre, il y a autant de tours assistants que de messages utilisateur. Sinon `Waiting` (fil au travail) ou `Ambiguous` (fil au repos) et la demande reste sans réponse. Deux tests ajoutés (tour sans réponse, tour de sous-agent) |
| 2. Perte de corrélation si le pont meurt entre le 200 de t3code et l'écriture de l'attente | Relu l'ordre des lignes : l'attente était écrite après le dispatch | oui | L'attente est écrite avant l'appel, avec un `messageId` déterministe (`stable_uuid`) ; un refus de forme la retire, une panne réseau la conserve. Un rejeu reconstruit la même commande, que t3code déduplique par `commandId` |
| 3. Réponse dupliquée : le retrait de l'attente n'était sauvegardé qu'en fin de passage | Relu `settle_pending` : envoi puis sauvegarde différée | oui | Identifiant de réponse déterministe (`reply_id`) et sauvegarde de l'état après chaque décision |
| 4. Le renouvellement n'était borné qu'à une génération : après le second 401, un appel suivant renouvelait encore, soit une session par minute | Relu `Session::call` : `generation` incrémentée puis plus aucun état terminal | oui | Verrou `auth_failed` posé quand une session neuve est refusée ; plus aucun renouvellement jusqu'à réinstallation. Oracle renforcé : le compte d'émissions reste à 2 dans la durée |
| 5. Sessions administratives orphelines : écriture du jeton échouée après émission, révocation échouée avalée, `uninstall` effaçant l'état malgré un échec | Relu `renew`, `uninstall`, `install` | oui | Révocation immédiate d'une session non conservée ; identifiants non révoqués consignés dans `orphan-sessions.json` et repris au retrait ; `uninstall` échoue sans rien effacer tant qu'une révocation n'est pas acquise ; rollback ajouté à l'écriture du manifeste |
| 6. Pas d'élévation possible : le décideur t3code recopie la politique du fil et ignore celle transmise ; une valeur périmée ne cause qu'un refus de schéma | Confirmé par la revue sur `apps/server/src/orchestration/decider.ts` | close | Test de non-régression ajouté ; la politique n'est plus mise en cache mais lue au moment du dispatch (le cache donnait une valeur périmée, ce que le test a prouvé) |
