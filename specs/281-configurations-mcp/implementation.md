# Observations 281

Le refus E9 du 12 novembre à 04:00:54 Paris porte la corrélation
92c44cea-8d73-4b9d-8bc4-71538a81d79f. La sonde KS a pris 50 819 ms,
avec quatre contrôles de 13 188, 12 294, 12 678 et 12 657 ms. Le budget
est de 45 s. La fin de sonde transforme alors les candidats admissibles en
resource_limit. La durée interne de chaque contrôle n’est pas ventilée.
Aucune cause CPU, RAM ou file pleine n’est déduite. E9 et H1 ne sont pas rejoués.

Le reçu worker de ce6d9d08-50cb-44de-99e8-48bd0cad2454 est terminal,
native_ready=false, startup_failure=exited_before_ready, terminal_exit_code=0.
Le diagnostic privé de 137 octets indique que le fichier MCP de l’instance
7b0d4426-67cd-4dd2-8061-97322e6acc09 est absent. La purge du démon retire
tous les fichiers mcp-*.json sans preuve de propriété ni d’arrêt. Le fournisseur
peut en avoir besoin avant même son inscription. Son garde RAII nettoie déjà
son propre fichier lors de la fermeture normale. Les données métier ne sont
ni reproduites dans les tests ni remises à un autre fournisseur.

Le premier montage de tests a copié des fichiers de test de main qui dépendaient
de travaux réservés absents du runtime cumulatif. Il n’a pas compilé. Le montage
corrigé applique seulement les deux nouveaux tests à la recette 280 vérifiée.

La revue 8b85bda7 valide la logique et la garde propriétaire. Une ligne vide
doublée est retirée et les deux références documentaires à la purge sont corrigées.
La reproduction avant correction échoue bien sur le fichier absent et sur
l’erreur générique inchangée ; les deux tests passent après correction sur Mac
et Debian 12. Le test existant de nettoyage propriétaire et les quatre tests
du moniteur de démarrage passent sur Mac. Le premier essai du montage corrigé
avait encore hérité du TMPDIR long de l’application ; le lancement qualifié
retire les variables de liaison et utilise le dossier temporaire court existant.

Qualification : huit tests ciblés passent sur Mac (deux régressions, un garde
propriétaire, un témoin de politique, quatre tests du moniteur). Les deux
régressions et le témoin passent sur Debian 12 ; le complément est en cours.
Clippy strict workspace/all-targets et les premières compilations release
passent sur les deux plateformes. La compilation finale suit les seules
corrections de commentaires et de ligne vide. Linux exige glibc 2.34, sous
le plafond 2.36. La recette cumulative reproduit les huit fichiers sans fuzz.
Android 0.8.46/8046 est signé avec la même identité que 0.8.45.
