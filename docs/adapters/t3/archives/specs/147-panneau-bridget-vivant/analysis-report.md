# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/analysis-report.md](../../../../../../specs/147-panneau-bridget-vivant/analysis-report.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le statut0/26 ci-dessus décrit la passe initiale avant GO. Le principal a depuis autorisé les lanes ; les preuves courantes restent dans le journal et les résultats147. Les trois cas observés en implémentation ont été intégrés aux artefacts : ready coalescé côté atome malgré protection wire ; rupture CLI/daemon sous WebSocket T3 sain ; intervalle de plus de50 nouveautés.

<a id="extrait-2"></a>

## Extrait 2

Référentiel courant : 18 exigences, 10 critères, 5 scénarios utilisateurs et 30 tâches. Les tests finaux couvrent les branches de protocole, d'autorité, de processus, de reprise, de pagination, de sélection et de présentation. Rust 16PASS ; transport 298PASS ; tests threads 16PASS ; T3 contracts/Reader/auth/runtime 136PASS ; WebSocket 1PASS ; Web 116PASS. Les tests ignorés et avertissements préexistants restent dans le journal et les reçus, sans faux résultat zéro alerte.

<a id="extrait-3"></a>

## Extrait 3

Les deux audits v14 sont bornés au diff : 6 fichiers source Rust et 17 fichiers source/test T3. Les constats historiques sont conservés, les contre-revues finales sont APPROVE, le scoring readonly ne trouve aucun défaut résiduel confirmé. Les deux validateurs sortent RC0 avec zéro erreur et zéro warning. Aucun constat documentaire CRITICAL persistant. Cette consolidation a précédé le gate final du principal, désormais validé. T025/T026/T030 sont cochées sur les preuves. Aucun commit, fusion, push, installation ou activation.

<a id="extrait-4"></a>

## Extrait 4

| Exigence | Points d’entrée vérifiés | Preuve / responsabilité |
|---|---|---|
| FR147-01 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:12939 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts:241 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:664 | Signal→stream→relecture native, fixtures interop et recette séparées. |
| FR147-02 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:568 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:12964 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:253 | Done≠Replayed, mutations disponibles, noms pertinents et tri récent. |
| FR147-03 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:11541 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts:35 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:171 | Aucun pollingcorps, garde idle sans scan, silence natif69s. |
| FR147-04 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:807 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:130 | File16/coalescence/overflow et un groupe courant+dirty ; rafale100. |
| FR147-05 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts:50 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:316 | Retry technique borné, newready, rattrapage sous S. |
| FR147-06 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts:346 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/runtime.ts:346 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:97 | EOF/cancel scoped, visibilité et visite obsolète, dix cycles. |
| FR147-07 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/rightPanelStore.ts:503 | Clé environnement/projet/conversation, choix indépendant. |
| FR147-08 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:220 | show/history explicites hors page1, A103 restauré. |
| FR147-09 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:449 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts:43 | Contexte/panne masque mais gardeUUID ; seul refus confirmé du fil retirechoix. |
| FR147-10 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:12795 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts:105 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/auth/RpcAuthorization.ts:26 | Autorité serveur avant lectures/flux, réattestation avant émission. |
| FR147-11 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-transport/src/protocol.rs:2205 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/bridget.ts:19 | Schémas fermés contrôle seul, canaris et champs inconnus rejetés. |
| FR147-12 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/rightPanelStore.ts:942 | UUID seul réhydraté ; aucune copie de corps/titre/membres. |
| FR147-13 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:332 | Pont nouveautés/anciensegment, to_seq S commun, staging atomique. |
| FR147-14 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:414 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:849 | Identité DOM et keys stables ; recette focus/Range/détails/copie brute. |
| FR147-15 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:11255 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/ws.ts:2144 | Tables/cursors/verdicts identiques, aucun Deliver/dispatch/ACK/modèle. |
| FR147-16 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:76 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts:43 | Codes neutres, masque/refus, pas refonte visuelle large. |
| FR147-17 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:877 | Auteur→destinataires ou Sans sollicitation, date unique même ligne. |
| FR147-18 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:731 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:827 | Targets effectifs auteur exclu, noms autorisés/fallbackUUID, aucun appel extra. |

