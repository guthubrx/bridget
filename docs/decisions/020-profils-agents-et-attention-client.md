# ADR-020 - Profils d'agents séparés et attention par client

**Date**: 2026-08-31
**Statut**: Proposé
**SPEC liée**: SPEC-078

## Contexte

Bridget utilise aujourd'hui le nom d'agent comme adresse de routage, clé de
flotte, clé d'historique et libellé UI. Cette superposition interdit un surnom
stable et sûr: renommer la chaîne technique peut déplacer une reprise ou perdre
un lien de conversation. L'apparence n'est aujourd'hui qu'une préférence de
navigateur. Les notifications sont déduites de la page, donc ne survivent pas à
une reconnexion et sont trop limitées pour rendre compte d'une flotte.

Bridget Desktop est une coque Tauri qui charge l'UI commune via un tunnel dont
le port local change. Une préférence `localStorage` dépendante de cette origine
ne peut donc pas définir correctement une préférence de notification Desktop.

## Décision

Créer dans la SQLite du daemon:

1. un registre d'identités opaques associé aux noms de routage actifs et
   historiques;
2. un profil serveur partagé par identité, avec display name, labels, apparence
   et instructions versionnées;
3. un journal sémantique d'attention; et
4. un état de préférence et de lecture scoped par client UUID stable.

Le profil ne modifie jamais le nom technique. Toutes les surfaces utilisateur
utilisent le display name. Les routes UI transportent une référence opaque
nécessaire à leur fonctionnement mais ne l'affichent pas.

Les instructions sont des consignes individuelles, pas des personas. Elles sont
injectées uniquement à une naissance/reprise fournisseur confirmée et restent
inférieures aux règles Bridget, permissions et mandats.

Le Web partage le panneau et le centre d'activité fournis par le relais.
Bridget Desktop conserve un client_id stable dans son répertoire de données et
assure seul la notification macOS via une capacité Tauri explicite. La WebView
relayée n'obtient aucun privilège Tauri.

## Conséquences

### Positives

- L'utilisateur peut changer le nom affiché sans migration de messages ni
  risque sur la flotte.
- Web et Desktop convergent sur les propriétés réellement communes.
- Deux appareils n'imposent jamais leur bruit de notification l'un à l'autre.
- Les événements sont rattrapables et dédoublonnés après une reconnexion.
- Un fournisseur sans mise à jour de contexte active ne produit pas un faux
  succès: le statut reste en attente jusqu'à sa prochaine naissance.

### Coûts et limites assumés

- Nouvelles tables SQLite, migration et projection de compatibilité.
- Un client_id est nécessaire malgré l'absence de comptes, afin de préserver
  l'autonomie de deux appareils.
- Le plugin Tauri officiel est une dépendance supplémentaire, justifiée pour ne
  pas exposer Tauri à la WebView et éviter une commande macOS ad hoc.
- Deux agents anciens ayant exactement le même nom historique ne peuvent pas
  être distingués rétroactivement par le ledger actuel.
- L'application totalement arrêtée ne reçoit pas de notification native. Le
  journal permet seulement le rattrapage à la relance.

## Alternatives écartées

- Étendre `fleet.json`: trop couplé au lifecycle et incompatible avec les agents
  non gérés ou historiques.
- Renommer le champ `name`: il est une adresse de protocole, non une propriété
  de présentation.
- Créer une seconde UI de profils dans Tauri: elle dupliquerait le relais et
  contredirait la frontière de sécurité de SPEC-074.
- Déduire les alertes des flux d'outils: bruit, pertes à la reconnexion et pas
  de déduplication fiable.
- Employer une persona réutilisable: hors périmètre de cette décision.
