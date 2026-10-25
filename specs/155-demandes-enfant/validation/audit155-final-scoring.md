# Audit de code — session155

**Score global** : A (99.85/100)
**Couverture basse du diff** : 27.46% ; note sur cet échantillon déclaré.
**Livraison bloquée** : False

## Tendance

Première session pour ce scope155. Aucune note154 transposée.

## Résumé exécutif

| Domaine | Note | Numérique | Findings |
|---|---|---:|---:|
| security | A | 100 | 0 |
| llm_security | A | 100 | 0 |
| supply_chain | A | 100 | 0 |
| observability | A | 100 | 0 |
| reliability | A | 100 | 0 |
| tests | A | 100 | 0 |
| performance | A | 99 | 1 |
| complexity | A | 100 | 0 |
| quality | A | 100 | 0 |
| architecture | A | 100 | 0 |
| ai_hygiene | A | 100 | 0 |
| duplication | A | 99 | 1 |
| minimalism | A | 100 | 0 |
| ux | N/A | N/A | 0 |

## Findings

### PERF-082 — LOW — Le tick renouvelle une seconde pour chaque émission séquentielle

crates/bridget-daemon/src/daemon/native_child_requests.rs:907
Le lot de followups est borné à512 lignes, mais chaque appel emit recrée son propre budget d’une seconde. Le tick natif attend la fin de cette boucle avant ses propres transitions. Plusieurs writers lents peuvent donc accumuler leur attente. La condition de saturation n’a pas été mesurée ; les timeouts et la fermeture des sockets limitent chaque émission.

```text
for (id, conn, writer, frame) in outgoing {
    emit(state, &id, &conn, &writer, &frame, false);
}
```

Remédiation : Conserver ce risque comme limite connue. Avant toute optimisation, utiliser les logs de tick existants pour établir une baseline réelle ; si un retard est observé, borner le budget du lot tout en conservant les claims et les recontrôles FR027. Aucun patch de confort demandé dans cette lecture.
Vérification : Lecture finale907–909/929 et native_delegation2402. Aucun délai mesuré ni benchmark.

### DUP-001 — LOW — Renvoi après reconnexion copié dans deux branches

crates/bridget-daemon/src/wrapper.rs:4367
Les lignes4367–4379 et4611–4623 sont identiques : elles relisent les demandes waiting, excluent uncertain et remettent les trames au daemon. Les deux chemins terminent une reconnexion avec le même writer et le même état. Aucun helper équivalent de ce renvoi collectif n’a été trouvé ; send_wrapper_message reste le point unique d’envoi d’une trame.

```text
                if forward_child_requests {
                    let pending: Vec<_> = child_requests
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .waiting
                        .iter()
                        .filter(|entry| !entry.uncertain)
                        .map(|entry| entry.request.clone())
                        .collect();
                    for request in pending {
                        send_wrapper_message(&writer, WrapperToDaemon::ChildRequest { request });
                    }
                }
```

Remédiation : Réunir ce seul bloc dans une fermeture locale au parcours géré, appelée après les deux reconnexions. Garder les deux branches et leurs contrôles. Aucun module, export, framework, table ou outil neuf.
Vérification : Les deux appels doivent conserver le filtre !uncertain et envoyer le même tuple ; rejouer les tests existants native155_wrapper et la reconnexion149 concernés, sans nouveau scénario.


## Minimalisme & Frugalité

Potentiel minimalisme : ~0 lignes suppressibles signalées dans les findings MIN.
Somme des estimations fournies ; aucun décompte global ni conclusion de checklist inventés.

## Vertus LLM & Responsabilité Future

Synthèse fournie par le module12 :
```json
{
  "minimalism_summary": {
    "removable_lines_estimate": 0,
    "basis": "0 ligne directement supprimable à comportement constant étayée dans les seules lectures admissibles ; aucune conclusion globale. DUP-001 reste exclusivement au module07.",
    "cross_domain_note": "DUP-001 appartient uniquement au module07 et propose une réduction locale après refactor ; aucun finding MIN doublonné."
  },
  "virtues": {
    "cognitive_load": "Le champ cwd explicite supprime une ambiguïté de lecture humaine ; null et verified=false évitent une confiance implicite.",
    "volume": "Ajout limité au transfert, contrôle type/borne et citation ; assertions dans les scénarios existants. Les autres volumes restent hors relecture ciblée.",
    "abstractions": "Aucune abstraction nouvelle ; validations séparées car frontière fournisseur puis DTO/droits. Pas de factorisation générale justifiée par deux expressions Debug.",
    "human_responsibility": "Contrats précisent1024octets/champ,4096faits, absence/null, contrôle visible et aucun confinement. L’intention et les limites sont expliquées.",
    "ununderstood_zones": "Aucune glue opaque étayée dans le delta cwd. Les anciennes analyses ont leur provenance historique ; les autres hunks ne sont pas crédités fraîchement."
  }
}
```

