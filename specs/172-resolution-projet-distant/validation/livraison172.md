# Livraison 172 — préparation autonome des projets distants

Date : 18 octobre 2026. Correction active, recettes Cartae réussies.
Périmètre : résolution implicite du projet, préparation Git/SSH, démarrage et retour.

## Résultat utile

Un appel Cartae depuis un dépôt Git local n'exige plus un project_id connu
ni une correspondance préparée manuellement. Bridget retrouve la racine du
parent, transporte le commit exact et crée la correspondance après préparation.
Le transport utilise la fédération SSH existante. GitHub n'est pas nécessaire :
un bundle Git peut contenir un commit non publié. Un transfert ultérieur
importe seulement les objets manquants quand un delta est possible.
La préparation appartient à Bridget ; elle n'appelle pas T3.

T3 fournit la référence de parcours (héritage du projet, préparation, lancement).
Il ne fournit pas le transfert Mac vers Cartae. Références et différences :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/plan.md.

## Versions actives

- Maître Mac : a53b05c23426160dd34c5987613e7d0d878e273f, build172-a53b05c2,
  PID53473, actif depuis 09:59:02 CEST.
  Binaire : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.
  SHA256 : 8451acbd43c563e5ee05fcc851cd06bb213ed669bd5360afe6616b2bba73fc9f.
  Signature ad hoc vérifiée.
- Worker Cartae : 780d5df4ebfd5bb72f38fce509310b5ebb2523c9,
  build172-780d5df4, PID3893226, actif depuis 10:06:07 CEST.
  Binaire : /home/moi/.local/lib/bridget-communication/172-780d5df4ebfd/bin/bridget.
  SHA256 : f4e5bf79b6a903178a4d01f3b8f2a5eeeed26aa98979cce24e865c6a853ee8e2.
- Le dernier delta concerne l'enregistrement du wrapper distant ; il est
  exécuté sur Cartae. Le maître Mac a53 reste suffisant pour ce parcours.
- T3 PID637/1068 et pont898 conservés. Les anciens clients MCP déjà en mémoire
  conservent leur code jusqu'à leur prochaine ouverture ; aucun arrêt imposé.
- Aucun schéma SQL ou droit de fournisseur modifié. AGENTS.md local étranger préservé.

## Recettes réelles

### Projet Bridget connu

Request : recette172-bridget-cartae-connu-20261018-r3.
Task : 68f69ce3-62bb-4b17-8bbc-853399a6346f.
Sans project_id, machine Cartae, posture discovery, espace partagé.
Lecture réelle de /home/moi/10.Scripts/64.bridget/.git/HEAD, de sa référence
et de /home/moi/10.Scripts/64.bridget/Cargo.toml.
SHA a53b05c23426160dd34c5987613e7d0d878e273f, version0.4.1.
Résultat : BRIDGET172_CARTAE_CONNU_OK.
Résultat lu, ACK, rejeu strict : même tâche, même résultat.
État durable result_available, error=null, cleanup_done=true, un lancement.

### Projet Revue initialement absent

Source : /Volumes/8TB2/50-repos-archives/11.Repositories/revue.
Aucune déclaration manuelle avant la recette r1.
La préparation r1 a créé automatiquement :
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository.
Le premier lancement et son résultat ont confirmé la création ; son brief
demandait par erreur un shell interdit en discovery, donc la lecture finale
n'était pas validée à ce stade.

Recette finale :
Request : recette172-revue-cartae-neuf-20261018-r5.
Task : 800dae46-f45a-4e7c-bc51-9d1e7dabba7d.
SHA ac8f9295b89d1475f32af85f9e3e9d12d3f89008 transféré automatiquement.
Ce commit a remplacé le SHA initial pendant les essais à la suite d'une fusion
faite par une autre session. L'appel r4 avec l'ancien SHA a été correctement refusé.
Fichiers lus :
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/.git/HEAD
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/package.json
Nom revue, version0.1.0, SHA exact, BRIDGET172_REVUE_NEUF_OK.
Résultat lu et acquitté par le parent Revue. Rejeu strict : même tâche,
même résultat, acknowledged. État durable result_available, error=null,
cleanup_done=true, un lancement.
Aucun fichier Revue modifié par les agents de recette.
La création initiale et la lecture finale sont deux essais distincts, pas
présentés comme un seul lancement ininterrompu.

## Défauts révélés et corrigés pendant les essais

1. Identifiant projet absent : résolution unique depuis le dossier attesté.
2. Projet absent : préparation Git/SSH et publication atomique des registres.
3. Réattestation après Ready plus longue que cinq secondes :
   bail de60s négocié ReleaseLeaseV2, ancien worker limité à5s ;
   contrôles de droits/révision/époque conservés.
4. Enregistrement du wrapper limité à1s : échec réel os error11 reproduit
   avec Registered retardé de1,5s. Attente initiale10s puis lecture de session1s.

Les échecs historiques sont conservés dans le journal d'implémentation.
Aucun refus de droits ou de commit n'a été transformé en réussite.

## Vérifications et limites

- Préparation autonome Python : 9 PASS, vrais dépôts Git et SSH simulé, sans T3.
- Résolution/routage : 2 PASS172, 35 PASS168, 1 PASSincident171.
- Délais : 3 PASS172, 8 PASST3MCP, 2 PASSrécupération admission.
- Worker : 21 PASS en série ; première exécution parallèle20PASS/1 verrou
  occupé. Rejeu exact PASS. Cause de concurrence non corrigée ici.
- Protocole : 9 PASS ; négociation ancienne/nouvelle capacité : 1 PASS.
- Enregistrement/reconnexion : 69 PASS/1 ignoré ; reproduction RED puis GREEN.
- Pas de suite globale, pas de succès global déduit.
- Test supplémentaire d'un parent natif hors T3 :
  recette172-parent-natif-autonomie-20261018-r1 refusée resource_saturated
  sur la machine locale, sans reçu ni tâche. Aucun repli ni changement de
  seuil. Ce parcours complet reste non validé en réel. Les recettes modèles
  ci-dessus partent de parents T3 ; la préparation autonome est testée séparément.
- Nouveaux projets Git seulement. Les fichiers non committés ne sont pas
  transférés. Les projets non Git déjà déclarés gardent leur parcours existant.
- La fédération SSH, Python3 et Git doivent déjà être installés.
- Correction technique : aucun changement d'interface, pas de diaporama requis.

## Reçus et conservation

Reçus privés Mac : /Users/moi/.cache/bridget-delivery172.
État final des tâches :
/Users/moi/.cache/bridget-delivery172/final-task-receipts.json.
Reçu worker :
/home/moi/.cache/bridget-delivery172/linux172-register.json.
Ancien binaire conservé :
/Users/moi/.cache/bridget-delivery172/bridget-before172.
Unités et binaires précédents conservés sur Cartae ; aucun effacement de base.
Worktree de correction conservé :
/home/moi/10.Scripts/64.bridget/.worktrees/172-resolution-projet-distant.
