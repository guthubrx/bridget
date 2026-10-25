# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/validation/converge164-p1.md](../../../../../../../specs/164-conversation-t3-distante/validation/converge164-p1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- H : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/packages/client-runtime/src/bridgetJournalPresentation.ts
- W : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/web/src/components/BridgetTaskJournal.tsx
- M : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/mobile/src/features/threads/BridgetTaskJournal.tsx
- C : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/web/src/components/ChatMarkdown.tsx
- R : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/packages/client-runtime/src/state/orchestration.ts
- E : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/packages/client-runtime/package.json
- T : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts

<a id="extrait-2"></a>

## Extrait 2

Hypothèses : séquences propres au journal d’une tâche, enveloppes natives validées par les contrats actuels, lacunes explicites seules faisant autorité. Le journal ne prouve pas des tours fournisseur ni un statut distant frais. Aucune donnée média/outil ne s’exécute. Aucun import T3 n’entre dans Bridget.

<a id="extrait-3"></a>

## Extrait 3

36 PASS de tests ciblés ne prouvent ni rendu natif iOS/Android, ni capture Electron, ni paquet installé. AST stricte RC1 et qualification Babel RC0 restent deux faits distincts. Aucun GREEN global, recette, audit code final, push, installation, nettoyage ou changement de runtime T3 déduit.
