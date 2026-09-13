# Spécification 096 — Fédération depuis le binaire

**Branche** : session-096-federate-cli · **Date** : 2026-09-07 · **Statut** : Validée par l'utilisateur
**Demande** : `bridget federate ssh://cartae.app -p 2222`, statut et retrait depuis la même interface, sans chemin de script à connaître.
**Tests** : à exécuter.

## Scénarios utilisateur et tests

### US1 — Retrouver une liaison connue (P1)
L'humain saisit une destination SSH et son port. Une liaison déjà configurée et identique est reconnue sans réinstallation ni interruption.
Test indépendant : la liaison cartae-core existante, enregistrée avec son IP, est reconnue par son nom DNS et son port ; le PID et les fichiers du service restent inchangés.

1. Étant donné une liaison privée valide vers Cartae, quand la commande demandée est lancée, alors elle indique la liaison réutilisée sans créer un second service.
2. Si plusieurs liaisons conviennent, aucun choix arbitraire n'est fait : préciser le label résout l'ambiguïté.

### US2 — Installer une nouvelle liaison (P1)
L'humain peut établir une liaison persistante depuis le seul binaire distribué. Les paramètres manquants peuvent être fournis par options ou demandés dans un terminal ; aucune valeur sensible ou identité distante n'est inventée.
Test indépendant : dans un environnement isolé sans dépôt ni script adjacent, les paramètres sont transmis au gestionnaire existant et l'installation reste autonome après sortie du CLI.

1. Paramètres complets valides : création par le gestionnaire natif approprié, vérifications de sécurité095 conservées.
2. Paramètres manquants sans terminal : diagnostic actionnable et aucune installation ; avec terminal : demander seulement les valeurs nécessaires.

### US3 — Observer et retirer (P1)
`bridget federate status` liste l'état des liaisons enregistrées. Le retrait exige une destination ou un label non ambigu et conserve les garde-fous095.
Test indépendant : statut sans SSH ni mutation ; retrait d'une fixture uniquement, erreur transmise sans faux succès.

### Cas limites
URL malformée, schéma non SSH, mot de passe dans l'URL, chemin/query/fragment, option inconnue, ports contradictoires, configuration corrompue ou liée, DNS inconnu, homonymie, destination non installée, EOF pendant saisie, outil système indisponible.

## Exigences

- FR001 : accepter `federate ssh://[utilisateur@]hôte [-p PORT]`, `federate status` et `federate remove ssh://[utilisateur@]hôte [-p PORT]` ; aide dédiée.
- FR002 : conserver un label explicite pour départager, et les paramètres avancés nécessaires à095 ; aucune collision silencieuse.
- FR003 : réutiliser la liaison reconnue sans modifier configuration, empreintes ou service. Le DNS sert à trouver un candidat ; jamais à changer la cible SSH ou à contourner sa clé d'hôte enregistrée.
- FR004 : fonctionner avec le binaire seul, sans ancien dépôt, worktree ni script externe choisi depuis un chemin non attesté.
- FR005 : réutiliser l'unique implementation095 pour installation, statut, retrait, validation et rollback. Aucun protocole daemon ni outil MCP supplémentaire.
- FR006 : transmettre les arguments comme données, refuser injections/contrôles, borner les découvertes et conserver les erreurs du gestionnaire.
- FR007 : respecter le mode non interactif ; ne pas lire un pipeline de travail comme des réponses humaines.
- FR008 : conserver le sens maître local → socket cliente distante. Aucun lancement fournisseur, restart daemon ou migration DB.
- FR009 : aligner aide, README FR/EN, skill et inventaire des commandes.

## Entités
Destination demandée (hôte, utilisateur facultatif, port), installation095 attestée (label/config/reçu), invocation d'administration (installer/statut/retirer).

## Critères de succès
- SC001 : l'exemple humain fonctionne après installation du nouveau binaire sans autre argument pour Cartae déjà enregistré.
- SC002 : zéro interruption ou second service lors de la réutilisation ; zéro mutation pour statut et entrées invalides.
- SC003 : une copie isolée du binaire suffit au parcours complet de fixture.
- SC004 : tests de frontière et régressions095 verts ; recette Cartae de réutilisation et annuaire réussie.

## Hypothèses et limites
macOS/Linux, OpenSSH et gestionnaire utilisateur disponibles comme095 ; accès SSH préautorisé, aucune acceptation automatique d'une nouvelle clé. Les usages administratifs restent humains. Une fédération n'est pas un ordonnanceur distant.