<a id="extrait-5"></a>

## Extrait 5

Deux passes Converge lecture seule validées, zéro exigence orpheline, zéro finding actif. Les limites de preuve native/synthétique et les avertissements préexistants sont conservés. Les audits Rust et T3 sont mécaniquement conformes (RC0, zéro erreur/alerte). Session Implemented/Validé en isolé, NON installée et NON activée. Aucun chantier148 incorporé.

<a id="extrait-6"></a>

## Extrait 6

Les audits initiaux du08/10 ne couvrent pas les extensions. Audits frais Rust/T3 du09/10 délégués aux reviewers, reçus et validateurs encore attendus. Ancien timeout CLI de qualification US8 corrigé par propriétaire : sortie64KiB/attente2s/global10s et garde PID. Revue du correctif et succès natif ne valent pas injection de timeout RED/GREEN. Vérification finale code↔tous FR/SC, absence d'orphelin et stabilité byte-à-byte des tâches restent à faire par le principal après gel des sources. Aucun commit/installation/activation/restart ou modèle.

<a id="extrait-7"></a>

## Extrait 7

- UUID/ACK : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs ; CLI : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs.
- MCP : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeMcp.ts ; Adapter : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.ts.
- Store : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/store/threads.rs ; Daemon : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs.
- Refresh : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/t3code_identity.rs ; Panel : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx.

<a id="extrait-8"></a>

## Extrait 8

| Gates réellement restants | État |
|---|---|
|FR31 titre/membres complets | RED1FAIL puisGREEN122PASS et statiquesRC0 ; main relit les deux paragraphes wrapping. Correctif code reçu, validation native dans QA restante. |
|Recette native US9 | Aperçu interrompu après resize ; evaluate échoué ne prouve pas0h2. Charte/clavier/DOM/copie complets encore à reprendre en isolé. |
|Audit frais T3 extensions | Final encore à recevoir. Audit Rust neuf validé RC0/0erreur/0warning par reviewer et principal, A/zéro actif limité à US6/US8/T040 et8fichiers/hunks ; pas toutSPEC147 ni réutilisation grade08/10. |

<a id="extrait-9"></a>

## Extrait 9

Le diagnostic assaini regional-wrkr-1/claude_glm/.claude-glm constate configuration sans Bridget et injection T3 t3-code seulement ; aucun secret ni modèle lancé. Après gate US7, FR21/22/23↔SC13/14↔T035–T041 possédaient reçus SDK292PASS/1SKIP, façade MCPnative3PASS et qualification refresh1PASS avec régressions22/14. Deux revues locales finales APPROVE puis GO principal permettaient leur cochage. Ce paragraphe est historique, avant T049/T050 et les gates globaux ci-dessous. Les preuves demeurent distinctes, pas SDK+modèleE2E.

<a id="extrait-10"></a>

## Extrait 10

Audits frais Rust A et T3 A99.86, zéro actif ; validateurs relus/re-exécutés par principal RC0/0erreur/0warning. Historique MEDIUM corrigé conservé, pas grade initial réutilisé ni dépôt global certifié. QA /Users/moi/.cache/t3-spec147-preview.457DX8/us9-final-native.json lue intégralement. Panneau CSS360px, viewport1843×1152, captures1280×800 ; composants/CSS/store réels, RPC/données/clipboard synthétiques. Aucune chaîne transport/provider/modèle E2E intégrale revendiquée.

<a id="extrait-11"></a>

## Extrait 11

L'utilisateur autorise ensuite commit/fusion/push/installation et redémarrage T3. Livraison EN COURS par principal, sans PR T3 ni cleanup142/145. Ce mandat ne transforme pas les preuves isolées en preuves de production. Reçus de sauvegarde ciblée app/binaire/DB Bridget (pas DB T3), installation/hashes/processus et smoke encore attendus. Aucun finding de développement rouvert ni task décomptée ;54/54 demeure validé.
