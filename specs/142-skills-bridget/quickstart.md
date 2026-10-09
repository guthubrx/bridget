# Vérifications ciblées — SPEC142

## Avant toute publication

Travail Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/`.
Travail dotfiles : `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/`.
Sauvegarde initiale : `/Users/moi/.cache/bridget-skills-142.eTqmgl/`.

Lire `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/specs/142-skills-bridget/reuse-audit.md` et vérifier le gate avant implémentation. Relever les vrais liens et empreintes des scripts actifs. Ne pas démarrer `agent_loop.py` pour tester les noms.

## Contrôles locaux

Depuis le worktree dotfiles, vérifier la syntaxe du publisher :

```bash
bash -n /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/sync_codex_skills.sh
```

Exécuter les tests ciblés de synchronisation isolée créés en T002/T004 : `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/tests/test_142_bridget_skill_sync.py`. Ils doivent utiliser des racines temporaires via les paramètres du publisher, jamais les racines actives. Consigner leur commande exacte et leurs résultats dans le journal d'implémentation. Aucun package nouveau requis.

Valider les nouveaux frontmatter et métadonnées avec le validateur local existant : `/Users/moi/.codex/skills/.system/skill-creator/scripts/quick_validate.py`. Donner à chaque invocation un dossier exact, pas une racine large. La validation de forme ne prouve pas la découverte native.

Inspecter le diff et les huit appelants. Vérifier que les quatre lignes CLI historiques de `horizon-new-episode` n'ont pas changé. Vérifier l'absence de fichiers moteur, daemon, T3 ou LaunchAgent modifiés.

## Recette de publication

Le principal coordonne la publication avec sauvegarde. Vérifier d'abord les cibles résolues, dont le handoff live externe. Préserver les anciens arbres scripts. Publier seulement les instructions, métadonnées et liens prévus. Archiver pre-104 hors catalogue, sans supprimer son contenu. Relire les fichiers installés et comparer les empreintes scripts/config aux relevés initiaux. Exécuter une seconde publication et vérifier l'idempotence.

Réutiliser la sonde native Codex isolée `/tmp/bridget-spec142-catalog.jreUv2/probe.cjs` ou une fixture équivalente sans appel de modèle. Rapporter la version et le périmètre exacts. Pour Claude, rapporter la parité des fichiers ; ne pas annoncer une sonde native non réalisée.

## Limites et livraison

Publication globale des six identités vérifiée. Tests synchronisation 16/16 et boucle isolée 152/152 PASS par l'équipier. La seconde publication est un no-op sur 36 fichiers d'instructions/métadonnées, trois racines, avec octets, chemins réels, dates et modes inchangés. Le lecteur natif Codex et le lecteur Claude de T3 découvrent les six identités invocables via fixtures isolées des fichiers publiés ; aucun modèle n'a été exécuté.

Archives récupérables : `/Users/moi/dotfiles/archives/skills/spec142/codex/agent-bridge.pre-104-20260713-082347/` et `/Users/moi/dotfiles/archives/skills/spec142/claude/agent-bridge.pre-104-20260713-082347/`. Reçu détaillé : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/specs/142-skills-bridget/results.json`.

Pas de commit/fusion/push142 : autorisation distincte non reçue. Aucun restart T3, daemon ou LaunchAgent. Aucun appel de modèle ni API payante. Une session déjà ouverte peut conserver son catalogue ; ne pas prétendre l'avoir rechargée. Aucun rendu du menu natif Claude ni comportement implicite du modèle n'est garanti par ces contrôles.
