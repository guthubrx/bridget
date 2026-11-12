# 280 : observations et portée

E6 : observed_at1794451066,12novembre02:37:46UTC. Les journaux maître et source
portent69486240-6ad7-4d30-853a-79926f02eccd, inspect/255, signatureunobserved.
L'horodatage et la jointure279 sont donc observés en fonctionnement. Le texte
SSH brut n'ayant pas été conservé, sa cause historique n'est pas reconstituée.
Une inspection native actuelle des quatre liaisons donne une cibleKS attestée,
sans besoin de bundle, et trois machine_mismatch ; toutes les commandes SSH
rendent0. Les temps sont0,349/0,452/0,350/4,281s. Aucun réglage SSH n'est changé.

E5 : les traces du fournisseur montrent deux contrôles de rattachement de10164
et10035ms à02:31UTC, avant toute commande de lecture de tâche. Cet instant
précède le passage279. Ni cette maintenance ni une révocation ne sont donc
inférées comme cause. Les journaux disponibles ne distinguent pas où le budget
interne a été consommé. Le résultat durable d'E5 et celui d'E6 sont conservés ;
aucune nouvelle mission, annulation, migration ou relance de travail n'est faite.

Le test ajouté reproduit le défaut avant correction : quatre sous-cas cible
valide/liaisonindisponible échouent, et le transport masque aussi l'ambiguïté.
Après correction, les16tests Python du helper passent surMac. Le nouveau test
exerce la fonction prepare avec ses gardes restantes : machine/authority
inchangées, un seul lien prepare, un seul publish. Il couvre l'absence de cible
avec erreur originale conservée et les refus de politique toujours bloquants.

La revue e67d9d04 ne trouve aucun défaut prouvé. Sa remarque de latence est
retenue dans la décision280 : les probes restent séquentielles, la recherche
peut attendre plusieurs budgets existants et aucune promesse de10s n'est faite.
Aucun second modèle n'est lancé pour qualifier le même point.


Qualification finale :16tests Python et7tests Rust ciblés passent surMac et
Debian12. Clippy strict workspace/all-targets et les builds release passent.
L’exigence Linux reste glibc2.34 sous le plafond2.36. Les autres résultats279
encore valides sont réutilisés, le code Rust fonctionnel étant inchangé hors
embarquement du helper et version. La recette reproduit les quatre fichiers
compilés sans fuzz, et conserve le cumul279 et les travaux précédents.
Android0.8.45/8045 est signé et qualifié contre0.8.44.
