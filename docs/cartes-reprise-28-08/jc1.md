# Carte de reprise — jc1

> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> Executer `bridget domain bridget` et verifier la sortie.
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
5) PIÈGES — Toujours vérifier SHA/branche et `origin/main` avant mesure ; ne jamais confondre precheck partagé et garde d’autorisation. Les motifs publics sont uniformes, l’audit interne est la seule raison détaillée. Les mutants doivent viser l’occurrence exacte et être restaurés par empreinte. Les chemins de policy/audit sont lus depuis l’environnement ; sans injection sûre, impossible de prouver le branchement daemon. Les tests ciblés peuvent manquer les binaires auxiliaires ; compiler `--no-run` avant comptage. Ne pas lancer de banc daemon sans fenêtre explicite.
6) DÉLÉGATIONS — Je porte trois délégations à l’état créée/non dispatchée ; je n’ai pas leurs trois identifiants ni leur portée exacte dans le contexte courant. Elles ne doivent pas être considérées comme des missions actives tant qu’un mandat complet objectif/délégation/message n’est pas transmis.
