# Parcours de validation - SPEC-078 Profils d'agents et notifications

## Préconditions

- Relais Bridget compilé avec les migrations SPEC-078.
- Deux clients reliés au même relais: navigateur Web et Bridget Desktop.
- Un agent Claude ou GLM/DeepSeek, un agent Codex et un agent Cursor/ACP si ces
  fournisseurs sont disponibles dans l'environnement de test.
- Une tâche contrôlée qui peut demander une information humaine, réussir et
  échouer de façon terminale sans commande destructive.

## 1. Migration et identité visible

1. Démarrer le daemon avec un ledger contenant un agent historique et au moins
   un agent géré.
2. Ouvrir la liste des agents dans Web puis Desktop.
3. Vérifier que chaque agent affiche un nom lisible et aucune chaîne UUID, nom
   de routage ou clé de persistence.
4. Renommer un agent dans le panneau de profil.
5. Recharger les deux clients puis redémarrer le daemon.

Résultat attendu: le nouveau display name est conservé partout, la conversation
historique et le cycle de vie sont inchangés. Tenter le même nom pour un second
agent est rejeté sans perte de formulaire ni révélation de son identité.

## 2. Labels et apparence partagés

1. Saisir `coordinateur, recherche` dans l'éditeur de labels puis confirmer.
2. Ajouter par Entrée un troisième label, puis ressaisir `recherche` avec des
   espaces superflus.
3. Choisir une forme et une couleur existantes.
4. Vérifier les pastilles dans la liste, l'en-tête et le centre d'activité.
5. Rechercher chacun des labels et le display name depuis le second client.

Résultat attendu: trois pastilles distinctes, aucune pastille vide ou dupliquée,
et exactement la même bouille dans les deux clients.

## 3. Consignes individuelles et fournisseurs

1. Ouvrir le profil d'un agent inactif et enregistrer une instruction courte.
2. Vérifier que le panneau affiche une révision et `en attente de relance` ou
   `appliquée` selon l'état réel.
3. Démarrer ou relancer l'agent, puis lui confier une tâche sûre permettant de
   reconnaître la consigne.
4. Contrôler que le verdict devient appliqué seulement après la naissance
   fournisseur confirmée.
5. Modifier la consigne pendant un tour actif.

Résultat attendu: le panneau ne promet pas une prise d'effet rétroactive, le
prochain démarrage applique la nouvelle révision, et les diagnostics, argv,
logs de test et messages publics ne contiennent pas le texte de la consigne.
Répéter pour Claude/GLM/DeepSeek, Codex et ACP/Cursor disponibles.

## 4. Événements d'attention et clients séparés

1. Client Web: activer attente humaine et échec pour un agent.
2. Client Desktop: n'activer que tâche terminée pour le même agent.
3. Provoquer successivement une attente humaine, une fin de tâche, un échec
   terminal, puis un outil et plusieurs chunks de streaming.
4. Placer chaque client en arrière-plan au moment approprié.
5. Ouvrir le centre d'activité et cliquer un événement.

Résultat attendu:

- Web reçoit seulement ses deux événements choisis, Desktop seulement la fin de
  tâche.
- Chaque occurrence produit au plus une notification native locale.
- Le centre et le badge gardent les éléments non lus, même si la permission
  système est refusée.
- Outil, commande, chunks et statuts routiniers ne changent ni centre ni badge.
- L'activation ouvre l'agent concerné sans déplacer le focus avant l'action de
  l'utilisateur.

## 5. Reconnexion, ports et extinction

1. Avec Desktop, reconnecter un profil afin que le port loopback du tunnel
   change.
2. Vérifier que les préférences Desktop et les événements non lus restent
   identiques.
3. Quitter complètement Bridget Desktop, provoquer un événement, puis relancer
   l'application.
4. Refaire un événement déjà reçu après une reconnexion du relais.

Résultat attendu: pas de promesse de notification pendant l'arrêt complet, mais
l'événement est rattrapé au redémarrage. Le même événement n'est jamais notifié
deux fois après reconnexion ou changement de port.

## 6. Accessibilité et non-régression

1. Utiliser Tab, Maj+Tab, Entrée, Échap et les flèches pour ouvrir, éditer et
   fermer le panneau, le picker d'avatar et le centre d'activité.
2. Vérifier qu'un clic sur la bouille ouvre le profil tandis que les trois
   points ouvrent toujours les actions SPEC-077.
3. Contrôler à 200 % de zoom et à 1280 x 720 pixels.
4. Avec un lecteur d'écran, vérifier l'annonce polie des nouvelles entrées sans
   vol de focus.
5. Lancer les suites Rust, Node et Desktop prévues au plan, puis
   `git diff --check`.

Résultat attendu: aucun menu contextuel régressé, aucune information interne
exposée, aucune erreur de layout bloquante, et toutes les suites ciblées passent.
