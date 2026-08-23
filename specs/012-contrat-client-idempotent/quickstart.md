# Quickstart : valider le contrat client idempotent

Prérequis : daemon avec socle 012, un équipier ACP (007) comme destinataire.

## 1. Cycle nominal avec ID choisi

```bash
SCOPE=$(cat ~/.config/monclient/scope)   # opaque, généré une fois par le client
bridget send --to codex-1 --id 018f3c... --issued-at 2026-08-22T21:00:00Z --issuer-scope "$SCOPE" "délégation X"
# → Accepted { id, expires_at }
```

## 2. Retry après perte d'accusé (le cœur)

Relancer la même commande (mêmes trois valeurs) : résultat identique rejoué,
**une seule** livraison au destinataire (vérifier au journal 007 : un seul
`turn_start`). Redémarrer le daemon entre les deux : comportement inchangé.

## 3. Divergence et refus typés

Même `--id` avec un corps modifié d'un octet → `EnvelopeMismatch` ;
`--issued-at` vieux au-delà de l'horizon → `IdempotencyExpired` ; futur
au-delà de la tolérance → `InvalidIssuedAt` ; `--id` sans `--issuer-scope` →
`invalid_params`.

## 4. Lookup

`bridget lookup --issuer-scope "$SCOPE" --id 018f3c...` → l'un des quatre
résultats, avec `expires_at`. Pendant un envoi en vol : `OutcomeUnknown`.

## 5. Deux clients, même UUID

Deux scopes différents, même `--id`, même destinataire → deux livraisons
distinctes (compteur de prompts = 2), retries de chacun sans supplément.

## 6. Robustesse aval

Scénarios automatisés (matrice SC-001) : crash daemon à chaque point
(avant réservation / après `Prepared` / après remise avant issue / après
issue avant accusé client), redélivrances → comptage des frames
`session/prompt` lues par le faux adaptateur. `rename` du destinataire avant
redélivrance → `DeliveryIndeterminate`. Corruption ciblée d'un reçu `Acked` →
quarantaine, zéro réinjection.

## 7. Non-régression historique

`bridget send` classique (sans `--id`) : comportement 007 inchangé, zéro
`idempotency_record` créé (sonde) ; suite complète 007/008 au vert.
