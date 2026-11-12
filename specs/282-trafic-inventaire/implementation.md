# Observations 282

Le 12 novembre vers 07 h 40 Paris, le chef Parix signale environ 12,5 Mo/s
reçus par le connecteur local. Une première mesure nettop reproduit deux
incréments de 8 949 397 octets en deux secondes. Une lecture authentifiée
unique de shell retourne 8 948 987 octets en 0,834 s pour 3 235 fils et
26 projets. Les éléments sont des résumés de fils, pas leurs historiques
complets. Aucun corps de message ni secret n’est conservé dans les mesures.

La boucle commune relit l’inventaire toutes les trois secondes. Chaque file
non vide lance aussi drive_queue toutes les 500 ms, puis relit le même
inventaire avant de constater que son tour occupé interdit une remise de pair.
Cette amplification est le défaut ciblé. Les lectures de détails et le poids
de l’inventaire commun restent hors du correctif ; aucun gain n’est encore
annoncé avant la mesure du binaire chargé.

Le nouvel état observed_busy ne permet que de différer ces lectures sans effet.
Le relevé commun le réactualise. Avant toute remise effective, le snapshot frais
et les contrôles existants restent obligatoires. Les expirations sont traitées
avant cette optimisation, et un guidage explicite dans la file garde son accès
à la lecture fraîche. Aucun nouveau cache d’autorité, droit ou champ de fil.

La reproduction avant correctif échoue sur l’assertion d’absence de requête
HTTP, avec le code 101. Le même test passe après correction sur Mac et Linux.
Les nouveaux tests vérifient aussi la lecture fraîche pour un guidage derrière
une tête bloquée, les messages autorisés pendant le tour et les expirations.
Un filtre de qualification nommé spec156 ne sélectionnait aucun test : le
script l’a refusé. Son nom réel guidance156_t3 sélectionne six tests ; seuls
les contrôles restants sont repris. Aucun succès vide n’est retenu.

La recette cumulative 281 + delta 282 reproduit exactement les quatre fichiers
modifiés du runtime. Le seul ajustement d’application concerne la forme compacte
d’un bras Tick dans le runtime, différente de main ; aucun fichier Rust complet
de main n’est transplanté. Les travaux réservés du lot 260 restent exclus.

La revue indépendante 6756e57d accepte le code de production. Elle relève deux
preuves insuffisantes dans les tests : état d’un thread serveur utilisé comme
oracle d’absence de requête, et expiration d’une file devenue vide avant la
nouvelle porte. Les tests utilisent désormais un listener non bloquant, gardent
un voisin valide pendant l’expiration et vérifient le refus envoyé. Un quatrième
cas couvre l’occupation découverte par une lecture fraîche, sans Tick suivant.

Compromis : la détection de fin d’un tour occupé dépend désormais du prochain
relevé commun, puis du passage de file (500 ms au plus hors blocage). Le poll
par défaut vaut trois secondes ; les temps HTTP et la charge s’ajoutent. Une
configuration de poll plus lente retarde donc cette détection. Pendant une
indisponibilité du relevé, l’observation occupée continue seulement à différer
les lectures ; elle ne permet jamais une remise. La fermeture et la conservation
des messages restent celles du parcours existant. Les demandes sans attestation
ouverte sur un fil au repos peuvent encore provoquer des lectures répétées ;
ce cas est à mesurer séparément si le trafic résiduel le justifie.

La preuve renforcée échoue avant le correctif sur une connexion HTTP observée
(code 101, 10,28 s) puis passe après correction sur Debian 12. Les quatre tests
finaux passent aussi sur Mac. Avec les 36 contrôles voisins, le périmètre utile
couvre 40 tests par plateforme. Clippy strict workspace/all-targets passe sur
les deux plateformes, avec reprise ciblée après les seules retouches des tests.
Les images release 0.8.47 sont construites sur Mac et Debian 12 ; Linux exige
glibc 2.34. Android 0.8.47/8047 conserve l’identité de signature précédente.

## Activation et mesure

Le commit 3675adf3da71467da162cada8344c48ca5954e49 est poussé. La publication
0.8.47 contient six fichiers téléchargés sans authentification et vérifiés.
Une première archive Linux avait été préparée pendant sa copie : le contrôle
interne a détecté sa taille incomplète avant publication. L’archive a été recréée
après copie, puis son exécutable comparé par SHA-256 au candidat installé.

Le maître, le service mobile, les cinq workers et le connecteur local chargent
282-trafic-inventaire-r1. Empreintes réellement chargées et pointeurs de retour
281 vérifiés. Sur M1, aucun état de mise à jour absent auparavant n’est inventé.
Le service mobile répond HTTP 200. La coupure du maître a duré 4,22 s ; les
wrappers contrôlés ont conservé leur identité de processus. T3 conserve le PID
77762 et sa naissance du 11 novembre à 14:10:41. Les anciennes images encore
utilisées par des sessions restent conservées ; le correctif interne est chargé
dans le connecteur sans changement du catalogue des outils déjà montés.

Avant : 116 195 884 octets sur 15 intervalles d’une seconde, soit 7,746 Mo/s.
Après : 83 850 100 octets sur 30 intervalles, soit 2,795 Mo/s, environ 64 % de
moins. Les deux premières captures après chargement contenaient des lignes de
processus absentes pendant l’amorçage de nettop : elles ne servent pas au bilan.
La capture finale ignore l’amorçage, contrôle la monotonie du compteur et utilise
les indices réels des observations. Ce sont des fenêtres successives de travail
réel, pas une charge artificielle identique. Aucun gain CPU n’est attribué.
L’inventaire commun reste volumineux ; ses historiques complets ne sont pas dans
la réponse mesurée. Les propriétaires affectés ont reçu les avis silencieux.

## Ménage et travaux distincts

1,35 Gio d’intermédiaires Cargo sont retirés sur le Mac et 19,54 Gio sur le volume
de compilation SYSLE2. Le premier contrôle Mac avait rencontré un descripteur
non vérifiable ; une lecture ultérieure l’a identifié comme socket IPv4, puis le
contrôle natif complet a réussi. Aucun processus n’a été arrêté pour nettoyer.
Deux copies de recette et trois exports Mac (315 730 434 octets logiques) sont
retirés. Les exports admissibles SYS, KS et M1 sont retirés. Une ancienne image
KS et une M1 sont retirées ; actif, retour et images encore chargées sont gardés.
Les contrôles incomplets Cartae/SYS conservent les autres images ; les exports
Cartae non qualifiés restent aussi conservés. Aucun worktree supprimable trouvé.
L’espace système libre mesuré sur le Mac est de 73 812 742 144 octets ; l’activité
simultanée interdit d’attribuer toute sa variation à ce ménage.

François a demandé deux chantiers de développement supplémentaires pendant cette
livraison. Leurs fichiers sont distincts et conservés non commités dans le checkout
partagé SYSLE2. Les deltas sont également sauvegardés dans les reçus privés du lot.
Ils ne font pas partie du binaire 282 : 283 ventile les phases des sondes ; 284
corrige une incohérence de présence locale dans la projection de continuation.
Leur qualification et leur intégration suivent leurs propres tâches.
