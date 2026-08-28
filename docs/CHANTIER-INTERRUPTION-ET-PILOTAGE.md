# Chantier — pilotage et interruption d'un tour en cours

Écrit le 28/08/2026 au soir, sur mandat de l'humain, avant qu'il n'aille dormir.
Ce fichier est la source. Rien ici n'est une supposition : chaque affirmation
porte sa mesure ou sa référence.

## L'ordre des travaux, et il n'est pas négociable

**ZÉRO — d'abord livrer ce qui est déjà fait.** Avant tout développement
nouveau : intégrer les branches livrées et sans conflit, recompiler, relancer.
Mesure du 28/08 à 18h42 : le service tourne sur `f7658d4d9746-dirty`, un commit
**absent de `main`**, avec 138 commits de retard. Tant que ce n'est pas fait,
rien de ce qui est développé ne sert à personne.

**UN — le pilotage d'un tour en cours.** Le reste de ce document.

**DEUX — le reste**, qui attendra.

---

## Le besoin, en une phrase

Quand l'humain écrit à un agent, cet agent doit le prendre en compte **sans
attendre la fin de son tour**. Il décide ensuite : il reprend, il abandonne, ou
il propose autre chose. C'est lui qui propose ; l'humain n'administre pas.

Ce qui n'est **pas** demandé : arrêter tout le parc, un simple accusé de
réception, une file prioritaire pour les messages humains.

---

## Ce qui est mesuré

**Le défaut.** Test du 28/08 sur un agent en mode flux : commande de 90 s
de 16:12:39 à 16:14:09, message envoyé à 16:13:26 donc à mi-parcours, vu à
16:14:28 — **19 secondes après la fin**. L'agent déclare : « aucun message vu
pendant le sleep ».

Même protocole sur un agent en terminal : message vu **16 secondes avant la
fin**. Le mode terminal a la propriété, le mode flux ne l'a pas — et la
passation en cours fait basculer les agents du premier vers le second.

**Les quatre tests d'interruption**, tous isolés, aucun composant de production
touché :

| Fournisseur | Requête | Délai | Sous-processus | Bridget l'envoie déjà ? |
|---|---|---|---|---|
| Codex | `turn/interrupt` | 6 ms | **continue** | oui, `codex_app_server.rs:806` |
| Cursor | `session/cancel` **sans champ `id`** | 4 ms | non vérifié | oui, bonne forme |
| Claude | `control_request` / `interrupt` | 25 ms | **arrêté** | **non, absent** |

---

## Les formes exactes, validées par la mesure

**Codex — interruption**
```json
{"jsonrpc":"2.0","id":4,"method":"turn/interrupt",
 "params":{"threadId":"…","turnId":"…"}}
```
Réponse `{}`, puis `turn/completed` avec `status: "interrupted"`.
La sortie tardive de la commande abandonnée arrive ensuite, **correctement
attribuée à l'ancien `turnId`** — un adaptateur qui filtre par identifiant de
tour l'ignore proprement. Bridget indexe déjà par `turn_id`.

**Cursor — annulation**
```json
{"jsonrpc":"2.0","method":"session/cancel",
 "params":{"sessionId":"…"},"headers":[]}
```
**PIÈGE MESURÉ, LE PLUS IMPORTANT DE TOUS** : avec `id: ""` au lieu d'un champ
absent, Cursor Agent 2026.08.25 a produit **8 235 fragments pendant 87 secondes**
après l'annulation et n'a jamais répondu. La bibliothèque ACP de référence
produit cette forme fautive. Bridget, lui, omet correctement le champ.

Solder les demandes en attente **avant** d'annuler : autorisations marquées
`cancelled`, saisies avec une réponse vide. Sans cela l'agent reste suspendu.

**Claude — interruption en flux brut, sans le kit officiel**
```json
{"type":"control_request","request_id":"…",
 "request":{"subtype":"interrupt"}}
```
Réponse :
```json
{"type":"control_response","response":{"subtype":"success",
 "request_id":"…","response":{"still_queued":[]}}}
```
Puis un résultat terminal `aborted_tools` ou `aborted_streaming`.
**Conclusion d'architecture** : Bridget n'a pas besoin de basculer vers le kit
officiel ni de changer de transport. Il lui manque seulement cette trame.

