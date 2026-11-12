# 278 : défaut fonctionnel établi pendant la qualification277

Après activation277, le log mesure douze résolutions vides sur124,3s,
2278octets, onze identités et au plus deux événements par identité.
Quatre causes sont origin_worker_unavailable. Après reconnexion vérifiée
des workers, les quatre agents sont encore busy et project:null :
556cc099/e61c966a/373c4ac5/dc0fefcc. La première annonce n’a donc pas été
reprise. Les résultats et commandes natives restent conservés.

Le wrapper ne réannonce son cwd qu’à sa reconnexion et ignore le verdict
ProjectContextResult. Le daemon ne gardait pas le fait en échec. Il conserve
maintenant un seul fait en mémoire par connexion native pour les refus transitoires
origin_worker_unavailable et origin_proof_busy. Un heartbeat authentifié du worker actuel ouvre
un lot de lectures asynchrones ; le lecteur du canal reste libre de recevoir
la preuve. Un garde libère le créneau même si le thread ne peut pas démarrer.
La fin d’une connexion retire son fait. Aucun stockage durable ni wire nouveau.

La même fonction d’annonce revalide commande, propriétaire/instance, worker,
root et révocations. Le fait attendu est comparé avant et après la RPC.
Une annonce plus récente ou une tâche devenue invalide ne peut pas être
écrasée par une réponse tardive. Seule la lecture de projet est reprise :
aucun prompt, envoi métier, lancement, nouveau modèle ou droit ajouté.
Le petit point d’entrée d’origine est conservé pour les nombreux appelants ;
le paramètre de fait attendu reste exclusivement sur la voie interne de reprise.

Le test278 et les fixtures253/254 exercent le retour par heartbeat, sans
changement des octets de la Task. Ils refusent un mauvais propriétaire,
puis simulent une annonce plus récente pendant la RPC et une annulation.
La régression d’avant est celle observée sur les quatre sessions réelles277.

Le premier essai Cargo a détecté une modification trop large de version dans
le lockfile : encoding_rs0.8.42 ne doit pas devenir0.8.43. Le lock est restauré
puis seules les trois entrées bridget sont changées. Aucun checksum n’est
contourné. Le test278 passe sur Mac après correction.


R1 demande de fermer un ABA, de conserver busy et de purger on_unregister.
Une révision monotone par annonce d’origine distingue deux faits identiques.
Le même fait conserve sa révision pendant une reprise ; la comparaison avant
et après preuve refuse donc une réponse calculée avant la dernière annonce.
Le test appelle désormais le parcours public pendant la RPC et vérifie que
seule la révision distingue les deux faits. Le refus origin_proof_busy conserve
le fait ; le lot s’arrête alors jusqu’au heartbeat suivant pour ne pas saturer
le worker. Un heartbeat authentifié d’une autre machine ne déclenche rien.
La purge Unregister est exercée par le parcours public. Main avait ce chemin
extrait dans wrapper_dispatch ; le runtime le conservait inline et le purgeait.
Les textes en attente sont bornés à4096octets de root et255d’hôte.


R2 904ea935 ne trouve pas de défaut bloquant. Ses corrections ciblées sont
intégrées : ordre déterministe par ancienneté, avertissement de reprise
automatique sans délai de succès promis, clauses d’identité et de commande
réellement atteintes dans le test, succès direct de l’annonce publique conservé.
Les échéances de fixture passent à 15 secondes. Les nouveaux blocs sont formatés.
Le tri ajoute O(n log n) à ce chemin de reprise après incident, au lieu de O(n).
Cette dépense bornée par les connexions natives en attente rend l’ordre stable
et traite les annonces les plus anciennes en premier. Une capacité commune
encore occupée arrête toujours le lot au prochain heartbeat ; le tri seul ne
prétend pas garantir une récupération tant que le worker refuse ses lectures.
Aucun TTL, nouvelle file durable ni relance métier ne sont ajoutés.


Le premier export R2 avait retiré des fonctions voisines des tests, car la
base runtime et main n’avaient pas la même indentation. La compilation de
qualification a refusé cet export ; il n’a pas été installé. La restauration
reprend exactement le suffixe de la base runtime à partir de la fixture237.
Cette égalité est contrôlée avant le nouvel essai. Le test278 complet passe
alors, avec les nouvelles clauses d’identité, de commande et de guidage.


Qualification finale : quatorze tests Rust ciblés par plateforme passent après
les corrections R2, ainsi que Clippy strict workspace/all-targets et le build
release. La recette reproduit les quatre fichiers compilés sans fuzz. Les
douze tests Python inchangés restent couverts par la qualification277.
Le paquet Android0.8.43/8043 est signé et compatible avec la signature0.8.42.
