# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/research.md](../../../../../../specs/145-bridget-thread-viewer/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

L'identité `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/t3code_identity.rs` atteste des processus pour le contexte MCP. Elle fournit un modèle de refus en cas d'ambiguïté, mais un lecteur humain dormant ne doit pas dépendre d'un PID de fournisseur. La connexion primaire est préparée dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/t3code.rs` vers1277. Le principal a figé un nouveau fait de liaison de cette connexion après enregistrement et publication du projet.

<a id="extrait-2"></a>

## Extrait 2

T3 possède le store natif `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/rightPanelStore.ts`, les onglets `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/RightPanelTabs.tsx` et le montage `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/ChatView.tsx`. Leurs unions sont fermées ; les extensions doivent couvrir toutes les branches et la migration persistée. Aucun contrat Bridget n'existe à réutiliser dans T3 au point de départ.

<a id="extrait-3"></a>

## Extrait 3

Le service `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/processRunner.ts` accepte déjà commande, argv, délai et plafond de sortie. `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/auth/RpcAuthorization.ts` couple les scopes de lecture au groupe RPC. Les résolutions de conversation et projet existent dans `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/orchestration/Services/ProjectionSnapshotQuery.ts`.

<a id="extrait-4"></a>

## Extrait 4

Choisir un rafraîchissement manuel et une recherche locale dans les pages chargées. Garder les séquences historiques ASC déjà présentes. Utiliser la racine projet autoritative T3, pas un chemin fourni par le navigateur ou l'annuaire `last_known`.