---

## Ce qui est mieux que l'interruption, et que personne n'a encore mesuré ici

Codex expose **`turn/steer`** : injecter un message dans un tour en cours **sans
l'interrompre**. C'est exactement le besoin, sans le coût.

Référence : `~/11.Repositories/openclaw/extensions/codex/src/app-server/`,
fichiers `attempt-steering.ts` et `run-attempt-active-turn.ts`. Openclaw offre
deux modes au choix — `queue`, ce que fait Bridget aujourd'hui, et `steer`. Leur
défaut est `steer`.

**Trois pièges qu'ils ont documentés, à ne pas redécouvrir :**

- *L'acceptation n'est pas la remise.* Une interruption efface les entrées
  acceptées mais non consommées. Garder le message non soldé tant que Codex n'a
  pas confirmé l'avoir traité.
- *`turn/steer` est un accusé, rien ne garantit une réponse.* Sans délai
  d'attente, l'appelant reste bloqué jusqu'à la fermeture du client et **bloque
  tous les pilotages suivants derrière lui**.
- *Préserver l'ordre après un rejet* : un message rejeté ne doit pas être doublé
  par le suivant.

**Non résolu ailleurs** : openclaw n'implémente le pilotage que pour Codex —
vingt-quatre fichiers pour Codex, un pour Anthropic, zéro pour ACP.

---

## Ce qui reste à trancher par la mesure

1. **`turn/steer` sur Codex** — jamais mesuré ici. Le plus rentable.
2. **Un équivalent pour Claude** — peut-on écrire un message utilisateur pendant
   un tour actif sans interrompre ? Inconnue réelle, aucune trace nulle part.
3. **Un équivalent pour Cursor** — le protocole ACP prévoit-il autre chose que
   l'annulation ? Aucune trace chez openclaw.

**Décision de l'humain** : ne pas attendre les trois. Le coordinateur passera
sur Codex, où le pilotage sans interruption existe. Les autres suivront avec
l'interruption, qui est déjà prouvée pour les trois.

---

## Ce qui est demandé, concrètement

**A — Coordinateur Codex.** Relancer le référent en Codex plutôt qu'en Claude,
avec sa carte de reprise et son contexte. Le fournisseur est indifférent à
l'humain ; ce qui compte est que le pilotage y soit possible.

**B — La trame Claude.** Écrire dans `claude_stream_json.rs` l'émission de
`control_request`, la corrélation de `request_id`, et la reconnaissance des deux
états terminaux. Aucun de ces trois mots n'apparaît aujourd'hui dans le fichier.

**C — Le déclencheur.** Pour les trois transports, relier l'arrivée d'un message
humain au mécanisme d'annulation ou de pilotage. Pour Codex et Cursor le
mécanisme existe déjà et n'attend qu'un déclencheur ; c'est la pièce manquante,
pas la pièce difficile.

**D — `turn/steer` d'abord pour Codex**, l'interruption en repli pour les autres.

---

## Contraintes, tirées de ce qui a échoué aujourd'hui

**Le transport est le point de passage de tout le parc.** Une régression y coupe
la communication de tous, référent compris — donc sans moyen de se faire aider à
réparer. C'est arrivé ce matin : service arrêté, `attach` inutilisable, plus
aucune observation possible.

**Une clôture doit citer un effet mesuré, pas une intégration.** Deux clôtures
du 28/08 ont été posées sur des correctifs bien présents dans `main` et sans
aucun effet sur la route réelle. La preuve d'ancestralité prouve qu'un code est
livré, pas qu'il fonctionne.

**Ne jamais conclure sur une déclaration d'agent.** Les quatre tests de la
journée ont été tranchés par des horodatages relevés. Deux annonces de
« c'est fait » se sont révélées fausses en quelques minutes de vérification.