## Limites et preuves historiques

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/155-demandes-enfant/validation/audit155-final-scoring.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/155-demandes-enfant/validation/audit155-final-scoring.md#extrait-2).

## Points positifs

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation.rs:1385 — q² et borne8Kio annoncés exactement.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs:529 — Debugcwd borné1024octets, facts4096.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/native_permissions.rs:73 — cwd chaîne<=1024octets ou null ; autre forme refusée sans I/O ni changement des droits.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs:529 — cwd cité entier avec Debug ou null, cwd_verified=false ; budgetglobal4096 maintenu.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-transport/src/codex_app_server.rs:2605 — absence conservée comme null, aucun fallback vers dossier session/parent.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation_chain155_tests.rs:487 — fixture existante vérifie fin du dossier et contrôles visibles, sans confondre citation et confinement.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs:611 — Le canal Human reste horsstate aprèscwd.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/human_inbox.rs:549 — Budgetcommun3s et stderr retenu1024octets maintenus.
- native_child_requests.rs:521–546 : commande/cwd entiers cités avec Debug, cwd_verified=false, effets complets et refusfacts>4096.
- native_permissions.rs:50–80 et codex_app_server.rs:2463–2475,2602–2618 : cwd string/null exact, forme invalide et1025octets refusés ; aucun gain de droit149.
- native_child_requests.rs:920–1002 : contrôle avantwrite et reprise M2 restent conservateurs aprèsdelta.
- codex_app_server.rs:429–452,1082–1093,3238–3310 : O_NONBLOCK, budgetSTDIO3s partagé et incertitude conservés aprèsdelta.
- chain155:398–543, human155:1–117, codex:7705–7737 : oracles existants inspectés pourcwd complet/null/contrôles/1025octets ; leurs noms PASS recoupés dansROOTcwd.
- native155-cwd-r1.json/log :71PASS=63daemon+1E2E+7transport,214sources stables et SHA égaux à la capture finale ; aucun full final déduit.

## Suppressions baseline
```json
[]
```


