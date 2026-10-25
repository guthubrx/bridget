# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/adversarial-review-project-scope-inventory.md](../../../../../../specs/138-priorite-projet/adversarial-review-project-scope-inventory.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Objection | Vérification par le principal | Retenue | Action |
|---|---|---|---|
| La connexion background retire l'identité T3 ; le projet doit venir d'un contexte autonome | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:350, environnement background | Oui | Contexte Client propre négocié, racine/hôte validés sans Register ni identité empruntée ; plan/modèle/contrat et T004 corrigés |
| Une reprise de notification peut réutiliser un mandat courant modifié | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:3602 et /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:3626, création/reprise d'outbox | Oui | Figer corps, destinataire, racine et motif avant tentative ; T011 teste échec puis changement de mandat |
| Un ancien serveur peut ignorer le champ nouveau de BridgetMessage | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-core/src/message.rs:74, absence de deny_unknown_fields | Oui | Exiger la capacité138 côté client avant de transmettre une enveloppe à motif ; pas de downgrade ni supposition de refus serveur |
