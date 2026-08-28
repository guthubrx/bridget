# Carte de reprise — jc6

> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> AVANT d executer quoi que ce soit : LIRE `/home/moi/.cache/bridget/agent-domains/<ton-nom>`
> et `bridget who`, et RAPPORTER les deux. `bridget domain` REECRIT ce fichier a chaque appel,
> meme sans changement : le geste de correction detruit la trace qu il faudrait observer.
> ENSUITE seulement, executer `bridget domain bridget` et verifier la sortie.
> Le domaine NE SURVIT PAS au remplacement du processus : il est derive du nom du repertoire
> de la racine git (`derive_domain`, crates/bridget-daemon/src/wrapper.rs:983), jamais de `bridget`.
> Un successeur qui omet ce geste reste hors domaine, sort du champ de la ronde, et est perdu en
> silence sans que rien ne le lui signale. Aucune carte ne portait cette ligne avant le 28/08 15h53.


- Agent : `jc6`
- Emise : 2026-08-28T14-29-50Z
- Identifiant ledger : `mcp-1854300-6a919b5e-78`
- Recueillie par : bridget, referent, sur decision humaine du 28/08

---

CARTE DE REPRISE JC6 — passation tmux vers successeur en flux

1. ETAT
- Aucune mission active. Gel humain en vigueur : ne rien reprendre ni ouvrir sans mandat complet comportant objectif, délégation et message.
- Copie locale : /home/moi/revue/jc6
- Branche locale : session-026-operations-greffe-central ; HEAD connu cf21b3e3af5f690c947207f11fffbcf22368bfdb. Cette copie était en retard de 293 commits lors de la dernière lecture : ne jamais la prendre pour main.
- Aucun mutant en vol, aucun répertoire temporaire de mes dernières mesures, aucun CARGO_TARGET_DIR à nettoyer. Les copies /tmp de la dernière composition ont été supprimées et leur absence attestée.
- Dernier main observé pendant mon activité : 265930ccc0001d2a99205dddc0177ed9bee1079b. Il a pu avancer ; vérifier, ne pas supposer.

2. FAIT LIVRÉ
A. Session 026, opérations du greffe central.
- Branche : origin/session-026-operations-greffe-central
- Tête livrée et relue favorablement : cf21b3e3af5f690c947207f11fffbcf22368bfdb.
- Attesté : refus terminal durable de Delegate côté guichet ; persistance avant retour ; aucun repli local ; rejeu idempotent octet pour octet ; réutilisation unique de citation::unclassified_known_citations ; SQL ouvert et vocabulaire Rust fermé/fail-closed ; migration reconstruisant seulement guichet_refusal_receptions.
- Limites déclarées : citations par branche/SHA non détectées ; delegate applicatif, registre_add, objective_close et contre-tests profile_approve/routine_approve reportés à une seconde tranche.
- Je n’atteste PAS ici l’inclusion de cf21b3e dans le main actuel : vérifier par élément nommé ou ascendance.

