# Quickstart opérateur : Reprendre le contrôle

Prérequis : daemon et relais UI sur le binaire livré, au moins un projet actif avec deux agents missionnables, une routine déclarée pour observer les différés. Les libellés ci-dessous sont ceux de l'interface livrée.

## 1. Mettre en pause

1. Ouvre l'interface. Le bandeau en tête de page affiche « Autonomie active », un bouton « Pause », le plafond d'objectifs automatiques et « Aucune décision en attente ».
2. Clique « Pause ». Attendu : le bandeau passe en fond orangé, « Pause depuis 0 min · Pause demandée depuis l’interface », le bouton devient « Reprendre ».
3. Attends une occurrence de routine, ou lance `maicie status --json` et lis `control`. Attendu : une occurrence différée avec le motif `pause`, aucun objectif nouveau.
4. Redémarre le service Bridget (`systemctl --user restart bridget-daemon`, puis relance les agents gérés par UUID). Attendu : le bandeau affiche toujours la pause après rechargement.
5. En ligne de commande, `bridget who` se termine par `Contrôle : pause depuis … · plafond objectifs automatiques N`. `bridget control status --history` montre la ligne `pause_on` avec l'acteur `humain`.

## 2. Imposer un focus

1. Dans la colonne Projets, sélectionne ton projet. Sous l'en-tête, un champ « Travaille sur… » et un choix « Après le focus actuel » ou « À la place du focus actuel ».
2. Tape une phrase, clique « Lancer ». Attendu : « Demande déposée. Maicie l’ouvre à sa prochaine relève, deux minutes au plus. »
3. Attendu en moins de deux minutes : un objectif apparaît en tête de liste, marqué focus, avec l'origine « demandé par toi ». Une délégation est partie vers un agent du projet ; son instruction contient la base gelée et le bloc `IDENTIFIANTS DE DÉPÔT`, que tu n'as pas saisis.
4. Lance une seconde fois avec un autre texte et « Après le focus actuel ». Attendu : le second passe en file derrière le premier.
5. Vérifie qu'une occurrence de routine due pendant ce temps est différée avec le motif `focus`.

## 3. Traiter la boîte de réception

1. Provoque une intervention requise : mets l'agent cible en DND et laisse une délégation atteindre l'épuisement de sa chaîne de réassignation, ou envoie-toi une demande suivie avec `--reply` qui expire.
2. Attendu : le bandeau passe à « 1 décision vous attend » ; si `~/.config/bridget/human-channel.json` est configuré, une notification arrive sur ton canal, sinon rien ne se perd.
3. Clique « 1 décision vous attend ». Le panneau « En attente de toi » liste l'item avec son résumé et ses boutons : « Vu », « Annuler », « Réassigner à … ».
4. Choisis. Attendu : l'item disparaît de la liste ouverte, le bandeau revient à « Aucune décision en attente », et la délégation reflète ta décision à la relève suivante, deux minutes au plus.
5. En ligne de commande : `bridget inbox list`, puis `bridget inbox resolve <id> ack`.

## 4. Vérifier le budget

1. Réglages, « Paramètres du serveur », section « Objectifs automatiques » : mets le plafond à 1, « Enregistrer ». Attendu : « Enregistré : plafond 1. »
2. Laisse deux routines produire. Attendu : un seul objectif automatique ouvert, la seconde occurrence différée avec le motif `budget`, un item « budget reached » dans la boîte, une seule fois.
3. Ferme l'objectif automatique. Attendu : la prochaine occurrence repart sans intervention.
4. Ouvre un focus pendant que le plafond est atteint. Attendu : accepté.

## 5. Essai adverse

Depuis un agent, par son outil MCP ou `bridget send`, tente d'ouvrir une délégation avec une origine humaine ou un focus, ou de lever la pause. Attendu : refus explicite (`human_origin_forbidden`, `human_principal_required`, capacité non négociée), aucun objectif ouvert, aucun changement d'état. Ouvre une seconde connexion sous l'identité humaine alors que l'interface est connectée : un item « human route replaced » apparaît dans la boîte.

## Validation

Chaque étape produit une trace relisible : `bridget control status --history`, `bridget inbox list --all`, `routine_occurrences`, `reassignment_reductions`, ledger. Consigner les identifiants observés dans `implementation.md` avant de cocher T045.
