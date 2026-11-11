# Faits et qualification 273

Les refus quota_exceeded:24 sont historiques. Le maître courant charge le quota
persistant 128. Les deux tâches 82a04885 et 9148274c sont failed avec cleanup_done=true,
sans instance démarrée : le défaut de clôture avant lease corrigé en 269 s’applique.
Les gardes restent des processus vivants et comptent dans la flotte ; un ACK ne
libère pas la garde. Aucun plafond changé dans ce lot.

La revue 9d94efd5 a rendu son résultat le 11 novembre à 19:04:47 Paris. La garde
était de 900 secondes, soit une échéance à 19:19:47. Le reçu worker passe terminal
à 19:19:53, avant le continue signalé vers 19:20. Résultat et dossier conservés.

Le journal b34e74d8 atteste la remise du brief complet et une fin de tour normale,
sans appel d’outil. La réponse fournisseur est une simple acceptation. Ce fait ne
prouve pas qu’un modèle soit généralement incapable de développement. Le parent
n’avait pas demandé de garde. Aucun classement automatique des réponses, nouvelle
inférence, relance ou substitution de moteur n’est ajouté. L’enveloppe native
précise maintenant le travail attendu dans le tour et les limites du mandat.
Le test 148 vérifie le texte effectivement remis au transport, l’enveloppe durable
intacte, l’identité exacte, la rétention après échec d’injection et l’absence de
consigne ajoutée aux autres messages. Cela vérifie l’instruction livrée, pas une
garantie de qualité des réponses futures.

Le statut expose round_limit=8 et rounds_remaining dès la première admission.
Le nouveau test échoue sur 272 (champ absent), puis passe après le changement.
Il couvre 0/1/7/8/9, la saturation du compteur à zéro, l’absence d’ouverture de
garde et l’absence de mutation du store. Le test 153 conserve le refus de la
neuvième ronde et l’idempotence. La limite n’est pas relevée.

Les anciennes remarques de revue 271 sont déjà intégrées : le test d’exemption
asserte Ok ou un code différent de intent_not_permitted et couvre StoreOnly,
échange natif, réponse suivie et rejeu d’un message déjà en file. Les trois
missions de revue reçues dans ce lot étaient déjà accusées et nettoyées.

Qualification : 1 test 273, 2 tests 148, 1 test 271, 40 tests 153 et 1 test 187
réussis sur Mac et Debian 12. Compilation et contrôles finaux consignés après
activation. Reconstruction cumulative 272 puis delta 273, contrôle inverse à sec
sans fuzz. Aucune recompilation du chantier 260 réservé.


Revue indépendante sur le commit réel 32ca19fc : aucun défaut bloquant prouvé.
La remise à zéro de deferred_native_context après relance ne concerne pas les
missions natives : persistent_relaunch exige explicitement l’absence de
NATIVE_MISSION_BOOTSTRAP_ENV avant d’entrer dans cette branche. Pas de relance
native ajoutée. La consigne comporte déjà le cas d’accès indispensable manquant,
qui doit rendre un blocage. current_task_view appelle directement task_view :
les réponses d’admission reçoivent les champs additionnels.

Le catalogue automatique avait une propriété keep_alive_secs sans description.
Elle reçoit maintenant le même texte que l’admission directe. Le test 273 vérifie
les deux catalogues, puis les compteurs et l’absence de mutation. Le test repasse
sur chaque plateforme et le contrôle final Mac est réussi.

François demande ensuite la clôture Git et la libération de disque. Les caches
Mac mesurés occupent environ 38 Go (264 et 271). Le serveur documentaire Python
PID 33346, avec cwd disparu, bloquait l’observation globale. François autorise
explicitement son arrêt et celui de l’auxiliaire 52798. Après SIGTERM du serveur,
l’auxiliaire a terminé seul. T3 garde PID 77762, né à 14:10:41. Aucun processus
Bridget de mission n’est arrêté pour ce ménage.
