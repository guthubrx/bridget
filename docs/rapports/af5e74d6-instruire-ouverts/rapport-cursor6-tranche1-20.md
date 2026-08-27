# Tranche 1 — 20 constats non instruits

Objectif `af5e74d6` · base `08c0bd8` · IDs extraits : `/tmp/bt/cursor6-tranche1-ids.txt`  
Pied au moment : **168 ouverts / 69 blockers** · catalogue non touché

Légende : **VERIFIE** / **NON REGLÉ** / **RESULTAT-OU-PROCESS** (pas un dû code) / **SUPPOSE** (reste ouvert)

| # | ID exact | Statut | Preuve / note |
|---|---|---|---|
| 1 | `review_amender:constat/agent-federe-ni-lisible-ni-recreable` | NON REGLÉ VERIFIE | Même famille fédération ; aucun lot cycle-de-vie distant trouvé. |
| 2 | `constat/is-control-ne-voit-pas-les-caracteres-bidi` | NON REGLÉ VERIFIE | `registry.rs` / `daemon.rs` utilisent encore `char::is_control` (Cc seulement). Bidi Cf passe. |
| 3 | `constat/la-liste-blanche-ne-borne-pas-le-contenu-de-ses-champs` | DEJA REGLÉ VERIFIE | `provider_fingerprint` opaque dans `codex_app_server.rs` (introduit `6b390ab`, sur main). Oracles `test_019_*_est_opaque`. |
| 4 | `constat/un-nom-d-agent-avec-saut-de-ligne-fabrique-un-message-dans-le-ledger` | NON REGLÉ VERIFIE | Même motif que validate-agent-name : filtre CLI/MCP partiel ; rendu ledger sans échappement ; Register socket non aligné. |
| 5 | `constat/une-panique-attendue-s-affiche-dans-un-test-qui-passe` | RESULTAT-OU-PROCESS | Faute de lecture du référent, déjà « mesuree et fermee » dans le texte — nature resultat/leçon, pas défaut code. Reclasser, ne pas `fermer` comme dû. |
| 6 | `review_amender:constat/disculpation-de-redelivery-repose-sur-un-instrument-aveugle` | NON REGLÉ (SUPPOSE partial) | STOP sur bancs-instables ; pas de preuve de levee du mutant « instrument aveugle » retrouvée ici. Reste ouvert. |
| 7 | `constat/la-carte-de-reprise-est-injectable-et-c-est-un-prompt` | NON REGLÉ VERIFIE | `managed_resume_context` fait encore `lines.join("\n")` (`wrapper.rs:180`) ; nom non échappé. |
| 8 | `constat/f38-voie-2-doit-elire-sur-le-chemin-complet-pas-le-nom-nu` | NON REGLÉ | Défaut de conception F38/carte criticité ; pas de preuve de correctif chemin complet instruit ici. |
| 9 | `constat/garde-fou-du-regime-promis-a-midi-et-non-tenu-l-apres-midi` | RESULTAT-OU-PROCESS | Promesse humaine non tenue ; pas un bug code. |
| 10 | `constat/sept-agents-en-echec-silencieux-comptes-occupes` | PARTIEL | Echéance 600→2700 livrée (fermetures liées) ; la confusion OCCUPE vs BLOQUE / busy permanent peut rester. Pas de preuve bout-en-bout de la composition ici → reste ouvert pour la propriété « compte occupé ». |
| 11 | `constat/le-daemon-perime-n-a-jamais-bloque-les-merges` | RESULTAT-OU-PROCESS | Aveu d'erreur du référent ; le défaut était une fausse cause, pas un code à corriger. |
| 12 | `constat/une-meme-propriete-violee-produit-un-faux-refus-ici-et-un-faux-accord-la` | SUPPOSE reglable | Merge 025 `0252558` sur main ; **non rejoué** l'oracle Linux d'évasion ici → je ne ferme pas mentalement. Reste ouvert jusqu'à mesure. |
| 13 | `constat/le-tour-qui-n-aboutit-pas-rend-l-agent-definitivement-non-mandatable` | NON REGLÉ | Composition busy+refus+échéance ; pas d'issue « busy expire / mandat empilé » vérifiée. |
| 14 | `constat/un-mandat-envoye-en-notification-se-lit-comme-informatif` | NON REGLÉ (process+produit) | Défaut de forme reply=no ; parade `--reply` est procédure, pas preuve que Maicie force une remise non facultative. |
| 15 | `constat/un-agent-qui-demande-l-autorisation-d-emettre-reste-muet` | RESULTAT-OU-PROCESS | Mode de défaillance agent ; pas un trou code démontré comme fermé. |
| 16 | `review_amender:constat/deux-charges-du-meme-jure-ne-peuvent-pas-etre-corrigees-independamment` | NON REGLÉ | Problème d'orchestration de jury ; pas de lot dédié trouvé. |
| 17 | `constat/open-mute-l-original-sur-une-base-au-ddl-incomplet` | SUPPOSE partiel | Code `store.rs` distingue base neuve vs peuplée à user_version 0 ; **non mesuré** le cas « open mute l'original » ici. Reste ouvert. |
| 18 | `constat/le-raisonnement-des-agents-arrive-deja-et-nous-le-jetons` | DEJA REGLÉ (partiel) VERIFIE | L4 ACP `cd9cfee` + Codex `c024fc5` sur main : `agent_thought_chunk` → `event=reasoning`. **Borne** : Cursor / autres familles non vérifiées ici. |
| 19 | `constat/mcp-contourne-la-grammaire-canonique-des-identites` | PARTIEL VERIFIE | `mcp_identity::valid_name` exige alphanumeric/`-`/`_` (rejette LF). **Mais** `is_alphanumeric` Unicode accepte encore des lettres non-ASCII ; et Register socket peut rester plus large. Ne pas fermer comme « totalement réglé ». |
| 20 | `constat/lien-distant-tombe-agents-vivants-mecanisme-inconnu` | NON REGLÉ | Incident réseau/lien ; mécanisme toujours « inconnu » dans le texte ; rien de livré d'instruit. |

## Candidats fermeture (pour le référent) dans cette tranche

Seulement ceux à preuve nette de **fond** :
- `constat/la-liste-blanche-ne-borne-pas-le-contenu-de-ses-champs` — `sha` de merge session-19 / commit `6b390ab` (préférer le merge exact sur main si tu fermes).

À ne **pas** fermer comme dû (reclasser nature) :
- panique-attendue, daemon-perime, garde-fou-regime, agent-autorisation-emettre.

## Semblent réglés mais non (ajouts tranche 1)

- `constat/is-control-ne-voit-pas-les-caracteres-bidi` — toujours `is_control`.
- `constat/la-carte-de-reprise-est-injectable-et-c-est-un-prompt` — toujours `join("\n")`.
- `constat/mcp-contourne-la-grammaire-canonique-des-identites` — LF filtré, Unicode large encore possible.
- `constat/le-raisonnement-…` — ACP/Codex OK, pas attestation trois familles.

## Suite

Tranche 2 = 20 suivants parmi les ~136 restants (après exclus déjà instruits + cette tranche).
