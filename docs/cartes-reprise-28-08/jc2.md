# Carte de reprise — jc2

> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> AVANT d executer quoi que ce soit : LIRE `/home/moi/.cache/bridget/agent-domains/<ton-nom>`
> et `bridget who`, et RAPPORTER les deux. `bridget domain` REECRIT ce fichier a chaque appel,
> meme sans changement : le geste de correction detruit la trace qu il faudrait observer.
> ENSUITE seulement, executer `bridget domain bridget` et verifier la sortie.
> Le domaine NE SURVIT PAS au remplacement du processus : il est derive du nom du repertoire
> de la racine git (`derive_domain`, crates/bridget-daemon/src/wrapper.rs:983), jamais de `bridget`.
> Un successeur qui omet ce geste reste hors domaine, sort du champ de la ronde, et est perdu en
> silence sans que rien ne le lui signale. Aucune carte ne portait cette ligne avant le 28/08 15h53.


- Agent : `jc2`
- Emise : 2026-08-28T14-30-32Z
- Identifiant ledger : `mcp-2017281-6a919b88-79`
- Recueillie par : bridget, referent, sur decision humaine du 28/08

---

CARTE DE REPRISE jc2 — PASSATION VERS AGENT EN FLUX

1. ETAT

Aucune mission active, aucun mutant en vol, aucun processus ni matériel temporaire détenu. Je suis en réserve nommée. Le dernier travail était la session 059, mesure falsifiable du rapport vérification/production. Il est livré, relu hostilement et attesté par Bridget, mais son intégration appartient à l’humain.

Le worktree 059 a été retiré proprement. Dernier contrôle avant retrait : arbre propre, tête locale=origin. /tmp/jc2-s059.bDcTvY est absent et git worktree list ne le référence plus.

2. FAITS LIVRÉS

Branche : origin/session-059-mesure-verification-production
Base contractuelle : 394c0f5c8d352a606c85db7232bb1ad4fa79446e
Tête attestée : b52b7369ef0fb5c5765a76d1c09c0c7c46d716fc
Quatre commits :
- b984563 docs(spec): Formaliser la mesure verification production
- ec6880f test(ronde): Poser oracle rouge du rapport vp
- e266fcb feat(ronde): Publier la mesure verification production
- b52b736 test(ronde): Attester le ratio par racines

Delta attesté : 7 fichiers, +841/-3, aucun fichier de session 058. Divergence mesurée 0/4 depuis la base ; merge-tree rc=0, arbre 7088417083cbd3d36e67663d9c34b30b03a715e9.

Résultat métier :
- baseline humaine reproduite : 169 identifiants, 6253 octets avec LF final, SHA-256 e2a279624ecd4fbd9ca7f8f97effa45777c15b6b3b9574f3d43a9d90298d5a55, reference.matches=true;
- seulement 5/169 objectifs classables de manière réfutable, soit 2,96 % ; 164 restent indeterminate avec causalite_historique_indisponible;
- production=1, vérification=4 au niveau objectifs ; production=1, vérification=1 au niveau racines ; facteur d’expansion réel=4,0;
- conclusion : ratio historique honnête impossible. Les deux ratios sont unavailable/classification_coverage_incomplete;
- target=null et origin_used=false; aucune cible décidée ou cachée.

Publication : la mesure est ajoutée au JSON et au résumé de la ronde passive existante. Elle lit uniquement la copie SQLite jetable ; aucune écriture DB, décision ou émission. La baseline gelée Paris et la glissante 24 h sont distinctes.

Preuves attestées deux fois, par jc2 puis Bridget :
- rouge avant production après JSON réel : AssertionError: la ronde réelle ne publie pas encore la mesure V/P;
- nominal vert : rapport V/P réel : baseline, couverture et invariance de racine vérifiées;
- mutant root_id -> objective_id : AssertionError: le découpage a déplacé le ratio par racines: 4.0;
- restauration SHA-256 de bridget-ronde.py : 62fedf91a7a5d17c55600a830a27ae9fce43a1eb89d73d06d6a2b73fd2a81a2b;
- harnais historique, py_compile, bash -n, ruff check, diff-check et banc d’installation verts;
- ruff format --check rouge sur base ET tête, défaut préexistant attesté indépendamment, aucun vert de format revendiqué.

3. RESTE

Prochaine étape autorisée : attendre la décision humaine d’intégrer ou non 059. Ne pas intégrer, rebaser, amender, fixer une cible ni compléter les 164 classes sans mandat humain.

