# Développement 168 — lot D

Date : 2026-10-15.

## Réalisé

- `decision_service.rs` porte la charge normalisée168, ses validateurs fermés, le refus explicite de capacité Laya et le permis de chaîne.
- `claude_stream_json.rs` porte l'appel Claude isolé : environnement vidé puis allowlisté, répertoire temporaire privé, flags qualifiés, stderr nul, stdout 64 Kio et résultat 4 Kio.
- Le parseur n'accepte aucun `tool_use` sauf `StructuredOutput`, outil interne exigé par `--json-schema` dans la qualification réelle Haiku ; la seule sortie retenue est `result.structured_output`.
- Correctif revue ROOT : commande alignée sur la préqualification (`--print`, entrées/sorties stream-json, `--verbose`, `--strict-mcp-config` sans valeur et `--mcp-config` vide). Init, capacité, outil, sortie structurée et texte final sont contrôlés et concordants.
- Oracles source ajoutés dans les bancs existants : défaut natif sans effort, IDs v2 séparés de163, formateur seul, capacités interdites, sorties contradictoires et surtaille. Ils ne sont pas exécutés.

## Contrôles exécutés

- `rustfmt --edition 2024` sur les fichiers D : exécuté, succès.
- `git diff --check` sur les fichiers D : exécuté, succès.

Aucun Cargo, test, build ni appel modèle réel n'a été exécuté : permis ROOT absent.

## Raccord restant

C conserve `RoutingDecisionPermit` de la consultation service jusqu'au drain Claude, puis valide strictement `ClaudeIsolatedDecisionResult` par `validate_routing_choice` avant CAS. Les oracles T021 et le raccord de chaîne T020 restent à exécuter par leurs propriétaires.
