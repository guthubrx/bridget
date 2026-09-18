# Registre des dépréciations

Ce registre recense les chemins maintenus temporairement pendant une migration.
Une entrée est supprimée avec son chemin dès que la condition indiquée est
atteinte.

| Chemin | Remplacé par | Supprimable quand |
|---|---|---|
| Champ fil `AgentInfo.rate_limit` (Option mono-fenêtre) | `AgentInfo.rate_limits` (Vec par fenêtre) ; lecture wire : l'ancien champ se relit en Vec d'un élément | Dès que tous les consommateurs who/UI/MCP lisent `rate_limits` |
| Liste blanche de binaires dans `crates/bridget-daemon/src/wrapper.rs` | Registre `~/.config/bridget/agents.json` | Supprimée en T703 |
| Affichage stderr sans transport dans `crates/bridget-daemon/src/wrapper.rs` | Livraison structurée ACP ou log applicatif | Supprimé en T705 |
| Micro-banc SC-005 ignoré dans `crates/bridget-transport/src/acp.rs` | Banc à deux vues attach réelles dans `crates/bridget-daemon/tests/sc005_attach_budget.rs` | Après suppression de la couverture historique ACP au prochain cycle de nettoyage des tests |
| Pont npm `@zed-industries/codex-acp` et `@zed-industries/claude-code-acp` (lancement via `npx` / binaire du paquet) | Pilotes natifs `codex app-server` et `claude_stream_json` (défauts registre + ADR 010) | **Retiré le 2026-08-24 (G10)** : refus `SpawnRefusal::EnvUnfit` sur **tout** chemin d'entrée (spawn géré + `--equipier` / `launch_acp_with_status`), après normalisation basename + retrait `@version`. L'ACP générique **reste** pour Cursor (`cursor-agent acp`) et Gemini (`gemini --acp`). |
| Protocole ACP comme chemin d'investissement pour Codex/Claude | Ponts natifs uniquement pour nouvelles capacités (modèle, quota, coût) | Dès G10 : aucune nouvelle capacité ne passe par ACP pour ces deux fournisseurs. ACP reste le chemin vivant de Cursor/Gemini ; retirable le jour où Gemini (et éventuellement Cursor) exposent un accès natif, ou sortent du périmètre. |

Relecture T711 : ces deux chemins hérités sont les seuls retirés par la
migration ACP ; ils restent étiquetés ici jusqu'à la suppression de leur
référence historique.

Relecture G10 (2026-08-24) : le pont Zed n'était pas une couche de code
distincte — c'était le protocole ACP générique pointé vers des paquets npm
tiers figés. Le retrait consiste donc à (1) garder les défauts natifs déjà
posés pour `codex`/`claude`, (2) refuser explicitement les paquets Zed au
spawn géré **et** au wrapper `--equipier`, (3) laisser `acp.rs` et le chemin
ACP pour les fournisseurs qui le maintiennent. Les specs 007/009/010 qui
documentent l'ancien pin npm restent en archive historique et ne sont pas
réécrites.

Durcissement revue G10 (2026-08-24) : la comparaison porte sur le basename
normalisé (chemins absolus / relatifs, suffixe `@version`) pour que
l'annonce « refuse toute définition » soit vraie aussi pour le style vivant
du projet (`/opt/homebrew/bin/codex-acp`, etc.).

Recensement AVANT retrait (G10, 2026-08-24, tête main `3dcca2c`) — aucun
appelant vivant du pont dans le code de lancement ni dans
`~/.config/bridget/agents.json` (seule entrée : `cursor-agent` + `acp`).

| Lieu | Nature | Décision |
|---|---|---|
| `registry.rs` `default_agents()` | Déjà `codex app-server` / `claude_stream_json` | Conservé |
| `acp.rs` + protocole ACP | Couche générique Cursor/Gemini (ADR 010) | Conservé |
| Fixtures le service compagnon `codex-acp` / `claude-code-acp` | Définitions de test encore zed | Remplacées par les pilotes natifs |
| `daemon.rs` test runtime géré `npx`+`codex-acp` | Fixture ACP, pas un pont | Remplacée par `fixture-acp` |
| `npx` + `fixture-acp` (fleet, cli, parity, protocol) | Adaptateur ACP de test | Conservé (pas Zed) |
| `managed_process` / `mcp_identity` tests `npx` | Cycle de vie du groupe, pas le paquet | Conservé |
| Specs 007/009/010, catalogue, ADR 010 (contexte) | Archive historique du pin npm | Conservée, non réécrite |
| `attach.rs` noms `claude-acp`,`codex-acp` | Libellés d'erreur, pas une commande | Renommés pour ne plus coller au pont |

Relecture T911 : la gestion du cycle de vie par le daemon est additive. Le
lancement historique par wrapper-terminal reste supporté et aucun nouveau
chemin n'est déprécié par la session 009.

Relecture T1305 : le rendu compact de `attach` est sélectionné uniquement sur
stdout TTY. Le rendu historique non-TTY reste le chemin de compatibilité et
n'est pas déprécié ; aucune nouvelle entrée n'est donc ajoutée au registre.
