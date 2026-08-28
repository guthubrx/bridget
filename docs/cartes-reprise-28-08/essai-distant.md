# Carte de reprise — essai-distant

> **TOUTE AFFIRMATION D'INTEGRATION DE CETTE CARTE EST DATEE DU 28/08 ET NON REVERIFIEE DEPUIS.**
> Avant de t'appuyer sur une integration annoncee ici : `git fetch` PUIS
> `git merge-base --is-ancestor <sha> origin/main`. Ne deduis pas, mesure.
> ET NOMME LE REMOTE : `origin` ne designe pas le meme depot selon les checkouts du parc —
> deux d'entre eux pointent un miroir local fige, pas github.
> AVERTISSEMENT DE rc7-flux, qui vaut pour les deux sens : une affirmation « X est integre »
> ne se defait pas, mais une affirmation « X n'est PAS integre » PERIME DANS L'AUTRE SENS —
> il suffit qu'on integre pour que la carte fasse croire a une prochaine incarnation qu'il
> reste du travail alors qu'il est fait. C'est le mensonge le plus probable d'une carte.
> Bandeau ajoute le 28/08 17h00 par le referent, sur signalement de rc7-flux.


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


- Agent : `essai-distant`
- Emise : 2026-08-28T14-35-43Z
- Identifiant ledger : `mcp-1418654-6a919cbf-41`
- Recueillie par : bridget, referent, sur decision humaine du 28/08

---

1. État

Je suis en réserve, sans mission active. Derniers travaux : lots 040 (observateur disque), 042 (fixtures), M2 republication v19, puis relecture UI. Aucun travail courant.

2. Faits livrés

- `session-040-observateur-espace-disque` : chaîne de commits jusqu’à `c76c23798e0ad1d155dfbbd27f7179d397e73c0d`, puis amendements ultérieurs sur une autre branche de travail ; M1/M2/M3 ont été mesurées, mais la composition finale a été reprise par d’autres.
- `session-042-fixtures-integration-ephemeres` : `5a055b9477f6577b5848cee8d821de1dc41013cc`, espaces terminaux corrigés, `git diff --check` vert.
- `fix/maicie-republish-before-migrate` : `afda09fb956aac1153971d8485c5856eeacc5673` puis artefact M2 publié jusqu’à `f33cd0b...`; fixture v19 et attestations ajoutées. La contre-vérification a confirmé deux mutants discriminants.
- Relecture UI `session-ui-correctifs` : jointure par identifiant confirmée, records sans `message_id` conservés, trois codes d’erreur séparés ; couverture de mutation du premier volet non établie.

3. Reste

Aucune reprise autorisée actuellement. Les réserves sont conditionnelles à un mandat portant objectif, délégation et message. Ne pas relancer les mesures anciennes sans nouvel objet explicitement figé.

4. Chemins absolus

- `/home/moi/revue/hygiene-disque-ToHHyG`
- `/home/moi/revue/fixtures-integration-ephemeres-42`
- `/tmp/maicie-republish` (clone temporaire supprimé depuis)
- `/home/moi/revue/essai-distant/bridget-src` (dépôt tiers, à préserver)

5. Pièges coûteux

- Ne pas amender une tête déjà relue : empiler les commits pour conserver l’ancêtre du verdict.
- Les mutants doivent être rejoués après chaque amendement.
- Les cibles filtrées neuves ne construisent pas toujours `managed_test_binary`.
- Une fixture peut lire la configuration réelle si tous les chemins ne sont pas isolés.
- `user_version=19` seul ne prouve pas un schéma v19.
- Les lignes de test doivent inclure `passed`, `failed` et `filtered out`.

6. Délégations

Je ne connais pas l’état courant des objectifs/délégations Maicie ni des relectures après la dernière composition. Les derniers échanges indiquaient : 040 en contre-vérification, M2 republication validée, et aucune nouvelle mission ouverte.
