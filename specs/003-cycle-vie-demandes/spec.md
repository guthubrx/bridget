# Spécification : cycle de vie des demandes Bridget

**Branche** : `session-03-cycle-vie-demandes`  
**Créée** : 2026-08-14  
**Statut** : Implémenté et validé  
**Input** : Permettre à un agent d'annuler une demande antérieure afin d'arrêter les relances et d'éviter les réponses devenues inutiles.

## Scénarios utilisateurs et tests

### US1 — Annuler une demande en attente (Priorité : P1)

Un agent qui a demandé une réponse peut annuler cette demande lorsqu'elle n'est plus utile, par exemple après un changement de priorité ou parce qu'un autre agent a terminé le travail.

**Pourquoi cette priorité** : évite du travail inutile et les relances coercitives qui parasitent la coordination.

**Test indépendant** : créer une demande nécessitant une réponse, l'annuler puis vérifier que le destinataire est prévenu et qu'aucun rappel ni timeout ne suit.

**Scénarios d'acceptation** :

1. **Étant donné** une demande ouverte émise par un agent, **quand** cet agent l'annule avec son identifiant, **alors** la demande devient annulée et les relances cessent immédiatement.
2. **Étant donné** une demande annulée, **quand** le destinataire reçoit l'information, **alors** il voit explicitement qu'aucune réponse n'est requise.
3. **Étant donné** une demande ouverte, **quand** un autre agent tente de l'annuler, **alors** l'annulation est refusée sans modifier son état.

---

### US2 — Conserver un état fiable après redémarrage (Priorité : P1)

Un opérateur peut consulter l'état final d'une demande et le daemon ne relance jamais une demande déjà répondue, annulée ou expirée, y compris après son redémarrage.

**Pourquoi cette priorité** : une relance erronée après un redémarrage détruirait la confiance dans le protocole.

**Test indépendant** : enregistrer des demandes dans plusieurs états, redémarrer le daemon puis vérifier que seuls les états ouverts peuvent encore générer un rappel.

**Scénarios d'acceptation** :

1. **Étant donné** une demande ayant reçu une réponse, **quand** le daemon redémarre, **alors** elle reste terminée et ne génère aucun rappel.
2. **Étant donné** une demande annulée, **quand** le daemon redémarre, **alors** elle reste annulée et ne génère aucun rappel.
3. **Étant donné** une demande encore ouverte, **quand** le daemon redémarre avant son délai, **alors** son état reste consultable et sa prochaine relance respecte le délai restant.

---

### US3 — Distinguer demande suivie et notification (Priorité : P2)

Un agent peut savoir si un message ordinaire ne demande aucune suite, tandis qu'une demande suivie expose un identifiant et un état consultable.

**Pourquoi cette priorité** : le protocole ne doit pas transformer toute communication en tâche administrative.

**Test indépendant** : envoyer une notification et une demande suivie, puis vérifier que seule la seconde apparaît comme ouverte et annulable.

**Scénarios d'acceptation** :

1. **Étant donné** une notification sans réponse attendue, **quand** elle est livrée, **alors** aucune demande suivie n'est créée.
2. **Étant donné** une demande suivie, **quand** l'émetteur liste ses demandes, **alors** il obtient l'identifiant, le destinataire, l'état et le délai restant.

### Cas limites

- Une annulation répétée de la même demande est sans effet supplémentaire et retourne son état terminal.
- Une annulation après réponse, expiration ou annulation ne réouvre jamais la demande.
- Une réponse arrivée après annulation est livrée comme information si son destinataire existe, mais ne modifie pas l'état annulé.
- Un identifiant inconnu ou inaccessible ne révèle pas de détail sur les demandes d'autres agents.
- Si le destinataire est déconnecté lors de l'annulation, l'état reste annulé et aucune relance ne reprend à sa reconnexion.

## Exigences

### Exigences fonctionnelles

- **FR-001** : le système DOIT donner un identifiant unique à chaque demande pour laquelle une réponse est attendue.
- **FR-002** : le système DOIT conserver l'émetteur, le destinataire, la date de création, le délai et l'état de chaque demande suivie.
- **FR-003** : l'émetteur d'une demande ouverte DOIT pouvoir l'annuler en indiquant son identifiant, avec un motif optionnel.
- **FR-004** : le système DOIT refuser l'annulation par un agent autre que l'émetteur.
- **FR-005** : l'annulation DOIT arrêter immédiatement les rappels et notifications de dépassement associés.
- **FR-006** : le destinataire connecté DOIT recevoir une notification explicite d'annulation ne demandant pas de réponse.
- **FR-007** : le système DOIT conserver les états terminaux `answered`, `cancelled` et `timed_out` de façon durable afin qu'un redémarrage ne les rouvre pas.
- **FR-008** : le système DOIT empêcher qu'une réponse ultérieure modifie un état terminal.
- **FR-009** : le système DOIT permettre à l'émetteur de lister ses demandes suivies et leur état actuel.
- **FR-010** : une notification sans réponse attendue NE DOIT créer aucune demande suivie.
- **FR-011** : l'annulation répétée d'une même demande par son émetteur DOIT être idempotente.
- **FR-012** : chaque wrapper DOIT publier l'OS de son hôte lors de son enregistrement ; l'annuaire et `bridget who` DOIVENT l'exposer de façon lisible.

### Entités principales

- **Demande suivie** : unité de travail créée lorsqu'une réponse est attendue ; porte un identifiant, les deux agents, un délai et un état.
- **Événement de cycle de vie** : trace d'une création, réponse, annulation ou expiration associée à une demande suivie.

## Critères de succès

### Résultats mesurables

- **SC-001** : 100 % des demandes annulées ne génèrent plus de rappel ni de timeout durant une fenêtre de test supérieure à leur délai initial.
- **SC-002** : 100 % des annulations effectuées par un autre agent sont refusées et la demande conserve son état antérieur.
- **SC-003** : après redémarrage du daemon, 100 % des demandes terminales créées par le test restent terminales.
- **SC-004** : un opérateur peut identifier l'état et le destinataire d'une demande ouverte en une commande.
- **SC-005** : `bridget who` permet d'identifier l'hôte et l'OS de chaque agent connecté sans désaligner ses colonnes.

## Hypothèses

- Une demande suivie correspond à l'actuel usage `--reply`; les messages ordinaires gardent leur comportement sans état.
- La première version ne cherche pas à interrompre physiquement un processus IA déjà engagé : elle annule l'obligation protocolaire de répondre et les relances associées.
- La conservation utilise la base locale déjà employée par Bridget et respecte sa politique de rétention existante.
- Les identités Bridget connectées constituent le périmètre d'autorisation initial ; aucune gestion d'identité réseau supplémentaire n'est ajoutée.

## Dépendances

- `002-federation-ssh` fournit la présence durable et la reconnexion utilisées par les notifications d'annulation.
