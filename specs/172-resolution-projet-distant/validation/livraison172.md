# Livraison 172 — préparation autonome des projets distants

État courant du 18 octobre 2026, après le complément de sondes et de lecture bornée : **In Progress**.
Le complément corrige aussi les défauts rencontrés lors des vraies délégations.
T3 est installé sur disque ; son activation reste à faire selon la consigne utilisateur.

## Versions courantes et activation

- Bridget source40e4d0c4dc2c7d493513a25508730ffd8c0be5ab, intégrée et poussée.
- Client Mac installé : build172-40e4d0c4, signé, SHA256
  a79e0f88f32312cd8f7ffdf66e06decf58130e720ef89fb289e1dc433e58a5de.
  /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.
- Maître Mac51581 conservé : build172-efeeb08f, source
  efeeb08fad26d0a394390b1264493409fd7c8abc. Le dernier delta ne change que
  les façades clientes ; aucun redémarrage du maître nécessaire pour ce delta.
- Worker Cartae3988809 actif depuis11:17:27 CEST, build172-40e4d0c4.
  /home/moi/.local/lib/bridget-communication/172-40e4d0c4dc2c/bin/bridget.
  SHA256968f961841be9322d49419abc529f63946ed04d5c4c9bf1e108434ab1bd7a6f2.
  Client /home/moi/.local/bin/bridget aligné. Aucune tâche native active au préflight.
- T3 source f3ddfca24278ab1ebb689087e5c42c6bc00c6551, intégrée et poussée
  vers le fork. Paquet0.0.45-local.172 signé installé dans
  /Applications/T3 Code (Local).app.
  ASAR4d57cee627d68079504e613cf93d532fa6399c7026a14c2b8f24fc26aa52a9b3.
  Application637, serveur1068 et pont898 préservés. Leur code T3 chargé reste
  la version171.5 ; ce n’est pas une preuve d’activation172.
- Le profil cx-pro pointe sur le même fichier que le profil Codex :
  /Users/moi/.codex/config.toml.
  Un seul champ du serveur officiel Bridget changé : tool_timeout_sec30→3700.
  Tous les autres champs comparés identiques, sauvegarde privée conservée.

## Recettes complémentaires prouvées

- Parent natif Cartae → enfant Cartae, sans service T3 ni project_id :
  request recette172-parent-natif-cartae-20261018-r3 ;
  Task parent c5d2017b-2e2e-47c7-8168-49daeb476ee0 ;
  Task enfant0caa190a-0fb4-4c0e-a493-254137804e24.
  Vrais modèles Haiku sélectionnés automatiquement. Lecture réelle du Git et
  de Cargo.toml au SHAefeeb08f, version0.4.1, marqueurs parent/enfant exacts.
  Chaque résultat lu/ACK ; rejouage strict, mêmes Tasks et mêmes résultats.
  Deux nettoyages terminés. Ce parcours remplace la limite native du premier reçu.
- Codage réel sur Cartae dans un worktree isolé :
  request recette172-codage-bridget-cartae-20261018-r1 ;
  Task e125950f-9ceb-4dc3-92e6-2841e0566f05 ;
  /home/moi/10.Scripts/64.bridget/.worktrees/e125950f.
  Un test ajouté à tests/test_spec172_project_preparation.py, 27lignes.
  python3 -m unittest -v tests/test_spec172_project_preparation.py :10PASS.
  HEADefeeb08f avant/après, aucun commit ni service modifié. Marqueur
  BRIDGET172_CODAGE_CARTAE_OK, résultat complet4676octets lu via l’outil
  officiel, ACK puis rejeu même Task/résultat. Worktree sale conservé, cleanup_done=true.
  Le test possède un contrôle positif de hook. Il ne prouve pas à lui seul
  que core.hooksPath est la protection active ; cette limite est conservée.

## Échecs réels qui ont déclenché le complément

- Deux lectures du résultat de codage ont d’abord expiré avec budget JSONL dépassé.
  La récupération ultérieure a réussi, sans présenter cette réussite comme un fix.
  Les piles du daemon ont ensuite montré l’attente de réattestation HTTP T3.
  Cinq cas RED avant correction ; budget complet réservé après correction.
- Revue r6 : premier appel et reprise ont expiré à30s dans l’hôte Codex, sans Task.
  Aucun reçu durable trouvé pour cette clé au contrôle. Une nouvelle reprise
  strictement identique a été demandée après installation, sans nouvelle clé.
