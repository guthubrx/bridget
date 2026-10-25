# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md](../../../../../decisions/150-feuille-de-route-delegation-cinq-etoiles.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- La suspension des missions pendant une pause, prévue en 150, passe en 152 :
  elle partage le mécanisme de reprise d'une mission avec son identifiant
  d'origine et le gel des horloges (contre-revue 150, objections 1 à 4 et 7).
- Constat pour 157 : `bridget_capabilities` annonce `discovery: true` pour
  `codex-pro`, mais `bridget_delegate` refuse `permission_source_unavailable`
  depuis un fil T3 Claude (2026-10-11).
- Les sessions modifient les mêmes fichiers (`native_delegation.rs`) : exécution
  séquentielle dans un seul worktree, sans commit automatique, diff enregistré
  par session.