B. L3 Codex reasoning et actes.
- Branche : origin/feat/codex-reasoning-et-actes
- Tête livrée connue seulement par préfixe : 58f33449 ; ne pas inventer le SHA complet, le relever sur origin.
- Attesté : trois événements reasoning exercés ; contrôle positif avec contenu exact avant available=false ; approval affichée mais non validable ; mutant « ignorer item/reasoning/* » tué par l’oracle de présence ; restauration SHA-256 5c23b0787e52e4d0687c9faf50fc100b524425cb1adfef2d5a0e06d80fb565b6.
- Relecture/composition favorable par rc5. Non mesuré : fournisseur Codex authentifié réel et rendu navigateur.

C. Recherche graphe de contrôle.
- Artefacts attendus : /home/moi/revue/jc6/specs/034-orchestration-graphe-de-controle/recherche.md et /home/moi/revue/jc6/specs/034-orchestration-graphe-de-controle/decision.md
- Résultat acquis : gouverner les engagements, pas la pensée ; refus machine aux arêtes critiques ; état structuré + réconciliation, pas simple journal ; vérificateur séparé utile seulement avec signal orthogonal ; entrées latérales nécessaires.
- Je n’ai pas conservé le SHA de publication dans mon contexte : vérifier présence et provenance, ne pas supposer.

D. Revues et mesures importantes.
- session-20 pre-push : verdict AMENDER puis levée vérifiée ; tête corrigée 7c151f9df83f6829d7e41fe6e4b66f05e1acd9fa annoncée fusionnée.
- busy tour non abouti : deux charges causales levées puis lot annoncé fusionné ; vrai oracle traverse ListAgents → Maicie candidates/delegate → SendIdempotent → socket.
- Suite bridget-daemon sur main 846d105 : ne terminait pas, rc124 ; 593 résultats sur 594, un seul témoin bloqué, donc verdict agrégé perdu mais pas une traîne entière de couverture.
- Seconde passe Maicie composée : f9d311d636b23c03c8be833eb1f48f1c31973659 + 9422c19961348bbd864d9a8f3ebded8c7136f195 ; arbre mesuré dbed94f1631e48439bbdaa6b0827ec03ea5868ff ; check all-targets rc0 ; 12/0/0/0 ; mutant C1 mort à l’assertion métier ; restauration SHA-256 0ec489cf9415892081f2dda6391be559416b8c95d13794e9aa26d65a5584cf8f. Bridget l’a ensuite intégré.
- Canal humain : l’asymétrie entrée OK / retour unknown_recipient a été causée plus tard par deux services sur la même socket ; Bridget annonce service orphelin arrêté à 10h12 et canal bidirectionnel rétabli depuis 10h13. Toujours vérifier status=accepted, jamais confondre texte rédigé et remise attestée.

3. RESTE
- Réserve historique : trois artefacts de tests à corriger seulement après décision/mandat explicite :
  1) sigkill_daemon… attend un exit réussi sous cfg(test) alors que les handlers sont volontairement inactifs ; contrôle positif dédié vert.
  2) TEMOIN_carte_de_reprise_instruction_lf… : propriétés de sécurité vertes, égalité terminale vieillie après ajout des trois identifiants.
  3) tour_non_abouti_redevient_mandatable… : fixture sans identité daemon locale attestée, échec isolé déterministe daemon_store_not_local.
- Trois priorités humaines encore dues, aucune en cours :
  1) corriger l’attribution fausse « humain » dans la vue — 428/433 lignes fausses mesurées ;
  2) rendre durable la règle « aucun travail auto-généré tant qu’une demande humaine attend » — actuellement seulement gel manuel ;
  3) rendre explicite le ratio vérification/production — mesure 127/169 = 75 %, cible non arbitrée.
- 85 objectifs qui ne se ferment pas avec annuaire_bridget_indisponible alors que le daemon voit les agents : cause inconnue, explicitement hors mandat. Ne pas s’en saisir seul.
- Le lancement des successeurs en flux échoue actuellement selon Bridget ; cause inconnue. La carte reste autoritaire, ne pas respawn un tmux pour contourner.

4. CHEMINS ABSOLUS UTILES
- /home/moi/revue/jc6
- /home/moi/revue/jc6/crates/bridget-daemon/src/daemon.rs
- /home/moi/revue/jc6/plugins/maicie/src/main.rs
- /home/moi/revue/jc6/plugins/maicie/src/install_publish.rs
- /home/moi/revue/jc6/plugins/maicie/tests/install_republish_before_migrate.rs
- /home/moi/revue/jc6/docs/catalogue-du-du.md
- /home/moi/.cache/bridget/bridget.db
- /home/moi/.local/bin/bridget-dette-humaine
Ces chemins désignent ma vieille copie sauf la base/cache ; pour juger main, utiliser un clone frais de https://github.com/guthubrx/bridget.git.

5. PIÈGES COÛTEUX
- Toujours base et tête dans la même passe ; une base mesurée plus tôt ne vaut rien.
- Lister l’univers avant le compte ; rendre passed/failed/ignored, jamais seulement rc.
- Conserver la sortie brute intégrale ; filtrer uniquement l’affichage. J’ai perdu une cause SC-005 en filtrant le panic.
- TMPDIR court : les sockets Unix cassent au-delà de 103 octets et produisent de faux rouges. Utiliser un chemin privé court.
- CARGO_TARGET_DIR privé : un target partagé a déjà produit un faux vert avec le binaire d’un autre worktree.
- Construire explicitement le binaire requis avant les tests ; sinon un banc peut terminer vite parce qu’il échoue avant d’atteindre la propriété.
- Un merge-tree vert ne prouve pas la composition sémantique. Comparer aussi les propriétés et les fichiers.
- Un mutant survivant peut avoir visé la mauvaise occurrence ; prouver le diff exact, puis restaurer et attester SHA-256.
- Un agent « connected » n’est pas une preuve d’activité ; lire console/ledger. Un message envoyé n’est pas remis tant que le statut n’est pas accepted/recu.

6. DÉLÉGATIONS
- Je ne connais aucune délégation jc6 encore active. Les objectifs visibles de mes dernières revues ont été annoncés clos.
- Je n’ai PAS interrogé Maicie pour cette carte, conformément à l’ordre de ne lancer aucune campagne. Ne pas déduire l’état administratif de mes souvenirs.
- Le gel humain interdit toute auto-attribution. Le successeur attend un mandat complet du référent ou une demande humaine directe, puis répond directement à l’humain.