- Opus2D r2 : Task b00ae037-ed93-4106-9e4b-68fbee5af3d2 admise, puis
  failed/remote_start_failed:permission_attestation_unavailable.
  Lecture et ACK de cet échec demandés au parent propriétaire ; recette r3
  distincte demandée après correction. Aucun succès Opus2D annoncé à ce checkpoint.

## Validation ciblée du dernier delta

- Neuf cas172 PASS, dont réponse volumineuse tardive, refus tardif et même clé au rejeu.
- Client Unix :19PASS ; façade de délégation :19PASS (ensembles partiellement communs).
- HTTP/attestation :8PASS, droits refusés et redirections toujours fermés.
- Contrats d’attente MCP :16PASS/1 ancien oracle qui assimilait une socket
  absente à unsupported ; oracle corrigé et rejoué seul1PASS.
- Montage Codex T3 :42PASS ; format et lint des deux fichiers conformes,
  zéro erreur et zéro avertissement. Deux cas nouveaux RED puis GREEN.
- Les tests de conservation des droits T3 du complément précédent restent
  prouvés séparément : trois nouveaux cas, dix goals et onze permissions,
  contrôle de21cas sur les overlays171 conservés. Aucune nouvelle FULL.
- Build Linux103,64s et build Mac127,23s, tous deuxRC0. Deux liens documentaires
  node_modules historiques exclus de l’extraction privée Mac ; sources compilées intactes.
- Aucun autre serveur que Cartae testé. Activation T3 et recettes finales
  des parents Revue/Opus2D restent ouvertes à ce checkpoint.

## Reçus du complément

/Users/moi/.cache/bridget-delivery172/coding-model-final-task.json.
/Users/moi/.cache/bridget-delivery172/native-model-final-tasks.json.
/Users/moi/.cache/bridget-delivery172/mac172-transport.json.
/Users/moi/.cache/bridget-delivery172/t3-install/bounded-attestation-install.json.
/Users/moi/.cache/bridget-delivery172/config-timeouts/receipt.json.
/home/moi/.cache/bridget-delivery172/linux172-transport.json.
Journaux de tests dans /home/moi/.cache/bridget-delivery172.
Aucune base modifiée par les diagnostics. AGENTS.md étranger du Mac préservé.
Correction technique, sans changement d’interface : pas de diaporama requis.

## Complément de sondes prêt à activer

Les défauts de réponse tardive, déconnexion V2 et verrou worker ont leur preuve
RED/GREEN ; détails et logs dans implementation.md. Worker21PASS, protocole9PASS,
client19PASS, banc de sonde retardée1PASS. Le lot initial de routage35 a donné
28PASS/7FAIL ; oracle corrigé1PASS et Laya7PASS en reprises séparées.
Le correctif T3 de lecture bornée ajoute16PASS, sans retirer les contrôles de droits.
Les sources du nouveau daemon/worker restent à compiler et activer à ce checkpoint.
Les versions40e4 ci-dessus décrivent encore les clients effectivement installés.
Revue r6 a désormais rendu expected_base_commit_mismatch ; Opus2D r3 a rendu
worker_unavailable. Ces refus ne sont pas transformés en succès. Aucun nouvel
essai des projets demandé avant la correction du worker et l’activation T3.

## Premier reçu conservé pour l’historique

Les états et chemins ci-dessous décrivent le premier lot, avant les compléments
ci-dessus ; ils ne décrivent pas les processus courants.

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

## Versions du premier reçu (historiques)

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


## Complément de preuve d'autonomie
Le banc natif complet est désormais PASS : deux scénarios avec vrai maître,
worker et wrapper, fournisseur simulé, sans service T3. La demande automatique
omet project_id ; le dossier et le commit sont vérifiés, puis le résultat et
l'unicité du rejeu. Le second scénario retarde Ready de6s et conserve le bail60s.
Le banc historique garde son protocole antérieur et ses quatre cas passent.
Commandes exécutées sur Cartae :
cargo test -p bridget-daemon --features test-support --test native_delegation_e2e spec172_parent_natif_auto_sans_t3_resout_projet_et_attend_ready -- --nocapture
cargo test -p bridget-daemon --features test-support --test native_delegation_e2e remote162_e2e_delegate_maitre_reel_resultat_unique_cwd_et_modele -- --nocapture
Logs privés :
/tmp/bridget172-native-e2e.log
/tmp/bridget172-native-legacy-e2e.log
La recette avec modèle parent natif réel reste distincte et non validée.
Le tour automatique a aussi rencontré permission_attestation_unavailable
dans l'adaptateur T3, avant toute admission ; aucun contournement effectué.
