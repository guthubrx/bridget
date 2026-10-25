# Archives techniques de l’adaptateur T3

Document d’origine : [specs/166-rattachement-bridget-reprise/research.md](../../../../../../specs/166-rattachement-bridget-reprise/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Alternative | Motif du rejet |
|---|---|
| Réinstaller Bridget | Répare parfois l’émission, mais pas le montage dans un fil déjà chargé. |
| Augmenter les limites système | Sans rapport avec l’incohérence d’identité. |
| Créer une identité globale de secours | Casse l’isolation entre les fils. |
| Redémarrer T3 à chaque reprise | Interrompt les autres conversations et masque la cause. |
| Renouveler selon un minuteur | Ajoute un état et laisse une fenêtre de course. |

<a id="extrait-2"></a>

## Extrait 2

La correction appartient à l’adaptateur Codex de T3. Bridget applique déjà le refus sûr attendu.
