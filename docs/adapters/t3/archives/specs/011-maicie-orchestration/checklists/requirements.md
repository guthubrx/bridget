# Archives techniques de l’adaptateur T3

Document d’origine : [specs/011-maicie-orchestration/checklists/requirements.md](../../../../../../../specs/011-maicie-orchestration/checklists/requirements.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] CHK001 Les objectifs sont créés explicitement, jamais par un message direct.
- [x] CHK002 Le mode collaboratif et le mode délégué sont distingués sans verrouiller la conversation.
- [x] CHK003 GUI, TUI, DSH, T3 Code, DAG et scheduler sont explicitement hors périmètre MVP.
- [x] CHK004 Tous les futurs fichiers source Maicie appartiennent à `plugins/maicie/`.
