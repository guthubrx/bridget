# Chantier — pilotage et interruption d'un tour en cours

Écrit le 28/08/2026 au soir sur mandat de l'humain.

---

# ⛔ LES TESTS SONT FAITS. NE LES REFAITES PAS.

**Quatre tests isolés ont été menés ce soir, sur les trois fournisseurs, avec
horodatages à la milliseconde et traces conservées.** Tout est reproduit
intégralement plus bas : formes JSON exactes des requêtes, réponses reçues,
délais mesurés, chemins des traces.

**Ce qui est demandé est une IMPLÉMENTATION, pas une validation.**

Il est interdit de rouvrir une campagne de tests sur ce qui est mesuré ici. Si
quelqu'un pense qu'une mesure ne vaut pas, il doit écrire **pourquoi** — quelle
condition diffère, quelle version change — avant de la refaire. Un « je préfère
vérifier » ne suffit pas : l'humain a fait faire ces tests exprès pour que
personne ne les refasse.

**Contrôle minimum acceptable après implémentation** : un horodatage montrant
qu'un agent a été interrompu ou piloté en cours de tour, sur la route réelle.
C'est tout. Pas de campagne, pas de contre-validation croisée, pas de jury.

---

## L'ordre des travaux, en trois phases

**PHASE 1 — livrer ce qui est déjà développé.** Intégrer les branches livrées
sans conflit, recompiler, relancer avec `bridget-relance`.
*Fin de phase, critère unique* : la route `v1/journal?agent=humain` rend un
nombre d'octets **non nul**. Elle rendait 0 à 19h13 alors que le correctif était
intégré et en service ; `agent=jc2` rendait 26170 au même instant.

**PHASE 2 — ce chantier, et rien d'autre.** Mené **au bout** : livré, intégré,
en service, vérifié par son effet.
*Fin de phase, critère unique* : l'humain peut interrompre un agent en cours de
tour. Tant que ce n'est pas vrai, la phase n'est pas finie, quel que soit le
nombre de lots produits.

**PHASE 3 — le reste repart**, et seulement alors. Le gel de l'expansion est
levé à ce moment-là, pas avant.

---

## Le besoin, en une phrase

Quand l'humain écrit à un agent, cet agent doit le prendre en compte **sans
attendre la fin de son tour**. Il décide ensuite : il reprend, il abandonne, ou
il propose autre chose. C'est lui qui propose ; l'humain n'administre pas.

**Ce qui n'est PAS demandé** : arrêter tout le parc, un simple accusé de
réception, une file prioritaire pour les messages humains.

---

# LES MESURES

## Test 0 — le défaut, mesuré des deux côtés

**Agent en mode flux** (`essai-distant-flux`) :

| Événement | UTC |
|---|---|
| Commande `sleep 90` démarrée | 16:12:39 |
| Message envoyé, à mi-parcours | 16:13:26 |
| Commande terminée | 16:14:09 |
| Message vu par l'agent | **16:14:28** |

Soit **19 secondes après la fin**. L'agent déclare : « aucun message vu pendant
le sleep ».

**Agent en mode terminal** (`jc2`), protocole identique :

| Événement | UTC |
|---|---|
| Début | 17:43:38 |
| **Message vu** | **17:44:52** |
| Fin | 17:45:08 |

Soit **16 secondes AVANT la fin**. Le mode terminal a la propriété, le mode flux
ne l'a pas — et la passation en cours fait basculer les agents du premier vers
le second.

---

## Test 1 — Codex, `turn/interrupt`

