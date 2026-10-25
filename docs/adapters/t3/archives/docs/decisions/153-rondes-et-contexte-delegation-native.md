# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/153-rondes-et-contexte-delegation-native.md](../../../../../decisions/153-rondes-et-contexte-delegation-native.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Décision | Choix retenu | Alternative rejetée |
|---|---|---|
| D1 | Une Task et un enfant pour plusieurs rondes ; chaque ronde change la mission. | Une nouvelle Task liée : casse les index par enfant et le nettoyage. |
| D2 | Enfant inactif en `result_available` avec `idle_deadline_at`. | Un état `idle` : change le contrat fermé de statuts149/T3. |
| D3 | Fin explicite par Cancel et arrêt humain existants ; preuve durable de garde levée. | Un outil release : ajoute une surface publique inutile. |
| D4 | Brief validé et rendu par `handoff::validate_and_render`, mêmes champs et borne de 16 384 octets. | Lecture de l'historique parent ou message séparé : sources indisponibles, ordre et atomicité perdus. |
| D5 | Huit rondes au plus ; garde de 60 à 7 200 secondes. | Contexte et garde sans borne. |
| D6 | Échec d'une ronde : tâche `failed` et nettoyage existant. | Nouvel essai automatique : hors contrat153. |
| D7 | Rejeu par `RoundRecord { round, request_id, digest }`, huit entrées au plus. | Table de clés de ronde supplémentaire. |
| D8 | `native_launch_registry` commun au lancement et à la reprise152 ; contrôles locaux de Continue. | Extraction ou wrapper de droits supplémentaire. |
| D9 | Recalcul de la politique actuelle avec la posture demandée initialement ; égalité complète avec la politique figée. | Révocations seules, posture effective ou retrait de champs de sécurité : réduction de droits non détectée. |
| D10 | Table indexée `native_delegation_past_missions(mission, task_id)` et JOIN de lecture. | Balayage du JSON de toutes les tâches à chaque réponse tardive. |
| D11 | Preuve de remise par ronde dans `unacked_rounds` ; aucune purge tant qu'une preuve manque. | Exiger l'ACK avant Continue : bloque une correction dans le tour où le parent lit le résultat. |
| D12 | `begin_round` atomique sous verrou : heure fraîche, historique, CAS, resets et projection dans une transaction ; signal après commit. | Sauvegardes séparées : ronde partielle ou réveil après rollback. |
