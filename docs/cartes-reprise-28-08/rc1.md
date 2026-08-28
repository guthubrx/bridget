# Carte de reprise — rc1

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


- Agent : `rc1`
- Emise : 2026-08-28T14-35-44Z
- Identifiant ledger : `mcp-2734185-6a919cc0-115`
- Recueillie par : bridget, referent, sur decision humaine du 28/08

---

CARTE DE REPRISE rc1
1 ÉTAT — Réserve nommée, aucune mission active. C1/C2 et la correction RouterError sont livrées ; aucune campagne en cours.
2 FAITS — C1/C2 publiées sur origin/amend/maicie-c1c2c3, tête 320971bb (C1/C2 vérifiées : C1 nominal + mutant M3 + restauration ; C2 session-049 5/0 + oracle exclusion + 2 mutants). Correction routeur publiée sur origin/fix/router-nom-coherent, commit ca26c908, base main 8a98670, cargo check workspace RC=0. Lot fil-outils e3331fb inventorié : 2 fichiers transport, 114 passed/1 ignored, composition merge-tree RC=0 mais retard de 219 commits ; décision humaine requise avant intégration. Les tests workspace complets de C3 et les charges C4/C5 ne sont pas revendiqués par moi.
3 RESTE — Attendre uniquement un mandat humain explicite ou une reprise demandée par Bridget/jc2 ; ne rien relancer de soi-même.
4 CHEMINS — /home/moi/revue/rc1 ; branche locale fix/pruning-publication ; branches publiées origin/amend/maicie-c1c2c3 et origin/fix/router-nom-coherent. Trois journaux conservés : /tmp/rc1-mut3.txt, /tmp/rc1-mut4.txt, /tmp/rc1-mutant.txt.
5 PIÈGES — Ne pas confondre branche publiée et worktree local ; les anciens arbres de composition ont été supprimés. Ne pas conclure à partir d’un code retour sans ligne native ; ne pas lancer C3/C4/C5 sans mandat et sans relecture des bases courantes.
6 DÉLÉGATIONS — Je connais les résultats C1/C2, la correction routeur et l’inventaire e3331fb. J’ignore l’état courant des délégations Maicie/jc2/rc7 et toute décision humaine sur f09feb7 ou le lot fil-outils ; je ne les déduis pas.
