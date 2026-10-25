# Archives techniques de l’adaptateur T3

Document d’origine : [specs/146-bridget-panel-lisible/convergence.md](../../../../../../specs/146-bridget-panel-lisible/convergence.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Tâches strictement byte-identiques avant et après le passage : SHA256 `343054f4c6109c00986b2811b80dad353f565c1b3057fa9157f7fc71a95da539`. Aucun manque fonctionnel trouvé,0 tâche ajoutée. La relecture porte sur10 FR et8 SC, le delta de production6 fichiersRust +3 T3 et les tests pertinents, dont nouvelle CLI. Les cases T008 et suivantes évoluent seulement après cette phase, hors Converge.

<a id="extrait-2"></a>

## Extrait 2

| Exigences | Réalisation fichier:ligne | Test ou preuve fichier:ligne | Résultat vérifié |
| --- | --- | --- | --- |
| FR146-01, SC146-01 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/store/threads.rs:167 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/store/threads.rs:1897 | Tri global activité dernière séquence DESC puis UUID ASC, vide et pagination. |
| FR146-02 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx:231 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx:302 | Date courte cohérente et création explicite d'un fil vide. |
| FR146-03, SC146-02 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/store/threads.rs:1563 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx:524 | Historique snapshot DESC, borne inclusive, octets et has_more par existence ; interop137 corps. |
| FR146-04 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx:444 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx:260 | Liste compacte, auteurs visibles, séparation et styleT3. |
| FR146-05, SC146-03 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx:518 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx:315 | Clamp4 mesuré, corps exact développé, observer nettoyé. |
| FR146-06, SC146-04 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx:502 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx:348 | Copieoriginal entier CRLF/Unicode et recherche partie cachée sans requête. |
| FR146-07, SC146-05 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx:540 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx:390 | Détails typeFR et remplacements disponibles, sans résumé. |
| FR146-08 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/daemon.rs:11790 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx:474 | Singleton autorisé, contexte attesté, stale A→B→A et refus. |
| FR146-09, SC146-06 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/daemon.rs:11876 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/daemon.rs:11686 | Refus avant maintenance, deux gardes ; état métier inchangé etCLI froide. |
| FR146-10, SC146-07 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx:460 | /Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx:369 | Focusclavieravanttoggle retiré, panneau360 px sans débordement ; recetteDOM réelle. |
| SC146-08 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/specs/146-bridget-panel-lisible/validation/integration-receipt.md:7 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/specs/146-bridget-panel-lisible/validation/rust-receipt.md:7 | RED/GREEN distincts,603tests T3,14 nouveaux Rust,typesfmtlint/buildprivé. |

<a id="extrait-3"></a>

## Extrait 3

Le principal a vérifié le navigateur isolé sur composants réels :137 messages en ordre descendant uniques en3 pages50/50/37,snapshot 137 ; copie exacte1746 octets ; clavier Espace/Entrée etfocus sur Copie ; panne/refus/stale sans ancien article ; clamp de 4 lignes à360 px sans débordement. Source `/Users/moi/.cache/t3-spec146-preview.650MHJ/browser-receipt146.json`.

<a id="extrait-4"></a>

## Extrait 4

Recherche d'un mot uniquement caché dans le corps deMessage 137replié : corps exact présent, aria-expanded false,46 requêtes avant/après saisie. Ce complément est une observation rapportée par le principal, distincte du receiptJSON initial. Aucun appel distant de recherche ajouté. Aucun PNG disponible : captures T3 ont échoué, aucune alternative ni capture prétendue.
