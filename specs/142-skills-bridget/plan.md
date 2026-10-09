# Plan — SPEC142

Date : 2026-10-07. Statut : Implemented — publié, non committé. Plan et revue finale APPROVE. Tests ciblés et publication vérifiés ; aucune exécution de modèle.

## Décisions

Réutiliser les compétences et le publisher existants. Renommer leur présentation, pas leur moteur. Les alias sont de petits `SKILL.md` distincts, pas des copies intégrales ni des dossiers liés au canon. Les scripts anciens restent la seule implémentation ; les nouveaux dossiers pointent vers eux.

Sources de travail :

- Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/skills/bridget/` ; source publiée stable `/Users/moi/Nextcloud/10.Scripts/64.bridget/skills/bridget/` après intégration autorisée.
- Loop moderne : `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/agent-loop/SKILL.md`, repris comme canon `bridget-loop` dans le même arbre Codex.
- Handoff : `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/agent-handoff-ledger/SKILL.md`, repris comme canon `bridget-handoff` dans le même arbre Codex.
- Publisher : `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/sync_codex_skills.sh`.

Les anciens fichiers Claude ne sont pas sources d'autorité pour ces noms. Le publisher garde une correspondance explicite source/cible pour les six identités. Il ne suit pas la date d'une copie pour choisir son contenu.

## Constitution et périmètre

Une branche par session, travail isolé, tests avant correction et sauvegardes avant publication. Aucun nouveau framework, service, modèle persistant ou dépendance. Les standards Next.js et tests Cartae ne s'appliquent pas à cette modification documentaire/Bash : utiliser des contrôles de format et des fixtures de publication ciblées. Aucun build Rust/T3 ni contrôle global des autres compétences. La session141 et l'application T3 restent intactes.

## Items et réutilisation de l'existant

1. Canon Bridget : conserver `/Users/moi/Nextcloud/10.Scripts/64.bridget/skills/bridget/SKILL.md:1`, ajuster seulement la description de présentation si nécessaire et ajouter les métadonnées.
2. Canon loop : reprendre les instructions modernes du fichier Codex existant ; changer les noms de compétence et références documentaires, pas les commandes ni les règles de boucle.
3. Canon handoff : reprendre les instructions existantes ; changer la présentation et références sans ajouter une obligation d'usage.
4. Alias : remplacer uniquement les trois `SKILL.md` historiques par de courts pointeurs absolus. Donner aux alias une identité distincte et une présentation française de compatibilité.
5. Métadonnées : réutiliser le format `agents/openai.yaml` existant dans agent-loop. Le prompt des canons contient `$bridget`, `$bridget-loop` ou `$bridget-handoff`. L'affichage des alias mentionne la compatibilité ; préserver leurs noms frontmatter historiques.
6. Scripts et LaunchAgent : conserver les arbres historiques. Ajouter seulement les liens relatifs `scripts` des nouveaux dossiers. Vérifier les cibles résolues avant écriture ; ne pas remplacer un lien runtime existant ni publier un moteur ancien par inadvertance.
7. Publisher : ajouter une branche ciblée six noms avant la logique mtime existante. Copier seulement les artefacts d'instruction nécessaires ; ne pas supprimer/remplacer un dossier historique entier ni ses scripts. Conserver les autres modes du publisher.
8. Sauvegardes : déplacer les anciennes sauvegardes pre-104 vers un emplacement explicite hors des racines de découverte, avec inventaire et contrôle de contenu. La sauvegarde réelle de préparation est `/Users/moi/.cache/bridget-skills-142.eTqmgl/`.
9. Appelants : modifier les huit `SKILL.md` recensés dans `research.md`. Garder les chemins CLI de `horizon-new-episode:52,63` pour Codex et Claude.
10. Tests publisher : ajouter une fixture indépendante ciblée au publisher existant, sans dépendance, si aucun test équivalent n'existe. Racines temporaires distinctes, sentinelles script, sauvegarde et seconde exécution.
11. Découverte : réutiliser le protocole local Codex `initialize`/`initialized`/`skills/list` sans lancer de modèle. La sonde préparatoire montre les alias distincts ; la publication réelle doit vérifier les sources installées. Ne pas revendiquer une découverte native Claude sans preuve.
12. Documentation : artefacts SpecKit, journal de publication et limites. Aucun nouvel outil de synchronisation concurrent.

## Séquence technique

1. Relever les cibles réelles, empreintes scripts/config et sauvegardes. Relire le gate réutilisation.
2. Écrire des tests isolés rouges pour promotion legacy, suppression de scripts, alias, sauvegarde et idempotence.
3. Produire les trois canons, trois alias, métadonnées et liens, puis corriger les huit appelants.
4. Étendre la branche ciblée du publisher, rendre les tests verts et vérifier la syntaxe Bash.
5. Relire diff, cohérence frontmatter/prompts et non-régression hors liste blanche.
6. Faire la publication autorisée avec sauvegarde et preuves, sans restart. En cas de source manquante ou cible non sûre, arrêter avant mutation.
7. Converge lecture seule, audit ciblé et journal final. Commits/merge/push attendent une autorisation explicite distincte.

## Contrôles et limites

Les tests vérifient les fichiers et le comportement du publisher, pas le respect futur des instructions par un modèle. La sonde native Codex 0.160.1 a réussi, mais sur fixture isolée. Deux alias explicites produisent deux entrées ; on ne promet pas trois seules entrées UI. Claude ignore actuellement le nom d'affichage OpenAI dans l'observation préparatoire ; son alias doit être clair par son propre frontmatter.

## Divergences volontaires

Trois nouveaux dossiers de présentation sont justifiés par trois nouvelles identités de catalogue. Ils ne créent pas trois moteurs. Les alias restent distincts car une liaison unique dédupliquée perd l'identité historique. Un test dédié de publisher est justifié : les tests des anciens moteurs ne couvrent pas la copie d'instructions entre racines.
