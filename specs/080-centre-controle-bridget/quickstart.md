# Vérification locale - SPEC-080 Centre de contrôle Bridget

1. Construire et lancer le daemon de test avec son relais UI et une politique de racines temporaire valide.
2. Ouvrir le panneau via un profil SSH déjà approuvé ou une fixture relay locale.
3. Vérifier que l'engrenage est atteignable au clavier, ouvre l'identité du serveur et affiche les badges Ce serveur, Ce projet ou Lecture seule.
4. Lire la clé `project_roots.allowed_roots`, modifier une valeur valide, contrôler le delta, refuser une première fois puis confirmer une seconde fois.
5. Vérifier que le reçu indique la génération attendue et résultante. Rejouer le même `command_id` et vérifier que le même reçu est retourné.
6. Créer une prévisualisation, effectuer une modification concurrente dans une fixture, puis vérifier le refus de génération obsolète sans écriture partielle.
7. Insérer des échantillons usage attestés avec et sans modèle ou tarif, vérifier les libellés Estimation API, Inconnu et Non tarifé.
8. Dans Bridget Desktop, modifier le thème et le fuseau et vérifier que seul le rendu des fenêtres locales change, sans requête ou modification du tunnel.
9. Ne jamais appliquer une mise à jour ni une commande d'hôte pendant cette vérification.
