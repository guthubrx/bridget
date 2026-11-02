# Signalement sans redésignation répétée

Correction de procédure du 2 novembre 2026 après le retour utilisateur sur
l’agent piétons. La recherche du catalogue limitée au projet masquait le
mainteneur ; la procédure conduisait alors à redemander une désignation déjà
confirmée dans le contexte humain. Le rôle persistant reste limité au projet
Bridget et son extension native demeure en attente : aucun droit n’est modifié.

La consigne partagée utilise désormais une désignation humaine déjà connue
et applicable, résout l’identité et vérifie la présence native. Le refus
role_scope_denied de lecture du catalogue ne remplace pas le verdict d’envoi
direct, qui conserve ses propres gardes. Sans désignation et sans candidat
qualifié, conserver le signalement ; ne pas déduire du seul refus technique
qu’il faut reposer une question de désignation. Une ambiguïté réelle entre
plusieurs responsables reste une raison de demander une décision ciblée.

Fichier canonique actif :
/Users/moi/.agents/skills/bridget-signalement/SKILL.md.
Alias local inchangé : /Users/moi/.codex/skills/bridget-signalement.
Delta versionné dans signalement-designation.patch, sans destinataire fixé
en dur dans la procédure. Aucune copie de cette skill trouvée sur les trois
hôtes Linux contrôlés ; aucune installation ni diffusion générale prétendue.

Validation : quick_validate.py passe. Contrôle de comportement réel :
message bridget226-pietons-signalement-direct-20261102-01, reçu
d71c6c97-4c0f-4203-9029-72c4448c1f65, remise accepted après relecture stricte.
Dans le fil 95269e5e-91fb-4cee-a459-6fb24485a178, l’agent relit la version
corrigée et annonce l’envoi du dossier sans nouvelle question humaine.
La skill n’élargit pas l’accès aux rôles et n’autorise aucune nouvelle revue,
réémission d’une mission incertaine ou opération sur le monde.

Incident technique distinct conservé : tâche Bridget
7024f34c-22df-4b6a-b16f-8b37907351b8, requête
opus2d-pietons-r56-capacite-index-revue-20261102-02, état starting avec
remote_start_failed:remote_start:WorkerUnavailable, sans résultat.
Son propriétaire garde ses références. La revue T3 ultérieure a été
explicitement autorisée par François et retourne un résultat favorable
22/22 ; aucune migration ni duplication de cette tâche par le mainteneur.
Le défaut distant reste à diagnostiquer ; la correction de consigne ne vaut
ni réparation de WorkerUnavailable ni activation métier.
