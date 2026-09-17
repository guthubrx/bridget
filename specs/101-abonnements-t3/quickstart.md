# Recette101

Toutes les recettes automatiques utilisent un home/socket privé explicite et
aucun fournisseur payant. Ne pas exécuter globalement les suites qui utilisent
SIGKILL ou ciblent des services réels ; sélectionner les tests sûrs.

1. Deux fils même dossier : rattachement à leurs identifiants distincts ;
   sous-agent Codex plus récent ne change pas le fil parent. Ambiguïté refusée.
2. Deux recettes complémentaires : HTTP local contrôlé → projection du journal
   et capacités ; puis journal réel → socket et daemon privés → un seul Deliver
   pour un abonnement once attesté. Le pompage du relais est explicite dans le
   second test ; cela ne remplace pas la recette T3 réelle de l'étape9.
3. Démarrage avec historique : aucune notification. Notification entrante :
   son tour induit ne crée pas une nouvelle observation. Fin sans texte testée.
4. État inconnu, outil échoué, lecture seule, activité tronquée : aucun faux fait.
5. Types/capacités, cible inexistante, auxiliaire tentant de publier : refus.
6. Déconnexion/reconnexion source et redémarrage daemon : état explicitement
   indisponible/interrompu, jamais surveillance active fictive.
7. Régressions100 : extrait transmis sourcé, deux écritures concurrentes averties,
   DND/TTL/once, wrappers non T3 toujours utilisables.
8. Compilation, fmt, clippy ; tests ciblés101/100/099/MCP/attach/protocole.
9. Usage T3 réel : valider identité de ce fil puis notification depuis un fil
   témoin disponible, ou fin observée Horizon sans lui envoyer une mission parasite.
   Conserver reçu et ID de notification, pas les contenus privés dans les specs.

Les étapes9 et adoption sont distinctes : aucun remplacement silencieux du
binaire de production à partir d'un diff non relu/non commité.
