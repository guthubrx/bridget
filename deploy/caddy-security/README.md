# Caddy : construction de sécurité

Le paquet officiel 2.11.7 corrige les avis Caddy mais embarque Go1.26.8 et x/net0.59.0. Les avis GO-2026-6603/6610/6611/6612/6617, GO-2026-6809, GHSA-mgcc-cww5-fp2q, GO-2026-6865, GHSA-q7gw-24fr-5p69, GO-2026-5932 et GO-2026-6860/6866 demandent des versions plus récentes. Cette construction utilise Caddy 2.11.9 avec Go 1.27.3 et tous les modules standards HTTP/TLS/reverse proxy/logging/metrics, tout en omettant `caddypki`/`acmeserver` (CA locale non utilisée, qui tirait `go-jose/v3`). `go.mod` et `go.sum` figent le graphe sans vulnérabilité OSV.

Depuis ce dossier avec Go1.27.3 : `go build -mod=readonly -trimpath -o caddy .`. Valider la configuration complète du serveur avec le candidat avant installation. Conserver la configuration privée et le binaire précédent, puis vérifier tous les domaines et le flux Bridget après activation. Ne pas remplacer le paquet corrigé par le paquet amont2.11.7 compilé avec Go1.26.8.

Le paquet local porte 2.11.7-1+bridget.security178. Une prochaine version amont devra être rescannée avant remplacement. Les clés et fichiers Caddy privés ne font pas partie du dépôt.
