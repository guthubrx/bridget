# T011 - route humaine reelle du 2026-08-29

## Deploiement observe

- La route humaine a ete activee par le daemon `3e0cbcb6c550` pret a `19:13:34Z`.
- Le relais UI associe etait pret a `19:14:55Z`.
- Le test a cible `priorite-reponse-codex`, agent d'essai de la SPEC-063, jamais un agent de travail.

## Protocole isole

- Le depart demande exactement `sleep 90`, sans ecriture de fichier.
- Le second message humain demande l'arret immediat et le marqueur `T011_ACK_STOP`.
- Les deux messages passent par `POST /v1/send` du relais UI, pas par une fixture ni par un client transport direct.

## Horodatages releves

| Evenement | Horodatage UTC | Preuve |
| --- | --- | --- |
| envoi UI START | 19:19:23.550Z | message `c34e4e4fb5004` accepte |
| tour et prompt START | 19:19:23Z | journal Codex `turn_start` puis `prompt_dispatched` |
| commande sleep commencee | 19:19:33Z | `item/started` pour `/bin/bash -lc 'sleep 90'` |
| envoi UI STOP | 19:19:42.464Z | message `308f687d16ac4` accepte |
| steer enregistre | 19:19:42Z | journal Codex `turn_steer` |
| message humain remis au tour | 19:20:03Z | `prompt_dispatched` via `turn/steer` |
| marqueur emis par l'agent | 19:20:07Z | fragments texte `T011_ACK_STOP` |
| tour termine | 19:20:07Z | `turn_end`, raison `completed` |
| fin naturelle attendue | 19:21:03Z | debut du `sleep 90` plus 90 secondes |

## Verdict

- Le travail etait actif avant le message STOP : la commande avait demarre neuf secondes avant son envoi.
- Le tour s'est termine 56 secondes avant sa fin naturelle, apres remise du message humain et emission de son marqueur.
- La prise en compte fournisseur a demande 21 secondes entre l'envoi UI et `prompt_dispatched`, puis le tour a termine quatre secondes plus tard.

## Non verifie

- Cette route Codex a employe `turn/steer`, capacite active du tour, et non le repli `turn/interrupt`.
- Le meme essai reel Claude n'est pas possible avant le 31/08 06:00 UTC : les deux agents Claude d'essai ont retourne leur limite hebdomadaire avant tout travail.
- Le rendu visuel final du relais UI est couvert par ses tests et par l'asset servi, pas par une seconde capture navigateur apres le correctif `b9bd9f8`.

## Sources brutes

- Journal: `~/.cache/bridget/sessions/priorite-reponse-codex/2026-08-29.jsonl`.
