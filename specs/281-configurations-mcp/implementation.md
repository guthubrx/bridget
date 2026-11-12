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

## Activation, publication et clôture

Le code est poussé au commit 3391784b220d6493edceda269eaaa76c9ad4612e.
Le complément Linux passe : huit tests ciblés par plateforme. Les compilations
finales passent sur Mac et Debian 12 après les corrections de revue. La publication
0.8.46 contient six fichiers téléchargés sans authentification et vérifiés par
empreinte. Les deux archives contiennent les exécutables complets qualifiés.

Le maître, le service mobile et les cinq workers chargent les images 0.8.46.
Le contrôle porte sur les exécutables réellement chargés, avec retour 0.8.45
qualifié et pointeur rollback vérifié par les écrivains natifs. Le M1 garde son
absence préalable d’état de mise à jour ; aucun état n’y est inventé. Le contrôle
mobile répond HTTP 200. La lecture native de la revue réussit après activation.
La coupure du maître mesurée est de 3,63 s ; les wrappers contrôlés conservent
leurs PID et dates de naissance. T3 conserve son PID 77762 et sa date de
démarrage du 11 novembre à 14:10:41. Sa façade MCP déjà montée reste en 266,
sans effet sur ce correctif interne dont le protocole reste inchangé.

Les propriétaires de commandes réellement affectées ont reçu les avis avant et
après le passage, conservés sans réveil. Devantures a reçu le diagnostic E9 ;
Économie a reçu la cause du démarrage avorté et la correction. Aucune de leurs
missions n’a été rejouée, annulée ou remplacée par cette maintenance.

Ménage : 5,77 Gio de caches Cargo supprimés sur le Mac ; 1,80 Gio de release
sur le volume de compilation SYSLE2. Le contrôle d’un descripteur du worker SYS
est ensuite incomplet : les autres caches restent conservés. Les exports SYS
ont pu être retirés lors d’un contrôle distinct complet. Sept images obsolètes
Cartae, deux sur Mac, une sur KS et une sur M1 sont retirées après vérification
d’absence d’usage. Les images actives, le retour et les anciennes images encore
chargées restent conservés. Les exports Cartae dont le contrôle est incomplet
restent conservés, sans arrêt de processus pour nettoyer.

Les exports propres et deux copies de recette reconstructibles sont retirés du
Mac. Le contrôle SpecKit ne trouve aucun worktree ni autre retrait admissible.
L’espace système disponible mesuré après ce passage est de 62 014 107 648 octets
sur le Mac, 127 450 521 600 sur Cartae, 39 552 901 120 sur SYSLE2 et
81 815 572 480 sur KS. L’activité simultanée interdit d’attribuer toutes les
variations à ce seul ménage. Les reçus restent dans
/Users/moi/.cache/bridget281-incidents. Les configurations MCP abandonnées dont
l’absence d’usage n’est pas prouvée ne sont pas purgées.

## Contrôle après reconnexion : continuation Vie des habitants

Les deux refus primary_absent du 12 novembre à 04:54:56.040 et 04:55:17.366
Paris précèdent la commande métier pendant la reconnexion de maintenance.
Le même parent relit ensuite la tâche 76395328-f7d7-493a-aa31-c3904f883df4
à 04:55:47.191 : result_available, ronde 4, continuable=true, error=null.
À 05:00:18.020, la même demande vdh-population-region-r10-r5-20261112-0455
est admise : mission_pending, ronde 5, error=null. Le diagnostic ne rejoue
aucune commande. La présence native actuelle du parent est confirmée.
Les métadonnées assainies, sans corps métier, sont conservées dans
/Users/moi/.cache/bridget281-incidents/vdh-reconnection-observed.json.
Cet épisode transitoire n’appelle ni nouveau correctif ni autre redémarrage.
