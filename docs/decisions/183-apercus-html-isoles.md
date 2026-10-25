#183 — Aperçus HTML interactifs isolés

Décision du25octobre2026. Bridget ouvre les documents autonomes dans une vue dédiée. Le document généré est non fiable, y compris ses scripts, balises, liens et formulaires. La session mobile et ses clés ne sont jamais transmises au moteur HTML.

## Frontière

Un document hôte fixe porte une iframe srcdoc échappée intégralement. Son sandbox autorise seulement les scripts. L'absence de allow-same-origin donne une origine opaque. Le document enfant reçoit une CSP avant tout contenu : réseau, sous-cadres, travailleurs, formulaires, bases et polices distantes refusés. Les scripts et styles locaux sont permis ; images et médias intégrés en data seulement. L'hôte n'écoute aucun message de l'enfant et ne contient aucun état utilisateur.

Le WebView refuse réseau, fichiers, fournisseurs de contenu, stockage DOM, cookies tiers, téléchargements, fenêtres, navigations et permissions. Il n'a aucune interface JavaScript native. Les dialogues de script sont annulés. Le moteur doit satisfaire la version corrigée déjà requise par Bridget. Une version ancienne ou inconnue n'obtient que le code et le guidage de mise à jour. Le mode statique conserve son refus des scripts. Le bouton Fermer appartient à l'interface native et reste hors de la page.

## Arbitrage et réutilisation

Le WebView statique existant reste le mécanisme de rendu. Aucun serveur HTML, nouveau composant de navigateur ou bibliothèque ajouté. Activer JavaScript dans le document principal ne séparerait pas le contenu de son hôte ; cette option est écartée. L'iframe opaque ajoute une frontière de document, complétée par les refus natifs. Les diagrammes, formulaires et compteurs autonomes fonctionnent ; une bibliothèque chargée depuis un CDN doit être intégrée au document. Aucun proxy réseau implicite.

## Risques et oracles

Tentatives à vérifier sur le moteur installé : fermeture de srcdoc, lecture/écriture du parent, stockage, session/native bridge, fetch/XHR/WebSocket, images distantes, file/content, navigation, popup, permission, fichier choisi et téléchargement. L'absence de requête utile et les erreurs capturées doivent être observées ; une simple comparaison de chaînes ne suffit pas. Vérifier aussi compteur cliqué0->1, retour au fil et fermeture d'une page défaillante. Une boucle de script peut rendre son moteur inutilisable ; la fermeture native et la source restent les voies de récupération, sans prétendre borner le CPU d'un document arbitraire.

Sources officielles : https://developer.android.com/privacy-and-security/risks/insecure-webview-native-bridges et https://web.dev/articles/sandboxed-iframes . La décision n'élargit ni les droits du téléphone ni ceux des agents.
