# Corpus 089 — intégrité acquise, gel encore partiel

Les 13 fichiers de fixtures sont copiés **octet pour octet**, LF compris, depuis le commit source. manifest.json nomme leur chemin dans Git, leur SHA-256, leur famille et le lecteur historique identifié. Aucune re-sérialisation ni normalisation n'a été appliquée.

Depuis la racine de ce worktree :

```sh
sh scripts/verify-089-contracts.sh
sh scripts/verify-089-contracts.sh --self-test
sh scripts/verify-089-contracts.sh --require-complete
```

Les deux premières commandes contrôlent ce qui existe. La troisième retourne actuellement **1**, intentionnellement : elle interdit de déclarer T002 terminée. Six familles restent à matérialiser :

| Famille manquante | Point de départ dans la référence |
|---|---|
| message-idempotency | protocol.rs:3895–3992 et :4680–4743 ; daemon.rs:7524 et test :19423 pour le canon stocké |
| directory-ledger | protocol.rs:4168–4250 et :5286–5345 ; clients réels dans cli.rs:6508+ |
| managed-lifecycle | protocol.rs:4816–4969 et :5128–5181 |
| attach-wire | protocol.rs:4295–4327, :4971–5096, :5228–5265 ; les contenus de journal ne remplacent pas ces trames |
| guichet-claim-reply | protocol.rs:4328–4555 ; la fixture 015 de sept lignes ne couvre pas token/lease/réponse finale |
| provider-native-wire | codex_app_server.rs:5235 interdit jsonrpc sur le fil ; la fixture historique de forme en contient |

Les références protocol.rs désignent crates/bridget-transport/src/protocol.rs ; daemon.rs et cli.rs désignent crates/bridget-daemon/src/. Les numéros de ligne valent au commit épinglé.

## Ce que vérifie l'auto-test

Une copie temporaire privée reçoit les mutations ; aucun fichier du dépôt ni objet Git n'est modifié. Le même vérificateur refuse : un octet altéré ; son empreinte réécrite pour masquer l'altération ; une source Git absente ; une fixture absente ; une déclaration de gel complet privée d'une famille requise. La relation d'ancêtre est testée avec deux commits existants dans le sens invalide, pas seulement avec un SHA inventé.

Le contrôle d'intégrité n'est pas une preuve de conformité du fournisseur. En particulier, provider-codex-0.150.1.jsonl est une fixture historique de **forme** ; son lecteur ne vérifie que des méthodes et des capacités construites localement. Elle ne peut pas servir de recette d'app-server ni remplacer une capture réelle. Les 65 tests protocol:: de la référence sont verts ; ils ne matérialisent pas à eux seuls tous les octets de leurs constructions typées.

## Suite de T002

Matérialiser les sorties du codec de référence avec provenance de génération explicite, les faire lire par des consommateurs réels et verrouiller leur émission avec des goldens indépendants. Ne pas modifier le protocole pour le faire correspondre à une fixture périmée. Tant que cela manque, garder la revue avant suppression ouverte.
