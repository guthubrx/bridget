# Livraison 170 R2 — clôture des faux délais distants

## État

Correctif prêt pour ROOT dans le worktree. Aucun commit, push, activation ou
changement de production effectué par cet agent.

## Cause et comportement

Avant : `capture_remote_result` pouvait accepter et envoyer un résultat distant
sans transition de `tracked_requests`; l’ACK transport était le seul chemin de
`answered`, donc `delivery_acked=false` laissait les rappels expirer.

Après : un reçu `Accepted` corrélé au couple mission/ronde/owner/enfant appelle la
transition canonique `mark_answered`. La transition reste distincte de
`delivery_acked`. Après commit, `wait_epoch` et `pending_replies` sont réveillés
uniquement si la ligne passe réellement de `open` à `answered`. L’entretien
réessaie depuis les demandes SQL ouvertes avant de lire `pending()`, y compris
si la tâche native a déjà `cleanup_done=true`. Les refus, conflits, propriétaires
étrangers, reçus d’une autre mission/ronde et vraies expirations ne clôturent
pas. Le collecteur bloque les rappels pendant une panne SQL transitoire, puis le
tick suivant reprend la clôture.

## Preuves

Tests ciblés : 49 PASS (46 régressions `remote162_`, 3 oracles `remote170_`).

SHA-256 :

```text
fde53ecb8eea879e3f2e0c202417ab63ce7341c19fad91599114abb68dfc0fb5  crates/bridget-daemon/src/daemon.rs
72de37e9e8e57053a6ac15bebe0ccc0ddab919832d5e4264494007b1106bd195  crates/bridget-daemon/src/daemon/native_delegation.rs
66c9a1fec33dc5564366504c8fed9dc68c530ad94fa846736697dd354006ce67  specs/170-delais-reponses-distantes/spec.md
adad5109e1a681e274bce3302254c0273885a55e8fdea1524aa6886d12e7104b  specs/170-delais-reponses-distantes/plan.md
81d74a7401ac52ebd7f3cf5a593c9d10eb0ca485c5dbb86170fa4ef572f43677  specs/170-delais-reponses-distantes/tasks.md
c9e09c386cc997677c20e02777ec78cfd1b3f2bc61fcb47b8bf0532d2732f725  specs/170-delais-reponses-distantes/validation/tests.md
66c9a1fec33dc5564366504c8fed9dc68c530ad94fa846736697dd354006ce67  specs/170-delais-reponses-distantes/spec.md
adad5109e1a681e274bce3302254c0273885a55e8fdea1524aa6886d12e7104b  specs/170-delais-reponses-distantes/plan.md
7f24ad9af9e50d0e5741e4bc3ce54c8cc8e8b210e81347f665b011630651bb6e  specs/170-delais-reponses-distantes/tasks.md
6f2653b07002aa240efd5870e14d118896c6556ab6bc62430d5d35a513f32a63  specs/170-delais-reponses-distantes/validation/tests.md
ee30cbd6150614b19425d2941180bef3df4bcc75454e65b607d6861dcf73906b  /home/moi/.local/state/bridget-delivery170/diagnostic.md
```

## Limites

`RemoteResultEnvelope::validate` refuse un corps vide. Le cas « résultat vide
valide » n’est donc pas déclaré PASS, car l’extension de `bridget-transport`
dépasserait le périmètre mono-fichier demandé.
