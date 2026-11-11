# Qualification 274

La tâche aca41e28, désormais annulée et nettoyée par son propriétaire, conservait
une définition et une politique de worktree différentes de la sonde source.
Leur égalité était exigée avant tout nouvel appel worker. La revue indépendante
R1 confirme cette comparaison incorrecte et demande de conserver la sonde pour
shared, les axes figés et la revalidation fraîche du worker.

Le nouveau test reproduit le refus definition,permissions sur 273. Après correction,
la matrice worktree/commit_view et effort identique/distinct passe. Elle vérifie
les refus de définition altérée, politique altérée ou absente, préparation absente
et dérive réelle du contexte shared. La projection ne modifie ni tâche ni store.

Un premier export partiel avait omis deux conditions compactées et gardé le test
en shared. La lecture du fichier exporté a identifié cette erreur de raccord.
La fonction complète et le helper de test sont ensuite recopiés depuis la source.
Le test passe sur cet export et Clippy strict ne relève aucun avertissement.

Les deux revues 271 reçues tardivement portent sur des changements déjà intégrés
et testés. Le diagnostic de préparation courant indique le journal réel du service
chargé et la corrélation par heure/opération/statut, sans prétendre que le reçu
contient correlation_id. Aucun nouveau défaut prouvé de spool n’est signalé.

Sur M1, le dépôt mappé existe depuis avant les refus signalés ; le SHA demandé
est présent. L’absence du chemin Nextcloud source ou de .bridget ne prouve pas
une absence de préparation. Le journal maître à observed_at=1794423932 retrouve le même admission_id et
correlation_id que le journal M1 : le premier refus est une projection de
permissions non mappable (permission_mapping_unavailable), accompagnée d’une
définition indisponible pour l’autre candidat. Le code ne conserve pas quel
chemin a échoué ; il serait faux de désigner les pièces jointes sans cette preuve.
Le second refus garde seulement le code de transport ancien, sans diagnostic
source exploitable conservé. Aucun droit élargi ni admission rejouée.

Le refus KS a été suivi d’un rejeu strict admis et d’un résultat nettoyé. Aucune
cause CPU ou mémoire n’est déduite du code resource_limit.

Le pilote Claude stream-json refuse explicitement le steer strict pendant un
tour. La file attend le tour suivant. L’interruption d’une mission native active
par message est refusée ; l’arrêt passe par le contrôle natif du propriétaire.
Ce lot ne prétend pas ajouter un guidage pendant un tour.

La revue indépendante R2 ne trouve pas de contournement dans le parcours isolé.
Elle demande une preuve explicite de succès shared inchangé et un refus quand
seule l’empreinte de placement est altérée. Ces assertions sont ajoutées à la
même matrice. Tous les tests du module admission_tests sont ensuite exécutés,
y compris ceux qui ne portent pas le numéro 198. L’effort, le modèle et les droits
sont encore revalidés par le worker sur son record préparé lors du Start.

Avant intégration : 54 tests ciblés réussis sur Mac et Debian 12 (spec274,
spec198, spec202 et spec241). Après les remarques R2, les 27 tests du module
admission_tests et Clippy strict passent sur Mac. La recette cumulative 273 puis
274 est reconstruite sans fuzz et ses quatre fichiers modifiés sont identiques
à l’export qualifié. Le retour de la compilation et du contrôle complémentaire
Linux, puis l’activation, sont consignés dans la clôture.

Clôture : les 27 tests du module, dont les deux assertions demandées par R2, et
Clippy strict workspace/all-targets passent sur Mac et Debian 12. Les deux builds
release réussissent ; Linux exige glibc 2.34, sous le plafond 2.36. L’APK 0.8.40
est signé et sa compatibilité avec la signature précédente est vérifiée. Les
six fichiers publics de v0.8.40 sont téléchargés anonymement et leurs empreintes
revérifiées. Le commit de code publié est a80b5d9dac1dabb6044afb077d704de51adc2881.

Activation : maître, service mobile et workers Mac, M1, Cartae, SYS et KS chargent
0.8.40 / 274-reprise-contrat-prepare-r1. Le retour 273 est qualifié et les pointeurs
privés sont mis à jour sous leur verrou natif. M1 n’avait pas d’état de mise à
jour : aucun état n’est inventé, son image de retour et son reçu sont conservés.
Les 19 superviseurs vivants des fenêtres de passage gardent leur identité de
processus. La coupure du maître dure 3,27 secondes. Le mobile répond HTTP 200,
enveloppe version 1, et la lecture MCP de notre tâche de revue fonctionne après
le passage. T3 garde PID 77762 et sa naissance à 14:10:41. Son ancien catalogue
reste chargé par choix explicite de l’utilisateur. Aucun travail métier rejoué.

Les cinq responsables de processus concernés ont un avis avant et après, conservé
sans réveil. Les avis après passage donnent les causes constatées et la limite
Claude, sans présenter un message conservé comme lu. Les deux anciennes revues
271 étaient déjà accusées ; la nouvelle revue R2 est lue et accusée.

Ménage : 7,10 Gio logiques d’intermédiaires retirés sur Mac et 4,48 Gio sur le
volume externe SYS. Les tailles logiques APFS ne sont pas des octets physiques
libérés. Après un lecteur incomplet sur SYS, le descripteur signalé est vérifié
séparément puis un nouvel inventaire complet natif réussit ; aucune garde ne
change. Quinze copies exécutables inutilisées sont retirées sur Cartae, SYS, KS
et M1 (654 683 232 octets). Les images encore utilisées, l’active et le retour
qualifié sont conservés. Les exports temporaires propres à cette livraison sont
retirés (459 800 075 octets) après observation native complète. Les archives
publiques, reçus, sauvegardes privées utiles et source cumulatif de reconstruction
restent conservés. Rapport puis application de menage.py : zéro worktree, branche
ou reconstructible supplémentaire éligible. Un seul checkout, seule branche
main. Espace libre final observé : environ 62 Gio sur Mac et 38 Gio sur la racine
SYS ; ces valeurs évoluent avec les autres travaux.
