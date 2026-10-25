# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/analysis-report.md](../../../../../../specs/138-priorite-projet/analysis-report.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Finding | Preuve observée par le principal | Correction retenue | Vérification prévue |
|---|---|---|---|
| Client background sans contexte utilisable | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:350 retire l'identité T3 héritée | Contexte autonome sur connexion Client négociée, racine/hôte validés, sans Register d'agent ni identité empruntée ; plan/modèle/contrat et T004 alignés | T003/T004 et scénario du client de fond |
| Reprise d'outbox sous un mandat modifié | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:3602 et :3626 décrivent l'enveloppe de notification/reprise | Racine, motif, destinataire et corps figés avant la première tentative ; T011 teste modification après échec | T011/T012 et scénario de reprise d'outbox |
| Ancien serveur susceptible d'ignorer un champ | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-core/src/message.rs:74 ne porte pas deny_unknown_fields | Le client exige la capacité138 avant l'enveloppe à motif ; aucun downgrade sans motif | T009/T010 et scénario serveur sans capacité |
| Membre inconnu susceptible de masquer une divergence connue | Matrice du fil A/B/U relue avec FR-13804/05/08 | Vérifier chaque lecteur ; l'inconnu U ne supprime pas le contrôle A/B ; T014 complété | Fil A/B/U sans motif refusé avant dépôt |
