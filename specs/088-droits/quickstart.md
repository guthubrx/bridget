# Quickstart: Droits

## 1. Un lien, un refus lisible
1. Paramètres › Sécurité du contenu : tout désactivé (état neuf). Fais publier par un agent un message avec un lien HTTPS.
2. Attendu sous le lien : « Ouverture au clic désactivée · Sécurité du contenu › Liens externes » et un bouton « Autoriser les liens ». Clique. Attendu : le bouton « Ouvrir dans le navigateur » apparaît sans rechargement, et la case est cochée dans Paramètres.

## 2. Un shell bloqué, un refus attribué
1. Sur le serveur, lance un agent Codex en posture découverte (profil Prudent), envoie-lui « exécute `ls` ».
2. Attendu dans le fil, attribué à Bridget : « Signalement de sandbox (non attesté) : la commande a échoué avec un diagnostic de sandbox · codex · posture découverte, lecture seule » avec le bouton « Droits › Shell ». Un simple `echo` de la même ligne avec sortie 0 ne produit rien. Limite connue : aucune détection pour Claude dans ce lot.

## 3. La page Droits
1. Paramètres › Droits. Trois blocs, chaque ligne avec sa phrase. Profil affiché : Prudent.
2. Choisis Équilibré. Attendu : lignes serveur mises à jour (posture complète, réassignation active, plafond 5), lignes locales de CE navigateur passées « au clic ». `bridget control status --history` montre le plafond et une ligne `rights_set` avec la génération incrémentée.
3. Ouvre la page dans un second navigateur : ses lignes locales n'ont pas bougé, profil affiché « Personnalisé », mention « Réglage local à ce navigateur ».
4. Mode expert : déplie « Shell » ; attendu : mécanisme « posture d'agent », stockage « serveur, control_state.agent_posture », valeur brute, et les trois lignes d'agent marquées liées.
5. Depuis un agent (`bridget send` ou outil MCP), tente `POST /v1/control/rights/apply` : refus `human_principal_required`, consigné dans le journal du relais.

## 4. Tester un droit
1. Droits › Shell › Tester, agent : un Codex en posture découverte. La ligne passe « En cours » ; « Actualiser » après la fin du tour. Attendu : « Refusé par la sandbox du fournisseur · <heure> · <agent> », ligne brute `bwrap: …`. Si l'agent est occupé, la demande attend son tour : la ligne reste « En cours » puis « Inconnu » au-delà de 120 s sans fin de commande.
2. Passe en Équilibré, relance l'agent (`bridget relaunch <uuid>`), Tester à nouveau. Attendu : « Réussi · <heure> ».
3. Droits › Internet › Tester. Attendu : « Réussi » (le code HTTP est dans la ligne brute), ou « Injoignable » avec la ligne `curl`.
4. Reviens le lendemain : le résultat est marqué « ancien ».

Mesure préalable (T006a) : la première fois qu'un agent Codex termine une commande sur le serveur mis en service, vérifier dans son journal (`~/.cache/bridget/sessions/<agent>/*.jsonl`) qu'un `update` de `kind: command` avec `detail: item/completed` et `exit_code` apparaît. S'il n'apparaît pas, Codex n'émet pas de fin structurée sur cette version : aucun signalement ni test ne peut conclure, et il faut le dire dans `implementation.md`.
