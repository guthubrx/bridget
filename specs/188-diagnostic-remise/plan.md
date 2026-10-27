# Plan
1. Extraire une vue empruntée des lectures locales existantes, sans dupliquer les règles ; conserver les signatures runtime par délégation.
2. Ajouter diagnostic local CLI protégé du plafond de délégation, avec tuple complet, lecture des fichiers privés, vérification de l’enrôlement et contrôle de stabilité avant/après.
3. Réutiliser la liste fermée des codes de permissions et rendre une étape utile, sans transposer un diagnostic local en preuve d’autorité.
4. Tester sur un store déjà verrouillé : aucun octet changé, refus identité étrangère, préparation absente, lecture instable, permissions et ancrage Git refusés. Rejouer les contrôles worker existants concernés.
5. Revue indépendante, candidat via script de build, exécution sur le cas réel sans services puis livraison source/candidat. Chargement du maître reste distinct et non autorisé par cette correction.
Complexité : O(b) pour le reçu borné et les lectures de configuration existantes, sans inventaire global ni sonde réseau nouvelle. Le rattachement vivant lit le tableau noyau des verrous (borne2Mio), puis les seuls argv/environnement du PID détenteur vérifié.
