# Plan d implémentation - SPEC-072

## Résumé technique

Étendre le registre existant avec un seul chemin de configuration Claude Code isolé. Ne pas créer de transport par fournisseur : Cursor reste dans AcpTransport, et Anthropic, GLM ainsi que DeepSeek restent dans ClaudeStreamJsonTransport. Faire passer le type déclaré jusqu au contexte fournisseur afin que le journal distingue anthropic, glm et deepseek.

## Frontières de propriété

| Périmètre | SPEC-072 | SPEC-071 |
|---|---|---|
| Registre, environnement, lancement, journal fournisseur | propriétaire | consommateur futur |
| Runtime, logos, fiche visuelle | ne modifie pas | propriétaire actuel |
| Appel fournisseur et transport | étend l existant | hors périmètre |

## Phases

### P1 - Contrat de profil non secret

1. Ajouter claude_config_dir aux définitions du registre et aux définitions résolues, au digest et aux tests de stabilité.
2. Le valider comme chemin absolu réservé à claude_stream_json.
3. Construire l environnement d un enfant à partir de ce chemin, en refusant un répertoire absent ou non sûr plutôt que de réutiliser Anthropic.
4. Mettre à jour chaque constructeur de test de définition résolue afin que les preuves de reprise et de digest couvrent ce nouveau champ.

### P2 - Provenance exacte des flux Claude Code

1. Passer le type d agent au ClaudeStreamJsonOptions.
2. Remplacer les contextes provider_kind = claude codés en dur par ce type.
3. Prouver qu un flux historique claude reste claude, tandis que glm et deepseek restent distincts dans leurs événements et leur binding.

### P3 - Registre et profils privés

1. Conserver cursor natif, déclarer anthropic, glm et deepseek dans le registre privé du serveur avec protocoles et capacités explicites.
2. Créer les profils GLM et DeepSeek à partir des fonctions locales, sans afficher leurs valeurs et avec les permissions 0700/0600.
3. Ne pas modifier le profil Anthropic actif. Déclarer le type anthropic comme profil dédié seulement après avoir prouvé son emplacement.
4. Vérifier qu aucun secret n apparaît dans git diff, dans le journal ou dans les réponses d erreur.

### P4 - Validation vivante par fournisseur

1. Lancer un agent Cursor géré, lui demander une réponse courte sans écriture, puis vérifier texte, contexte et clôture de tour.
2. Lancer GLM et DeepSeek dans des worktrees de test sans écriture, vérifier texte, modèle observé, outil de lecture et fin de tour.
3. Lancer Anthropic pendant un essai GLM afin de prouver que son upstream ne change pas.
4. Vérifier l interruption, les refus et les erreurs avec leurs motifs utiles.
5. Ne déclarer un fournisseur actif que si sa preuve vivante est complète.

## Tests attendus

- Tests Rust ciblés du registre, de l environnement, du digest et du transport Claude Code avant puis après chaque changement.
- Test de contrat ACP déjà existant, plus échange réel Cursor ACP.
- Tests d intégration réels GLM et DeepSeek, isolés et sans écriture.
- Test de non-régression Codex, Claude historique, interruption et remise.
- cargo fmt --check, tests ciblés puis cargo test --workspace et build release avant livraison.

## Sécurité et exploitation

- Les fichiers secrets ne sont pas dans le dépôt ni dans le worktree.
- Les profils sont créés de manière atomique avec droits restrictifs, sans afficher leur contenu.
- Aucune mise à jour de compte, de binaire fournisseur ou de session globale.
- Déploiement seulement après les preuves, redémarrage unique de bridget-ui.service, puis canari fournisseur et lecture de journal.

## Critères de sortie

- Cursor répond dans un agent géré et sa provenance est cursor.
- GLM et DeepSeek répondent avec leurs profils isolés, ou restent explicitement non configurés avec la preuve de leur blocage externe.
- Anthropic historique reste fonctionnel et non redirigé.
- Le journal conserve le fournisseur déclaré, le transport et le modèle observé sans heuristique.
- Aucun fichier de SPEC-071 n a été modifié.
