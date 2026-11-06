# Correction 239 — maintenance visible depuis tous les projets

François demande de corriger le catalogue de maintenance. Le rôle actuel est
limité au projet Bridget ; les autres projets ne peuvent ni le découvrir ni
vérifier son mandat. Le mainteneur doit être découvrable et joignable par rôle
depuis chaque projet, sans destinataire fixe dans les procédures.

Utiliser les ports natifs configure_mandate puis declare_roles. Le mandat est
limité à la maintenance de Bridget ; la portée transversale n'ajoute aucun
droit sur les fichiers, mémoires métier, comptes ou permissions des projets.
Conserver auto_start=false, durable=false, les budgets existants et les tâches.

La modification permanente exige la décision humaine native corrélée à son
canon. Aucun champ forgé, SQL direct ni contournement du contrôle humain.
Ne pas redémarrer T3. Poser une question visible si son rechargement devient
nécessaire. La configuration actuelle permet une application sans redémarrage.
