# Contrat UUID agent — extension US147-06

Date :2026-10-09. Statut :plan UUID validé par le principal avant implémentation. Tests RED/GREEN encore attendus.

Entrées concernées : champs UUID des commandes CLI et de l'outil MCP de fils partagés create/post/read/ack/history/show/close : thread/operation_id/membres/targets/reçus. Admissibilité : forme hyphénée36 caractères équivalente au UUID parsé par eq_ignore_ascii_case. Sortie : forme hyphénée minuscule avant hash d'idempotence et accès SQL. Pas de trim, accolade, URN ou forme compacte ajouté aux champs UUID.

Réutiliser require_uuid pub(crate) dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs et thread_members dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs. Le CLI conserve les noms/préfixes non UUID et leur résolution existante. Ne pas réécrire un nom ni le corps.

Create/Post/Close/ACK : une même requête dont seules les casses d'identifiants changent conserve le même effet idempotent. ACK normalise le reçu avant lookup, mais garde ReceiptInvalid pour son refus métier. Corps et références non UUID déclarées restent inchangés.

canonical_uuid humain145/146/147 et parse_recent_cursor restent stricts et inchangés. Normaliser une référence n'accorde aucun droit ; la garde d'identité/projet/rattachement reste souveraine. Aucun nouveau helper, dépendance, table, migration, champ wire ou configuration de production.

Tests comportementaux : sept scénarios minimaux Create/Post/Close/ACK/membres-targets/invalides/invariants humains ; parsing CLI et résolution des noms ; replay avec majuscules et reçus exacts. Fixtures privées sans modèle, agent actif ni restart. Les écritures interagents de ces tests ne sont pas des effets du canal humain FR147-15.

Portée UUID US6 : seulement les actions et l'outil de fils partagés create/post/read/ack/history/show/close et leurs références UUID thread/operation/membres/targets/reçus. Aucun changement des IDs opaques send ou d'autres outils, de l'acteur d'autorité, des noms/préfixes UUID partiels, de canonical_uuid humain ni des curseurs fermés.
