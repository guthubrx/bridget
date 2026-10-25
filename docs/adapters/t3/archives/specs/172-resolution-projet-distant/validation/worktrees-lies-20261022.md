# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/worktrees-lies-20261022.md](../../../../../../../specs/172-resolution-projet-distant/validation/worktrees-lies-20261022.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Maître Mac PID 68244, SHA-256 248755a6f69f20665743e7fb289b73c50083bb12a2cd67849421425b2f934c4e.
- Worker Cartae PID 2219043, SHA-256 4cc657c2c8e287a85160d9359a8d587422308210fe00c250557254bf396040cd.
- Application T3, serveur et pont : PID 28528, 28591 et 898 conservés, ASAR inchangé.
- Zéro commande worker et zéro wrapper Bridget actif avant les deux bascules.
  Ce contrôle ne prétend pas inventorier les calculs T3/SSH indépendants ; aucun n'a été arrêté.
- 21 tests Python, quatre tests Rust distincts affectés et contre-revue APPROVE.
  Le test intégré final comprend cinq scénarios. Aucun succès de suite globale déduit.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

Code relu : /Users/moi/11.Repositories/t3code-local/apps/server/src/vcs/GitVcsDriverCore.ts,
fonction resolveRepositoryPathsUncached, lignes 1290–1367.
T3 distingue le répertoire Git commun, résolu en chemin réel, du dossier
de travail. Bridget reprend cette preuve pour reconnaître un checkout lié.

<a id="extrait-4"></a>

## Extrait 4

Un recu prive est ecrit avant extraction, puis marque termine seulement apres
le succes de Git. Un recu incomplet est refuse au rejeu, sans suppression ni
reprise automatique. Un ancien dossier sans recu et avec des fichiers suivis
manquants ou modifies est aussi refuse. Le format du commit est verifie au
depart et a destination. T3 arme une transaction de liaison et un rollback
apres creation dans WorktreeMcpService (235–278) ; Bridget conserve plutot le
recu distant incomplet et les fichiers, pour eviter une suppression distante
non verifiee.

<a id="extrait-5"></a>

## Extrait 5

Cause observee : SSH non interactif choisit /usr/bin/git (2.43.0), alors que le
service choisit deja /home/moi/.local/lib/bridget-tools/git-2.56.0/bin/git.
Le choix est maintenant place avant le retour non interactif dans
/home/moi/dotfiles/bash/.bashrc, cible du lien /home/moi/.bashrc, conserve.
Le lien stable /home/moi/.local/lib/bridget-tools/git-current pointe vers 2.56.0.
Une connexion SSH neuve et un shell de connexion choisissent tous deux 2.56.0,
sans PATH ajoute a leur commande. Sauvegarde et empreintes privees conservees.
T3 resout aussi la commande Git depuis l'environnement du processus
(GitVcsDriverCore.executeGit) ; cette correction de shell appartient a Cartae.