Si l’humain autorise l’intégration :
1) fetch et relever le origin/main courant ;
2) vérifier que b52b7369ef0fb5c5765a76d1c09c0c7c46d716fc est toujours l’objet jugé ;
3) remesurer merge-base/merge-tree contre ce main courant ;
4) matérialiser et tester l’arbre de fusion avant merge ;
5) activer la ronde seulement DEPUIS un main propre et admis, jamais depuis la branche.

La suite conceptuelle est une provenance daemon réellement attestée pour les flux futurs. L’humain doit d’abord l’arbitrer. Elle n’appartient pas à 059 et ne doit pas être ouverte automatiquement.

4. CHEMINS ABSOLUS

Checkout local historique : /home/moi/revue/jc2
ATTENTION : dernier état connu avant passation = ancien checkout sur session-024 avec changements utilisateur sans rapport. Ne pas y changer de branche, ne pas le nettoyer et ne pas supposer qu’il contient 059.

Ancien worktree 059, désormais ABSENT : /tmp/jc2-s059.bDcTvY
Ancien dépôt 059, désormais ABSENT : /tmp/jc2-s059.bDcTvY/repo

Base Maicie réelle : /home/moi/.cache/bridget/maicie-state/maicie.sqlite3
Configuration Maicie : /home/moi/.config/maicie/config.json
Ronde installée actuelle : /home/moi/.local/bin/bridget-ronde
Archives de ronde : /home/moi/.cache/bridget/rondes

Les fichiers source 059 n’ont actuellement AUCUN chemin local attesté après nettoyage. Ils vivent dans l’objet Git distant b52b7369. Dans un nouveau worktree choisi et vérifié par le successeur, les quatre zones sont scripts/bridget-ronde.py, scripts/test-bridget-rapport-vp.sh, scripts/test-bridget-ronde.sh et specs/059-mesure-verification-production/ ; ne pas prétendre qu’elles sont présentes avant checkout.

Fichiers explicitement NON touchés par 059, alors disputés par 058 :
/home/moi/revue/jc2/crates/bridget-daemon/src/ui.rs
/home/moi/revue/jc2/plugins/maicie/src/lib.rs
/home/moi/revue/jc2/plugins/maicie/src/main.rs
/home/moi/revue/jc2/plugins/maicie/src/greffe_service.rs

5. PIÈGES

- Ne pas reconstituer la population 169 avec un jour civil UTC ou la glissante actuelle. La baseline exacte est (1787788534, 1787874934], fenêtre gelée Europe/Paris, ordre cree_at,id, LF final.
- Ne pas utiliser origin : les deux objectifs qui le portaient étaient auto_generated malgré une cause humaine. Le test impose origin_used=false.
- Ne pas améliorer la couverture par mots-clés. 2,96 % honnêtes valent mieux qu’un faux 100 %.
- Ne pas compter seulement les objectifs : quatre charges réelles appartiennent à une racine. Le mutant prouve que remplacer root_id par objective_id déplace le ratio 1,0 vers 4,0.
- ruff format --check est rouge historiquement ; ne pas l’imputer à 059 ni annoncer un vert global.
- L’installateur de ronde refuse correctement une branche non intégrée. Ne pas contourner ce refus.
- main peut avoir avancé après la dernière mesure. Un nom ou une ancienne valeur de main n’est pas une base.
- Le gros corpus ne doit pas être dumpé dans une conversation ; agréger et rendre les compteurs.
- L’ancien checkout /home/moi/revue/jc2 contient des changements qui appartiennent à l’utilisateur.

6. DÉLÉGATIONS

Mandat 059 :
- objectif 924ccb3c-2ac1-49e7-85a9-746d32d36a11
- délégation f3ca9807-bdf2-4be9-88bb-bbeef803a86e
- message 78cba682-d9e5-434c-b4fb-c3078606118e
- dernier fait reçu : charge fermée, lot attesté par Bridget.

Je sais que l’intégration de 059 est réservée à l’humain et n’était pas faite au dernier message. J’IGNORE si elle a eu lieu depuis ; vérifier, ne pas déduire.

Je ne connais aucune délégation active à mon nom. Je N’AI PAS interrogé Maicie pour cette carte, conformément à l’ordre de ne lancer aucune campagne ; ne pas transformer cette absence de mesure en affirmation durable.

Le lancement des successeurs échoue actuellement après trois hypothèses réfutées ; cause inconnue. Ne pas en instruire la cause sans mandat.

FIN DE CARTE — aucun travail à reprendre automatiquement.
