# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-revue-native-r1.md](../../../../../../../specs/148-identite-delegation/validation/glm-revue-native-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **par le parcours natif Bridget** : aucun appel T3 dans le moteur (grep vide). Le recette e2e tourne daemon+superviseur+wrapper réels, fournisseur fermé. Les variables de session T3 ne passent jamais aux enfants (`lifecycle.rs:461-465`, testé).
- **Saga SQLite** : insertion idempotente `UNIQUE(owner_instance, request_id)`, rejeu par canon exact, états fermés. La reprise après redémarrage du daemon est testée e2e (tâche, IDs et résultat stables).
- **Sélection exacte et définition gelée** : `for_delegation` (`registry.rs:314-363`) refuse modèle/effort hors catalogue, strippe tout modèle/effort du profil, réinjecte le choix exact. `from_resolved` revérifie le digest avant tout re-spawn : le registre courant n'est jamais consulté. La recette prouve `glm5.3`/`high` réels au lancement.
- **Mission SendIdempotent, réponse avant ACK** : `capture_reply` persiste le résultat PUIS `handle_idempotent_send` finalise Accepted (`daemon.rs:8643-8648`). Le parent ne reçoit qu'une livraison, même après coupure entre remise et checkpoint (testé, double garde `replayed`).
- **Attente enfants, annulation, cleanup, reprise managed** : `waiting_for_children` + `descendants_busy` (test petit-fils actif) ; `cancel_tree` parcourt tâches et liens flotte, stoppe feuilles d'abord, SC006 vérifié e2e ; `recover_child` exige digest identique + lien flotte `delegation_id` + parent d'origine.
- **Refus sans mutation** : testé (store vide après refus modèle, effort, cwd, posture, identité). Le rejeu précède la révocation et le changement de registre (voulu, testé).
- **Grants humains** : CLI interactive + puits `Grant` exigeant rôle Client, acteur contrôle humain, `ControlStateV1`. Jamais un outil d'agent.
- **Limites** : 4096 tâches / 128 actives / 16 par instance, mission 64 Kio, résultat 256 Kio, profondeur 8, parcours borné 4096.
