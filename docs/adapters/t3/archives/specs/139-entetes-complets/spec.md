# Archives techniques de l’adaptateur T3

Document d’origine : [specs/139-entetes-complets/spec.md](../../../../../../specs/139-entetes-complets/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Exigences et succès
FR01 : tous les formats Bridget émis par le pont T3 sont couverts.
FR02 : seul le premier paragraphe de l'enveloppe complète est atténué.
FR03 : texte, copie, historique, transport et relances inchangés.
FR04 : tests ciblés et aperçu isolé ; aucune écriture dans les données actives.
FR05 : arrêter proprement l'application avant remplacement ; livrer un paquet
traçable avec sauvegarde, contrôle santé et retour arrière. Ne pas modifier les données.
Succès : tests positifs/négatifs PASS, mesures visuelles et archive vérifiées.
