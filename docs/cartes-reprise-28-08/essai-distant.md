# Carte de reprise — essai-distant

> **PREMIER GESTE OBLIGATOIRE AVANT TOUTE LECTURE DE CETTE CARTE**
> AVANT d executer quoi que ce soit : LIRE `/home/moi/.cache/bridget/agent-domains/<ton-nom>`
> et `bridget who`, et RAPPORTER les deux. `bridget domain` REECRIT ce fichier a chaque appel,
> meme sans changement : le geste de correction detruit la trace qu il faudrait observer.
> ENSUITE seulement, executer `bridget domain bridget` et verifier la sortie.
> Le domaine NE SURVIT PAS au remplacement du processus : il est derive du nom du repertoire
> de la racine git (`derive_domain`, crates/bridget-daemon/src/wrapper.rs:983), jamais de `bridget`.
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