## Corrections et traces du cycle1
```json
{
  "resolved_findings": [
    {
      "original_id": "PERF-041",
      "original_fingerprint": "6f75818e3962",
      "original": {
        "id": "PERF-041",
        "category": "performance",
        "subcategory": "algorithmic_complexity",
        "severity": "MEDIUM",
        "file": "crates/bridget-daemon/src/daemon/native_delegation.rs",
        "line": 1385,
        "title": "Respond annonce un coût linéaire mais compare les IDs en O(q²)",
        "description": "Le commentaire du chemin Respond annonce O(log tâches + log demandes + B). La validation parcourt les clés de réponse et recherche chacune dans la liste des IDs attendus. Cette partie coûte O(q²+B), sous le verrou métier ; q reste borné par les limites de projection et de réponse. Le problème retenu est l’annotation inexacte, pas un ralentissement runtime démontré.",
        "evidence": "|| input\n    .keys()\n    .any(|key| !ids.iter().any(|id| id.as_str() == Some(key.as_str())))\n|| ids.iter().any(|id| {\n    id.as_str().is_none_or(|id| {",
        "exploitability": "N/A",
        "impact": "L’annotation sous-estime le coût de vérification ; les limites4096/8192 octets empêchent une croissance non bornée sur le chemin normalisé.",
        "path_type": "hot",
        "remediation": "Corriger uniquement le commentaire existant en O(log tâches + log demandes + q² + B), en citant les bornes déjà présentes. Aucun nouvel index, helper ou scénario.",
        "verification": "Relire le commentaire contre les boucles existantes et les limites à codex_app_server.rs:2605 et native_delegation.rs:1486 ; vérification documentaire sans nouveau test.",
        "effort": "XS",
        "regression_risk": "LOW",
        "complexity_current": "O(log t + log n + q² + B)",
        "complexity_target": "Même code et mêmes bornes ; annotation exacte",
        "references": [
          "Constitution Article XVIII",
          "audit-code-v14 module04",
          "crates/bridget-transport/src/codex_app_server.rs:2605",
          "crates/bridget-daemon/src/daemon/native_delegation.rs:1486"
        ],
        "fingerprint": "6f75818e3962"
      },
      "status": "resolved",
      "proof": {
        "file": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation.rs",
        "line": 1385,
        "sha256": "51905c9443f3a47ede4799b176827039192ab20c583f6cf2b2fb9d1e40ccf99b",
        "evidence": "            // O(log tâches + log demandes + q² + B), réponse<=8Kio ; même propriétaire\n            // direct que recover_owner, sans écriture pour un refus ou un rejeu.\n            if [task_id, request_id].iter().any(|id| {"
      }
    },
    {
      "original_id": "PERF-081",
      "original_fingerprint": "41c2c45e2996",
      "original": {
        "id": "PERF-081",
        "category": "performance",
        "subcategory": "blocking_io_under_business_lock",
        "severity": "HIGH",
        "counter_audit": "CONFIRMED",
        "file": "crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 527,
        "title": "Le canal humain peut garder le verrou commun sans limite",
        "description": "receive garde le MutexGuard DaemonState pendant publish_human. Le nouveau dépôt Human appelle daemon_deposit_human_inbox puis deposit_and_notify, qui attend synchroniquement le processus du canal via write_all et wait_with_output. La transaction du dépôt est déjà committée ; le problème porte sur le verrou métier commun. Précondition : canal valide configuré et processus externe qui ne termine pas. La lecture indépendante de root et audit_performance151 confirme le flux ; aucun blocage runtime n’a été reproduit.",
        "evidence": "if verdict == ChildRequestVerdict::Human {\n    publish_human(&st, &task, &row)?;\n} else {\n    publish_parent(&mut st, &task, &row, &mut controls)?;\n    st.delegation_store.wait_epoch().bump();\n}",
        "exploitability": "LIKELY",
        "impact": "Sous cette précondition, une demande Human peut empêcher les autres lecteurs, les réponses, les échéances et le contrôle du daemon d’acquérir DaemonState. Aucun temps de blocage mesuré.",
        "path_type": "hot",
        "remediation": "Séparer le dépôt/la liaison durable de la notification. Réutiliser deposit_and_notify avec canal None sous state, préparer le résumé, puis appeler push_to_channel après libération de state et enregistrer la tentative après reprise brève du verrou. Conserver les clés, le CAS et les recontrôles ; aucun nouvel outil ou framework. Root fournit le raccord final dans les fonctions existantes.",
        "remediation_code": "// Dans la section existante de dépôt, sous state :\nlet deposited = crate::human_inbox::deposit_and_notify(\n    st.store.connection(), request, None,\n).map_err(|error| error.to_string())?\n .map_err(|refusal| format!(\"human_inbox_refused:{refusal:?}\"))?;\nst.delegation_store.link_child_inbox(&row.request_id, &deposited.item_id)?;\n// Si deposited.created : préparer (config, summary, item_id, now)\n// dans la liste locale de notifications, sans lancer le canal ici.\n// Après la fin du bloc qui détient state :\nfor (config, summary, item_id, now) in human_notifications {\n    let result = crate::human_inbox::push_to_channel(&config, &summary);\n    let st = state.lock().unwrap_or_else(|error| error.into_inner());\n    let _ = crate::human_inbox::record_notify_attempt(\n        st.store.connection(), &item_id,\n        &crate::human_inbox::NotifyAttempt {\n            at: now, ok: result.is_ok(), error: result.err(),\n        },\n    );\n}",
        "verification": "Root : étendre le scénario Human existant avec le canal privé déjà utilisé ; pendant l’attente du canal, une lecture/échéance indépendante doit acquérir state. Vérifier le même item lié et la même trace de tentative après retour ; aucune exécution faite par cet auditeur.",
        "effort": "S",
        "regression_risk": "MEDIUM",
        "references": [
          "audit-code-v14 module08 checklist DB/I-O",
          "crates/bridget-daemon/src/daemon/native_child_requests.rs:344",
          "crates/bridget-daemon/src/daemon/native_child_requests.rs:278",
          "crates/bridget-daemon/src/daemon.rs:9943",
          "crates/bridget-daemon/src/human_inbox.rs:157",
          "crates/bridget-daemon/src/human_inbox.rs:545",
          "crates/bridget-daemon/src/human_inbox.rs:557"
        ],
        "counter_audit_evidence": {
          "source": "Message du principal du2026-10-12 : contre-lecture root et audit_performance151 confirmée ; rapport cache counter-audit155-channel.md/json annoncé en clôture.",
          "scope": "Flux/verrou confirmés par lecture ; canal configuré et non-terminaison conditionnels, aucun runtime reproduit."
        },
        "fingerprint": "41c2c45e2996"
      },
      "status": "resolved",
      "proof": {
        "file": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 611,
        "sha256": "3be800076fe7523adc083dce913f4d38988dc7da540fb6d4fa6bc3c2adb2af51",
        "evidence": "    })();\n    if let Some((config, summary)) = human_notification {\n        // Le programme externe ne détient jamais le verrou métier. Seule la\n        // consignation locale de sa tentative reprend brièvement state.\n        let pushed = crate::human_inbox::push_to_channel(&config, &summary);"
      }
    },
    {
      "original_id": "SEC-001",
      "original_fingerprint": "573cf471637c",
      "original": {
        "id": "SEC-001",
        "category": "security",
        "subcategory": "incomplete_human_authorization_facts",
        "severity": "HIGH",
        "counter_audit": "CONFIRMED",
        "file": "crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 457,
        "title": "Human reçoit des faits tronqués sans indication de perte",
        "description": "Une commande source de513–1024 octets passe la normalisation, puis sa projection Human conserve512 octets. Les contrôles et chemins peuvent aussi être transformés ou coupés ; effects.command/file_write sont omis. L’accord répond au callback fournisseur original, donc porte sur une demande plus complète que les faits présentés. La chaîne est confirmée statiquement ; aucun rendu ou exploit runtime observé.",
        "exploitability": "LIKELY",
        "exploitation_scenario": "Demande commandExecution valide sous dangerFullAccess, avec suffixe significatif après512 octets et offre accept/decline. Human voit une commande recoupée sans marqueur ; accept s’applique au callback original. Aucune commande exécutée par l’audit.",
        "impact": "L’humain peut autoriser un effet absent de sa présentation. Le plafond de droits149 reste inchangé.",
        "path_type": "hot",
        "remediation": "Refuser avec details_incomplete avant Human si la projection change ou coupe une commande/chemin. Copier les trois effets normalisés dans facts sous la garde4096 existante. Réutiliser les gardes et fonctions existantes.",
        "remediation_code": "let display_changed = request.details[\"command\"].as_str().is_some_and(|raw| quoted_text(raw,512) != raw)\n    || request.details[\"paths_cited\"].as_array().is_some_and(|raw| raw.len() != paths.len()\n        || raw.iter().zip(&paths).any(|(value,shown)| value.as_str() != Some(shown.as_str())));\nif verdict == ChildRequestVerdict::Human && display_changed {\n    verdict = ChildRequestVerdict::Forbidden;\n    code = Some(\"details_incomplete\");\n}\nlet mut presented: Value = serde_json::from_str(&facts).map_err(|error| error.to_string())?;\npresented[\"effects\"] = request.details[\"effects\"].clone();\nfacts = presented.to_string();\n// Insérer avant la garde facts.len()>4096 existante et avant toute publication Human.",
        "verification": "Contre-relire absence de publication Human lors de transformation/perte et copie des trois effets. Réemployer les oracles155 existants de faits et texte hostile ; aucun nouveau test exécuté ici.",
        "effort": "S",
        "regression_risk": "MEDIUM",
        "references": [
          "counter-audit155-display-stdio.json:H1",
          "codex_app_server.rs:2553–2562,1012–1027",
          "native_child_requests.rs:13–26,443–467,276–290"
        ],
        "runtime_reproduced": false,
        "status": "open",
        "evidence": "        let mut facts=json!({\"method\":quoted_text(request.details[\"method\"].as_str().unwrap_or(\"unsupported\"),128),\n            \"duration\":if request.details[\"duration\"]==\"once\"{\"once\"}else{\"unsupported\"},\n            \"command\":quoted_text(request.details[\"command\"].as_str().unwrap_or(\"\"),512),\n            \"paths_cited\":paths,\"paths_verified\":false,\"network\":request.details[\"effects\"][\"network\"].as_bool().unwrap_or(false),\n            \"child_posture\":if snapshot.child_policy[\"sandbox_policy\"][\"type\"]==\"readOnly\"{\"read_only\"}else{\"development\"},\n            \"parent_sandbox\":snapshot.parent.provider_policy[\"sandbox_policy\"][\"type\"],\"verdict_code\":code,\n            \"question_ids\":if verdict==ChildRequestVerdict::Parent{request.question_ids.clone()}else{Vec::new()}}).to_string();",
        "counter_audit_evidence": {
          "path": "/Users/moi/.cache/bridget-delivery155.um4xccv4/counter-audit155-display-stdio.json",
          "sha256": "85f7de19475ff0b8bedc4da6f13cb83bf1d09aa0107ca22f07cde02f4c97d91b",
          "method": "independent static source review",
          "runtime_reproduced": false
        },
        "path_type_basis": "sensitive callback path; runtime frequency not measured",
        "fingerprint": "573cf471637c"
      },
      "status": "resolved",
      "proof": {
        "file": "crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 521,
        "sha256": "3be800076fe7523adc083dce913f4d38988dc7da540fb6d4fa6bc3c2adb2af51",
        "evidence": "        let raw_command = request.details[\"command\"].as_str().unwrap_or(\"\");\n        // Debug rend les contrôles et caractères de direction visibles, sans\n        // modifier ni couper la commande qui sera exécutée par le fournisseur.\n        let command = if raw_command.is_empty() {\n            String::new()\n        } else {\n            format!(\"{raw_command:?}\")\n        };\n        let cwd = request.details[\"cwd\"]\n            .as_str()\n            .map(|cwd| format!(\"{cwd:?}\"));\n        let mut facts=json!({\"method\":quoted_text(request.details[\"method\"].as_str().unwrap_or(\"unsupported\"),128),\n            \"duration\":if request.details[\"duration\"]==\"once\"{\"once\"}else{\"unsupported\"},\n            \"command\":command,\n            \"cwd\":cwd,\"cwd_verified\":false,\n            \"paths_cited\":paths,\"paths_verified\":false,\"paths_cited_truncated\":paths_cited_truncated,\"paths_cited_changed\":paths_cited_changed,\n            \"effects\":{\"command\":request.details[\"effects\"][\"command\"].as_bool(),\"file_write\":request.details[\"effects\"][\"file_write\"].as_bool(),\"network\":request.details[\"effects\"][\"network\"].as_bool()},\n            \"network\":request.details[\"effects\"][\"network\"].as_bool(),\n            \"child_posture\":if snapshot.child_policy[\"sandbox_policy\"][\"type\"]==\"readOnly\"{\"read_only\"}else{\"development\"},\n            \"parent_sandbox\":snapshot.parent.provider_policy[\"sandbox_policy\"][\"type\"],\"verdict_code\":code,\n            \"question_ids\":if verdict==ChildRequestVerdict::Parent{request.question_ids.clone()}else{Vec::new()}}).to_string();\n        if raw_command.len() > 1024 || facts.len() > 4096 {\n            verdict = ChildRequestVerdict::Forbidden;\n            code = Some(\"details_incomplete\");\n            facts=json!({\"method\":\"unsupported\",\"verdict_code\":\"details_incomplete\",\"question_ids\":[]}).to_string();\n        }",
        "reviewed_at_utc": "2026-10-12T03:37:28.083936+00:00"
      }
    },
    {
      "original_id": "REL-001",
      "original_fingerprint": "a33380318509",
      "original": {
        "id": "REL-001",
        "category": "reliability",
        "subcategory": "unbounded_private_stdio_callback",
        "severity": "HIGH",
        "counter_audit": "CONFIRMED",
        "file": "crates/bridget-transport/src/codex_app_server.rs",
        "line": 1065,
        "title": "Le callback155 peut rester bloqué sur le STDIO privé",
        "description": "answer_provider_request tient la table transport pendant write_value ; le wrapper tient aussi sa table dans son lecteur de commandes. ChildStdin est bloquant et writeln/flush n’ont pas d’échéance. shutdown attend les mêmes table/writer. write_value est une dette héritée dont le callback155 étend l’impact ; le scénario reste conditionnel et non reproduit.",
        "exploitability": "LIKELY",
        "exploitation_scenario": "Fournisseur vivant qui ne lit plus stdin ; pipe plein pendant une réponse. L’écriture attend sans borne et bloque le lecteur géré, ses contrôles et la progression de l’arrêt. Aucun banc de saturation exécuté.",
        "impact": "Une session peut rester sans ACK ni clôture sûre. Aucune élévation de droits ou panne globale du daemon n’est affirmée.",
        "path_type": "hot",
        "remediation": "Borner le verrou writer et les écritures réelles du pipe avec une échéance absolue et mode non bloquant selon le patron existant. Préserver la sérialisation, l’incertitude de ligne partielle, alive=false et l’absence de replay. request_with_timeout seul ne borne pas le write préalable.",
        "remediation_code": "use std::os::fd::AsRawFd;\nlet fd = stdin.as_raw_fd();\nlet flags = unsafe { libc::fcntl(fd,libc::F_GETFL) };\nif flags < 0 || unsafe { libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK) } < 0 {\n    return Err(TransportError::Io(std::io::Error::last_os_error().to_string()));\n}\n// Avant boxing du STDIO. Réemploi du write_value existant, avec le même deadline\n// avant writer.try_lock() et chaque write, offset conservé, WouldBlock/Interrupted\n// traités jusqu'à expiration ; aucune réinitialisation du délai par fragment.\n// Sur tout échec partiel conserver le chemin existant :\npending[index].uncertain = true;\nself.alive.store(false, Ordering::SeqCst);\nself.observations.1.notify_all();\nreturn Err(TransportError::Io(format!(\"child_request_write_uncertain: {error}\")));\n// Adapter l'acquisition du writer dans write_value à try_lock jusqu'au deadline.\n// Garder la sérialisation answer/cancel. Le writer socket garde sa propre borne.\n// Proposition d'intégration non compilée, pas un patch exécuté.",
        "verification": "Contre-relire O_NONBLOCK avant boxing, acquisition/écriture bornées par la même horloge et échec partiel uncertain sans ACK. Réemployer les oracles callback/answer-cancel155 existants ; aucun nouveau scénario de confort exécuté.",
        "effort": "M",
        "regression_risk": "MEDIUM",
        "references": [
          "counter-audit155-display-stdio.json:H2",
          "codex_app_server.rs:427–437,967–1074,3177–3185,781–789,1272–1293",
          "wrapper.rs:408–425,4521–4530,5560–5617"
        ],
        "runtime_reproduced": false,
        "status": "open",
        "evidence": "        // Le verrou sérialise answer/fin/resolved. Une écriture partielle ne\n        // prouve aucun refus : garder l'incertitude et fermer l'admission.\n        if let Err(error) = write_value(&self.writer, reply) {\n            pending[index].uncertain = true;\n            self.alive.store(false, Ordering::SeqCst);\n            self.observations.1.notify_all();\n            return Err(TransportError::Io(format!(\n                \"child_request_write_uncertain: {error}\"",
        "counter_audit_evidence": {
          "path": "/Users/moi/.cache/bridget-delivery155.um4xccv4/counter-audit155-display-stdio.json",
          "sha256": "85f7de19475ff0b8bedc4da6f13cb83bf1d09aa0107ca22f07cde02f4c97d91b",
          "method": "independent static source review",
          "runtime_reproduced": false
        },
        "path_type_basis": "sensitive callback path; runtime frequency not measured",
        "fingerprint": "a33380318509"
      },
      "status": "resolved",
      "proof": {
        "file": "crates/bridget-transport/src/codex_app_server.rs",
        "line": 3238,
        "sha256": "ac13e0cfd0becdec45a3e24ae7973ddbb94c18e74b00ce40ddf0a93fca76c5e8",
        "evidence": "fn write_value(writer: &Writer, value: Value) -> Result<(), TransportError> {\n    // Un budget TOTAL pour le verrou, les écritures STDIO partielles et flush.\n    // Le SocketWriter interactif conserve son propre flush borné à3s ; il\n    // peut ajouter ces3s après contention, sans prétendre au budget STDIO.\n    let deadline = Instant::now() + Duration::from_secs(3);\n    let mut bytes =\n        serde_json::to_vec(&value).map_err(|error| TransportError::Io(error.to_string()))?;\n    bytes.push(b'\\n'); // mêmes octets JSON compacts + LF que writeln!(\"{value}\").\n    let mut guard = loop {\n        if Instant::now() >= deadline {\n            return Err(TransportError::Io(\"budget writer Codex dépassé\".into()));\n        }\n        match writer.try_lock() {\n            Ok(guard) => break guard,\n            Err(std::sync::TryLockError::Poisoned(error)) => {\n                return Err(TransportError::Io(error.to_string()));\n            }\n            Err(std::sync::TryLockError::WouldBlock) => {\n                let millis = deadline\n                    .saturating_duration_since(Instant::now())\n                    .as_millis()\n                    .clamp(1, 25) as i32;\n                if unsafe { libc::poll(std::ptr::null_mut(), 0, millis) } < 0 {\n                    let error = std::io::Error::last_os_error();\n                    if error.kind() != std::io::ErrorKind::Interrupted {\n                        return Err(TransportError::Io(error.to_string()));\n                    }\n                }\n            }\n        }\n    };\n    let result = (|| {\n        let output = guard.as_mut().ok_or(TransportError::AgentDead)?;\n        let mut offset = 0;\n        loop {\n            if Instant::now() >= deadline {\n                return Err(TransportError::Io(\"budget writer Codex dépassé\".into()));\n            }\n            let progress = if offset < bytes.len() {\n                output.write(&bytes[offset..]).map(Some)\n            } else {\n                output.flush().map(|()| None)\n            };\n            match progress {\n                Ok(None) => return Ok(()),\n                Ok(Some(0)) => {\n                    return Err(TransportError::Io(\"écriture Codex sans progrès\".into()));\n                }\n                Ok(Some(count)) => offset += count,\n                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,\n                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {\n                    // Box<dyn Write> ne donne aucun fd. poll sans descripteur\n                    // attend au plus25ms sous la MÊME échéance, sans busy-loop.\n                    let millis = deadline\n                        .saturating_duration_since(Instant::now())\n                        .as_millis()\n                        .clamp(1, 25) as i32;\n                    if unsafe { libc::poll(std::ptr::null_mut(), 0, millis) } < 0 {\n                        let error = std::io::Error::last_os_error();\n                        if error.kind() != std::io::ErrorKind::Interrupted {\n                            return Err(TransportError::Io(error.to_string()));\n                        }\n                    }\n                }\n                Err(error) => return Err(TransportError::Io(error.to_string())),\n            }\n        }\n    })();\n    // Une trame partielle n'est pas une réponse négative attestée. Fermer le\n    // canal empêche qu'une écriture suivante complète ses octets incertains.\n    if result.is_err() {\n        let _ = guard.take();\n    }",
        "reviewed_at_utc": "2026-10-12T03:37:28.083936+00:00"
      }
    }
  ],
  "persistent_history": [
    {
      "original_id": "DUP-001",
      "original_fingerprint": "c0dcaaf9b24b",
      "original": {
        "id": "DUP-001",
        "category": "quality",
        "subcategory": "duplication",
        "severity": "LOW",
        "file": "crates/bridget-daemon/src/wrapper.rs",
        "line": 4367,
        "title": "Renvoi après reconnexion copié dans deux branches",
        "description": "Les lignes4367–4379 et4611–4623 sont identiques : elles relisent les demandes waiting, excluent uncertain et remettent les trames au daemon. Les deux chemins terminent une reconnexion avec le même writer et le même état. Aucun helper équivalent de ce renvoi collectif n’a été trouvé ; send_wrapper_message reste le point unique d’envoi d’une trame.",
        "evidence": "                if forward_child_requests {\n                    let pending: Vec<_> = child_requests\n                        .lock()\n                        .unwrap_or_else(|e| e.into_inner())\n                        .waiting\n                        .iter()\n                        .filter(|entry| !entry.uncertain)\n                        .map(|entry| entry.request.clone())\n                        .collect();\n                    for request in pending {\n                        send_wrapper_message(&writer, WrapperToDaemon::ChildRequest { request });\n                    }\n                }",
        "exploitability": "N/A",
        "impact": "Une évolution du filtre ou de la remise doit être reportée dans deux copies ; risque de divergence de maintenance. Aucun défaut runtime actuel ni taux global de duplication n’est déduit.",
        "path_type": "cold",
        "remediation": "Réunir ce seul bloc dans une fermeture locale au parcours géré, appelée après les deux reconnexions. Garder les deux branches et leurs contrôles. Aucun module, export, framework, table ou outil neuf.",
        "verification": "Les deux appels doivent conserver le filtre !uncertain et envoyer le même tuple ; rejouer les tests existants native155_wrapper et la reconnexion149 concernés, sans nouveau scénario.",
        "effort": "XS",
        "regression_risk": "LOW",
        "duplication_type": "STRICT",
        "duplication_lines": 13,
        "occurrences_count": 2,
        "all_locations": [
          "crates/bridget-daemon/src/wrapper.rs:4367",
          "crates/bridget-daemon/src/wrapper.rs:4611"
        ],
        "fingerprint": "c0dcaaf9b24b"
      },
      "status": "persisting",
      "proof": {
        "file": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/wrapper.rs",
        "line": 4367,
        "sha256": "659a1e09f6ac4c351d6d906e1065f4614680e016cf36cd0c066b039df074278d",
        "evidence": "                if forward_child_requests {\n                    let pending: Vec<_> = child_requests\n                        .lock()\n                        .unwrap_or_else(|e| e.into_inner())\n                        .waiting\n                        .iter()\n                        .filter(|entry| !entry.uncertain)\n                        .map(|entry| entry.request.clone())\n                        .collect();\n                    for request in pending {\n                        send_wrapper_message(&writer, WrapperToDaemon::ChildRequest { request });\n                    }\n                }"
      },
      "current_id": "DUP-001",
      "current_fingerprint": "c0dcaaf9b24b"
    },
    {
      "original_id": "PERF-082",
      "original_fingerprint": "19cf86e44509",
      "original": {
        "id": "PERF-082",
        "category": "performance",
        "subcategory": "sequential_ipc_budget",
        "severity": "LOW",
        "file": "crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 811,
        "title": "Le tick renouvelle une seconde pour chaque émission séquentielle",
        "description": "Le lot de followups est borné à512 lignes, mais chaque appel emit recrée son propre budget d’une seconde. Le tick natif attend la fin de cette boucle avant ses propres transitions. Plusieurs writers lents peuvent donc accumuler leur attente. La condition de saturation n’a pas été mesurée ; les timeouts et la fermeture des sockets limitent chaque émission.",
        "evidence": "for (id, conn, writer, frame) in outgoing {\n    emit(state, &id, &conn, &writer, &frame, false);\n}",
        "exploitability": "THEORETICAL",
        "impact": "Retard potentiel du tick natif et des échéances. Le budget nominal des attentes IPC peut cumuler jusqu’à k secondes, k<=512 ; les relectures sources/SQL ont un coût distinct non mesuré. Ce n’est pas une durée runtime observée.",
        "path_type": "hot",
        "remediation": "Conserver ce risque comme limite connue. Avant toute optimisation, utiliser les logs de tick existants pour établir une baseline réelle ; si un retard est observé, borner le budget du lot tout en conservant les claims et les recontrôles FR027. Aucun patch de confort demandé dans cette lecture.",
        "verification": "Vérification structurelle : budget renouvelé à :833 et appel préalable au tick natif à native_delegation.rs:2402. Une éventuelle mesure future réutilise les logs existants ; aucun benchmark ou scénario exécuté ici.",
        "effort": "S",
        "regression_risk": "MEDIUM",
        "references": [
          "crates/bridget-daemon/src/daemon/native_child_requests.rs:833",
          "crates/bridget-daemon/src/daemon/native_delegation.rs:2402",
          "audit-code-v14 module08 profiling-first"
        ],
        "fingerprint": "19cf86e44509"
      },
      "status": "persisting",
      "proof": {
        "file": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 907,
        "sha256": "3be800076fe7523adc083dce913f4d38988dc7da540fb6d4fa6bc3c2adb2af51",
        "evidence": "    for (id, conn, writer, frame) in outgoing {\n        emit(state, &id, &conn, &writer, &frame, false);\n    }"
      },
      "current_id": "PERF-082",
      "current_fingerprint": "19cf86e44509"
    }
  ]
}
```
