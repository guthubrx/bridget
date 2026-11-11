# Faits 272

KS : les journaux bridget_prepare_capacity prouvent held_bootstrap, held=1 et
prepared=0 pour 6afe051a et 681bb87c. Ne pas en déduire un CPU ou disque saturé.
Les deux tâches restent queued avec refus de préparation, sans instance créée.
L’admission Rendu annoncée pending est devenue admitted en 52 secondes et porte
681bb87c. Son annulation par bridget_cancel cherchait dans les demandes de
messages suivies, pas dans les reçus d’admission.

La qualification MEP fe52581e est ensuite admise (dca9a0d6), résultat disponible,
accusé et nettoyage terminé. La copie Git du nouveau SHA en ronde 2 du prévol
avait été réalisée par le parent, même tâche désormais ronde 4 terminée.

Les pertes de lien Économie à 16:17 et 17:29 CET coïncident avec les redémarrages
maître 15:17:07Z et 16:28:32Z. L’attestation session ne réinscrit pas le primaire.

La tâche SYS a6e7d08d n’a pas demandé keep_alive_secs et guarded=false. Sa fin
avec cleanup_done=true rend la continuation impossible, sans perte de résultat.
La tâche d87d03e4 a 29 provider_result success entre 13:12:36Z et 13:18:22Z, mais
aucun turn_end avant l’annulation à 13:32:24Z. Les anciennes trames SDK de tâches
de fond ne sont pas dans le journal ; ne pas inventer leur compteur ni statut.

Correctifs nécessaires : annulation explicite du reçu avant Task, refus de
contrôle donnant le parcours natif et diagnostic borné de l’attente fournisseur.
Aucun transfert automatique de clés SSH ou données externes.


Qualification : les trois tests 272 couvrent le store SQLite réel, les phases
Pending/DecisionStarted/Chosen, la course avec admit_auto_task, le propriétaire,
les états terminaux, la compatibilité du catalogue et le fil Unix avec un ancien
maître. Les tests de guidage 156 (3), de transport 174 (25) et de contrat historique
168 (1) passent sur Mac et Debian 12. La qualification finale et l’activation sont
consignées après leur exécution.

La première revue a relevé deux défauts corrigés : conserver la première cause
terminale face à un retour tardif et signaler un worktree créé avant l’annulation,
sans le supprimer. La seconde revue, sur le delta complet joint et la base, ne
trouve pas de blocage. Son point sur le chemin conservé est ajouté au refus rendu.
Le mappage admission_cancelled contient déjà le texte de résolution. Le raccord
T3 réatteste puis vérifie identité/projet avant le même puits natif ; aucun autre
contrôle de types d’opérations n’y interdit CancelAdmission. Le contrôle actif
sera fait après activation, sans annulation d’une mission métier.

Le test de contexte a d’abord exposé un ajout manquant dans la variante cumulative
du module de guidage (espacement différent de la source principale). Correction
ciblée, puis reprise du même test sur les deux plateformes. Aucun changement de
format de stockage. Le delta cumulatif se vérifie en sens inverse à sec, sans fuzz.

Limites constatées : le journal historique de d87d03e4 ne permet pas de reconstruire
les événements SDK des tâches de fond. Les nouvelles instances journalisent leur
nombre et l’indicateur unconfirmed ; aucun succès fournisseur isolé ne clôture
artificiellement une mission. Un pilote qui refuse steer reste explicite : queue
attend un tour, task_continue exige une garde, le propriétaire utilise task_cancel
si le travail est devenu périmé. Aucun contournement ni copie d’accès SSH.


Activation : 0.8.38 / 272-annulation-admission-r1 chargée sur le maître,
le service mobile et les workers Cartae, SYS, KS, Mac et M1. Contrôle des images
réellement chargées et des pointeurs de retour 271. Publication v0.8.38 : six
fichiers retéléchargés sans authentification et contrôlés par empreinte.
Le quota 128 est chargé et persistant sur le maître.

Le raccord MCP existant répond avec le nouveau guidage natif après réattestation.
Sa façade ouverte depuis 14:10 utilise encore 266 et n’expose pas kind. Une façade
neuve lancée depuis un shell n’a pas la preuve de session injectée par le client ;
elle refuse identity_not_found sans effet. Aucune preuve copiée ou fabriquée.
Le contrôle SQL/Unix du nouveau parcours est exécuté, mais son essai authentifié
dans cette session attend le remontage natif du catalogue. T3 reste PID 77762,
né le 11 novembre 2026 à 14:10:41. Aucun arrêt ou remplacement autorisé dans ce lot.


Clôture : les tâches 681bb87c et 6afe051a ont été annulées par leur parcours
propriétaire, cleanup_done=true, sans instance enfant. Les sept propriétaires
réellement concernés ont reçu un avis conservé sans réveil, puis le retour au
service et les faits propres à leurs incidents. stored ne signifie pas lu.

Les trois tests 272, les 3 tests 156, les 25 tests 174 et le contrat historique
168 passent sur chaque plateforme. Clippy workspace/all-targets avec refus des
avertissements et builds release passent sur les deux. Linux exige glibc 2.34,
sous le plafond 2.36. Empreintes exécutables :
- Mac/M1 : cdb63de8a0a19e748c65d6194123d6ab1fadbf56e8a175740ae145281673794a.
- Linux : 1229715a0ad3f50ae72a643dc919f7c71d849381dcdb3fb40276e0fceca8525f.
Le mobile répond HTTP 200, enveloppe version 1. La version de retour est 271,
qualifiée dans les deux sens et inscrite par l’écrivain atomique sous verrou.
M1 n’avait pas d’état d’outil de mise à jour : aucun état inventé.

Ménage : 88 704 007 octets d’exports propres retirés après installation et
vérification publique. Les caches Cargo restent conservés : sur Mac, PID 33346
avec cwd introuvable ; sur SYS, descripteur du worker non vérifiable dans la
photographie lsof. Une seconde observation après disparition du premier FD reste
incomplète sur un autre FD ; aucun troisième essai ni contournement. Cartae reste
aussi conservateur sur le worker. Les deux anciennes images KS restent réellement
utilisées. Les images encore chargées par les missions et anciens MCP sont gardées.
Le cache SYS est sur volume externe, dans le budget commun ; / a environ 39 Gio
libres. Aucun processus indépendant arrêté pour nettoyer. Aucun worktree ni branche
auxiliaire créé dans ce lot.

Décision de François : garder T3 ouvert. L’essai authentifié du nouveau champ dans
son MCP déjà monté est donc différé, sans empêcher les 128 places ni les services
272 de fonctionner. Le refus legacy dans cette même session rend bien le guidage
272 et n’annule rien. Le nouveau parcours complet est vérifié sur le store et le
transport Unix isolés ; aucune preuve de session n’a été copiée depuis un processus.
