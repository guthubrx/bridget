# ADR-022 - Bridget Desktop agrège la flotte et conserve la conversation par origine

**Statut** : Proposé  
**Date** : 2026-08-31  
**SPEC liée** : SPEC-081

## Contexte

Desktop mémorise plusieurs profils SSH et tunnels, mais son unique WebView enfant couvre la fenêtre. La navigation et la flotte restent ainsi celles d'un seul relais et l'entrée Serveurs locale devient inaccessible pendant une conversation.

## Décision

Desktop devient la coque de navigation des sources et de la flotte globale. Il lit un instantané borné de chaque tunnel, le réduit à des données non sensibles et rend localement Sources, filtres, tris et épingles.

Le relais local éventuel est découvert au moment de la lecture par `bridget ui endpoint --json`, une commande sans paramètre utilisateur. Il n'est jamais sauvegardé comme profil, ne réclame pas de jeton manuel et se nomme toujours « Cet ordinateur ». En cas d'absence, il n'est pas affiché.

Une seule WebView enfant reste autorisée. Elle est placée à droite de la coque et chargée pour l'agent choisi, via le tunnel de sa source. Le relais utilise `desktop_shell=1` et cache ses colonnes locales pour ne pas dupliquer la navigation.

Les cartes utilisent la paire `source_id, agent_name`. Une action ne peut donc atteindre que son tunnel correspondant. Les préférences de présentation restent locales à Desktop.

## Conséquences

### Positives

- Toute la flotte est visible sans casser l'isolation entre sources.
- Le gestionnaire de serveurs reste accessible en conversation.
- Les homonymes restent distinguables.
- Chemins de projets et secrets de tunnel restent hors de la page Desktop.

### Limites

- Une seule conversation distante est visible à la fois.
- Un serveur sans mode compact est isolé comme incompatible.
- Les coordinateurs historiques sans rôle explicite ne sont pas pré-épinglés.
- Aucun projet multi-serveur ni installation locale ne sont introduits.
