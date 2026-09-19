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

T3 Code fournit, sous licence MIT, un bon modèle de présentation : tours
structurés, réponses d'assistant sans bulle lourde, activités repliables,
Markdown GFM, blocs de code titrés et contrôles de copie. Son implémentation
React et son environnement local ne peuvent pas être copiés comme tels dans le
panneau HTML statique de Bridget.

## Décision

1. Le journal Bridget reste l'autorité des tours. Un adaptateur de projection
   pur construit des tours stables sans changer la remise ni inventer un nouvel
   protocole agent.
2. Les réponses agent sont des documents à gauche, les demandes humaines des
   bulles concises à droite. Les actes, erreurs, durées et raisonnements
   attestés deviennent un résumé repliable rattaché à leur tour.
3. Le Markdown reste analysé par `marked` puis assaini par DOMPurify. Les
   éléments enrichis sont ajoutés après assainissement avec une liste blanche
   étroite. Aucun HTML agent ne devient exécutable.
4. Les préférences `liens externes`, `références de fichiers` et `images`
   sont séparées, versionnées, désactivées par défaut et locales au client.
   Dans Bridget Desktop, le store Tauri est l'autorité : le WebView de relais
   ne reçoit qu'un instantané en lecture injecté au démarrage. Il ne possède
   aucune commande Tauri d'écriture de ces réglages.
5. Les liens externes exigent un geste explicite. Dans Bridget Desktop, le
   lien est transmis à un schéma local `bridget-open:` que la coque intercepte
   puis revalide avant de demander l'ouverture HTTPS au navigateur système ;
   le WebView distant ne reçoit aucune capability Tauri. Les références de
   fichiers ne donnent accès qu'à un aperçu relayé, en lecture seule, après
   clic et seulement si le chemin canonique appartient aux racines de projet
   déjà autorisées. Les images distantes n'emploient que HTTPS,
   `referrerpolicy` `no-referrer`, chargement différé et une taille bornée. Les
   formats actifs et les URI `data:`, `javascript:` et `file:` ne sont jamais
   rendus.
6. Les extraits T3 repris textuellement ou adaptés gardent un commentaire de
   provenance et l'avis MIT de T3 Tools Inc. est ajouté aux notices tierces.
   Le reste reprend ses principes, pas son architecture React.

## Conséquences

### Positives

- L'opérateur obtient une lecture proche de la clarté de T3 sans donner au
  contenu distant une permission native ou un droit d'exécution.
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
- Porter la pile React de T3 Code : elle ajouterait une chaîne de construction
  et des dépendances incompatibles avec l'UI statique existante, sans bénéfice
  proportionné.
