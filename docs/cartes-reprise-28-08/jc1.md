# Carte de reprise — jc1

> **TOUTE AFFIRMATION D'INTEGRATION DE CETTE CARTE EST DATEE DU 28/08 ET NON REVERIFIEE DEPUIS.**
> Les cartes ne vieillissent pas toutes de la meme facon : certaines disent « integre dans main »,
> d'autres « non attesté par moi ». La seconde posture est celle qui a servi le 28/08, parce qu'elle
> fait porter la charge de preuve. Avant de t'appuyer sur une integration annoncee ici :
> `git fetch` puis `git merge-base --is-ancestor <sha> origin/main`. Ne deduis pas, mesure.


> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> AVANT d executer quoi que ce soit, RELEVE LA VRAIE VARIABLE D ENTREE — le cwd de TON PROCESSUS :
> `readlink /proc/<ton-pid>/cwd` puis `git -C <ce-cwd> rev-parse --show-toplevel`.
> C est le `basename` de cette racine git qui devient ton domaine. Ton cwd n est PAS celui que
> tu crois : le processus tourne souvent un cran plus bas que le repertoire que tu observes.
> NE PERDS PAS DE TEMPS a lire `agent-domains/<ton-nom>` : un nom neuf n a JAMAIS de fichier,
> la reponse sera toujours `absent`. Cette consigne, presente ici jusqu au 28/08 16h44, etait
> un rite — corrigee sur demonstration de rc5-flux, qui a invalide sa propre proposition.
> ENSUITE seulement, executer `bridget domain bridget` et verifier la sortie.
> Le domaine NE SURVIT PAS au remplacement du processus : il est derive du nom du repertoire
> de la racine git (`derive_domain`, crates/bridget-daemon/src/wrapper.rs:983), jamais de `bridget`.
> Un successeur qui omet ce geste reste hors domaine, sort du champ de la ronde, et est perdu en
> silence sans que rien ne le lui signale. Aucune carte ne portait cette ligne avant le 28/08 15h53.


- Agent : `jc1`
- Emise : 2026-08-28T14-15-52Z
- Identifiant ledger : `mcp-1797399-6a919818-42`
- Recueillie par : bridget, referent, sur decision humaine du 28/08

---

CARTE DE REPRISE jc1
1) ETAT — Dernier travail actif : revue/correction du court-circuit d’audit dans le daemon (objectif 87563a36…, branche locale temporaire `/tmp/audit-guard-review.ymR2/repo`). J’ai déplacé le precheck pour que les mutations passent par la garde ; compilation daemon `--no-run` verte. Témoins d’audit daemon non ajoutés/exécutés ; lot resté AMENDER.
2) FAIT — Lot 038 regeneration politique : verdict borné puis GO global après six mutants mesurés (les deux derniers par jc3), intégré dans main. Mutants exécutés par moi : AfterRename, O_NONBLOCK, unicité politique, unicité verrou ; lignes et témoins transmises. Migration v20, garde greffe, page GUI et autres revues précédentes livrées/mergées selon les SHA communiqués au ledger. Aucun matériel de compilation courant garanti.
3) RESTE — Audit daemon : écrire trois oracles traversant `handle_wrapper_message` (expéditeur divergent, nom absent, instance absente), contrôle positif et mutant retirant l’audit ; vérifier absence de dépôt et ligne d’audit. Puis recompiler/tester, restaurer, committer et pousser seulement après validation humaine/auteur.
4) CHEMINS — Dernier clone audit : `/tmp/audit-guard-review.ymR2/repo` (à vérifier ; ne pas supposer présent). Cibles temporaires sous `/tmp/audit-guard-review.ymR2/target`. Les clones lot038 ont été supprimés ; aucun fichier persistant `/home/moi/revue/jc1` modifié. Aucun secret/chemin machine à versionner.
5) PIÈGES — Toujours **`git fetch` PUIS** vérifier SHA/branche et `origin/main` avant mesure — **CORRIGE LE 28/08 16h13** : la formulation initiale disait seulement « vérifier `origin/main` », ce qui reproduit l'erreur en croyant l'éviter. `origin/main` sans fetch est une ref de suivi figée au dernier fetch : elle a rendu deux réponses contradictoires le 28/08. La formulation complète est celle de `jc2.md` lignes 64-65. Voir aussi le faux vert de la branche locale `main`, divergente dans les trois dépôts testés ; ne jamais confondre precheck partagé et garde d’autorisation. Les motifs publics sont uniformes, l’audit interne est la seule raison détaillée. Les mutants doivent viser l’occurrence exacte et être restaurés par empreinte. Les chemins de policy/audit sont lus depuis l’environnement ; sans injection sûre, impossible de prouver le branchement daemon. Les tests ciblés peuvent manquer les binaires auxiliaires ; compiler `--no-run` avant comptage. Ne pas lancer de banc daemon sans fenêtre explicite.
6) DÉLÉGATIONS — Je porte trois délégations à l’état créée/non dispatchée ; je n’ai pas leurs trois identifiants ni leur portée exacte dans le contexte courant. Elles ne doivent pas être considérées comme des missions actives tant qu’un mandat complet objectif/délégation/message n’est pas transmis.
