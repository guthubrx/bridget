# ADR 001 — Modéliser les demandes Bridget comme un cycle de vie persistant

**🇫🇷 Français** · [🇬🇧 English](001-request-lifecycle.en.md)

**Date** : 2026-08-14  
**Statut** : Accepté

## Contexte

Une demande avec réponse attendue crée aujourd'hui des rappels en mémoire. Si la priorité change, l'émetteur ne peut ni stopper les rappels ni libérer le destinataire de l'obligation de répondre. Un redémarrage peut aussi perdre l'état de cette attente.

## Décision

Une demande suivie reçoit un identifiant et un état durable. L'émetteur peut l'annuler de manière idempotente. Les états terminaux ne sont jamais rouverts et aucune relance ne les concerne.

Les messages ordinaires restent de simples notifications : ils ne créent ni tâche, ni état, ni coût de suivi.

## Conséquences

- La coordination devient explicite : une réponse est requise uniquement pour une demande encore ouverte.
- La base locale contient un petit registre supplémentaire, réutilisant le stockage SQLite existant.
- L'annulation ne promet pas l'arrêt immédiat d'un modèle IA déjà en train de travailler ; elle arrête la contrainte de réponse et les rappels Bridget.
- Les futures évolutions peuvent ajouter une interruption coopérative sans modifier la sémantique de base.
