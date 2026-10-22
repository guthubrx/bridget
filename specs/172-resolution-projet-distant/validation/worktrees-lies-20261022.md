# Préparation des worktrees liés — 22 octobre 2026

## Livraison active et qualification réelle

Le build 172-76b682d6 est actif sur Mac et Cartae. Source de code :
76b682d6bfdf315a9a3d10de6dc75d0879a265ae, fusionnée dans main et poussée.

- Maître Mac PID 68244, SHA-256 248755a6f69f20665743e7fb289b73c50083bb12a2cd67849421425b2f934c4e.
- Worker Cartae PID 2219043, SHA-256 4cc657c2c8e287a85160d9359a8d587422308210fe00c250557254bf396040cd.
- Application T3, serveur et pont : PID 28528, 28591 et 898 conservés, ASAR inchangé.
- Zéro commande worker et zéro wrapper Bridget actif avant les deux bascules.
  Ce contrôle ne prétend pas inventorier les calculs T3/SSH indépendants ; aucun n'a été arrêté.
- 21 tests Python, quatre tests Rust distincts affectés et contre-revue APPROVE.
  Le test intégré final comprend cinq scénarios. Aucun succès de suite globale déduit.

Recette réelle depuis le contexte T3 Architecture Opus2D :
Task fdd9c6e9-e16d-4994-a763-e7185870cfca, machine cartae,
package opus2d@0.1.0, checkout :
/home/moi/10.Scripts/69.opus2D/.worktrees/158-integration-continuite256
HEAD directement lu et base attendue identiques :
ecef0730a9c3b5e7c6381c954537659f07baac99.
Marqueur effectivement émis : BRIDGET172_WORKTREE_SHARED_LIVE_OK.
Résultat intégral lu, ACK et clôture par son propriétaire ; aucune erreur,
continuable=false, cleanup_done=true. Mode Shared : aucune garde Worktree acquise.
Aucune copie du mandat métier parcs/eau ; sa fabrication reste sous T3.

La première recette dccaf64a a exécuté les lectures correctes, mais n'a pas
émis son marqueur : le parent avait ajouté une commande Git incompatible avec
les outils de cet enfant discovery. Ce résultat reste partiel. La recette
corrigée, strictement fondée sur les fichiers, est distincte et ne remplace
aucune mission métier. Le contrat de recette ne requiert aucun shell.

Git Cartae 2.56 est le défaut durable des nouveaux SSH et shells de connexion,
vérifié sans PATH temporaire. La configuration du worker emploie aussi le lien
stable /home/moi/.local/lib/bridget-tools/git-current/bin.
Configuration utilisateur :
/home/moi/dotfiles/bash/.bashrc
Le lien /home/moi/.bashrc est conservé. Mac utilise Git 2.55, compatible.
La sauvegarde de configuration et les empreintes restent privées.

Confirmation finale envoyée aux 12 agents Opus2D présents dans l'annuaire.
Les API ont accepté les envois ; cela ne prouve pas leur lecture. Aucune
migration des tâches déjà engagées ni nouvelle recette demandée à la flotte.

Nettoyage : checkout de compilation Mac et deux binaires intermédiaires
jamais activés retirés après contrôle lsof vide. Binaire courant, sauvegarde,
preuves et cache Cargo réutilisable conservés. Cinq worktrees Cartae propres
et fusionnés conservés : inventaire lsof incomplet, aucune suppression forcée.
Le worktree Opus2D de qualification reste à son propriétaire.

Réserve faible non bloquante de la revue : une erreur OS au lancement de Git
reste fermée par conservation du reçu. Pas de corruption ni élargissement de
droits ; aucune panne de ce type observée dans la recette réelle. La revue a
approuvé le correctif. Contrairement à sa lecture limitée, le worktree auteur
existe encore : ses empreintes et celles du worktree de revue ont été
comparées exactement par l'opérateur.

Preuves privées :
/home/moi/.cache/bridget-worktrees172
/Users/moi/.cache/bridget-worktrees172


## Cause observée

La demande opus158-156-parcs-publications-candidates-cartae-20261022-01
reçoit routing_candidate_unavailable avec dix refus mapping_unavailable,
avant toute Task. Aucun reçu pour cette clé dans le store maître.

