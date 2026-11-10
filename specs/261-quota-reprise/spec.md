# 261 : actualisation du quota de la session conservée

Correction demandée par François le 10 novembre 2026. Tests ciblés seulement.

Le pilote ne relisait le quota qu'au démarrage. Une ancienne fenêtre épuisée
restait bloquante après un reset fournisseur si aucune notification n'arrivait.

Lire account/rateLimits/read sur le même processus natif au repos, au plus une
fois toutes les trente secondes, avec le délai existant de deux secondes.
Le lecteur existant publie la réponse dans l'ordre, vers la même instance.
Un échec conserve les faits précédents, sans disponibilité inventée ni boucle.
Aucun tour, changement de compte, de modèle ou de droits. Les gardes de reprise
et de dépense restent inchangées. Les pilotes déjà chargés doivent être mis à
jour par une maintenance native qualifiée ; aucune injection dans leurs pipes.

Vérifications : reset100→0 puis100 ; même PID, aucune inférence, cadence ;
erreur de lecture sans faux0 ; régression de la garde scoped de l'enfant.
Sources sur main, livraison et activation vérifiées, conservation des fichiers.
