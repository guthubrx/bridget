# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/037-observations-t3-attestees.md](../../../../../decisions/037-observations-t3-attestees.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La session 100 apporte partage de journal, abonnements et risques de collision.
Le pont T3 possède déjà des journaux et des identités de fils, mais cela ne
prouve ni l'identité d'un appel MCP natif ni la capacité à produire chaque
événement. Accepter silencieusement une surveillance impossible serait trompeur.

<a id="extrait-2"></a>

## Extrait 2

Bridget reste le produit autonome ; T3 reste un adaptateur. Le cas utilisateur
est « préviens-moi ici quand Horizon-3D termine son tour », pas « orchestre et
valide son projet ». Aucun profil supplémentaire, verrou de travail ou composant
le service compagnon n'est introduit.

<a id="extrait-3"></a>

## Extrait 3

Réutiliser les preuves primaires, marqueurs typés et contrôles de naissance de
processus existants. Dans l'adaptateur seulement, lire en SQLite READ_ONLY trois
colonnes des sessions T3 : `thread_id`, `provider_name`, `resume_cursor_json`.
Limiter la requête aux fils actifs effectivement enregistrés dans Bridget.

<a id="extrait-4"></a>

## Extrait 4

Croiser l'identifiant natif avec un processus descendant du serveur T3 : tous
les rollouts ouverts et leur première métadonnée pour Codex ; flags explicites
`--resume` ou `--session-id` pour Claude. Une seule correspondance est nécessaire,
avec filiation et naissance revérifiées. Ne jamais utiliser cwd, titre, date de
modification ou choix d'identité par le modèle. Les formes inconnues sont refusées.

<a id="extrait-5"></a>

## Extrait 5

Publier un marqueur privé atomique sans écraser celui d'un autre propriétaire.
Le retirer sur perte ou ambiguïté. Après crash du pont, récupérer uniquement
ses marqueurs de namespace exact dont le propriétaire identifié par PID et
naissance est attesté mort ou recyclé ; préserver tiers et propriétaires vivants.
T3, ses fournisseurs et leurs conversations n'ont pas à être redémarrés.

<a id="extrait-6"></a>

## Extrait 6

Réutiliser le journal, son relais de faits et la file de notifications100.
Accepter les fins T3 `completed`, `error`, `interrupted`, y compris sans texte
assistant, uniquement avec une origine attestée. Dédupliquer après écriture
confirmée et exclure les tours issus de notifications système. Idle et
déconnexion ne prouvent jamais une fin.

<a id="extrait-7"></a>

## Extrait 7

Une permission `approval.requested` est une demande observée, pas la preuve
qu'une autorisation reste attendue. Les écritures T3 sont limitées aux faits
Codex structurés, terminés avec succès et portant des chemins confirmés.
La projection Claude T3 perd `input.file_path` et peut synthétiser une fin sans
résultat d'outil : **ne pas annoncer FileWritten pour Claude T3**. Les wrappers
structurés Bridget dans un terminal gardent leurs propres capacités.

<a id="extrait-8"></a>

## Extrait 8

Les lacunes du relais et de la projection T3 réutilisent une trame primaire
ObservationGap : compteur de pertes quantifiées distinct du nombre de lacunes,
quantité inconnue possible. Notice bornée, sans consommation once et sans
fabriquer un événement de fin. Les compteurs sont consultables sur l'instance
courante du daemon. La sauvegarde d'abonnement échoue sans attendre un verrou
SQLite, puis restaure le délai habituel des autres écritures ; le rollback
mémoire existant évite une fausse confirmation de souscription.

<a id="extrait-9"></a>

## Extrait 9

- La règle100 demeure : événements reçus par Bridget après abonnement, sans
  relecture de l'historique ; un fait en transit peut déclencher la notification.
- T3 ne publie que `latestTurn` : plusieurs fins entre deux lectures peuvent
  être manquées. Sa fenêtre d'activités est limitée à 500 avant compression,
  et sa projection à 12 chemins par activité. Les lacunes détectées sont
  signalées, sans prétendre reconstruire les faits absents.
- Les lectures OS, SQLite et rollouts sont bornées. Une collecte incomplète
  refuse le rattachement plutôt que de choisir une identité plausible.
- `once` est consommé au déclenchement ; absence, DND, saturation ou panne
  peuvent perdre une remise. La trace d'abonnement n'est pas une file durable.
- Une collision reste un risque entre deux auteurs sur le même hôte et chemin
  dans 30 secondes ; pas de verrou, de fusion Git, ni de surveillance du disque.
- Une fin de tour peut être une erreur ou une interruption. Elle ne prouve
  jamais le succès métier, la fin du projet ou la réponse à une demande `reply`.

<a id="extrait-10"></a>

## Extrait 10

Tests isolés d'identité, refus et appartenance, projections positives/négatives,
annonces primaires, état après redémarrage et parcours journal → notification.
Les recettes de la session101 consignent les commandes et résultats réellement
obtenus. Cette ADR ne constitue pas une preuve de déploiement ou de réception
dans un fil T3 réel : cette dernière recette et l'autorisation d'adoption restent
des conditions distinctes, sans arrêter les conversations actives.