Source : /Users/moi/Nextcloud/10.Scripts/69.opus2D/.worktrees/158-integration-continuite256
Destination attendue : /home/moi/10.Scripts/69.opus2D/.worktrees/158-integration-continuite256
Commit demandé : 37485568b9d94e22ab20db52a62650eeede4d9d8.

Les correspondances Opus2D des deux hôtes sont identiques. Le commit est déjà
présent sur Cartae. Le dossier distant attendu est absent. Le helper importe
les objets Git du dépôt principal, puis la sonde exige le dossier inexistant.

## Référence T3 et correction

Code relu : /Users/moi/11.Repositories/t3code-local/apps/server/src/vcs/GitVcsDriverCore.ts,
fonction resolveRepositoryPathsUncached, lignes 1290–1367.
T3 distingue le répertoire Git commun, résolu en chemin réel, du dossier
de travail. Bridget reprend cette preuve pour reconnaître un checkout lié.

Différence distante : le helper prépare ce checkout sur Cartae, via la
fédération SSH existante. Son .git doit être un fichier et son répertoire
Git commun doit être celui de la source déclarée. Seul le chemin relatif
vérifié est transmis. La destination crée un checkout en tête détachée au
commit demandé. Les hooks restent désactivés. Aucun nouveau mapping,
protocole worker, fournisseur ou droit.

Le checkout principal, ses fichiers locaux et ses branches restent inchangés.
Un checkout existant doit partager le même dépôt Git. Le helper ne le
réinitialise pas. Les contrôles worker de commit, droits, identité,
disponibilité et rafraîchissement partagé restent ceux déjà en place.

Un dossier étranger, un lien symbolique et un dépôt imbriqué indépendant
sont refusés. Les fichiers non committés ne sont pas transférés.

## Vérification ciblée

15 tests Python PASS sur de vrais dépôts Git. Le test Rust existant des gardes
mapping absent, ambigu, explicite et origine passe aussi (1 PASS). Le transport SSH est simulé par
un exécutable qui lance réellement le helper distant dans une seconde
arborescence. Les cinq nouveaux cas couvrent : création et rejeu, dossier
étranger, lien symbolique, dépôt indépendant et hooks de destination avec
contrôle positif.

Le RED utilise la fixture finale et le seul script de production rétabli à
la base 4c23fcc07ffa3cbbd4c0849913714663cdf58b75. La préparation réussit à
tort, puis la lecture du dossier absent échoue. La même fixture passe avec
le correctif.

Preuves privées :
/home/moi/.cache/bridget-worktrees172/red-final.json
/home/moi/.cache/bridget-worktrees172/red-final.log
/home/moi/.cache/bridget-worktrees172/tests.json
/home/moi/.cache/bridget-worktrees172/green-final.log

Ce checkpoint ne qualifie pas encore l'activation ni le lancement de la
mission Opus2D. Ils doivent être consignés séparément. Aucun parcours
d'interface modifié ; pas de diaporama requis.


## Contre-revue et correction du worker

La revue 0043865a refuse le checkpoint ad980493 : le helper prepare le bon
checkout, mais le worker sonde et rafraichit encore le principal. La mutation
est maintenant bornee au checkout reel. Le repertoire Git commun doit rester
celui du mapping. Une correspondance limitee a un sous-dossier ne gagne aucun
droit de rafraichir son parent.

Reference T3 relue : GitVcsDriverCore.refreshCheckedOutBranch (3668–3726)
applique statut, filiation et avance rapide dans input.cwd. Bridget conserve
son avance rapide seule, sans le reset que T3 autorise dans certains cas.
Les gardes de missions actives restent appliquees a ce checkout.

Le mode isole part aussi du checkout source reel. Son nouveau dossier et la
projection des droits sont relies a ce checkout, puis revalides avant remise.
Le mapping du registre reste inchange. Le mode partage ne bouge jamais le
principal pour satisfaire le commit d'un autre worktree.

Un recu prive est ecrit avant extraction, puis marque termine seulement apres
le succes de Git. Un recu incomplet est refuse au rejeu, sans suppression ni
reprise automatique. Un ancien dossier sans recu et avec des fichiers suivis
manquants ou modifies est aussi refuse. Le format du commit est verifie au
depart et a destination. T3 arme une transaction de liaison et un rollback
apres creation dans WorktreeMcpService (235–278) ; Bridget conserve plutot le
recu distant incomplet et les fichiers, pour eviter une suppression distante
non verifiee.

