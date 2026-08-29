# Preuve US5 - fournisseurs par contrat

Date de validation : 2026-08-29.

## Matrice prouvee

| Fournisseur | Chemin | Contrat exerce | Resultat |
| --- | --- | --- | --- |
| Codex | app-server | interruption et operations attestees | accepte seulement apres observation |
| Claude | stream-json | interruption, autorisation, EOF et evenement inconnu | refus ou repli type, jamais infere |
| Cursor | ACP commun | session/cancel via AcpTransport | aucun adaptateur Cursor separe |
| Inconnu | fixture | version ou capacite absente | refus sur |

## Oracles executes

```text
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-2 /home/moi/.cargo/bin/cargo test -p bridget-transport --test provider_contract_test
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-2 /home/moi/.cargo/bin/cargo test -p bridget-daemon --test capabilities_integration_test -- --nocapture
```

Verdict : 2 succes transport et 3 succes daemon. Les controles couvrent identite fournisseur, autorite, cible, generation, thread et tour incoherents.

## Limite nommee

Aucun executable Cursor ne fut configure sur hote de test. Cursor est donc prouve au niveau ACP commun et fixture contractuelle, pas par demarrage binaire externe. Codex et Claude ne furent pas lances contre conversations de production.

## Verification du binaire externe

Le 2026-08-29, `command -v cursor-agent` et `command -v cursor` ont retourne
absence sur le serveur de test. Le parcours T070 ne peut donc pas etre execute
sans installer ou declarer un binaire Cursor reel. Cette limite ne remet pas en
cause la preuve du transport ACP commun, mais interdit de cocher T070.
