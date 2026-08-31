# Parcours de validation - SPEC-076

Ce parcours sert après livraison des dépendances. Il ne déclenche aucun
déploiement.

## Préconditions

1. SPEC-065, SPEC-066, SPEC-067 et SPEC-075 sont intégrées sur une même main.
2. Une politique de racines valide est chargée et attestée.
3. Une configuration complète de coordinateur est compatible.
4. Les fixtures ne contiennent aucun secret réel.

## Création

1. Ouvrir réglages, vérifier les racines actives.
2. Créer sous une racine un dossier inédit.
3. Vérifier chemin final, option Git, configuration et durée.
4. Confirmer puis vérifier dossier seul et dépôt Git vide si choisi.
5. Vérifier identité, coordinateur unique et étapes dans la conversation.
6. Vérifier premier rapport sans nouvel agent ni fichier mémoire.

## Import et reconnexion

1. Importer un Git modifié, constater avertissement et absence de modification.
2. Importer un dossier non Git, confirmer ou refuser Git explicitement.
3. Rechoisir le même dossier, constater ouverture sans doublon.
4. Déplacer une fixture, constater path_missing, reconnecter explicitement.
5. Vérifier l'historique et l'absence de déplacement automatique.

## Retrait et sécurité

1. Retirer un projet avec coordinateur actif, constater arrêt propre puis retrait.
2. Vérifier conservation dossier, Git, conversation et historique.
3. Réactiver le même dossier, constater la même identité, une liaison active et
   exactement un audit appliqué.
4. Tenter chemin hors racine, lien sortant et racine large : refus avant effet.
5. Vérifier que découverte initiale ne crée ni fichier, commit ni agent.