Decision sur un checkout existant : le helper ne le deplace pas. Un commit
ancien valide reste soumis a la sonde du worker. Une avance rapide ne se fait
qu'au Prepare, si le checkout est propre et sans mission active. Une divergence
est refusee avant admission. Cette politique conserve le comportement Shared
existant ; imposer l'egalite dans le helper aurait interdit sa mise a jour sure.

## Git Cartae : correction durable de l'environnement

Cause observee : SSH non interactif choisit /usr/bin/git (2.43.0), alors que le
service choisit deja /home/moi/.local/lib/bridget-tools/git-2.56.0/bin/git.
Le choix est maintenant place avant le retour non interactif dans
/home/moi/dotfiles/bash/.bashrc, cible du lien /home/moi/.bashrc, conserve.
Le lien stable /home/moi/.local/lib/bridget-tools/git-current pointe vers 2.56.0.
Une connexion SSH neuve et un shell de connexion choisissent tous deux 2.56.0,
sans PATH ajoute a leur commande. Sauvegarde et empreintes privees conservees.
T3 resout aussi la commande Git depuis l'environnement du processus
(GitVcsDriverCore.executeGit) ; cette correction de shell appartient a Cartae.

Le checkpoint final des tests, la revue et les activations doivent suivre.


## Qualification finale de la revision corrective

18 tests Python et 4 tests Rust distincts PASS. Le nouveau test du worker parcourt quatre cas : partage/isole, base descendante/divergente du principal. Prepare et recheck passent, le principal reste identique, le dossier execute contient le commit demande. Une demande de retour arriere partage est refusee a la sonde.

RED final du worker : meme fixture, code de production ad980493. L assertion main must not move echoue sur deux HEAD differents. Le GREEN final utilise exactement la meme fixture. Aucun fournisseur reel dans ces tests, aucune suite complete. Les deux shells frais choisissent Git 2.56.0 ; les quatre tests finaux ne forcent pas PATH.

Preuves privees : /home/moi/.cache/bridget-worktrees172/rust-r3.json, /home/moi/.cache/bridget-worktrees172/python-r3.json, /home/moi/.cache/bridget-worktrees172/worker-red-final.json et leurs journaux. Empreintes source dans le JSON public voisin. Activation et recette MCP reelle encore non qualifiees a ce checkpoint.


## Levee des deux objections moyennes de c51603b8

Echec synchrone de creation : le recu est retire uniquement si Git a termine avec un refus et si la cible et son inscription dans l inventaire Git sont toutes deux absentes. Un delai expire, un inventaire illisible ou une inscription residuelle conserve le recu incomplet. Aucun retry automatique ni suppression de travail.

21 tests Python PASS. RED cible avec le script fadd74df : la liste des recus n est pas vide apres un echec propre, donc le test echoue. GREEN sur la meme fixture avec le correctif. Tests supplementaires sur inscription sans dossier, delai sans dossier et adoption sans recu. Le test du worker passe sur cinq scenarios, dont HEAD lie C0 vers descendant B : seule la cible avance, principal inchange, recheck reussi. Les trois autres tests Rust de r3 restent valides : code de production worker byte-identique, seul le test est etendu.

mapping.source_root n est pas une ecriture morte : inspect_candidate et recheck transmettent le mapping a native_permissions, qui lit source_root pour traduire le cwd du parent. Le registre reste inchange. Les fichiers ignore du projet ne sont pas modifies ; la proprete du principal reste une garde quand on demande son rafraichissement.

Reference T3 pour ce delta : WorktreeMcpService arme une transaction apres creation reussie et un rollback. Bridget exige une preuve d absence de toute creation avant de retirer son recu distant ; une issue inconnue reste fermee. Pas de nouveau parcours T3 equivalent pour un worker distant.

Preuves : /home/moi/.cache/bridget-worktrees172/tests-r4.json, /home/moi/.cache/bridget-worktrees172/clean-failure-red.log, /home/moi/.cache/bridget-worktrees172/python-r4.log, /home/moi/.cache/bridget-worktrees172/worker-green-r4.log.
