## Verdict : APPROVE_WITH_CHANGES

Aucune objection bloquante. Les cinq réponses de ROOT tiennent, à condition que la compatibilité 150 soit décrite comme additive dans le reçu de livraison. Changements demandés : documentaires uniquement, plus un point de semantique sur la correction quota (voir objection A). Je n'ai rien écrit, rien committé, rien lancé (ni Cargo, ni test, ni service).

## Réponses aux cinq objections

**1. Clé `substitution: null` : acceptée par la voie documentaire.**
- Code : `native_delegation.rs:260` écrit la clé pour toutes les réponses. Aucun consommateur de `crates/`, `skills/` ou `docs/` ne teste sa présence. Les seules autres occurrences sont des tests qui vérifient son contenu quand elle existe.
- Doc : `docs/delegation-native.md:87-91` dit que la clé est toujours rendue, `null` sans remplacement. La spec FR-015 (`spec.md:236-240`) dit la même chose.
- Limite à écrire : l'oracle 154 n'est plus identique au contrat 150 octet par octet. Une réponse sans fallback change d'octets, par ajout d'une clé `null`. La doc ne protège que les octets de la demande canonique, ce qui est exact.

**2. Fait « passé » sur une fenêtre connue : correction juste et bornée.**
- `provider_limits.rs:40` retire seulement la clé `fact.window` avant le `return`. Les autres fenêtres ne sont pas touchées.
- Le test `native157_limits_fenetre_perimee` (`provider_limits.rs:166`) couvre le cas : `five_hour` rejeté avec retour 500, puis fait passé à 95 retire l'ancienne entrée.
- Faiblesse : la dernière assertion (« autre fenêtre conservée ») ne vérifie que `!exhausted`. Elle serait vraie même si la fenêtre `unknown` était effacée. Limite à noter, pas un test à ajouter.

**3. `env_unfit` avant `cwd_gone` : documentée.** Vérifié dans `spec.md:389-393` et `docs/delegation-native.md:104-106`. C'est un changement de refus sur les lancements existants. La limite est déclarée.

**4. Horloge distincte : documentée.** Vérifié dans `spec.md:389-393` et `docs/delegation-native.md:58-59`. Le catalogue lit l'horloge au verdict (`native_delegation.rs:977`) puis à la projection (`:1473`). Je n'ai pas relu le chemin d'admission complet pour confirmer qu'il recontrôle le quota. ROOT l'affirme. Je ne peux pas le confirmer.

**5. Test flaky : documenté.** `converge157-r2.md:23` et `:69`, plus `tests-157.md:48`. L'échec sous créations Git concurrentes est signalé, avec le rejeu seul à 1/0 et l'absence de retry. Limite acceptée.

## Objection nouvelle (A) : LOW à MEDIUM, non bloquante

- Preuve : `provider_limits.rs:40` efface la fenêtre pour tout fait passé, quelle que soit la source. Une source faible peut donc effacer un rejet fort.
- Scénario : `five_hour` est `rejected` par `ClaudeStreamJson` avec retour 500. Plus tard, un `ClaudeStatusLine` `unknown` annonce `five_hour` avec retour 95 (passé). Le rejet est effacé et l'admission passe, alors que le fournisseur refuse encore. L'ancien code gardait le rejet jusqu'à 500.
- Effet réel : au pire, un lancement est tenté puis refusé par le fournisseur. Pas de corruption de données.
- Non prévu : la spec et le test ne couvrent que le cas même source.
- Changement minimal : soit documenter « le dernier fait de même fenêtre gagne, toutes sources confondues », soit limiter la suppression aux faits de même `source`.

## Questions de ROOT

- **Les cinq réponses satisfont-elles l'alternative documentaire ?** Oui pour 1, 3, 4 et 5. Chaque point figure dans la spec, la doc ou le reçu de converge.
- **La correction quota ferme-t-elle sans effacer une autre fenêtre ?** Oui par lecture : la suppression ne vise que la clé `fact.window`.
- **Reste-t-il une objection bloquante ou une régression liée aux deltas ?** Non bloquante. La seule régression de comportement possible est A, à documenter.

## Non vérifié

- Aucun test, build, Cargo, scanner ou service. Les chiffres 2396/9/74, 13 PASS ciblés et 44 natifs ne sont pas confirmés.
- Empreintes : seule `provider_limits.rs` a été recalculée (`6e69c7b5...`, conforme). Le manifeste et les 29 hashes n'ont pas été recontrôlés.
- Le contenu complet de `converge157-r2.json` et du plan n'a pas été relu.
