# Recherche - SPEC-086

## Sources internes examinées

- Le registre de projets de SPEC-065 ne porte actuellement aucun rôle système.
- `resolve_project_mounts` dans `crates/bridget-daemon/src/project_runtime.rs` possède déjà un booléen `writable` par mount.
- Les worktrees du projet sont déjà des chemins hôte explicites montés dans le conteneur.
- Le domaine Maicie est dans `plugins/maicie/`; il ne s'agit pas d'un dépôt autonome à découvrir.
- SPEC-066 fixe un conteneur par projet et SPEC-067 les ressources approuvées.

## Sources externes primaires

- Git, `git-worktree`: plusieurs worktrees liés partagent le repository commun, mais disposent de HEAD et index séparés. https://git-scm.com/docs/git-worktree
- Docker, bind mounts: le flag read-only applique la mutabilité au niveau du montage. https://docs.docker.com/engine/storage/bind-mounts/
- Anthropic, sécurité Claude Code: une restriction projet et les devcontainers réduisent l'étendue des écritures. https://docs.anthropic.com/en/docs/claude-code/security
- OpenAI, sécurité Codex: sandbox, approbations et observabilité doivent borner les capacités agentiques. https://openai.com/index/running-codex-safely-at-openai/

## Conclusions

1. Le dogfooding ne nécessite pas un nouveau type d'agent. La frontière utile est un projet système unique et sa politique de montage.
2. Une permission par agent ou mission ajoute des états de révocation difficiles sans gain immédiat. Le choix opérateur binaire est cohérent avec le modèle partagé par projet.
3. Le mode read-only doit être appliqué par Docker, pas seulement par une consigne à l'agent. Même activé, le checkout principal peut rester read-only au profit d'un worktree attribué.
4. Les worktrees rendent possible la coexistence interne/externe, à condition de préserver les chemins et de ne pas partager un même worktree actif.
5. L'auto-déploiement doit rester exclu: modifier une branche n'est pas une autorisation de remplacer le daemon qui porte le contrôle.
6. Les commits exigent l'écriture du git common dir. Le projet système est donc un domaine de confiance partagé, pas une sandbox contre un agent malveillant membre de ce projet.

## Alternatives rejetées

| Alternative | Rejet |
|---|---|
| Tous les projets peuvent modifier Bridget | Supprime la frontière projet et expose le système à toute mission. |
| Permission par agent | Administration lourde, incohérente avec le conteneur partagé. |
| Permission temporisée | Ajoute horloges, expiration et courses avec les travaux longs. |
| Copie du dépôt dans un volume Docker | Brise la coexistence avec les worktrees hôte et ajoute une synchronisation. |
| Merge et redémarrage automatiques | Un échec peut supprimer le plan de contrôle en cours d'utilisation. |
| Exécution Host pour le projet système | Ne permet pas à Bridget d'appliquer la frontière de montage. |
