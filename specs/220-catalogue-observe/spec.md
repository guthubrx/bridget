# Session220 — modèles observés par compte et serveur

Demande utilisateur autorisée : actualiser les modèles après changement de compte sur chaque serveur, après la comparaison219. La preuve publique doit séparer configuration, authentification, modèles annoncés et accès effectif au prochain tour. Aucun changement de compte, de droits ou de modèle automatique. Aucune mission métier ou qualification de modèle nécessaire à la découverte.

## Résultat attendu

Le propriétaire lit le catalogue natif et voit pour chaque connexion le serveur, le programme, l’état observé et sa date, la liste annoncée par ce programme, ses limites de fraîcheur et la prochaine action. Une ancienne observation ne vaut pas preuve actuelle. L’observation du compte reste opaque, sans adresse ni secret exposés. Les tâches admises et leurs canons ne sont pas réécrits.

Le cas initial est Codex sur Cartae, SYS et KS. Les autres adaptateurs qui ne disposent pas d’une découverte fiable restent explicitement non vérifiés. Cela n’interdit pas leurs parcours existants ; ne pas prétendre que tous les fournisseurs sont observés depuis le seul support Codex. Les connexions volontairement désactivées ne sont jamais réactivées par une observation.

## Invariants

- Le worker observe uniquement ses connexions et son contexte de lancement configuré. Aucun chemin ou environnement fourni par le lecteur du catalogue.
- Découverte sans thread/start, turn/start, login, logout ou copie de secrets. Identité propre de Bridget.
- Lecture account/read, model/list paginé et compte relu après la liste ; refus si les observations divergent. Une liste cache récente ne prouve pas l’accès payant.
- Objets typés et bornés, capacité négociée, identité/époque de connexion contrôlées. Les anciens pairs gardent leurs trames historiques.
- Une seule découverte en vol par worker, hors de sa file sérialisée de missions. Temps, sortie, nombre de modèles/pages et processus bornés. Pas de saturation par sondes répétées à chaque tick.
- Le worker est responsable de la fraîcheur locale ; le maître corrèle, invalide et projette. Le client MCP affiche les faits, sans les produire.
- Changement de configuration ou signal non secret de connexion : invalider l’observation puis refaire une découverte. Au redémarrage, à la reconnexion et sur échéance bornée : refaire le contrôle. Pour un login externe sans signal fiable : nommer la limite et proposer un rafraîchissement explicite.
- La liste fournisseur et la politique d’admission restent distinctes. Pas d’élargissement silencieux des modèles permis par un profil ; toute restriction configurée demeure visible. L’admission ne doit pas considérer un modèle précédemment absent comme autorisé par une simple déclaration de succès.
- Une panne, une source partielle ou un arrêt de sonde non confirmé restent inconnus et visibles. Aucun ancien choix présenté comme frais.
- Les messages nomment la connexion et le serveur, la portée, l’action et la décision humaine nécessaire.

## Vérification et livraison

Compte présent/absent/changé pendant lecture, changement externe, cache ancien, programme manquant/ancien, pagination et volume hostiles, coupure/reconnexion/époque ancienne, concurrence, arrêt de sonde, anciens pairs, absence de mutation des tâches et des droits. Réutiliser le décodage model/list si utile sans changer select_runtime. Mise à jour des guides FR/EN. Revue indépendante. Builds via scripts/build.py, distribution et activation sur les hôtes concernés, contrôle des images et du lien natif. Aucun rejeu métier ou changement de connexion KS pour qualifier ce travail.
