# ADR-170 — Passage à minreq 3 et remplacement de serde_yaml

**Date :** 2026-10-24. **Statut :** Accepté par ROOT dans la session 174.

## Contexte

L'audit 174 signale quatre dépendances ouvertes. Trois viennent du client HTTP `minreq` 2.14.1 : `rustls` 0.21.12, `rustls-webpki` 0.101.7 et `webpki-roots` 0.25.4. La quatrième est `unsafe-libyaml` 0.2.11, tirée par `serde_yaml`. Aucune version corrigée n'existe sur les lignes 0.21, 0.101 et 0.25. La correction passe donc par un changement de version majeure.

L'ADR-163 fixe le client `minreq` 2.14.1 comme seule frontière HTTP. Cette ADR fait évoluer ce pin. La frontière reste la même : un seul client, `https-rustls` seul, aucun mode TLS permissif.

## Décision

1. `minreq` passe de `=2.14.1` à `=3.0.0`, avec `default-features = false` et la seule feature `https-rustls`.
2. `serde_yaml` est retiré. `serde-saphyr` 2.0.0 le remplace, avec `default-features = false` et la seule feature `deserialize`. L'avis RUSTSEC-2024-0449 recommande cette alternative.
3. Le chargement du pin appelle `serde_saphyr::from_str_with_options` avec `reject_unsupported_tags: true`. Les fonctions externes ne sont pas compilées : `!include` (feature `include`), `${VAR}` (feature `properties`). Une balise `!include` est donc refusée.
4. Les doublons de clés restent refusés, et le budget de parsing reste celui par défaut. Ces valeurs viennent de la documentation de `serde-saphyr` 2.0.0.
5. Les appels HTTP gardent leurs protections : `send_lazy`, `with_max_redirects(0)`, les plafonds d'en-têtes et de ligne de statut, et un seul appel en vol. Le décideur n'utilise pas `with_timeout`, comme avant.
6. La garde `MINREQ_TIMEOUT` reste en place. `minreq` 3.0.0 lit toujours cette variable (`src/connection.rs`).

## Adaptations de code

- Le code de statut est désormais un `u16`. Le champ `HttpReply::status` du décideur et le variant `ContractError::Http` passent à `u16`.
- Les en-têtes sont une liste de paires `(String, String)`, sans normalisation des noms. Le contrôle de `Mcp-Session-Id` lit donc la liste sans tenir compte de la casse. Une réponse qui contient deux valeurs de cet en-tête est refusée.

## Conséquences

Positives :

- `rustls` 0.23.46, `rustls-webpki` 0.103.16 et `webpki-roots` 1.0.11 sont résolus. Ces versions dépassent les seuils corrigés des avis RUSTSEC-2026-0104, 0098, 0099, 0381 et 0382.
- `unsafe-libyaml` et `serde_yaml` disparaissent du graphe.

Négatives et limites :

- Le fournisseur cryptographique par défaut de rustls 0.23 passe par `aws-lc-rs`. Il ajoute `aws-lc-sys`, qui compile du C à la construction. Les machines de build doivent disposer d'un compilateur C, et de `cmake` selon la cible.
- L'analyseur YAML change. Son comportement sur des cas limites peut différer de `serde_yaml` 0.9. Les tests de pin couvrent le cas légitime et le cas refusé, pas toute la grammaire YAML.
- Les tests ciblés, le parcours distant complet et le scan des dépendances sont consignés dans la validation de la session 174. Ils ne constituent pas une validation de toute la grammaire YAML.

## Alternatives écartées

- Garder `minreq` 2.14.1 et accepter le risque : écarté, car ROOT a tranché pour la migration.
- Feature `https-rustls-probe` (magasin de certificats du système) : écartée, car la liste de confiance dépendrait de chaque machine.
- Features `https-native-tls` ou `https-openssl` : écartées, car elles ajoutent une seconde pile TLS, interdite par l'ADR-163.
- Forks de `serde_yaml` : écartés, car l'avis RUSTSEC-2024-0449 indique qu'ils sont touchés par le même problème.

## Validation prévue

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/170-minreq-3-et-serde-saphyr.fr.md#extrait-1).
