# Carte de reprise — rc1

> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> AVANT d executer quoi que ce soit : LIRE `/home/moi/.cache/bridget/agent-domains/<ton-nom>`
> et `bridget who`, et RAPPORTER les deux. `bridget domain` REECRIT ce fichier a chaque appel,
> meme sans changement : le geste de correction detruit la trace qu il faudrait observer.
> ENSUITE seulement, executer `bridget domain bridget` et verifier la sortie.
> Le domaine NE SURVIT PAS au remplacement du processus : il est derive du nom du repertoire
> de la racine git (`derive_domain`, crates/bridget-daemon/src/wrapper.rs:983), jamais de `bridget`.
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
