# Archives techniques de l’adaptateur T3

Document d’origine : [specs/176-observation-bornee/research.md](../../../../../../specs/176-observation-bornee/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T3 lu : GitVcsDriverCore.ts executeRaw lignes725–860 borne timeoutMs et maxOutputBytes, collecte les flux et expose les troncatures ; mécanisme repris. removeWorktree lignes3113–3158 borne le retrait et garde stderr hors fil. worktreeCleanup.ts vérifie si d'autres fils partagent un chemin ; il ne fournit aucune inspection de processus Linux. Le contrôle de processus reste donc propre à Bridget. Aucun nettoyage forcé T3 repris.

<a id="extrait-2"></a>

## Extrait 2

T3 : executeRaw borne les flux et le délai, principe repris. T3 n'a pas de scanner de processus Linux équivalent. Son retrait explicite après confirmation ne justifie pas l'assouplissement du retrait automatique Bridget.
