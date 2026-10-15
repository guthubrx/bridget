# ADR169 — Garder la préférence de délégation distincte de l'identité

Date : 2026-10-15. Statut : accepté pour la conception par ROOT.

Le réglage de délégation doit changer avant un tour sans faire tourner l'identité du fil. Le registre MCP existant efface des faits de permissions lorsque son objet de configuration est remplacé. Cette frontière traverse les réglages, la session et les adaptateurs ; une reconstruction banale de l'objet pourrait donc casser la reprise.

Décision : réutiliser le réglage, le contexte par tour `ProviderAdapterV2TurnInput` et la composition existants. Transmettre la préférence comme champ optionnel non autorisant, même sans session MCP. Ne pas modifier l'objet qui possède l'identité. Les droits, le moteur et le suivi restent leurs autorités existantes.

Le contrat complet, ses limites et les preuves attendues ont une seule source : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/contracts/delegation-preference.md.

Conséquence : aucun service ni stockage supplémentaire. Un test de continuité doit protéger la référence et l'identité lors d'un changement de préférence. L'option de contexte Claude et les autres détails de transmission restent dans le contrat, sans copie de leur mécanisme ici.
