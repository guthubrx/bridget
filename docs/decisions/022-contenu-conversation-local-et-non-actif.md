# ADR-022 - Contenu de conversation local et non actif

## Statut

Accepté - 2026-08-31

## Contexte

Les messages Bridget proviennent d'agents et restent non fiables. Le panneau
de conversation est servi par le relais distant à travers un tunnel SSH, alors
que les réglages de l'opérateur appartiennent au Mac et sont déjà conservés
par Bridget Desktop. Autoriser directement ce panneau distant à modifier des
préférences de sécurité locales lui donnerait le pouvoir de s'ouvrir sa propre
surface de liens, fichiers et images.

Le fil présente des tours structurés, des activités repliables, du Markdown GFM
et des blocs de code avec copie. Ces choix répondent au besoin de lecture des
conversations Bridget dans un panneau statique.

## Décision

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/022-contenu-conversation-local-et-non-actif.md#extrait-1).

## Conséquences

### Positives

- L'opérateur lit des tours explicites. Le contenu distant ne reçoit aucune
  permission native ni aucun droit d'exécution.
- Les chemins de fichiers de serveurs reliés restent bornés par la politique
  de racines existante et ne deviennent pas un explorateur de fichiers.
- La même préférence reste durable entre panneaux macOS et ne dépend pas du
  port loopback temporaire du tunnel.

### Négatives

- L'aperçu de fichier nécessite une route relay spécialisée, des limites de
  taille et des refus explicites.
- La coloration syntaxique doit être embarquée localement pour rester
  disponible hors ligne, avec son avis de licence.
- Le panneau navigateur sans Tauri conserve son store local distinct : il ne
  peut pas recevoir l'identité d'un Mac ni ses préférences natives.

## Alternatives écartées

- Donner au WebView relayé les commandes `preferences_save` Tauri : un serveur
  ou un contenu compromis pourrait activer ses propres garde-fous.
- Laisser tous les liens et images actifs puis demander une confirmation au
  cas par cas : l'image peut déjà provoquer une requête réseau avant toute
  confirmation.
- Ouvrir un shell, Finder ou éditeur distant pour les chemins reçus : cela
  élargit Bridget au-delà d'une consultation explicitement bornée.
- Ajouter une pile de rendu incompatible avec l'interface statique existante,
  sans bénéfice proportionné.
