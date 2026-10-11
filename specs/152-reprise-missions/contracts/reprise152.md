# Contrat - Reprise des missions natives 152

Compatible avec les formes d'appel 148, 149, 150 et 151. Lineage reste inchangé.
Les octets canoniques des appels restent identiques. Les comportements de pause,
reprise, annulation et refus changent comme décrit ci-dessous.
Cette session n'ajoute aucun paramètre d'appel.

## Lecture de statut (`bridget_task_status`, CLI équivalente)

Champs ajoutés à la vue courante de statut (`current_task_view`). Ils sont additifs.

| Champ | Type | Sens |
|---|---|---|
| `suspended` | booléen | vrai si la pause retient la tâche |
| `resume_attempts` | entier | redémarrages survécus, compteur sauvegardé avant la reprise, borne 2 |

L'état reste `working` (ou `queued`, `starting`, `mission_pending`). Un lecteur
qui ignore ces champs reste correct.

## Erreurs de tâche (champ `error`)

| Code | Quand |
|---|---|
| `native_resume_exhausted` | borne de 2 reprises après panne atteinte |
| `mission_reply_timeout` | existant ; aussi rendu quand l'échéance est dépassée au redémarrage |
| `request_unavailable` | suivi durable absent après admission, ou tâche `working` sans échéance ; clôture contrôlée et interruption de pause consommée avant l'échec, sans date fabriquée ni nouvelle continuation |
| `unreachable` | existant ; identité de flotte non vérifiée au redémarrage |
| `native_restart_identity_mismatch` | tâche dont l'identité ne peut pas être réconciliée ; le daemon continue son démarrage |
| `native_cancel_unverifiable:<enfant>` | annulation bloquée, la tâche reste `cancelling` (pas `failed`) |

La garantie de démarrage FR-029 porte sur la commande, l'instance et le lien
natifs dans un marqueur lisible. Une cible d'exécution étrangère, un échec de
clôture conditionnelle (CAS) ou une erreur de stockage conserve le refus sûr
bloquant. Aucun contrôle149 n'est assoupli pour permettre le démarrage.

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
| Échéance de mission | gelée ; échéance effective calculée depuis le journal des pauses, date brute inchangée |
| Suivi de demande de mission (relances, expiration) | gelé |
| Annulation (agent ou humain) | exécutée |
| Lecture de statut, catalogue | disponible |
| Envoi du résultat ou de l'avis d'échec déjà acquis | admis (ADR 027) |

`RequestWait` 151 lit l'échéance métier effective d'une demande native, sans
écrire son état ni sa date brute. Son budget `wait_timeout` reste celui de
l'appelant et continue pendant la pause. Une demande ordinaire garde son
comportement 151. L'admission d'un descendant compare également le délai
demandé au temps restant effectif du parent. La date brute exposée dans le
statut reste la date durable ; elle ne doit pas être prise pour le temps restant
pendant une pause.

## Lineage T3

Aucun changement de forme ni de clé. La tâche suspendue y apparaît `working`.
