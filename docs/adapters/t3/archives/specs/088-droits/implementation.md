# Archives techniques de l’adaptateur T3

Document d’origine : [specs/088-droits/implementation.md](../../../../../../specs/088-droits/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Lot 3 — 2026-09-04 : borne à l'écriture et fil virtualisé (après lecture du dépôt t3code)
Motif : le blocage au changement d'agent venait d'une carte de reprise de 1 034 232 caractères, produite par le wrapper lors d'une relance. Comparaison faite dans `~/11.Repositories/t3code` : leur contrat refuse un tour dont le texte dépasse 120 000 caractères (`packages/contracts/src/provider.ts`, `PROVIDER_SEND_TURN_MAX_INPUT_CHARS`), leur lanceur de processus plafonne la sortie à 8 Mio (`apps/server/src/processRunner.ts`), et leur liste de messages est virtualisée (LegendList, `apps/web/src/components/ChatView.tsx`). Chez nous, aucune des trois bornes n'existait sur ce chemin ; le ledger, lui, était sain (2 805 messages, maximum mesuré 9 167 caractères).
