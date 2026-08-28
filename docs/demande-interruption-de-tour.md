# Interruption d'un tour en cours — deux demandes

Le 28/08/2026. Deux demandes distinctes, à ne pas confondre : la première
**prouve** qu'un agent peut être arrêté en plein travail, la seconde **intègre**
cette capacité dans le transport.

## Le besoin, en une phrase

Quand l'humain écrit à un agent, cet agent doit s'arrêter, l'écouter, répondre —
puis reprendre ou non. Aujourd'hui il ne le voit qu'à la fin de son tour.

## Ce qui est mesuré, pas supposé

**En mode flux**, test du 28/08 sur `essai-distant-flux` : commande de 90 s de
16:12:39 à 16:14:09, message envoyé à 16:13:26 donc à mi-parcours, vu à 16:14:28
— soit **19 secondes après la fin**. L'agent déclare : « aucun message vu pendant
le sleep ».

**En mode terminal**, même protocole sur `jc2` : début 17:43:38, message vu
**17:44:52**, fin 17:45:08. Vu **16 secondes avant la fin**, donc en plein milieu.

Le mode terminal a donc la propriété, le mode flux ne l'a pas. Or la passation en
cours fait basculer les agents du premier vers le second : chacun **perd** cette
capacité en passant.

## Ce qui existe ailleurs, et qu'il ne faut pas réinventer

Le dépôt de référence `~/11.Repositories/t3code` pilote quatre fournisseurs et
implémente l'interruption pour chacun.

**Codex** — `apps/server/src/provider/Layers/CodexSessionRuntime.ts:1866` :
une requête `turn/interrupt` sur le protocole de l'agent, avec un délai de 3 s,
puis un second appel borné à 10 s. C'est une méthode standard de son interface.
L'adaptateur l'expose sous `interruptTurn` (`CodexAdapter.ts:1845`).

**Claude** — `apps/server/src/provider/Layers/ClaudeAdapter.ts` : ils passent par
le kit officiel `@anthropic-ai/claude-agent-sdk` et sa fonction `query()`, pas par
une écriture brute sur l'entrée. Le commentaire des lignes 395-400 documente le
comportement : interrompre pendant un appel d'outil produit `aborted_tools`,
pendant le flux produit `aborted_streaming`. Les deux sont traités comme des états
normaux.

**OpenCode** — `OpenCodeAdapter.ts:1558`, même forme, `interruptTurn`.

**Cursor** — `CursorAdapter.ts`, le moins outillé des quatre.

Point important : **Bridget écrit sur l'entrée du programme en brut**, un niveau
en dessous du kit officiel. C'est là que la capacité se perd.

---

# DEMANDE 1 — PROUVER, sur un agent d'essai

**Pour l'agent d'action ponctuelle. Une heure, jetable, mode dégradé assumé.**

**Objectif unique** : montrer qu'un agent Codex en cours d'exécution s'arrête
quand on lui envoie `turn/interrupt`, et qu'il reste utilisable après.

**Périmètre strict** : un agent d'essai — `essai-distant` ou un agent lancé pour
l'occasion. **Ne pas toucher au transport de production**, ni au service, ni au
référent, ni aux agents qui travaillent.

**Protocole suggéré, à adapter** : lancer une commande longue sur l'agent d'essai,
envoyer `turn/interrupt` à mi-parcours, relever l'horodatage des deux côtés, et
vérifier qu'un message suivant est bien traité après.

**Ce qu'on attend en rendu** : la forme exacte de la requête envoyée, ce que
l'agent a répondu, les horodatages, et surtout **ce qui n'a pas été vérifié**.
Si l'interruption laisse l'agent dans un état inutilisable, c'est un résultat
aussi utile qu'un succès — le dire.

**Ce qui ferait échouer la démonstration sans qu'on le voie** : conclure sur une
déclaration de l'agent plutôt que sur un horodatage mesuré. Les deux tests du
28/08 ont été tranchés par des heures relevées, pas par des impressions.

---

# DEMANDE 2 — INTÉGRER, une fois la preuve faite

**Pour le circuit normal : spécification, relecture croisée, intégration.**

**Propriété attendue** : un message humain adressé à un agent est présenté à cet
agent sans attendre la fin de son tour. L'agent décide ensuite s'il reprend ce
qu'il faisait, s'il l'abandonne, ou s'il propose autre chose.

**Ce qui n'est pas demandé** :
- Interrompre tous les agents. Seul le destinataire est concerné.
- Un accusé de réception. L'humain veut une analyse et une réponse, pas d'être
  rassuré.
- Une file prioritaire pour les messages humains. Ce serait le geste et non la
  propriété, et le défaut survivrait.

**Contrainte technique majeure** : le transport est le point de passage de tous
les messages de tous les agents. Une régression y coupe la communication du parc
entier — référent compris, donc sans moyen de se faire aider à réparer. C'est
arrivé le 28/08 au matin : le service arrêté a rendu `attach` inutilisable, donc
impossible d'observer ce qui se passait.

**Contrainte de langage** : le kit de référence est en TypeScript, Bridget est en
Rust. Deux voies — reproduire le protocole d'interruption en Rust en s'inspirant
de `t3code`, ou faire passer le pilotage par un programme intermédiaire. Le choix
appartient à celui qui prend le lot ; qu'il l'écrive.

**Point de vigilance sur les agents Codex** : ils sont majoritaires dans le parc
et leur protocole expose déjà `turn/interrupt`. C'est le cas le plus simple et le
plus rentable — commencer par lui.

**Ce que la clôture devra citer**, et pas seulement l'intégration du correctif :
un horodatage mesuré montrant qu'un message a été vu avant la fin d'un tour, sur
la route réellement utilisée. Deux clôtures ont été posées le 28/08 sur des
correctifs intégrés qui ne produisaient pas leur effet.
