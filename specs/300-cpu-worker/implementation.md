# Incident CPU worker du 19 novembre 2026

Le worker SYS PID 109486 fonctionne dans l'image 287-role-specialistes-r1.
Le passage à nice 19 du thread consommateur a réussi, sans arrêt ni changement
PID. Les autres threads observés étaient endormis, sans consommation CPU.
Les 44 zombies de la machine ne sont pas tous ses enfants : 1 sous le worker,
37 sous un autre parent et 6 sous un troisième. Aucun processus arrêté.

Cartae : le daemon PID 650292 porte environ 9 Go de mémoire résidente, presque
entièrement privée anonyme, avec de nombreux mappings de 64 Mio. Cette mesure
n'identifie pas les objets alloués et ne prouve pas une fuite. Aucun dump de
mémoire ni appel injecté dans le daemon. Son worker PID 651704 est distinct.

Hypothèse CPU : le suivi consomme du CPU parce que commands, pending_results,
reconcile_all et collect reconstruisent l'inventaire global, y compris les
préparations et leurs réservations, à chaque cycle. Le profil CPU observe ces
fonctions et la désérialisation JSON ; la boucle attend déjà 50 ms.

Régression : les lectures de suivi doivent lire et valider leurs reçus sans
parcourir les archives de préparation indépendantes. Les admissions, le
redémarrage et la collecte conservent le contrôle global. Aucun résultat
invalide, mauvaise identité, symlink ou store en quarantaine ne devient admis.

Qualification en cours : le test de lecture ciblée échoue avant correction puis
passe. Deux tests de régression couvrent les reçus corrompus, le contrôle global
à l'admission et à la collecte, et la cadence du suivi. Sur fixture locale de
1000 préparations, dix cycles de relais prennent 1,100942 s avec les deux anciens
inventaires et 0,004881 s avec les lectures ciblées. Profil debug, même fixture ;
ce gain sur le relais ne mesure pas le CPU global du worker en production.

La campagne worker donne 143 succès, un test de mesure ignoré par défaut et un
échec dans spec172_linked_checkout_prepare_preserves_main (préparation hors
racine avec fournisseur natif). Même échec reproduit sur HEAD aed70ea7 avant correction, dans le test isolé.
La préparation de dépôt n'est pas modifiée par ce lot ; échec préexistant conservé.

Activation : la préparation native bridget300-cpu-20261119 refuse
update_missions_active_or_unconfirmed. Aucun bail acquis, aucun service arrêté.
Ce refus bloque la mesure après correction sur les workers vivants ; nice 19
sur SYS est appliqué. Les objets des 9 Go de Cartae restent non identifiés.

Revue native en lecture seule admise sur le checkout Mac partagé : tâche
80ef0b09-3de0-4ccb-899f-da69b24384a6, clé
bridget-300-worker-cpu-review-20261119-r1. Aucun worktree créé ; le moteur de
routage a choisi le fournisseur. Résultat de revue en attente.

Revue reçue : deux coûts restaient sur les annulations et fins de commande.
Corrections ciblées : la rétention des signaux d'annulation passe avec la
collecte à dix secondes ; la fin de commande ne lit que les slots validés sous
le verrou du store. Tests ajoutés pour annulation et fin de commande malgré
une préparation indépendante illisible. La validation directe du dossier
payload empêche un lien symbolique même lorsque le blob n'existe pas.
Les lectures doubles de reçus sont préexistantes et conservées ; la mesure
est étendue à 359 commandes et 651 slots pour exposer leur coût résiduel.
Aucun résultat de production après activation n'est encore disponible.

Mesure finale après corrections de revue : même fixture locale, 1000
préparations, 359 commandes, 651 slots et dix relais. Ancien parcours :
4,183927 s ; parcours corrigé : 1,333311 s, soit un coût réduit de 68,1 %
(facteur 3,14). Les 5 tests du lot passent, mesure explicite comprise.
Cette fixture reproduit les nombres de reçus, pas les données ni l'activité
exacte de SYS. Elle ne remplace pas une mesure CPU après activation.
Revue lue et accusée par le port natif. Pas de nouvelle revue globale : les
remarques bloquantes ont reçu des corrections et tests ciblés.

Contrôle final : Clippy strict bibliothèque/tests réussi après corrections de
revue. Les profils bruts sont conservés en privé dans
/Users/moi/.cache/bridget300-cpu-memoire (4,4 Mio), hors Git. Les outils perf
extraits temporairement sur SYS (31 Mio) et les copies temporaires de profils
sur les deux serveurs ont été retirés après vérification de leur non-usage.
Aucun paquet système installé, aucun processus arrêté. PID 109486 toujours
présent à nice 19 au dernier contrôle. Aucun worktree ni branche créé.

Limites : la compilation et la mesure Linux du correctif actif restent à faire
lorsque le parcours natif de maintenance sera admissible. Ne pas remplacer un
worker à la main pour contourner update_missions_active_or_unconfirmed. Les
missions et résultats existants restent sous l'autorité de leurs propriétaires.
Le diagnostic mémoire nécessite une attribution des allocations, non fournie
par smaps ; ne pas annoncer une fuite ou un contenu métier sans cette preuve.

Après les corrections de revue, 15 contrôles existants ciblant annulation,
réconciliation et collecte passent également. Format et diff vérifiés. Le
nettoyage automatique Cargo a retiré 10,56 Gio d'intermédiaires au total et
laisse 2,63 Gio de caches. La compilation des tests vérifie le code Rust ;
aucune nouvelle image serveur n'est déclarée qualifiée ou active.
