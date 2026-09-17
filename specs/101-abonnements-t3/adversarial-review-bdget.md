# Contre-revue adverse101 — bdget

Date2026-09-16, demande16:57:59CEST, borne5min.
Fournisseur interrogé :Claude, agent bdget127bccff-8490-453a-8182-884b749ec41e.
Appelant :Codex, Bridget-enhanceb280f81d-a418-4dbc-bfd8-35c50d8fedd1.
Message MCPmcp-6800-6aaaae77-1, issued_at1789570679,
delivery_id4f044f1c-cfc5-477e-9a3a-47fe47422bb9 ; ledger confirme reçu.

## Question posée

Challenger identité sans usurpation, faits de fin/anti-boucle et limites de la
preuve T3. Verdict demandé APPROVE/APPROVE_WITH_CHANGES/BLOCKED et objections
vérifiables, corrections minimales. Interdits : fichiers, commits, déploiement,
lancement de fournisseur supplémentaire, API payante/GPU/média.
Artefacts et diff base1738a072 communiqués en chemins absolus :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3/specs/101-abonnements-t3/spec.md
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3/specs/101-abonnements-t3/plan.md
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3/specs/101-abonnements-t3/reuse-audit.md
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3/specs/101-abonnements-t3/tasks.md
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3/specs/101-abonnements-t3/implementation.md

Recette complémentaire explicitement demandée : abonnement MCP ponctuel à la
fin de son propre tour, puis réponse finale normale pour permettre la remise
d'une notification dans son fil T3 réel. Aucun message à Horizon ni autre projet.
Abonnement témoin confirmé en état durable :97a33158-195e-490e-aed2-aeb8c1879009,
owner=source=127bccff-8490-453a-8182-884b749ec41e. Recette réussie : fin
15:02:04Z, remise15:02:05Z, notification présente15:02:08Z dans le journal T3.
Précision : bdget n'avait pas d'outil MCP Bridget exposé ; repli CLI annoncé par
bdget, sans identité fournie ni secret copié. Notre appelant Codex utilise MCP.

## Verdict

APPROVE_WITH_CHANGES reçu dans le journal attesté bdget seq231 à15:02:04Z,
dans la borne5min. Pas de suite de tests exécutée par le relecteur (déclaré).
L'affirmation «20tests spec101_*» repose sur un motif de noms, pas sur le filtre
cargo spec101 : ce dernier exécute aussi les12tests du module identité spec101.

| Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|
| Construction non commitée, code exact introuvable | Accord utilisateur et skill interdisent commit ; cmp et SHA256 du binaire ; sauvegarde/archives locales | Partiellement | Traçabilité renforcée par archive source101.tgz ; commit/refonte déploiement rejetés car contraires à l'autorité explicite. Diff à relire/committer ultérieurement par décision humaine. |
| Identité JSONL couvre Claude, pas Codex | t3code_identity.rs:284,:447 ; propriétaire de l'abonnement Horizon vérifié, appel MCP réel Codex | Non | Objection factuellement inversée : Codex utilise rollouts JSONL, Claude flags natifs. Les deux identités ont créé un abonnement réel ; ne pas annoncer un faux refus Codex. |
| lsof indisponible ou lent, diagnostic utilisateur générique | runtime.rs inventaire borné ; t3code.rs:1030 journalise le refus ; erreur client identity_not_found | Partiellement | Limite ergonomique réelle : motif précis dans logs, message client générique. Aucun élargissement de droits ni blocage nominal observé ; amélioration possible hors critères101, documentée sans nouveau chantier. |
| T011 doit rester ouverte jusqu'à réception réelle | Journal T3 seq232–235, log remise séquence43313, abonnement consommé en base | Oui puis résolue | T011 cochée seulement après preuve ; fin de notification identifiée bridget-observation et exclue du collecteur. |

Réponse corrective adressée à bdget par MCPmcp-6800-6aaaafca-2, sans nouvelle
demande de réponse. Aucune régénération de code sur les hypothèses du relecteur.
