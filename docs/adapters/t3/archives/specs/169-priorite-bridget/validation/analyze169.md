# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/validation/analyze169.md](../../../../../../../specs/169-priorite-bridget/validation/analyze169.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le handler et le banc sont réutilisés : /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/http.ts:166 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/boundedSnapshotTransport.test.ts:235 (`withHttp`). La casse `http.ts` est requise sur Linux. Aucun Rust, helper produit, endpoint, RPC ou contrat public nouveau n'est demandé.

<a id="extrait-2"></a>

## Extrait 2

| Exigence | Tâches ou jalon | Preuve attendue |
|---|---|---|
| FR001 | T001, T005 | Préférence par fichier, sans UI. |
| FR002 | T001, T005, J4–J5 | Absence OFF, ON cible explicite. |
| FR003 | T001, T005 | Cache fiable, refus avant secrets/écriture. |
| FR004 | T002–T005, T007–T008 | Mode courant avant chaque tour. |
| FR005 | T003, T006–T008 | Exception T3 et outils hors délégation. |
| FR006 | T003, T006 | Contraintes explicites conservées. |
| FR007 | T003, T006 | Choix et exécution gardés par Bridget. |
| FR008 | T002, T004, T006, T009 | Identité et droits inchangés. |
| FR009 | T003–T004, T007–T009 | Blocage explicite, aucun repli d'hôte. |
| FR010 | T009–T010 | Rejeu et suivi sans doublon. |
| FR011 | T005, T010 | Tâches en cours gardées sur leur moteur. |
| FR012 | T010 | Projection Lineage et lectures sans effet. |
| FR013 | T006, T010 | Autonomie et résultat natifs conservés. |
| FR014 | T002–T004, T007–T009 | Six raccords avec limites explicites. |
| FR015 | T001–T003, T006, T008, T011 | Source unique et mécanismes bornés. |
| FR016 | T002–T009, T011–T012 | Périmètre minimal et correctif causal HTTP. |
| FR017 | T012, J3–J5 | Installation, activation et reprise réelles. |

<a id="extrait-3"></a>

## Extrait 3

| Critère | Tâches ou jalon | Preuve attendue |
|---|---|---|
| SC001 | T002–T005, T007–T008 | Matrice des modes et raccords. |
| SC002 | T001, T005 | Erreur explicite et fichier byte-identique. |
| SC003 | T003–T004, T006, T009, J5 | Tests ciblés puis recette réelle et compteurs séparés. |
| SC004 | T002, T004–T006, T009 | Contraintes, droits et identité. |
| SC005 | T003–T004, T007–T010 | Refus/rejeu sans secours ou doublon. |
| SC006 | T002, T004–T005, T010, T012, J5 | Transitions, suivi et reprise du gros fil. |
| SC007 | T003–T004, T006–T008 | Exception T3 et commandes natives intactes. |
| SC008 | T001, T005–T006, T008, T011 | Aucun mécanisme public ou stockage supplémentaire. |
| SC009 | T012, J3–J5 | Source installée/chargée, ON et reprise prouvés. |

<a id="extrait-4"></a>

## Extrait 4

Les jalons de livraison restent séparés pour éviter un commit circulaire. Le correctif HTTP réutilise handler et banc existants ; la commande Pi reste la seule création privée arbitrée. Les contrôles T3 restent ciblés. Pi réel demeure non attesté, binaire hors PATH ; seuls contrat/fixtures sont prévus, sans installation de fournisseur.

<a id="extrait-5"></a>

## Extrait 5

Relecture manuelle des artefacts et des preuves d’exploitation :14/14 cases inchangées,17FR/9SC reliés aux preuves existantes, FR017/SC003 réel/SC009 couverts par J5. Les deux corrections Analyze restent closes ; aucun manque nouveau ni changement de code. Source T3 58ddcf3a16205a24f940164cba500089f49ec6c0 inchangée ; P2 reste le dernier Converge. Scripts projet absents : même substitut manuel documenté, sans script simulé. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/validation/livraison169.md.
