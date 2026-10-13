# Données SPEC-161

Aucune nouvelle base, table, migration ou donnée métier. Le modèle décrit uniquement les preuves de livraison.

## Source approuvée

source_commit complet160, source_build_id12caractères, manifest_sha256 et liste de chemins/empreintes transférés. Validation : checkout propre, fichiers correspondant à la référence approuvée ; chaque fichier admissible transféré vérifié avant Cargo build. Les exclusions historiques (.git, target, DB, sockets, liens non suivis) sont explicites dans le manifest et ne dissimulent aucun fichier de compilation requis.

## Outil exécuté

runner_commit, runner_sha256, empreinte de federate-ssh.sh sourceable et commande sans secret. Ces valeurs ne remplacent jamais source_commit. Documentation_commit et ref distante sont relevés séparément.

## Livraison cliente

machine=cartae ; préfixe neuf ; binary_path ; binary_sha256 ; architecture ; mode ; identité source rapportée ; toolchain ; jobs=2 ; nice=10 ; codes retour et horodatages. État : préparée → compilée → candidat vérifié → active → recette validée. Une erreur avant active préserve la référence précédente. Une erreur après active ouvre un rollback conditionnel.

## Référence active et rollback

link_path=/home/moi/.local/bin/bridget ; previous_target ; previous_binary_sha256 ; installed_target ; observations avant/après. Refuser si lien inattendu, cible étrangère ou changement concurrent. L'ancienne cible096 reste conservée. Rollback : active → restaurée seulement si le lien est encore celui installé par161 ; sinon état intervention requise, sans écrasement tiers.

## Reçu durable

Preuves Git, disque et réseau distinctes ; empreintes des manifests et résultats de tests ; identités/naissances des processus protégés ; candidats cleanup avec décision, raison et chemin exact. Jamais de token, clé privée ni contenu de conversation. Aucun statut « Livré » sans vérification du push, de l'activation, des recettes et du nettoyage applicable.
