# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/permissions-doc-clarification-haiku-r5.md](../../../../../../../specs/149-sous-agents-lineage/validation/permissions-doc-clarification-haiku-r5.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Document | Passage avant | Passage après | Référence |
|---|---|---|---|
| `contracts/permissions.md`, lignes 43-46 | Un fait retiré, révoqué ou tourné n'est pas listé. « Version1 préserve identité, status/cancel et lecture historiques148 » sans limite. | Le refus couvre le fait « retiré (fin, échec ou fermeture du tour), révoqué ou tourné ». La v1 n'est préservée que « pour un credential sans fait jamais existé ». | G-P-07(b), R8.4 |
| `contracts/permissions.md`, lignes 441-455, branche (b) | « reçoit un refus nommé permission_attestation_unavailable ; jamais l'enveloppe v1 ; lecture et rejeu inchangés ». | Côté T3, refus nommé `permission_attestation_unavailable`. Côté Rust, la façade MCP ferme chaque appel avant effet avec `t3_session_unavailable` (`bridget_delegate`, `bridget_task_status`, `bridget_task_cancel`, `bridget_who`). Pas de repli v1, discovery ou PID. Aucun lancement. « Lecture et rejeu inchangés » est précisé en (i) et (ii). | Section 5 de la revue, R8.4, S5.1, S5.1b |
| `test-strategy.md`, ligne 189 (S149-32, branche b) | Même formulation que le contrat. | Même texte que le contrat. Ajout : les preuves existantes ne valent pas exécution de S149-32. | Section 5 de la revue |
| `test-strategy.md`, ligne 15 (en-tête) | Aucune entrée pour ce point. | Entrée datée « Clarification 2026-10-10 ». Le total reste 33 scénarios, aucun exécuté. | Historique des amendements |

<a id="extrait-2"></a>

## Extrait 2

1. `contracts/permissions.md`, ligne 30 : « Le moteur admis continue si T3 devient indisponible. L'annulation reste native. » Ce texte parle de T3 indisponible, pas d'un credential retiré. Il peut pourtant se lire comme une permission d'annuler via le credential retiré. Proposition : écrire « l'annulation humaine native ».
2. `contracts/permissions.md`, ligne 481 (ancienne 467-468) : « Les réponses identité148 version1 restent recevables pour les lectures historiques. » Si on lit « toute v1 reçue reste acceptée », cela contredit la branche (b). Proposition : écrire « les réponses version1 déjà stockées ».
3. `validation/recovery149.md`, section O1, lignes 88-89 : la proposition de clarification diffère du wording de la section 5. Elle ne distingue pas (i) et (ii). Ce fichier est une preuve. Il n'a pas été modifié. La version retenue est celle de la section 5.
4. Numéros de lignes décalés. `recovery149.md` ligne 74 cite « permissions.md lignes 29 et 40-46 ». La revue cite « lignes 44-46 et 440-448 ». Les nouvelles positions sont : version1 aux lignes 43-46, G-P-07 à la ligne 441, branche (b) à la ligne 443, branche (c) à la ligne 452, clarification aux lignes 462-464. Les références anciennes sont à mettre à jour par le principal si besoin.
