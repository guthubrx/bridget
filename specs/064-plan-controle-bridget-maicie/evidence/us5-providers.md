# Preuve US5 - fournisseurs par contrat

Date de validation : 2026-08-29.

## Matrice prouvee

| Fournisseur | Chemin | Contrat exerce | Resultat |
| --- | --- | --- | --- |
| Codex | app-server | interruption et operations attestees | accepte seulement apres observation |
| Claude | stream-json | interruption, autorisation, EOF et evenement inconnu | refus ou repli type, jamais infere |
| Cursor | ACP commun avec binaire reel | initialisation v1, session, prompt et `session/cancel` | terminal `cancelled` recu en 3 ms, aucun adaptateur Cursor separe |
| Inconnu | fixture | version ou capacite absente | refus sur |

## Oracles executes

```text
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-2 /home/moi/.cargo/bin/cargo test -p bridget-transport --test provider_contract_test
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-2 /home/moi/.cargo/bin/cargo test -p bridget-daemon --test capabilities_integration_test -- --nocapture
```

Verdict : 2 succes transport et 3 succes daemon. Les controles couvrent identite fournisseur, autorite, cible, generation, thread et tour incoherents.

## Execution Cursor ACP reelle - T070

Commande isolee :

```text
/home/moi/.local/bin/cursor-agent --mode ask acp
```

- Binaire : `/home/moi/.local/bin/cursor-agent` ; version `2026.08.25-3e8eec8` ; SHA-256 `2ccc9a8e167797641448b5e5c936f006ba137a2555f117f38c5eb76a5238a233`.
- Dossier de travail : `/tmp/bridget-spec064-cursor-acp.DQjTwb`, sans agent ni daemon Bridget de production.
- `initialize` v1 : envoi `18:39:31.143Z`, reponse v1 `18:39:31.536Z`.
- `session/new` : envoi `18:39:31.540Z`, session creee `18:39:33.621Z`.
- `session/prompt` lecture seule : envoi `18:39:33.625Z`, premier evenement de tour `18:39:34.410Z`.
- `session/cancel` : envoi `18:39:34.412Z`, terminal `stopReason=cancelled` `18:39:34.415Z`, soit 3 ms.

`session/cancel` annule le tour mais ne ferme pas un serveur ACP, ce qui est
conforme a son role. Le processus de test et ses descendants ont ensuite ete
arretes par SIGTERM simple, avec accord humain, sans signal force. Aucun
contenu de conversation de production, secret ou modification de workspace
nest inclus dans cette preuve.