**Requête exacte :**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "turn/interrupt",
  "params": {
    "threadId": "01a04992-18f5-75d3-803d-12133d24b495",
    "turnId": "01a04992-19fa-71f1-8f2a-dd969e27a926"
  }
}
```
Réponse `{}`, puis `turn/completed` avec `status: "interrupted"`.

| Événement | UTC |
|---|---|
| Commande longue démarrée | 18:11:54.486 |
| `turn/interrupt` envoyé, à mi-parcours | 18:12:39.635 |
| Réponse JSON-RPC | 18:12:39.641 — **6 ms** |
| Tour signalé `interrupted` | 18:12:39.641 |
| Tour suivant traité, `POST_INTERRUPT_OK` | 18:12:41.218 |
| `sleep 90` terminé **naturellement** | 18:13:24.488 |

**Le sous-processus n'est PAS arrêté.** Le tour protocolaire s'arrête, la
commande shell continue jusqu'au bout.

Trace : `/tmp/user/1002/codex-turn-interrupt-demo-9T8HjI/result.json`

## Test 2 — Codex, sort de la sortie tardive

| Événement | UTC |
|---|---|
| Commande marquée démarrée | 18:22:28.660 |
| Interruption envoyée | 18:22:58.765 |
| Réponse | 18:22:58.770 — **5 ms** |
| Ancien tour clos `interrupted` | 18:22:58.770 |
| Tour suivant clos, `SUIVANT_PROPRE` | 18:23:00.187 |
| Marque de l'ancienne commande | 18:23:28.661 |
| Ancienne commande terminée, `exitCode: 0` | 18:23:28.662 |

**Sortie tardive reçue :**
```json
{
  "method": "item/commandExecution/outputDelta",
  "params": {
    "threadId": "01a0499b-bdca-7f83-9330-10b6e8f972f6",
    "turnId": "01a0499b-be89-7133-9eff-761894647c76",
    "delta": "ORPHAN_MARKER_28AUG2026_1815\n"
  }
}
```

**Le `turnId` est celui de l'ancien tour, déjà interrompu.** La sortie n'est ni
perdue ni injectée dans le tour suivant. Un adaptateur qui filtre par `turnId`
l'ignore proprement — **Bridget indexe déjà par `turn_id`**. Un adaptateur qui
afficherait tout ce qui arrive sur le fil actif polluerait l'interface.

Trace : `/tmp/user/1002/codex-interrupt-output-demo-NUbf1A/result.json`

## Test 3 — Cursor, `session/cancel`

**Requête exacte, forme QUI FONCTIONNE — sans champ `id` :**
```json
{
  "jsonrpc": "2.0",
  "method": "session/cancel",
  "params": { "sessionId": "a7c5a9e2-8262-438a-b2eb-351c0afce09a" },
  "headers": []
}
```

| Événement | UTC |
|---|---|
| Premier flux de raisonnement | 18:43:00.309 |
| `session/cancel` envoyé | 18:43:01.324 |
| Prompt clôturé | 18:43:01.328 — **4 ms** |
| État | `stopReason: "cancelled"` |
| Fragments après annulation | **0** |
| Message suivant envoyé | 18:43:01.329 |
| Réponse suivante, `CURSOR_NO_ID_CANCEL_OK` | 18:43:03.888 |

### ⚠ PIÈGE LE PLUS COÛTEUX DE TOUS

Avec la forme `"id": ""` — celle que **produit la bibliothèque ACP de
référence** — Cursor Agent 2026.08.25-3e8eec8 a continué à produire
**8 235 fragments pendant 87 secondes** après l'annulation, et le prompt n'a
**jamais** répondu.

`id: ""` ne fonctionne pas. L'**absence** du champ fonctionne.
Bridget omet correctement le champ aujourd'hui — ne pas régresser là-dessus.

### Demandes d'autorisation en attente

Cursor émet :
```json
{
  "method": "session/request_permission",
  "id": 0,
  "params": { "toolCall": { "title": "…", "status": "pending" } }
}
```
Le client doit la solder **avant** d'annuler :
```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": { "outcome": { "outcome": "cancelled" } },
  "headers": []
}
```
Mesuré : demande en attente à 18:38:21.768, soldée à 18:38:21.811, tour rendu à
18:38:24.296, message suivant traité à 18:38:27.126 avec `CURSOR_POST_CANCEL_OK`.
**Sans ce règlement, l'agent reste suspendu indéfiniment.**

Traces :
- `/var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/cursor-acp-cancel-demo-RCcbiQ/result.json`
- `/var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/cursor-acp-cancel-no-id-demo-2wbPcQ/result.json`
- `/var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/cursor-acp-active-cancel-demo-8PG9GL/result.json`

## Test 4 — Claude, `control_request`

### Par le kit officiel

Appel : `await activeQuery.interrupt()`

| Événement | UTC |
|---|---|
| Premier flux reçu | 18:48:54.734 |
| `interrupt()` appelé | 18:48:55.758 |
| Promesse résolue | 18:48:55.760 |
| Terminal `aborted_streaming` | 18:48:55.764 — **6 ms** |

### SANS le kit, en flux brut — c'est le résultat qui compte

**Requête exacte, écrite sur l'entrée standard :**
```json
{
  "type": "control_request",
  "request_id": "raw-tool-interrupt-detached-001",
  "request": { "subtype": "interrupt" }
}
```
**Réponse exacte de Claude :**
```json
{
  "type": "control_response",
  "response": {
    "subtype": "success",
    "request_id": "raw-tool-interrupt-detached-001",
    "response": { "still_queued": [] }
  }
}
```
Puis un résultat terminal `aborted_tools` ou `aborted_streaming`.

**Test pendant un appel d'outil réel** (`sleep 60`, marque écrite à la fin) :

| Événement | UTC |
|---|---|
| Outil effectivement démarré | 18:58:10.346 |
| Interruption envoyée à mi-course | 18:58:40.348 |
| Accusé de contrôle | 18:58:40.351 — **3 ms** |
| Terminal `aborted_tools` | 18:58:40.373 — **25 ms** |
| Tour suivant envoyé | 18:58:40.373 |
| Tour suivant réussi | 18:58:41.967 — **1,594 s** |

**Le sous-processus EST arrêté** : la marque de fin attendue vers 18:59:10 était
toujours absente à 18:59:14.348. Contrairement à Codex.

**Aucun signal POSIX n'est nécessaire.**

**Conclusion d'architecture, ne pas la rediscuter** : Bridget n'a pas besoin de
basculer vers le kit officiel ni de changer de transport. Il lui manque
seulement cette trame.

Traces :
- `/var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/claude-sdk-interrupt-runtime-hxCCew/result.json`
- `/var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/claude-stream-json-interrupt-demo-xkDeN7/result.json`
- `/var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/claude-stream-json-tool-interrupt-tmux.sRnPl3zGyV/result.json`

---

# ÉTAT DE BRIDGET, VÉRIFIÉ DANS LE CODE

| Fournisseur | Requête | Délai | Sous-processus | Bridget l'envoie déjà ? |
|---|---|---|---|---|
| Codex | `turn/interrupt` | 6 ms | continue | **oui** — `crates/bridget-transport/src/codex_app_server.rs:806` |
| Cursor | `session/cancel` sans `id` | 4 ms | non vérifié | **oui**, bonne forme — `acp.rs:549` et `acp.rs:1384` |
| Claude | `control_request` / `interrupt` | 25 ms | **arrêté** | **NON — absent** de `claude_stream_json.rs` |

Vérifié : `grep -E "control_request|aborted_tools|aborted_streaming"` sur
`claude_stream_json.rs` ne rend **rien**.

Le déclencheur d'annulation existe pour Codex — `codex_app_server.rs:800`,
`if !interrupted && cancel.try_recv().is_ok()`. Il n'est actionné que par un
arrêt d'agent, jamais par l'arrivée d'un message.

---

# MIEUX QUE L'INTERRUPTION — `turn/steer`

Codex expose **`turn/steer`** : injecter un message dans un tour en cours
**sans l'interrompre**. C'est exactement le besoin, sans le coût.
**Bridget ne le connaît pas.**

## Référence : openclaw

`~/11.Repositories/openclaw/extensions/codex/src/app-server/`
- `attempt-steering.ts` — la file de pilotage, `createCodexSteeringQueue` ligne 45
- `run-attempt-active-turn.ts` — son usage dans un tour actif, ligne 63
- `run-attempt-lifecycle-controller.ts:73` — « Interrupt drops accepted pending »
- `attempt-steering.test.ts` et `run-attempt.steering.test.ts` — leurs tests

**Requête, telle qu'ils l'émettent** (`attempt-steering.ts:189`) :
```
client.request("turn/steer", {
  threadId,
  expectedTurnId,
  input,                 // items utilisateur
  clientUserMessageId,
}, { timeoutMs, signal })
```

**Modes offerts à l'utilisateur** (`ui/src/app/settings.ts:139`) :
`["queue", "steer"]`. `queue` = ce que fait Bridget aujourd'hui.
**Leur défaut est `steer`** (`follow-up-mode.ts:47`).

## Les trois pièges qu'ils ont documentés — ne pas les redécouvrir

1. **L'acceptation n'est pas la remise.** Une interruption efface les entrées
   acceptées mais non consommées. Garder le message non soldé tant que Codex n'a
   pas confirmé l'avoir traité par un `userMessage completion`.
   (`attempt-steering.ts:180-181`)
2. **`turn/steer` est un accusé, rien ne garantit une réponse.** Sans délai
   d'attente et sans signal d'exécution, l'appelant ne se débloque qu'à la
   fermeture du client, **et bloque tous les pilotages suivants derrière lui**.
   (`attempt-steering.ts:184-187`)
3. **Préserver l'ordre après un rejet** : un message rejeté ne doit pas être
   doublé par le suivant. (`attempt-steering.ts:218-219`)

**Non résolu ailleurs** : openclaw n'implémente le pilotage que pour Codex —
24 fichiers pour Codex, 1 pour Anthropic, 0 pour ACP.

---

# RÉFÉRENCE : t3code

`~/11.Repositories/t3code/apps/server/src/provider/`

**Codex** — `Layers/CodexAdapter.ts:1845` expose `interruptTurn`, qui appelle
`session.runtime.interruptTurn` (ligne 1847).
`Layers/CodexSessionRuntime.ts` envoie `turn/interrupt` avec un délai de 3 s puis
un second appel borné à 10 s.
**Contrainte à retenir** (`CodexSessionRuntime.ts:1853`) : « Codex accepte des
suites pendant que le tour courant tourne. La réponse contient l'identifiant du
tour en file, mais `turn/interrupt` n'accepte que celui qui est actif
maintenant. » Il faut donc suivre `activeTurnId`, pas l'identifiant retourné.

**Cursor** — `Layers/CursorAdapter.ts:1067`, `interruptTurn`. Séquence :
solder les autorisations en `cancelled`, solder les saisies avec réponse vide,
**puis** `session/cancel` sur ACP.

**Claude** — `Layers/ClaudeAdapter.ts:396-400`. Ils passent par
`@anthropic-ai/claude-agent-sdk` et `query()`. Le commentaire documente les deux
états : `aborted_tools` pendant un appel d'outil, `aborted_streaming` pendant le
flux.

**OpenCode** — `Layers/OpenCodeAdapter.ts:1558`, même forme.

---

# CE QUI EST DEMANDÉ

**A — Coordinateur Codex.** Relancer le référent en Codex plutôt qu'en Claude,
avec sa carte de reprise et son contexte. Décision de l'humain, à ne pas
rediscuter : le fournisseur lui est indifférent, ce qui compte est que le
pilotage y soit possible.

**B — La trame Claude.** Dans `claude_stream_json.rs` : émettre
`control_request`, corréler `request_id` sur `control_response`, reconnaître
`aborted_tools` et `aborted_streaming` comme états terminaux. Aucun de ces mots
n'existe aujourd'hui dans le fichier.

**C — Le déclencheur.** Relier l'arrivée d'un message humain au mécanisme
d'annulation ou de pilotage, pour les trois transports. Pour Codex et Cursor le
mécanisme existe et n'attend qu'un déclencheur — c'est la pièce manquante, pas
la pièce difficile.

**D — `turn/steer` pour Codex**, avec les trois pièges ci-dessus.
L'interruption reste le repli pour les autres.

## Ce qui reste réellement inconnu — les seules mesures encore permises

1. **`turn/steer` en conditions réelles** — jamais mesuré ici. Openclaw s'en
   sert, mais nous ne l'avons pas éprouvé.
2. **Un équivalent `steer` pour Claude** — peut-on écrire un message utilisateur
   pendant un tour actif sans interrompre ? Aucune trace nulle part.
3. **Un équivalent `steer` pour Cursor en ACP** — aucune trace chez openclaw.

**Tout le reste est mesuré. Ne le remesurez pas.**

---

# CONTRAINTES, TIRÉES DE CE QUI A ÉCHOUÉ LE 28/08

**Le transport est le point de passage de tout le parc.** Une régression y coupe
la communication de tous, référent compris — donc sans moyen de se faire aider à
réparer. C'est arrivé le matin : service arrêté, `attach` inutilisable, plus
aucune observation possible.

**Une clôture doit citer un effet mesuré, pas une intégration.** Deux clôtures
du 28/08 ont été posées sur des correctifs présents dans `main` et sans aucun
effet sur la route réelle. L'ancestralité prouve qu'un code est livré, pas qu'il
fonctionne.

**Ne jamais conclure sur une déclaration d'agent.** Les quatre tests ont été
tranchés par des horodatages relevés. Deux annonces de « c'est fait » se sont
révélées fausses en quelques minutes de vérification.

**Pendant les phases 1 et 2** : aucune revue de confort, aucun constat
auto-généré, aucune relance des agents en terminal.
