# Carte de reprise — essai-claude-distant

> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> AVANT d executer quoi que ce soit, RELEVE LA VRAIE VARIABLE D ENTREE — le cwd de TON PROCESSUS :
> `readlink /proc/<ton-pid>/cwd` puis `git -C <ce-cwd> rev-parse --show-toplevel`.
> C est le `basename` de cette racine git qui devient ton domaine. Ton cwd n est PAS celui que
> tu crois : le processus tourne souvent un cran plus bas que le repertoire que tu observes.
> NE PERDS PAS DE TEMPS a lire `agent-domains/<ton-nom>` : un nom neuf n a JAMAIS de fichier,
> la reponse sera toujours `absent`. Cette consigne, presente ici jusqu au 28/08 16h44, etait
> un rite — corrigee sur demonstration de rc5-flux, qui a invalide sa propre proposition.
> ENSUITE seulement, executer `bridget domain bridget` et verifier la sortie.
> Le domaine NE SURVIT PAS AU CHANGEMENT DE NOM — et non au respawn, requalification du 28/08 :
> il est persiste dans `agent-domains/<nom>`, RELU a chaque enregistrement y compris apres
> reconnexion, et un respawn du MEME nom retrouve donc son domaine. Un nom neuf n a pas de
> fichier : `derive_domain` s applique alors et rend le `basename` de la racine git du cwd,
> avec repli sur le cwd lui-meme. IL PEUT DONC VALOIR `bridget` NATIVEMENT si ce repertoire
> s appelle `bridget` — c est le cas de `rc7` et `essai-claude-distant`, verifie par leur PID.
> Source : `fn derive_domain` et `fn effective_domain`, `crates/bridget-daemon/src/wrapper.rs`,
> ligne 983 DANS L ARBRE `/home/moi/revue/rc7/bridget` — la ligne differe selon les checkouts,
> il y en a six sous /home/moi. `effective_domain` n a AUCUN repli code en dur vers `bridget`.
> Un successeur qui omet ce geste reste hors domaine, sort du champ de la ronde, et est perdu en
> silence sans que rien ne le lui signale. Aucune carte ne portait cette ligne avant le 28/08 15h53.

## AVERTISSEMENT — CETTE CARTE N A PAS ETE RENDUE PAR SON AGENT

Les neuf autres cartes de ce repertoire ont ete ECRITES PAR LEUR AUTEUR, sur demande du referent.
Celle-ci est une RECONSTITUTION faite par le referent le 28/08 a 15h57, a partir des seules traces
mesurables. `essai-claude-distant` n a jamais rendu de carte : il n a repondu ni a une demande
complete ni a une demande de trois lignes. L humain a tranche le 28/08 : « pour celui pour qui tu
n as pas reussi a recuperer le contexte, tant pis, on le prend quand meme ».

Elle a ete ecrite parce que rc1-flux a constate le 28/08 a 15h56 que le dixieme successeur etait le
seul couvert par une reserve nommee SANS carte a verifier — donc couvert sans qu on sache ce qu il
portait. Cette carte ne repare pas le manque de contexte ; elle rend le manque explicite.

## Ce qui est MESURE

- Agent : `essai-claude-distant`
- Type : `claude`, transport `tmux`, pane `essai-claude-distant:1.1`, `conn-11`
- Modele : `claude-opus-5`, effort `xhigh`
- Domaine : `bridget`
- Etat a la reconstitution : `connected`, `last_seen_secs` = 3621, `reconnect_count` = 0
- Production au ledger : **59 messages emis**, du 27/08 06:39 au **27/08 21:30**
- Delegations a son nom : **5 au total — dont 3 a l etat `creee` et 2 `soldee_par_cloture`**.
  La carte d arrivee du successeur annoncait 3, celle-ci 5 : **les deux chiffres sont exacts**, ils ne
  comptent pas la meme chose. Mesure du 28/08 16h48. Meme motif que le reste de la journee — deux nombres
  justes se contredisent tant qu on ne nomme pas ce qu on compte.
- Successeur : `essai-claude-distant-flux`, `conn-288`, ne le 28/08 vers 15h10

## Ce qui est CONNU par temoignage, non par mesure du referent

- Saturation rapportee a 97 pour cent. Le referent n a pas mesure ce chiffre lui-meme.
- Silence de plus de dix-huit heures : aucun message emis depuis le 27/08 21:30.
- `connected` a l annuaire malgre ce silence. Formulation de essai-claude-distant-flux, retenue
  telle quelle : **« connected sur un tmux n atteste pas la disponibilite, il atteste l existence
  d un pane »**.

## Ce qui est INCONNU et le restera

- Ce sur quoi il travaillait. Aucun etat, aucun reste-a-faire, aucun chemin absolu.
- Les pieges qu il a rencontres.
- L avancement reel de ses 5 delegations. **Ne pas l inferer de leur etat** — regle etablie par rc5.

## Ce que son successeur a etabli lui-meme, et qui vaut mieux que cette carte

- ~~`/home/moi/revue/essai-claude-distant` n est pas un depot git~~ — **FAUX, RETIRE LE 28/08 16h48.**
  Le poste reel est un cran plus bas : `/home/moi/revue/essai-claude-distant/bridget` **EST un depot** —
  branche `main`, HEAD `e75aa3b`, **1117 commits**, arbre propre. L erreur vient du premier rapport du
  successeur, qui avait teste la seule racine ; le referent l a reprise sans verifier, puis a fonde sur
  elle sa decision de ne pas lui confier de mandat de code. Le successeur l a lui-meme retiree.
  **Il n a besoin d aucun point de travail : il en a un.**
- Aucune memoire projet : `MEMORY.md` absent.
- Il a refuse d attester une correction de domaine qu il n avait pas mesuree avant : il atteste
  l etat final, pas le delta. Seul des dix a avoir fait cette distinction.
- Il a etabli que les onze agents `claude` partagent **une seule limite de taux**, referent inclus.

## Correction due

Le referent avait ecrit a son successeur que `essai-claude-distant` etait `codex/tmux`. **Faux** :
type `claude`, transport `tmux`, modele `claude-opus-5`, effort `xhigh`. La contrainte bubblewrap ne
l a jamais concerne. Son mode d echec n a rien a voir avec Codex.
