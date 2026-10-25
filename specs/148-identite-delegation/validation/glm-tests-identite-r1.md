# Rapport de test — session 148 identité/délégation

## Résultats d'exécution

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r1.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r1.md#extrait-2).

## Mouvement du code (worktrees édités en parallèle pendant les tests)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r1.md#extrait-3).

## Couverture vs contrat — ce qui est prouvé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r1.md#extrait-4).

## Trous — preuves nécessaires, par priorité

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r1.md#extrait-5).

**P2**
4. `lifecycle.rs:463` exclut la paire de `pass_env` (credential jamais hérité par un enfant). Non testé.
5. 401 d'un vrai serveur après `revokeThread` : la révocation est testée au registre (`resolve` → undefined), le 401 est simulé par le serveur factice.
6. Réponse 3xx explicite : `with_max_redirects(0)` est dans le code, pas dans les tests.
7. Credential auxiliaire réel : `Registered` avec `credential: None` suffit au test ; le chemin avec credential fourni et vérifié n'est pas prouvé.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r1.md#extrait-6).

## Limites

Tests exécutés sur des états mouvants. Les verts sont valables pour les hashes finaux listés ci-dessus. Ils peuvent être invalidés par la prochaine édition des agents Codex. Faux HTTP partout côté serveur : la conformité protocole et les refus sont prouvés en isolation. L'accord réel des deux bords reste à démontrer (P1-1 à P1-3).
