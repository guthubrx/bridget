# Contre-revue adverse — agent gui (Codex)

## Revue du plan

- Date : 2026-09-13 20:22 (CEST)
- Agent interrogé : `gui`, UUID `a005530d-9168-4887-9aee-0ff3dbfc1a39`, fournisseur Codex (gpt-5.6-sol), seul agent d'un autre fournisseur connecté
- Canal : `bridget send` sans demande suivie (l'expéditeur humain n'est pas adressable en retour) ; livrable demandé dans `adversarial-review-gui.verdict.md`, id ledger `03daf15ab3d04`
- Question : défaut logique, régression Codex 090 ou agents gérés, trou de couverture FR-09701..09713, abstraction inutile dans le plan PTY + présence `cli/claude_pty` + recette HOME réel/USER + refus sans voie de remise
- Verdict reçu : **pas de réponse dans le délai** (borne 10 min, contrôle à 20:31)

| objection | verifiee comment | retenue oui/non | raison |
|---|---|---|---|
| — | — | — | aucune réponse avant la borne ; le pipeline continue |

## Revue de l'implémentation

- Date : 2026-09-13 21:34 (CEST)
- Agent interrogé : `gui` (Codex), même UUID ; id ledger `8b41d19db1a24`
- Question : défaut logique, fuite de ressources (fd, thread, terminal), régression Codex 090 / agents gérés, trou FR-09701..09713, abstraction inutile dans le diff 097 (session PTY, PtyTransport, présence cli/claude_pty, refus sans pane, retrait du bypass, journal transcript « nés après », 128+signal, gate HOME réel + USER)
- Verdict reçu : **pas de réponse dans le délai** (borne 10 min, contrôle à 21:44) ; l'agent est resté `connected` sans produire le fichier demandé

| objection | verifiee comment | retenue oui/non | raison |
|---|---|---|---|
| — | — | — | aucune réponse avant la borne ; le pipeline continue |

Conséquence : aucune contre-revue d'un autre fournisseur n'a pu être obtenue dans ce tour ; la self-review Article XX et les recettes réelles tiennent lieu de vérification, sans prétendre à l'équivalence.
