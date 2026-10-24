# Restes transport après déconnexion — 24 octobre 2026

## Gigue de reconnexion (LIFE-JITTER)

Défaut confirmé par lecture de `reconnect_delay` dans `crates/bridget-daemon/src/wrapper.rs`.
Le plafond de 30 s était appliqué après le jitter. Les délais au-dessus de 30 s étaient donc ramenés à 30 s exactement.

Reproduction légère hors Git (`/home/moi/.cache/bridget-174-remaining/traces/reconnect_jitter.py`, 100 000 graines par tentative) :

| Tentative | Ancien : part égale à 30 000 ms | Nouveau : part égale à 30 000 ms | Moyenne ancienne | Moyenne nouvelle |
|---|---|---|---|---|
| 0 à 4 | 0 % | 0 % | inchangée | inchangée |
| 5 et plus | 50,3 % | environ 0,02 % (1 chance sur 6 001) | 28,5 s | 27,0 s |

Correction : la fenêtre ±20 % est bornée au plafond avant le tirage. Le backoff exponentiel, la borne haute de 30 s et la borne basse de 800 ms sont conservés.

Effet : en panne longue, la moyenne des délais baisse d'environ 1,5 s. Les délais ne se regroupent plus sur 30 s.

## Mutations WakeRequest et ArtifactPublish

Défaut confirmé par lecture de `crates/bridget-daemon/src/communication/client.rs` et `crates/bridget-daemon/src/mcp.rs`.
Ces deux envois utilisaient `exchange`. Une coupure après écriture pouvait donc afficher une erreur de connexion ou un refus, au lieu d'une issue inconnue.

Correction :
- `wake_request` utilise `send_then_wait` pour `Schedule`, `Cancel` et `CliOnce`. `List` reste en `exchange`.
- Une coupure après écriture rend `outcome_unknown`. Le conseil demande de relire la liste des réveils et de conserver l'identifiant et les paramètres d'origine.
- `execute_publish_artifact` utilise `send_then_wait`. Le pont dynamique ne rend plus `publication Bridget refusée` pour `outcome_unknown`. Le conseil demande de conserver la même clé d'idempotence.
- Aucune reprise automatique n'est ajoutée.

## Tests ajoutés (non exécutés)

- `wrapper.rs` : `reconnect_delay_plafonne_sans_regrouper_la_gigue`.
- `communication/client.rs` : `spec174_wake_coupure_socket_reelle_sans_rejeu_ni_lecture_modifiee`. Socket Unix réel, Disconnect puis EOF, une mutation et une lecture.
- `mcp.rs` : `spec174_publication_coupure_apres_ecriture_reste_issue_inconnue`. Socket Unix réel, Disconnect puis EOF, pont dynamique.

## Référence T3

Copies lues dans `/home/moi/.cache/bridget-174-remaining/t3`. `supervisor.ts` utilise une échelle fixe (3, 4, 8, 16 s) sans gigue. Ce fichier ne fournit donc pas de modèle de gigue à reprendre.

## Verdict

Correction écrite et relue. Aucune compilation, aucun test et aucun rustfmt ne sont lancés dans ce lot. Validation attendue par ROOT sur le cache unique.

## Limites

- Tests ajoutés mais non exécutés. Le comportement reste à confirmer par ROOT.
- Aucune coupure réelle n'a été capturée. Les tests simulent Disconnect et EOF.
- La publication n'a pas d'outil de relecture par clé d'idempotence. Le conseil se limite donc à conserver la clé.
- Pour une mutation, toute erreur d'écriture rend `outcome_unknown`, même si aucun octet n'a été écrit. Ce choix vient de `transmit` et reste inchangé. Il est conservateur.
- Les processus MCP déjà ouverts gardent l'ancien code jusqu'à rechargement.
- Fichiers non modifiés : `native_delegation.rs` et `daemon.rs`.
