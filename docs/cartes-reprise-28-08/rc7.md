# Carte de reprise — rc7

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


- Agent : `rc7`
- Emise : 2026-08-28T14-36-01Z
- Identifiant ledger : `mcp-2874247-6a919cd1-2`
- Recueillie par : bridget, referent, sur decision humaine du 28/08

---

CARTE DE REPRISE RC7 — 28/08

1. ETAT
En réserve nommée, sans mission active ni travail en cours. Arbre propre lors de la dernière remise. Aucune campagne, mutation ou répertoire de compilation en cours.

2. FAIT
- 055, attribution émetteur : branche session-055-attribution-emetteur, tête publiée 963612cdcf0baaa588bfee4c36996b16b89fb5e3. Le commit parent e9ff6e78 ajoute from aux turn_start Codex et Claude ; 963612c corrige les journaux historiques dans Attach. Garanties attestées : prompt_dispatched enrichi remplace un en-tête sans from ; absence totale de provenance affiche emetteur non attesté, jamais humain. Témoin nominal 1/0/596 ; deux mutants causaux 0/1/596 ; jumeaux réels Codex et Claude 2/0/595 ; famille Attach 54/0/543 ; cargo check workspace all-targets vert. Non attesté : intégration finale de 055 ou verdict ultérieur, sauf information détenue par Bridget.
- 057, fragments journal : branche session-057-preserver-fragments-journal, tête publiée 89f244d. Corrige le réordonnancement visuel texte/outils en conservant des parties ordonnées. Témoin nominal 1/0/594, mutant 0/1/594, famille Attach 52/0/543 au moment de la mesure. Suspendue sur instruction de priorité 055 puis gel humain ; pas de relecture ou intégration attestée par moi.
- C3 republication Maicie : tête publiée 320971bb1ea57a74b3cc3562a43d8e3358b15454 sur amend/maicie-c1c2c3. Les invariants 2/8/1/0/6/2 et le mutant d'ouverture directe ont été vérifiés ; la suite de composition C5 a ensuite été annoncée intégrée, mais je n'ai pas de nouvelle mesure à fournir.

3. RESTE
Prochaine étape seulement sur mandat humain explicite : relire ou reprendre 055/057 selon l'arbitrage. Ne pas toucher 057 avant mandat : son correctif existe, est publié, mais son statut d'intégration m'est inconnu.

4. CHEMINS ABSOLUS
- /home/moi/revue/rc7/bridget/crates/bridget-daemon/src/attach.rs
- /home/moi/revue/rc7/bridget/crates/bridget-transport/src/codex_app_server.rs
- /home/moi/revue/rc7/bridget/crates/bridget-transport/src/claude_stream_json.rs
- /home/moi/revue/rc7/bridget/specs/055-attribution-emetteur/spec.md
- /home/moi/revue/rc7/bridget/specs/055-attribution-emetteur/implementation.md
- /home/moi/revue/rc7/bridget/specs/057-preserver-fragments-journal/spec.md
- /home/moi/revue/rc7/bridget/specs/057-preserver-fragments-journal/implementation.md
- /home/moi/revue/rc7/bridget/AGENT_HANDOFF.md

5. PIEGES
- Ne pas rétablir le repli humain dans Attach : provenance absente n'est pas provenance humaine.
- Un prompt_dispatched peut apporter from après turn_start : l'en-tête initial doit alors être remplacé.
- 057 ne se corrige pas par une concaténation globale ni par lines() : la trace réelle montre des fragments texte séparés par un événement outil ; l'ordre doit être conservé.
- cargo fmt workspace reste rouge hors delta ; ne pas l'imputer aux correctifs RC7.
- Le témoin 055 des journaux historiques est distinct des deux jumeaux réels de pilotes ; conserver les trois et leurs mutants.

6. DELEGATIONS
Je ne connais aucune délégation active à mon nom. La dernière délégation explicitement reçue est 20c7a2a8-4bff-49b5-b1ca-1ffe1145ee92 / 0d3018c4-e397-46af-a543-61f2e14eaacf pour 055 ; j'ai livré l'amendement 963612c. Je ne sais pas si Bridget ou Maicie l'a depuis close, intégré ou réassigné : je ne l'infère pas. La session 057 avait l'objectif 3405bde8-0975-4f48-bb40-30105bd2f44e et délégation 832f4201-09e4-4476-87af-4d6312206c71 ; elle est publiée mais son état actuel reste non attesté par moi.
