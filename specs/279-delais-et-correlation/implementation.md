# 279 : faits et qualification

Le refus SSH du Moteur porte observed_at1794444627, soit le12novembre à00:50:27UTC.
Il précède le passage277. Aucun reçu d'admission ni tâche ne porte cette clé dans
le store. Sa sortie SSH n'a pas été conservée : l'exit255 ne prouve pas la cause.
À02:10:25UTC, l'inspection native de la même liaison installée vers SYS réussit.
Le projet Parix est reconnu ; le SHA423c92a6 exige encore le bundle du parcours
normal de préparation. Aucune mission ni message métier n'est rejoué.

Le refus ACK/clôture vers01h56CET coïncide avec277 : maître chargé à00:55:36UTC,
services Mac rechargés dans la même minute. Les cinq lectures refusées vers
02h52–02h53CET coïncident avec278 : maître chargé à01:52:45UTC, indisponible3,9s,
services Mac rechargés jusqu'à01:53:16UTC. Les opérations ensuite réussies sur
les mêmes références ne nécessitent aucun changement de permissions.

La demande mcp-85592-6af51a35-8a est créée à1794447925 et échue à1794447985.
Le ledger atteste timed_out à1794447986. À01:46:26UTC, le journal du connecteur
confirme son retrait avant démarrage. Le guidage absent est corrigé, sans
augmenter le délai ni transformer queue en guidage du tour actif. La phase de
saga indeterminate reste inchangée : le pont ne déduit pas les effets globaux.

Revue67a77219 : aucun défaut prouvé dans le diff. Les doutes d'import Uuid,
d'Option déplacée et d'hôtes de fixture sont levés par compilation et tests.
Le type de l'heure maître est i64 ; la première compilation a signalé la
signature initiale u64, corrigée sans conversion forcée. Les nouveaux tests
ont signalé qu'host_is_attested n'exclut pas les contrôles : le nouveau journal
les refuse explicitement. Une assertion textuelle est corrigée pour vérifier
la phrase effectivement rendue, « ce pont ne l'a pas injectée ».

Les informations aux trois propriétaires concernés sont conservées sans réveil.
Les deux revues278 reçues dans le même lot avaient déjà été traitées et intégrées
avant ces nouveaux travaux. Aucun de ces résultats n'a déclenché une mission bis.


Qualification :95tests ciblés distincts passent par plateforme, dont quatre
nouveaux. Après le changement de forme demandé par Clippy et la précision du
guidage du relais distant, les cinq tests concernés sont réexécutés avec succès.
Clippy strict workspace/all-targets et les builds release passent sur Mac et
Debian12 ; l’exigence Linux reste glibc2.34, sous le plafond2.36. Le premier
échec Clippy n’est pas ignoré : la condition imbriquée est remplacée par une
condition let chaînée, sans attribut de désactivation. Les autres90résultats
encore valides sont réutilisés. La recette reproduit les dix fichiers runtime
compilés sans fuzz et conserve les lots précédents, dont le travail260 réservé.
Android0.8.44/8044 est compilé, signé et qualifié contre0.8.43.
