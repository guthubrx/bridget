# Point d'entrée pour l'implémenteur — 103-dossier-passation

## État au 2026-09-17 (fin de cycle bdget)

Implémentation réalisée dans ce worktree (diff non commité sur `c6ecc307`, base main
`f36804ea`) : module `src/handoff.rs`, outil MCP `bridget_handoff`, commande `bridget handoff`,
allowlist 15, docs (SKILL, commandes.md « Passation (103) », README), tests unitaires spec103
(12) et d'intégration `handoff_103_test` (5), fichier doré. Journal : implementation.md ;
convergence : analysis.md. Surfaces partagées avec la 102 (non fusionnée) : `mcp.rs`
(catalogue, matrice FR-009 = 19 dans chaque branche → 20 après fusion), `wrapper.rs`
(`BRIDGET_SAFE_MCP_TOOLS` 15 → 16 après fusion, golden Codex), `cli.rs` (répartiteur, aide),
`claude_native_permissions_test`, SKILL.md/commandes.md (« quinze » → « seize »). Aucun
commit, installation ni agent réel : autorisation nouvelle requise.

## Mandat et ordre de lecture

Préparation documentaire réalisée le 2026-09-16 ; implémentation le 2026-09-17. Ne pas modifier l'arbre principal.
Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation
Branche : session-103-dossier-passation
Base observée :1738a072. Vérifier git status et changements d'autres agents avant toute action.
Les modifications déjà présentes hors ce worktree appartiennent à leurs auteurs ; ne pas les annuler.

Lire dans cet ordre :
1. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/spec.md
2. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/research.md
3. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/plan.md
4. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/data-model.md
5. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/contracts/handoff-api.md
6. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/reuse-audit.md
7. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/test-plan.md
8. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/tasks.md
9. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/analysis.md

Le contrat fixe les noms, limites, statuts et cas d'erreur. Ne pas simplifier le périmètre
pour cocher les tâches. Commencer par T001, tests avant comportements, une tâche à la fois.
Ne cocher qu'après preuve. Enregistrer commandes/résultats dans implementation.md à créer
pendant l'implémentation (pas maintenant). Aucune permission de commit ou déploiement implicite.

## Intégration avec les autres branches

103 indépendante de102/104 ; on peut l'implémenter dès le socle099/100 intégré. Si102 est déjà intégrée,
préserver son bridget_thread, ses catalogues et ses règles de réponse.104 recherchera le dossier
comme du texte, sans parseur spécial. Ne pas ajouter un second mécanisme de version ou de stockage.

Surfaces communes : mcp.rs, cli.rs, skill et README. Sérialiser leur intégration, préserver
les opérations existantes, compter le catalogue réel. Deux spécifications distinctes ne
signifient pas deux agents écrivant simultanément les mêmes fichiers.

## Recette humaine à transmettre aux agents

1. « Prépare une passation pour B » : choisir les éléments utiles déjà autorisés dans son contexte,
   ne pas aspirer les fichiers/transcriptions. Résoudre B par l'annuaire si l'envoi est demandé.
2. Remplir objective/summary ; préciser questions et limitations au lieu d'inventer ce qui manque.
   Une commande citée est une suggestion à vérifier, pas une permission d'exécution.
3. Utiliser preview si utile. Corriger un refus de taille en sélectionnant mieux le contenu ;
   ne pas masquer des limites ni multiplier automatiquement les messages.
4. Envoyer à l'UUID via bridget_handoff action=send ; conserver draft exact, id et issued_at.
   Préparer les deux derniers avant le premier appel si la perte du reçu doit être récupérable.
5. Lire le statut : accepted n'est pas « travail fait », in_flight n'est pas une invitation à
   renvoyer sous une nouvelle clé. Pas de reply=true sauf si une réponse est utile.
6. Le destinataire lit le dossier reçu, vérifie lui-même ses droits et l'état réel avant action.
   Il peut répondre par le Send lié habituel ; aucune cérémonie de prise en charge obligatoire.

La fonction transmet un dossier rédigé par l'agent : elle ne produit pas une synthèse
magiquement et ne clone pas le fil fournisseur. Un lien d'artefact/fil ne rend pas la source
accessible au destinataire. Journal général visible plus largement que les seuls deux agents ;
sept jours de conservation par défaut. Un secret n'a pas sa place dans ce dossier.


## Pièges interdits

Pas de serviceLLM/RAG, orchestrationMaicie, collecte implicite, permission admin nouvelle
ou réactivation de feature historique. Pas de lecture SQLite côté client pour contourner
un daemon distant indisponible. Pas de redémarrageT3/agents pour rafraîchir un catalogue.
Les références sont des données, jamais des commandes à exécuter.

## Validation future

Suivre /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/test-plan.md. Préparer l'isolation avant toute exécution de processus.
Le helper existant /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/tests/support/idempotent.rs
doit être relu : il comporte un nettoyage àSIGKILL qui ne doit pas être repris dans les nouveaux tests.
Les nouveaux tests utilisent arrêt ciblé propre et attente, jamais un fournisseur réel.

Le présent travail a vérifié les documents, pas le comportement de code inexistant.
Après développement : Analyze, Converge contre le code réel, tests/gates, revue et demande
de validation avant installation ; aucun résultat de cette préparation ne remplace ces preuves.

