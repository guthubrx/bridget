# Contre-revue adverse — `cursor5`

**Date** : 2026-08-25 21h45 · **Agent** : `cursor5` · **Fournisseur** : cursor
*(distinct du référent — c'est le motif du choix)*
**Périmètre** : lecture seule, rien écrit, rien commité.

**Question posée** : le découpage en six lots tient-il, et l'objectif est-il
atteignable en une nuit ? Trois angles prioritaires — la collision sur `ui.rs`,
le risque L5/L6, et les comportements oubliés.

## Verdict : `APPROVE_WITH_CHANGES`

> « Le cœur tient. La promesse *fichiers disjoints* est **fausse**. Objectif
> atteignable **si** L3/L4 réellement largués **et** L5/L6 coupés. Les 13
> critères : non. »

## Objections

| # | Objection | Vérifiée comment | Retenue | Suite |
|---|---|---|---|---|
| 1 | **Collision à trois sur `ui.rs`** — T5.2 retire le HTML embarqué, donc L5 touche le fichier de L1 et L2 | lu dans `tasks.md` T5.2 : *« retirer les 153 lignes embarquées de `ui.rs` »* — **contredit** le plan §2 qui annonce L5 sur les seuls assets | **oui** | **option A appliquée** : L1 absorbe le service des assets et le retrait ; `rc1` est **l'unique auteur de `ui.rs`** ; `jc1` ne touche que les assets |
| 2 | **L5→L6 est le risque de la nuit** — 153 → ~2000 lignes chez un agent | cohérent avec le plan §7, déjà identifié | **oui** | **ordre de coupe transmis à `jc1`** : couper d'abord les niveaux riches, puis le panneau, puis le dépli élaboré. Garder liste, saisie, envoi, réception, défilement, bandeaux |
| 3a | **Producteur de `relay_state` absent** — C5 décrit l'événement, personne ne l'émet → AC11 incomplet | contrat C5 relu : décrit la forme, **aucune tâche ne l'émet** | **oui** | **ajouté au lot de `rc1`** |
| 3b | `unread`, `last_excerpt`, `last_message_at` **non produits** ; `AgentInfo` porte déjà `state`/`host` → **DTO local**, ne pas toucher `protocol.rs` | `UiSnapshotV1.agents` sert `AgentInfo` brut | **oui** | **`UiAgentRowV1` local ajouté au lot de `rc1`** ; `protocol.rs` déclaré intouchable (ressource partagée) |
| 3f | **Rétention du focus au re-rendu** — cousin d'AC1 | non mesuré, mais **le raisonnement est décisif** : si le champ perd le focus à l'arrivée d'un message, l'utilisateur tape dans le vide — *le défaut d'origine sous une autre forme* | **oui, critique** | transmis à `jc1` : re-rendre **le fil seul**, jamais la zone de saisie ; focus, curseur et sélection conservés |
| 3g | **Entrée / Maj+Entrée** non spécifiés | absent de la spec | **oui** | tranché : **Entrée envoie, Maj+Entrée saute une ligne** |
| 3e | **Pastille de non-lu sans marquage comme lu** | `unread` défini, extinction non définie | **oui** | tranché : s'éteint quand le fil est affiché **et le bas visible** — pas au simple clic, sinon elle s'éteint sans lecture |
| 3c | **Accuser une demande** (§4.7) : ni contrat ni tâche | vérifié dans C1 et `tasks.md` | **oui** | **reporté hors nuit**, inscrit comme dû |
| 3d | **Sélecteur de machine** dans la spec mais **pas dans C1** | vérifié | **oui** | **hors nuit** — ne pas émettre un champ que le relais ne sait pas lire |
| — | « L0 déjà fait mais absent de `tasks.md` » | exact : `contracts/relais-v1.md` **existe** | **oui, sans effet** | les contrats sont figés avant les lots, ce qui était le but de L0 |
| — | « 5 auteurs + 4 relecteurs + composition = surcoût » | — | **non** | régime **un relecteur par lot**, sans jury. La mesure du 25/08 montre l'inverse du surcoût : douze lots sur douze en jury complet quand le critère en donnait quatre |

## Ce que cette contre-revue a évité

**Un conflit à trois sur `ui.rs`**, découvert au moment de la composition — donc
après que trois agents aient écrit dessus. C'est exactement ce qui est arrivé le
25/08 : deux lots ne compilaient plus une fois assemblés alors que chacun
passait ses tests.

**Le point 3f seul justifiait la revue** : sans lui, l'interface aurait pu
satisfaire AC1 à la lettre — le texte n'est pas modifié — **tout en dépossédant
l'utilisateur de sa saisie** par perte de focus.

## Réserve du référent

L'agent a rendu **en moins de dix minutes** sur la borne de vingt annoncée. Ses
objections **1, 3a, 3b, 3c, 3d, 3e, 3g sont vérifiables sur pièces** et l'ont
été. **3f n'est pas mesurée** — c'est un raisonnement, retenu parce qu'il est
solide, pas parce qu'il est prouvé.
