# 271 : messages après ronde et faits de préparation

Statut : appliqué dans les services 0.8.37/271, vérifications et conservations des instances antérieures suivies dans la session271.

Un enfant peut garder un processus après le résultat d’une ronde. Un nouveau
message de travail du parent ne doit pas être présenté comme un échange entre
projets physiques différents : il doit nommer la ronde terminée et le parcours
task_continue. La garde ne crée aucune ronde. Les dépôts silencieux et la remise
interne exacte restent distincts, ainsi que les réponses corrélées à une demande
encore ouverte. Les messages déjà admis ne sont pas rejoués ni reclassés.

La préparation distante conserve l’opération et le code de sortie non nul de
son processus. Les deux frontières valident une petite forme fermée, sans texte
de stdout/stderr. Une sortie excessive porte le code de borne existant. Un ancien
refus ne permet pas de reconstituer ces faits et conserve une cause inconnue.

Le journal de résultat fournisseur indique si sa finalisation native attend
encore des tâches de fond. Cette métadonnée ne clôt rien et ne publie aucun
résultat avant la fin effective déjà exigée par le pilote.

Les réponses ordinaires restent en file. Leur présence dans le suivi des demandes
atteste la réponse conservée, pas son injection dans le tour actif. La correction
ne transforme pas une réponse en steer ou interrupt sans contrôle explicite.

François demande de relever le plafond global à 128. Le défaut portable devient
128 ; la surcharge active à 24 est portée à 128 et chargée dans le superviseur.
Une variable explicite reste un choix administrateur, pas une valeur ignorée.
Les permissions, les plafonds de fournisseurs et les gardes mémoire ne changent
pas. La limite est lue au démarrage ; ce chargement ne nécessite aucun arrêt de T3.
