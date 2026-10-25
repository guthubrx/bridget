# Archives techniques de l’adaptateur T3

Document d’origine : [specs/146-bridget-panel-lisible/reuse-audit.md](../../../../../../specs/146-bridget-panel-lisible/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Source inspectée | Constat | Décision146 |
| --- | --- | --- |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/store/threads.rs:1184` | Liste145 UUID ASC avant limite, jusqu'à1212. | Étendre par une lecture récente distincte, activité du dernier message puis UUID ASC ; ne pas trier seulement la page T3. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/store/threads.rs:651` | Helper partagé de lecture ASC, borné en octets ; projection de correction665–670. | Réutiliser sérialisation et noms. Ajouter ordre interne DESC et borne globale de correction distincte de la borne de page. Garder les wrappers145 ASC. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/store/threads.rs:1431` | Historique145 fixe un instantané sans mutation, jusqu'à1469. | Réutiliser cette garantie pour `history_recent` ; curseur inclusif vers les séquences plus anciennes. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-transport/src/protocol.rs:2204` | Le résumé145 ne porte pas de date d'activité. | Créer un résumé récent distinct avec champ requis ; conserver145 et les usages `shown`. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/daemon.rs:11782` | Deux gardes exactes de singleton humain11782/11789 évitent la maintenance. | Étendre les deux gardes au singletonRecent exact. Préserver refus et whitelist13759. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/communication/client.rs:166` | Négociation et vérification de capacité humaine vers166/173. | Ajouter la capacité récente distincte ; aucun fallback récent vers145. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/packages/contracts/src/bridget.ts:96` | `shown` reprend le résumé145, jusqu'à100. | Garder ce modèle ; ajouter deux unions récentes séparées et leurs invariants d'ordre et de curseur. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/server/src/bridget/BridgetReader.ts:111` | CLI par argv, budgets et parsing fermé. | Étendre les deux actions et leurs options ; réutiliser annulation, refus assainis et exécution sans shell. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx:90` | Déduplication de liste existante90–95 ; ancien historique démarre à238 ; append304–315 ; pages419–426. | Réutiliser contexte et déduplication ; remplacer les lectures du panneau par les variantes récentes et une suite DESC. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/packages/client-runtime/src/state/orchestration.ts:11` | Cache natif avec stale3 secondes et idle10 secondes. | Réutiliser le client et son invalidation ; aucun polling ni second cache métier. |

<a id="extrait-2"></a>

## Extrait 2

`/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx:272` couvre trois pages et copie ;309 révocation ;334 recherche ;356 rafraîchissement et réponses tardives. Réutiliser ces scénarios avec les nouvelles réponses, puis ajouter aperçu, copie entière repliée et détails.

<a id="extrait-3"></a>

## Extrait 3

`/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/packages/client-runtime/src/state/orchestration.test.ts:131` couvre A → B → A. Conserver sa garantie réelle, pas seulement une simulation de composant.

<a id="extrait-4"></a>

## Extrait 4

- [x] Le socle145 et ses sources non committées sont distingués des changements146.
- [x] Les trois frontières stockage/transport/T3 ont été examinées.
- [x] Les actions145 restent inchangées ; l'incompatibilité récente est explicite.
- [x] Liste globale avant pagination et activité du dernier message sont exigées.
- [x] Snapshot historique, borne inclusive et correction globale sont séparés.
- [x] Les deux gardes humaines de maintenance sont explicitement couvertes.
- [x] Réutilisation des primitives et des tests est justifiée par l'existant.
- [x] Aucun helper SpecKit absent n'est prétendu exécuté ; application documentaire manuelle du protocole par le principal.
- [x] Aucune mutation, nouvelle dépendance ou action de production n'est prévue.
