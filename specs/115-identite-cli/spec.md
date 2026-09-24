# Spécification 115 — Un fil T3 peut envoyer par la ligne de commande

## Fiche synthèse
Spec: 115-identite-cli | Statut: In Progress | Priorité: P0 | Date: 2026-09-24
Branche: session-115-identite-cli | Dépend de 101 (marqueurs d'identité par filiation) et 114.

## Problème observé
Le 2026-09-24 à 06:44, le coordinateur `opus_city_ai` (Claude, dans T3) devait confier quatre
missions à ses équipiers. Ses quatre envois ont été refusés : « identité expéditeur non attestée ».
L'utilisateur signale que cela arrive souvent et rend Bridget inutilisable depuis Claude.

Deux causes indépendantes se sont cumulées.

**A. La ligne de commande ne pouvait jamais envoyer depuis un fil T3.** `bridget send` établit sa
connexion au daemon par la filiation de processus (`resolve_current_identity`), donc attestée au
nom de l'agent. Mais il signe le message avec `current_agent_id`, qui ne lit que
`BRIDGET_AGENT_ID_FILE` et `BRIDGET_AGENT_ID`, variables posées par les seuls lanceurs Bridget.
T3 n'en est pas un : le message part signé « human ». Le daemon voit une connexion attestée
prétendre parler au nom d'un humain et refuse l'usurpation. Reproduit depuis le fil `bdget` : ancien
binaire refusé sur l'identité, nouveau binaire accepté. Le même défaut touche `reply`,
`requests` et les deux crochets Claude qui déclarent le modèle, ce qui explique aussi la colonne
MODÈLE vide des agents Claude de T3.

**B. Le serveur MCP Bridget avait disparu de la configuration de Claude.** T3 ne transmet à Claude
que son propre serveur MCP ; Bridget dépend donc d'une déclaration au niveau utilisateur dans
`~/.claude.json`. Elle y avait été ajoutée à la main le 2026-09-23 (sauvegarde
`.claude.json.bak-bridget-20260923-074938`), puis écrasée : ce fichier est l'état interne de Claude,
réécrit en permanence par chacun de ses processus. Le processus relancé à 04:29 n'a donc reçu aucun
outil Bridget, et l'agent s'est rabattu sur la ligne de commande, cassée par A.

## Exigences
- **FR-001** : un envoi CLI depuis un fil T3 est signé par l'identité que la filiation atteste, la
  même que celle qui authentifie sa connexion.
- **FR-002** : l'ordre de preuve des lanceurs Bridget est inchangé ; la filiation n'est consultée
  qu'en l'absence de leurs variables.
- **FR-003** : une filiation qui ne livre pas un identifiant valide n'invente rien : « human ».
- **FR-004** : hors de tout agent Bridget, les crochets Claude restent inertes.
- **FR-005** : aucun changement de protocole, de daemon ni de pont.

## Hors périmètre
- Rendre durable la déclaration MCP de Claude contre les réécritures concurrentes de son fichier
  d'état : elle a été rétablie par la commande officielle `claude mcp add --scope user`, et la
  ligne de commande couvre désormais son absence.

## Critères de succès
- **SC-001** : depuis un fil T3, `bridget requests` liste les demandes de l'agent, pas celles
  d'un humain.
- **SC-002** : depuis un fil T3, un envoi franchit le contrôle d'identité.
- **SC-003** : recette complète verte.
