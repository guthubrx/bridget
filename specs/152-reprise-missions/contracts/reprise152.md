# Contrat - Reprise des missions natives 152

Compatible avec les contrats 148, 149 (Lineage) et 150. Un appel sans nouveau
champ produit les mêmes octets canoniques et le même comportement qu'en 150.
Cette session n'ajoute aucun paramètre d'appel.

## Lecture de statut (`bridget_task_status`, CLI équivalente)

Champs ajoutés à la vue de tâche (`task_view`). Ils sont additifs.

| Champ | Type | Sens |
|---|---|---|
| `suspended` | booléen | vrai si la pause retient la tâche |
| `resume_attempts` | entier | reconstructions faites après panne (redémarrages survécus, borne 2) |

L'état reste `working` (ou `queued`, `starting`, `mission_pending`). Un lecteur
qui ignore ces champs reste correct.

## Erreurs de tâche nouvelles (champ `error` d'une tâche `failed`)

| Code | Quand |
|---|---|
| `native_resume_exhausted` | borne de 2 reprises après panne atteinte |
| `mission_reply_timeout` | existant ; aussi rendu quand l'échéance est dépassée au redémarrage |
| `unreachable` | existant ; identité de flotte non vérifiée au redémarrage |
| `native_cancel_unverifiable:<enfant>` | annulation bloquée, la tâche reste `cancelling` (pas `failed`) |

## Refus définitifs rejoués (liste fermée)

| Code | Rejoué |
|---|---|
| `cwd_outside_parent_project` | oui |
| `cwd_unavailable` | oui |
| `model_unavailable` | oui |
| `effort_unavailable` | oui |
| `agent_type_unknown` | oui |

La liste est exactement ces cinq codes. Le registre renvoie pour un type inconnu
un texte dynamique (chemin du registre, liste des types, `registry.rs:157`) :
l'admission le ramène au code `agent_type_unknown` et ne stocke jamais le texte.
Tout autre code ou texte libre n'est pas enregistré.

Un rejeu avec la même enveloppe rend le même refus. Une autre enveloppe rend
`envelope_mismatch`.

## Refus transitoires, jamais gardés

`referent_paused`, `task_limit`, `retention_saturated`, `identity_unavailable`,
`reply_timeout_exceeds_parent`, `parent_task_unavailable`,
`parent_execution_ambiguous`, `delegation_grant_required`,
`permission_not_inherited`, `permission_attestation_unavailable`.
Pour ces refus, la clé reste libre.

## Comportement pendant la pause

| Opération | Pendant la pause |
|---|---|
| Lancement d'une tâche `queued` | différé |
| Remise de la mission à l'enfant | différée |
| Échéance de mission | gelée, repoussée de la durée de la pause à la levée |
| Suivi de demande de mission (relances, expiration) | gelé |
| Annulation (agent ou humain) | exécutée |
| Lecture de statut, catalogue | disponible |
| Envoi du résultat ou de l'avis d'échec déjà acquis | admis (ADR 027) |

## Lineage T3

Aucun changement de forme ni de clé. La tâche suspendue y apparaît `working`.
