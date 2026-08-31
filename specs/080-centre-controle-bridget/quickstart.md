# Vérification locale - SPEC-080 Centre de contrôle Bridget

1. Construire et lancer le daemon de test avec son relais UI et une politique de racines temporaire valide.
2. Ouvrir le panneau via un profil SSH déjà approuvé ou une fixture relay locale.
3. Vérifier que la roue, seule et lisible, est fixée tout en bas de la barre des agents et ouvre un overlay modal sans remplacer la conversation. Presser aussi `Commande + virgule` et vérifier que le même overlay s'ouvre.
4. Vérifier la colonne Projets sous les boutons macOS : en vue développée, la tuile « Toute la flotte » explicite « Tous projets confondus ». Un projet affiche une tuile initiales/couleur et son menu `…` ou clic droit propose la personnalisation et le retrait. En vue repliée, seules les tuiles restent visibles.
5. Dans Typographie, vérifier le gabarit de réglage : titre et sous-titre à gauche, deux sélecteurs encadrés et alignés à droite, aperçu immédiatement sous la ligne. Les fonds et l'overlay restent ceux de Bridget.
6. Modifier la police, une taille, le retour à la ligne, le thème et le fuseau. Vérifier qu'ils restent locaux et qu'aucune requête de contrôle serveur n'est produite.
7. Lire la clé `project_roots.allowed_roots`, modifier une valeur valide, contrôler le delta, refuser une première fois puis confirmer une seconde fois.
8. Vérifier que le reçu indique la génération attendue et résultante. Rejouer le même `command_id` et vérifier que le même reçu est retourné.
9. Créer une prévisualisation, effectuer une modification concurrente dans une fixture, puis vérifier le refus de génération obsolète sans écriture partielle.
10. Insérer des échantillons usage attestés avec et sans modèle ou tarif, vérifier les libellés Estimation API, Inconnu et Non tarifé.
11. Ne jamais appliquer une mise à jour ni une commande d'hôte pendant cette vérification.
