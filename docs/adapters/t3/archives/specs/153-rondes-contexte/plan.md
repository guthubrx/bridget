# Archives techniques de l’adaptateur T3

Document d’origine : [specs/153-rondes-contexte/plan.md](../../../../../../specs/153-rondes-contexte/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Décision | Raison | Alternative rejetée |
|---|---|---|
| Une tâche, plusieurs rondes ; `mission` change à chaque ronde | garde l'enfant, la lignée, les index par `child` et l'idempotence du résultat | tâche liée par `continues_task_id` : casse `for_child` et le nettoyage |
| Aucun nouvel état : enfant inactif = `result_available` + `idle_deadline_at` | liste de 9 états fermée par le contrat 149 | état `idle` : change le contrat T3 |
| Fin explicite = `bridget_task_cancel` et arrêt humain | un outil de moins ; même sens pour l'agent : « finir avec cet enfant » | outil `bridget_task_release` : surface en plus |
| `brief` = brouillon de `bridget_handoff`, rendu par `validate_and_render` | zéro validateur neuf ; mêmes bornes (16 Kio) et refus | historique du parent : Bridget ne le détient pas ; message handoff séparé : ni atomique, ni ordonné |
| Plafond de 8 rondes, garde de 60 à 7 200 s | borne le contexte de l'enfant et la taille du JSON de la tâche | pas de plafond : contexte non borné |
| Une ronde échouée termine la tâche | réutilise le nettoyage 149 ; la reprise est le sujet de 152 | nouvel essai automatique : hors périmètre |
| Rejeu d'une ronde par `rounds[] = {round, request_id, digest}` | 8 entrées au plus ; la `request_id` d'une ronde ne vit pas dans l'index unique `(owner, request_id)` des délégations | table dédiée : surface en plus |
| Droits revérifiés par `native_launch_registry` déjà commun au lancement et à la reprise152 | réutilise les révocations et sources existantes ; contrôle de politique actuelle local à Continue | nouvelle extraction : déjà disponible depuis152 |
| Droits actuels : recalcul de la politique de l'enfant avec l'attestation actuelle, refus si elle diffère de la politique figée | une réduction de droits sans révocation ne doit pas laisser l'enfant continuer | se limiter aux révocations et aux sources : ne voit pas la réduction (contre-revue Codex, objection 3) |
| Index `native_delegation_past_missions(mission, task_id)` | absorber une réponse tardive en O(log n) ; balayer `json_each` de 4096 tâches par message serait O(n) (Art. XVIII) | tableau JSON dans la tâche : lecture en O(n) |
| Preuve d'accusé par ronde (`unacked_rounds`) | ne perd aucun résultat indéterminé (invariant 150) sans bloquer le parent, qui continue souvent dans le tour où il lit | exiger l'accusé avant la ronde : le message n'est remis qu'à la fin du tour du parent, la ronde serait impossible dans le même tour |
| Ouverture atomique par `begin_round` (D12, I1/I5/I7, T004) | heure fraîche avant écriture, historique, CAS et resets dans une transaction ; projection et signal après commit | sauvegardes séparées : ronde partielle ou réveil après rollback |

<a id="extrait-2"></a>

## Extrait 2

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- Charge future ajoutée : 7 champs de tâche, 1 outil, 7 refus. Justifiée : sans
  eux, le cas « revue, corrections, revue » reste impossible.
- Charge supprimée : le parent n'a plus besoin de recoller le contexte à la main
  à chaque ronde.
- Non vérifiable ici : le lecteur T3 devant une tâche qui repart de
  `result_available`. Parade : mutation visible normale (relecture complète).

<a id="extrait-3"></a>

## Extrait 3

| Risque | Parade |
|---|---|
| Lecteur T3 mal à l'aise avec `result_available` vers `mission_pending` | mutation visible normale ; test de relecture ; consigné non vérifié côté T3 |
| 151 ou 152 ajoutent des champs par ronde que `begin_round` oublie | T001 liste les champs ; I5 ; test qui compare tous les champs de `Task` à une liste |
| L'identifiant de résultat par ronde casse la preuve d'accusé de 150 | une fonction unique `result_message_id` ; T016 modifie aussi la lecture de phase de 150 |
| Contexte de l'enfant qui grossit | plafond de 8 rondes ; brief delta ; documentation |
| Biais d'un enfant qui garde sa mémoire | documentation : contre-revue indépendante = nouvelle délégation |
| Délai d'inactivité non gelé pendant une pause | assumé : 152 ne gèle que les tâches non terminales ; aucun résultat perdu ; documenté |
| `prepare_restart` réécrit par 152 | T011(d) s'applique au code livré par 152 (lecture en T001) |
| Attente 151 sur une tâche inactive rend aussitôt le résultat de la ronde précédente | `round` ajouté à la vue et aux entrées d'attente (T018) ; sémantique documentée |
| `bridget_task_continue` juste après une attente : le résultat n'est pas encore envoyé par `tick` (1 s) | refus réessayable `result_delivery_pending`, documenté, testé (T013) |
| Redémarrage du daemon perd la mémoire de l'enfant | documenté ; reprise = 152 |
| Annulation reçue avant l'envoi du résultat, puis garde posée par `tick` | `guard_released` durable ; `tick` ne pose pas la garde ; tests agent et humain (T008) |
| Posture effective utilisée à la place de la posture demandée : politique Codex modifiée | recalcul avec la posture demandée (`native_permissions.rs:208-213`) ; tests posture absente, parents natifs, conversions (T014) |
| Politique figée et recalculée jamais identiques à tort (faux refus) | test : ronde immédiate sans changement admise ; si l'égalité complète du JSON échoue à tort, documenter le cas et demander arbitrage ; aucun champ de sécurité retiré (T014) |
| Corps de mission 80 Kio plus lourd | plafond de corps injecté 120 000 ; test de borne |
| Fuite d'information via le brief (journal lisible plus largement) | avertissements du dossier rappelés dans l'outil et la documentation |

<a id="extrait-4"></a>

## Extrait 4

Le principal assure Git, installation, activation coordonnée et nettoyage après
les gates. Préserver les autres fils T3. Distinguer les preuves Git, disque et
runtime. Diff153 enregistré dans `validation/diff-153.patch`.
