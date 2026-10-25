# Archives techniques de l’adaptateur T3

Document d’origine : [specs/141-messages-groupes/research.md](../../../../../../specs/141-messages-groupes/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Décision : étendre projectBridgetEnvelope et BridgetMessageBody. Leur responsabilité couvre déjà ce besoin. Preuves : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:105 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4034.

<a id="extrait-2"></a>

## Extrait 2

Décision : traiter seulement le format direct 📥 produit par batch_envelope. Preuve : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:3033. Les notifications utilisent une autre grammaire. Le transport Rust est conservé.

<a id="extrait-3"></a>

## Extrait 3

Décision : réutiliser Vitest et les deux suites de timeline. Preuves : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/package.json:13, /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts:208 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx:287.
