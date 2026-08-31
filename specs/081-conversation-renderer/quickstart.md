# Vérification manuelle - SPEC-081

## Préparation

1. Ouvrir Bridget Desktop puis un serveur déjà approuvé.
2. Dans les réglages Bridget Desktop, vérifier que les trois réglages de
   contenu sont visibles et désactivés après réinitialisation.
3. Ouvrir un fil fixture qui contient plusieurs tours, un tableau, deux
   blocs de code, un lien HTTPS, un chemin de projet et une image HTTPS.

## Fil de conversation

1. Vérifier que la demande humaine est une bulle alignée à droite.
2. Vérifier que la réponse agent est un document aligné à gauche, sans grande
   bulle identique à celle de l'humain.
3. Développer puis replier les activités d'un tour. Le résumé reste visible,
   l'ordre du fil ne change pas et le brouillon est conservé.
4. Remonter dans l'historique, provoquer une nouvelle activité et vérifier
   que le fil ne revient pas automatiquement en bas. Utiliser le rappel de
   retour au direct.

## Contenu technique

1. Vérifier le titre ou le langage du bloc de code, sa coloration dans le
   thème choisi et le bouton Copier.
2. Copier un bloc, puis vérifier que le presse-papiers contient le texte brut
   exact.
3. Faire varier le retour à la ligne d'un bloc sans modifier le réglage des
   autres blocs.
4. Copier un tableau et vérifier le Markdown ou CSV annoncé par le contrôle.

## Sécurité de contenu

1. Avec les trois réglages désactivés, vérifier qu'aucun lien ni image ne se
   charge et qu'une explication concise est rendue.
2. Activer seulement les liens externes. Vérifier qu'un lien HTTPS exige un
   clic et ne navigue pas automatiquement.
3. Activer les références de fichiers. Vérifier qu'un chemin sous une racine
   projet ouvre un aperçu borné, puis qu'un chemin hors racine est refusé.
4. Activer les images. Vérifier qu'une image HTTPS est chargée seulement à
   l'affichage, sans référent et sans accepter SVG, `file:`, `data:` ou
   `javascript:`.
5. Redémarrer Bridget Desktop. Vérifier que les choix de l'opérateur sont
   conservés. Réinitialiser les préférences puis vérifier le retour aux trois
   valeurs désactivées.

## Régression sécurité

1. Insérer du Markdown contenant `<script>`, `javascript:`, une image `data:`
   et une ancre HTML à événement. Vérifier l'absence d'exécution et de
   navigation.
2. Vérifier qu'aucun réglage de contenu n'apparaît dans les requêtes relay,
   dans un profil SSH ou dans les logs serveur.
