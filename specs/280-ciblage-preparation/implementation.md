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

## Activation et publication du 12 novembre 2026

Le code est poussé dans `main` au commit
`70e82aa1e0ec409b492fba87959872ad81bb6033`. Les six fichiers de la publication
0.8.45 sont accessibles sans authentification et leurs empreintes sont vérifiées.
Le helper extrait du binaire Mac correspond intégralement à la source qualifiée ;
ses 16 tests passent aussi contre cette extraction. Le binaire Linux installé
embarque les mêmes octets du helper.

Le maître, le service mobile et les cinq workers chargent les images 0.8.45.
Les contrôles portent sur les exécutables réellement chargés, pas seulement les
liens d'installation. Le retour 0.8.44 est vérifié dans les états privés des
outils de mise à jour ; aucun état absent n'est inventé sur le M1. Le point de
contrôle mobile répond HTTP 200 et une lecture native après le passage réussit.

Cartae a d'abord refusé le changement de worker pendant une transition de
lancement. Le même parcours a ensuite réussi après la fin de cette transition,
sans désactiver la garde. Les wrappers et résultats ont été préservés. T3 garde
son PID et sa date de démarrage. Sa façade MCP déjà montée reste en version 266 ;
le présent correctif côté worker en bénéficie sans changement de protocole.
Les propriétaires réellement affectés ont été prévenus avant et après le
passage, avec conservation sans réveil. Devantures a reçu le diagnostic et le
bilan du correctif, sans rejeu de ses missions E5 ou E6.

## Ménage gardé

Le script natif retire 3,67 Gio de caches Cargo sur le Mac. Les exports de
compilation et copies obsolètes vérifiés inutilisés sont retirés sur le Mac,
Cartae, KS et M1. Les images actives, le retour qualifié et les anciennes images
encore chargées restent conservés. Le contrôle SpecKit ne trouve aucun worktree
ni reconstructible supplémentaire admissible au retrait dans le dépôt principal.

Sur SYSLE2, le contrôle de processus du worker 4041060 ne permet pas de qualifier
certains descripteurs. Les caches et exports concernés restent donc conservés.
Le contrôle initial des anciennes images Cartae était aussi incomplet : ces
images restent conservées, bien que le contrôle distinct des exports ait réussi.
Aucun autre travail n'est arrêté pour nettoyer. Ces conservations restent à
réexaminer lors d'une livraison suivante.

Après ce passage, l'espace disponible mesuré sur le volume système est de
62 687 748 096 octets sur le Mac, 128 830 152 704 sur Cartae,
39 678 816 256 sur SYSLE2 et 82 454 044 672 sur KS. Ces mesures incluent
l'activité simultanée des machines ; les tailles logiques supprimées ne sont
pas présentées comme un gain physique exclusivement attribuable au ménage.
Les preuves et reçus sont conservés dans
`/Users/moi/.cache/bridget280-incidents`.

Les deux causes historiques restent inconnues : E5 a consommé environ 10 s dans
le contrôle de rattachement, E6 a reçu le statut SSH 255 sans texte brut conservé.
La reproduction du défaut corrigé ne suffit pas à lui attribuer cet ancien 255.
