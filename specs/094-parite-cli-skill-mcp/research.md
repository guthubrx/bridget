# Recherche de couture 094

Date : 2026-09-07. Sources : code local et deux audits indépendants.

| Sujet | Décision et preuve | Alternative écartée |
|---|---|---|
| Nom | Réutiliser client::rename_display_name, DisplayNameSet et validation agent_profile (80 caractères). UUID inchangé. | Nom utilisé comme adresse / mutation libre |
| DND | Availability propre, défaut 3600s ; durée 1s–604800s (7j), conversions checked. Fin calculée explicitement, volatile daemon. | Entier non borné pouvant paniquer dans Instant |
| Domaine | Domain propre + sauvegarde existante agent-domains/UUID, factorisée CLI/MCP, écriture atomique, échec explicite après ACK. | Nouveau store ou CLI masquant erreur disque |
| Autorité | Audit indépendant : Domain/Availability appellent des handlers sans conn_id (daemon.rs:11850), donc identité libre. Exiger live_connection_identity à chaque appel et résoudre présence par instance sous verrou. CLI doit rejoindre client auxiliaire partagé. | Ne fermer que le schéma MCP et garder un accès brut usurpable |
| Runtime | Déclaration propre source Declared, limites runtime existantes (100 caractères, contrôles/bidi refusés). Ne jamais utiliser SelectRuntime. | Présenter une déclaration modèle comme un changement effectif |
| Status | Réutiliser daemon::get_status ; projection MCP explicite running, disponibilité inventaire, compte nullable, build_id/hôte. Exclure socket/base/instance. | Lire SQLite ou inventer total messages |
| Control | Client role + capacité de lecture Lookup si suffisante, issuer_scope(instance) ; état et historique <=50, jamais scopes humains. | bridget-control-cli ou ControlStateSet depuis MCP |
| Reaper | CLI humain : observe_live lit ps/lsof/fichiers et écrit observations.jsonl. | Présenter report comme simple lecture daemon |
| Reprise | CLI humain : agrège Git, pin, chemins locaux et fallback base ; observations sûres séparées par who/ledger/status/control_status. | Paramètres de chemins libres au MCP |
| Permissions | Étendre liste exacte Codex approval_mode=approve et Claude allowedTools dans les chemins existants. | bypass global ou seulement ajouter tools/list |
| Documentation | Skill courte + référence d'inventaire dédiée, exemples exacts et catégories d'acteurs ; README FR/EN liés. | Recopier manuel complet dans skill chargée systématiquement |

Audit auteur reçu : message 6fd3400466ca4. Contre-revue indépendante Codex
review_authority_094 : trou de garde Domain/Availability confirmé. Aucun agent
Claude/Gemini joignable, donc pas de recette inter-fournisseur revendiquée.

Impact mainteneur : un seul client de commande, un seul stockage de domaine,
réutilisation des reçus et tests existants ; pas de dépendance ni registre neuf.
La limite DND 7j borne le nouveau contrat et le CLI partagé ; les durées énormes
n'étaient pas une capacité fiable, mais un chemin d'overflow.
