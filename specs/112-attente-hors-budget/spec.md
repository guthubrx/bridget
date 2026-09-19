# Spécification 112 — L'attente en file ne consomme pas le budget de tour

## Fiche synthèse
Spec: 112-attente-hors-budget | Statut: Implemented (livré 2026-09-19) | Priorité: P1 | Date: 2026-09-19
Branche: session-112-attente-hors-budget | Dépend de 111 (la file attend la fin du tour).

## Problème observé
Après la 111, une remise n'est plus jetée au bout de deux minutes. Mais le daemon pose sur chaque
remise poussée une échéance de tour, `deadline_at = poussée + notify_timeout` (2 700 s pour Codex),
conçue comme budget d'exécution du tour. Le pont l'appliquait aussi AVANT dispatch : un destinataire
occupé plus de 45 minutes perdait donc encore ses messages. Constaté le 2026-09-19 sur `horizon-3D`,
en tour continu depuis 16h14 : cinq remises en attente, la plus ancienne condamnée à 16h52.

## Exigences
- **FR-001** : avant dispatch, une remise n'est écartée pour échéance de tour que si cette échéance
  était déjà passée à sa réception par le pont.
- **FR-002** : au dispatch, le temps passé en file est reporté sur `deadline_at`, pour l'enveloppe
  injectée et les contrôles ultérieurs : le budget court à partir du dispatch effectif.
- **FR-003** : `reply_timeout` (attente déclarée par l'expéditeur), l'expiration de saga et l'état
  d'une demande suivie restent des motifs d'écartement inchangés.
- **FR-004** : aucun changement de protocole ni côté daemon.

## Critères de succès
- **SC-001** : une remise avec budget de 45 min reçue il y a une heure est toujours en file, et son
  échéance reportée conserve le budget entier.
- **SC-002** : une remise déjà périmée à la réception est écartée.
- **SC-003** : recette complète verte ; après livraison, les remises en attente pour `horizon-3D`
  survivent au-delà de 45 minutes d'attente.
