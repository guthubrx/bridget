La ronde est complète. Voici le rapport R2.

---

# Revue R2 — identité Bridget par session adaptateur de bureau (lecture seule)

**Verdict : APPROVÉ pour le code.** Un manque de preuve runtime reste ouvert. Il est déjà planifié pour la validation finale. Aucun défaut de code nouveau trouvé.

---

## Réponses aux findings R1 — vérifiées

### Finding critique R1 (Mcp-Session-Id ignoré) — résolu en code

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r2.md#extrait-1).

### Finding majeur R1 (sessions accumulées) — résolu

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r2.md#extrait-2).

### Mineur R1 (test deletedAt) — résolu

`BridgetSession.test.ts:63-69` couvre inactif, archivé et supprimé (5 tests au total). Le handler refuse `deletedAt` (`OrchestratorMcpService.ts:1780`) puis `assertLiveCaller` refuse archivé / sans run actif / provider étranger (`OrchestratorMcpService.ts:1028-1040`).

### E0502 — corrigé selon le principal. `--no-run` vert annoncé. Non réexécuté ici (lecture seule). Aucun problème de compilation visible dans les fichiers lus.

---

## Points demandés par le brief — vérifiés

**Point de branchement `mcp_identity`** (`mcp_identity.rs:279-345`) :
- Une preuve refusée ferme l'appel **avant** toute lecture PID (`transpose()?`, ligne 281).
- Identités native et session divergentes → refus (`T3SessionUnavailable`, lignes 331-334).
- Native valide → `delegated_origin` native conservée (lignes 337-340). La preuve de session ne promeut jamais un enfant interne en principal.
- Marqueur enfant présent, même invalide ou cassé, interdit le repli sur la session seule : `has_delegated_marker` utilise `symlink_metadata`, et toute erreur autre que NotFound retourne vrai (lignes 308-322). Couvert par `session_proof_preserves_native_child_limits_and_rejects_conflicting_identity` et les tests spec133.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r2.md#extrait-3).

**Pas d'identité en argument** : `BridgetMcp.ts:43-51` — args `["mcp"]` seuls, identité par l'env du sous-processus. Refus des identités Bridget héritées (lignes 12-17).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r2.md#extrait-4).

**Pas de cache** : chaque appel d'outil refait l'attestation complète. Aucun état partagé, aucun credential conservé.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r2.md#extrait-5).

**Mocks résiduels dans l'interop** : seuls la projection de stockage (`ThreadManagement`), `ServerEnvironment` et des dépendances vides sont simulés. Le transport Effect (admission stateful), la registry, le toolkit, le handler, le client Rust et le daemon Bridget sont réels. Le credential du thread est réellement émis par le daemon (`registerThread`, testkit lignes 133-172), pas forgé. `bridgetThreadUuid` doit rester synchrone de `stable_uuid` : toute divergence fait échouer le test, donc rien n'est masqué.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r2.md#extrait-6).

---

## Findings

1. **MINEUR — manque de preuve runtime, pas un défaut de code.** Le test interop est opt-in : sans `BRIDGET_148_RUST_EXECUTABLE`, il est skippé (`BridgetRustInterop.test.ts:32-33`). Il n'a pas encore tourné. L'exécution des tests Rust compilés (`--no-run` vert) n'est pas non plus observée. Ces preuves doivent venir de la passe de validation finale déjà planifiée. Tout l'édifice code est vérifié par lecture croisée des deux sources.

2. **OBSERVATION — coût par appel, pas un défaut.** Chaque appel d'outil coûte 4 allers-retours HTTP (initialize, initialized, call, DELETE) plus l'échange Unix. La réponse choisie élimine l'accumulation par sessions éphémères. La recommandation R1 d'une session réutilisée n'a pas été suivie, mais le problème de sécurité est clos. Choix défendable : aucun état partagé, aucune fenêtre de vol élargie.

---

## Verdict

**APPROVE** — pour le code de cette ronde. Les trois findings R1 reçoivent une réponse correcte et complète. Le branchement `mcp_identity` est fail-closed sur tous les chemins examinés. L'approbation reste conditionnée à la validation runtime finale : exécution du test interop réel avec le binaire isolé, et exécution de la suite Rust (TCP strict, test307, subprocess env). Aucun de ces points ne demande de changement de code sur la base de la lecture.
