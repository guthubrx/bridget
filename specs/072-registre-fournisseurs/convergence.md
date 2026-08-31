# Convergence manuelle - SPEC-072

## Vérification de cohérence

- Le registre reste la seule source de sélection fournisseur. Aucun texte, nom
  d'agent ou modèle ne produit de déduction d'upstream.
- Cursor réutilise `AcpTransport`. Aucun adaptateur Cursor séparé n'a été créé.
- Anthropic, GLM et DeepSeek réutilisent `ClaudeStreamJsonTransport`. Aucun
  client HTTP, crate ou backend d'observabilité n'a été ajouté.
- Le type fournisseur est transmis explicitement au transport et aux bindings.
- Les profils compatibles utilisent chacun leur `CLAUDE_CONFIG_DIR` privé. Le
  profil historique `claude` reste inchangé.
- La compatibilité de reprise couvre les formes figées antérieures sans accepter
  de digest forgé ni de profil non validé.
- Aucun fichier de la SPEC-071 n'a été modifié.

## Écarts et limites honnêtes

- DeepSeek est raccordé mais son compte a refusé le tour réel par manque de
  solde. Il est donc disponible techniquement, pas attesté comme fournisseur
  facturable.
- Anthropic a répondu par la limite hebdomadaire réelle. Cette erreur reste une
  erreur fournisseur visible, non une substitution silencieuse vers GLM.
- La présentation graphique de l'identité fournisseur reste hors périmètre de
  la SPEC-072 et sera consommée par la SPEC-071.
