# Modèle 136

discussion_entries immuable: kind nullable (legacy ou history/action/blocker/decision)
et supersedes_seq nullable (cible antérieure unique du même fil).
Cible même auteur/audience, non history, sans successeur. Dépôt+relation atomiques.
Audience effective: liste notify_json.targets, déjà stockée au dépôt de chaque
entrée (V1 compris). Aucun recalcul via wakes ; membres102 immuables.
Aucun statut mutable ajouté. Schéma fils v2, colonnes null pour les lignes legacy,
index unique partiel ; corps, compteurs, opérations, reçus et wakes inchangés.

read: history → body absent, presentation=history_reference ; remplacé au snapshot
→ body absent, presentation=superseded_reference, superseded_by_seq.
seq/message_id/auteur/date/notify restent ; références relues par history.
history_ref fournit thread_id/from_seq/to_seq pour l'action history bornée,
sans ACK ni déplacement du repère. Les droits restent ceux des membres du fil.
Les autres body restent intacts. History conserve tous les body exacts.
Reçu: bornes et snapshot conservés ; projection reconstruite à ce snapshot,
rejeu stable. ACK confirme les références et corps reçus, jamais une mission.
