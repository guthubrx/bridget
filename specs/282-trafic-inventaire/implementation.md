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
