# ADR 044 — Relais de sous-agent sous l'autorité du parent

- **Date** : 2026-10-04
- **Statut** : Accepté
- **Session** : 133

## Contexte

Un sous-agent interne peut voir Bridget sans posséder une identité indépendante.
Lui donner l'identité complète du parent autoriserait aussi les contrôles, le
journal, les artefacts et Maicie. Créer une identité par enfant ajouterait une
boîte de réception, un cycle de vie et des agents orphelins.

## Décision

Le pont publie une preuve enfant privée et éphémère. La façade MCP l'accepte
uniquement pour consulter l'annuaire et envoyer un message. Le message garde le
parent comme identité routable et porte une provenance enfant visible. La CLI et
les autres outils refusent cette preuve. Les réponses reviennent au parent.

## Conséquences

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/044-relai-sous-agent-sous-autorite-parent.md#extrait-1).
