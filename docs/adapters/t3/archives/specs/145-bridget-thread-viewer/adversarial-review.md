# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/adversarial-review.md](../../../../../../specs/145-bridget-thread-viewer/adversarial-review.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-07. Statut : final accepté. Revue du code en lecture seule terminée à 16:17:04 UTC. Verdicts : APPROVE Rust et APPROVE T3. Artefacts audit v14 validés deux fois par le principal, exit 0, zéro erreur, zéro warning. T020 suit Converge2 hors comparaison.

<a id="extrait-2"></a>

## Extrait 2

Deux worktrees145 seulement : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/` et `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`.

<a id="extrait-3"></a>

## Extrait 3

La revue finale porte sur31 sources/tests :8 Rust et23 T3. Hash agrégé avant et après la revue readonly : `7c765f38d24163def6fe80a1b4c792aca1b954e3f6612ee63ac1215a40456bc6`, identique selon le principal. Aucun source n'est modifié pendant ce cycle final. Les hashes du gel initial sont conservés séparément.

<a id="extrait-4"></a>

## Extrait 4

Sources corrigées : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/bridget/BridgetReader.ts`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/hooks/useCopyToClipboard.ts` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/cli.rs`.

<a id="extrait-5"></a>

## Extrait 5

Le vrai AtomRegistry et la factory native de `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/client-runtime/src/state/orchestration.test.ts` couvrent A → B → A avec ancien RPC non interruptible. Le résultat tardif ancien n'est pas publié. Cette preuve porte sur le runtime natif, pas seulement sur une UI mockée.
