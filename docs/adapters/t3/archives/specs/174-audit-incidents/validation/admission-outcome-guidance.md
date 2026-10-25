# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/admission-outcome-guidance.md](../../../../../../../specs/174-audit-incidents/validation/admission-outcome-guidance.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les choix de ce parcours relèvent du contrat Bridget : identité attestée,
état durable, budgets bornés et vérification avant annonce de réussite.
Les anciennes comparaisons de produits ont été retirées le25octobre2026.

<a id="extrait-2"></a>

## Extrait 2

- macOS: ad-hoc signature verified; SHA-256 `a3818666049c5185b59b8b28a94cd256926cfe8691b7109ec8df057b0c1ca27c`; daemon build ID `6d0f01096d10`, PID 45405. T3 and its bridge retained their processes.
- Cartae: SHA-256 `68b15bb493f1a374ed58c7e6554d65cc21caee19d61aee9adb4477e48417c78f`; worker PID 4150007, executable path verified. The unrelated daemon retained its process. The previously observed business wrapper had already ended with exit code 0 and a clean result before the worker restart; no preservation of a still-running wrapper is claimed.
- Live MCP verification: an intentionally mismatched source SHA returned the existing error code plus `detail.admission.state=not_recorded`, `task_id=null`. The admission and task tables contained zero rows for this diagnostic key. No provider or business mission started.
- Politique was sent the delivery and field usage guidance; recipient acknowledgment is not claimed.

<a id="extrait-3"></a>

## Extrait 3

The earlier four-test run covered the initial patch; the final four tests also cover a different body under the same key and owner isolation. No full-suite or second independent approval is claimed. Existing clients use generic detail rendering, so this daemon-side guidance is visible without restarting T3.
