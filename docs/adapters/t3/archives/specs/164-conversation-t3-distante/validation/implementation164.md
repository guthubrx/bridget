# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/validation/implementation164.md](../../../../../../../specs/164-conversation-t3-distante/validation/implementation164.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Mission code neuve : `3182cc37-7204-4769-9439-644b813a23c1` ; enfant `a9a25c7b-b021-41d7-a200-46006d257dcc`. Le reçu initial indique starting/working, pas un code terminé. Le développeur tient les sources T3 dans `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164`. Les documents distants ne sont pas resynchronisés pendant son travail.

<a id="extrait-2"></a>

## Extrait 2

T015 attend la **source T3 de163 validée, intégrée et poussée**, puis les gates source164 T013/T014. Elle n’attend pas G8/G9 du produit Bridget163. Le paquet commun163+164 est possible seulement après validation des sources164 ; les preuves et gates produit restent séparées. Si164 n’est pas prête, un paquet163 seul reste possible. Cette précision ne change ni les vingt responsabilités ni les exigences fonctionnelles.

<a id="extrait-3"></a>

## Extrait 3

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/plan.md : avant `5da953eca4309b0d8aa4a33cdaa70b1ac8f930f51b9c856dbea1e0459f9810f5`, après `6d744c60359e3e0f9f1859364f27116f79ef35a36185179912acce8600cea9cb`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/tasks.md : avant `155d2da50a7dd20a10a57777a144df0b12297ad7d1cd9a61f6abd385321be311`, après `3cfd71dc3d3288603bfa7445973942350d702db2f5722c0c6e2547026cb1318e`.

<a id="extrait-4"></a>

## Extrait 4

| Contrôle | Résultat réel | Limite restante |
|---|---|---|
| Vérification initiale du format sur11 chemins | RC1, neuf fichiers à formater | Ce contrôle initial reste historique |
| Format appliqué sur les neuf fichiers | RC0 ; log99ms, commande252ms | Aucun recheck du format ni rejeu des tests après formatage au checkpoint |
| Garde AST normalisée | RC1 ; six fichiers égaux, trois différents ; zéro erreur parseur avant/après | Caractériser les trois différences avant copie des sources ; ne pas les qualifier de simples espaces sans preuve |
| Lint du périmètre | RC1, chargement configuration bloqué | Plugin workspace @oxlint/plugins non exposé à oxlint-plugin-t3code ; ce résultat n’est pas un lint du code achevé |
| TypeScript client-runtime | RC1, une erreurTS2322 de fixture à la ligne22 : unknown n’est pas Json ; suggestions Effect également présentes | Annotation de payload corrigée en type wire BridgetJournalEvent["payload"] selon ROOT ; vingt charges de fixture validées JSON par gardeRC0, mais TypeScript non rejoué |
| TypeScript web | RC1, dix diagnostics | Log entier byte-identique au log163r1 ; correction des deux fixtures protégées déjà prouvée par163r2. Aucun PASS TypeScript164 intégré n’en découle |
| TypeScript mobile | RC0, log vide,6.676s | Sur snapshot contrôlé seulement ; ne prouve pas thème, sélection et rendu natif réel |

<a id="extrait-5"></a>

## Extrait 5

- /Users/moi/.cache/bridget-delivery163-20261013/t3-typecheck-web163-r1.log — SHA25609b78c3a62f183902acc04b49f086fac4f3ba5fafefa7cb26290a6c3f9c5845e ; identique byte par byte à types-web164-r1.log.
- /Users/moi/.cache/bridget-delivery163-20261013/t3-fixtures163-r2.json — SHA256ce697573dd3866f5e736e847c97e7cdce4e56d79d570c3814029cd6b908c5a36 ; typecheck web163r2 RC0 après correction de providerThreadId:null dans les deux fixtures MessagesTimeline. Ce reçu ne valide pas les nouvelles sources164.
- /Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-attestation.json — autorité des copies et limites. Les fixtures MessagesTimeline protégées ne doivent pas être modifiées par164.

<a id="extrait-6"></a>

## Extrait 6

La liste interop de `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts` ajoute uniquement le conteneur mobile, car les props Markdown natives ne consomment pas `className`. La règle garde `error` avec `allowUniwindTheme: true` dans la liste existante. Aucun changement global de règle.

<a id="extrait-7"></a>

## Extrait 7

**Converge passage1 NON CONVERGED, T021 ajoutée, In Progress2/21.** Les détails originaux et témoins masquent des métadonnées v/ts sur web/mobile ; les36PASS existants ne couvrent pas cette ouverture. Une correction ciblée et les deux bancs Journal149 existants sont requis. Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p1.md`. Aucun contrôle relancé par le responsable documentaire ni source modifiée. Contre-revue/audit, S164-12/T017, Git/paquet/install/cleanup/runtime restent futurs. Aucune case cochée.

<a id="extrait-8"></a>

## Extrait 8

Rapport `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p2.md`. Prochaine gate T014 contre-revue externe/audit, puis T015–T020 source163/Git/paquet/recette/install/cleanup. Recette web/mobile/native, livraison et runtime restent futurs. Aucun test/service/Git/code relancé ou modifié par cette lecture documentaire.

<a id="extrait-9"></a>

## Extrait 9

Mission corrective `3b3905be-3214-44bd-b160-37ab4e5edc9f`, ronde6, six fichiers déjà dans le scope. Code seulement, sans contrôles du développeur tant que163R8 réserve le permis. Aucun retour stable, hash final, test, P3 ou auditR6 présumé. Trois bancs existants affectés après permis ROOT ; pas de suite globale ou outil nouveau. Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r1.md`. T014 complète, recette et livraison restent ouvertes ; aucune case cochée.

<a id="extrait-10"></a>

## Extrait 10

Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md`. P1/P2 ettasks byte-identiques pendantP3 ; casesROOTtoujours2/22. Contre-revueR2/deltaaudit/T014complète puis Git/QA/paquet/installG8G9 futurs. Aucun succès produit/runtime déduit.

<a id="extrait-11"></a>

## Extrait 11

**P3 CONVERGED sur la source ; R2 APPROVE.** F1/F2/F3 sont fermées. Le compteur représente des occurrences, pas des versions uniques. F4 est accepté et non relu. Rapport et limites : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`.

<a id="extrait-12"></a>

## Extrait 12

Audit incrémental de six sources,309lignes lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01`. GradeA100 ciblé,0finding actif ; validateur canonique RC0,0erreur/0warning. AuditR5b, P1/P2 et tasks conservés byte-identiques. Aucun nouveau test ni contrôle source relancé. Les28PASS et trois TypeScript RC0 proviennent du manifeste finalR6, pas de la lecture documentaire.

<a id="extrait-13"></a>

## Extrait 13

Les reçus restent historiques et byte-identiques : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md` décrit le compteur2/22 au moment de sa lecture ; il ne sert pas de compteur courant. Preuves de source finale : `/Users/moi/.cache/bridget-delivery164-20261013/tests/validation164-r6-final.json` et `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r6-final.json`. Contre-revue : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`. Audit : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01/validator-receipt.json`.

<a id="extrait-14"></a>

## Extrait 14

**In Progress,19/22.** ROOT accepte les trois preuves réelles : intégration/push source T3, paquet signé et recette S164-12 du client isolé. T015/T016/T017 sont cochées. T018 attend la preuve G8 indépendante de ROOT ; T019/G9 et T020 restent ouverts. L'installation sur disque ne prouve pas l'activation du code dans les processus T3.

<a id="extrait-15"></a>

## Extrait 15

Source T3 intégrée/poussée : `511190beb307e0952066fe1a4b511cb9dd7adb18`, commit source164 `966f296ac70c3134bc6e91a722cfe86e726f50e3`, fork `refs/heads/local/main-20261009`. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/git-t3-source164-pushed.json`, SHA-256 `e7685a463e56580b511caa1afb76eb1fa6d7c53612025122506088be677c0259`.

<a id="extrait-16"></a>

## Extrait 16

Paquet commun163/164 version0.0.45-local.164 signé, une compilation native. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/t3-package164-result.json`, SHA-256 `2fb1b8f95140f09676d107f2429be07322c7ff77ff094993f1a82f95dff18984`. R1 conserve2574PASS/2FAIL des oracles ZIP ; R2 rejoue seulement les deux oracles affectés,2PASS/0FAIL. Aucun nouveau succès global2576 annoncé.

<a id="extrait-17"></a>

## Extrait 17

Recette S164-12 : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/t017164-final.json`, SHA-256 `fcf03a0e7f86ca30b13497df82a0fd9b512232e5db516eb0bd11f22fb9d47f6f`. Web réel issu du paquet final, données synthétiques, sept captures. Archive66membres vérifiés : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz`, SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. Viewports1280×900 et390×844 ; le second reste un rendu web mobile, pas une recette React Native. Aucun redémarrage Electron/T3, modèle ou agent réel utilisé dans la recette.

<a id="extrait-18"></a>

## Extrait 18

Installation signée sur disque : `/Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json`, SHA-256 `b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0`. Application `/Applications/T3 Code (Local).app` en0.0.45-local.164 ; app.asar SHA-256 `6eb2fbdfed44a603d0a66d81d92fadd6d3486ff1ff43a880fe788c7d39fc923f`. PID704/969 et dates de naissance inchangés. Le code162 reste chargé selon le checkpoint ROOT. Pas d'activation différée ni de faux handshake G8.

<a id="extrait-19"></a>

## Extrait 19

T3 : source `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée vers le fork/local/main-20261009. Application signée `/Applications/T3 Code (Local).app` installée en0.0.45-local.164 ; PID704/969 et naissances inchangés. Code162 encore chargé, aucun redémarrage ni activation différée. Bridget demeure natif Bridget.

<a id="extrait-20"></a>

## Extrait 20

Nettoyage T3 WT164 et branche attesté : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt164-fresh-stable-retirement.json`, SHA-256 `b0661c19c7243969d90035dfd8a9de062e2a823e5ae2183c78ac8fa6553f6b1b` ; retrait sans force après source intégrée/poussée et garde lsof complète,321991571octets logiques. Les anciens chemins WT164 sont historiques. G9 final : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json`, SHA-256 `08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3`. Onze ressources retirées : trois worktrees, deux stages Mac et six namespaces TMP ; trois branches retirées. Le WT T3 de163 et sa branche sont conservés : inventaire complet282139fichiers réussi, puis inventaire exact changé après archive. La garde refuse le retrait ; aucune cause de changement inventée, aucun contournement.

<a id="extrait-21"></a>

## Extrait 21

Tests sources :28PASSR6 distincts des10réducteurR4 et8MarkdownR1 conservés, soit46preuves de tests qualifiées par provenance, pas une nouvelle campagne46 ni un secondFULL. L'échec historique de rejeu/réalignement reste historique ; sa correction est couverte par l'oracleR6, aucun replay supplémentaire déduit de ce texte. Paquet :2574PASS/2ZIPFAILR1 conservés puis deux seuls oracles ZIP PASSR2 après réarchivage ; pas de nouveauGREEN2576. P1/P2/P3, contre-revues et audits ciblés restent inchangés ; aucun succès global/runtime de T3 inventé.

<a id="extrait-22"></a>

## Extrait 22

Les caches `/Users/moi/.cache/t3-final162.2nosz6zd` et `/Users/moi/.cache/t3-final163.zsb7fmfu` sont conservés pour leurs bases SQLite persistantes réelles,20480octets chacune, aux chemins `/Users/moi/.cache/t3-final162.2nosz6zd/source/node_modules/.vite/task-cache/v18/cache.db` et `/Users/moi/.cache/t3-final163.zsb7fmfu/source/node_modules/.vite/task-cache/v18/cache.db`. Ces dépendances ne sont pas conservées sous prétexte d'un futur build163. L'ancien échange `/Applications/.T3 Code (Local).app.new-163` reste utilisé par les processus T3 protégés ; le rollback et les preuves restent conservés. Cache Cargo partagé `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug` conservé, permis lourd rendu ; aucun nouveau FULL ou build163 en attente prétendu. Les trois namespaces SQLite `/tmp/b163.uqd3443p`, `/tmp/b163.dxmfq_la` et `/tmp/b163.07di472q` sont protégés par leurs attestations historiques ; leur existence actuelle n'est pas revendiquée. Aucun gain physique ni modification DB de production revendiqué.

<a id="extrait-23"></a>

## Extrait 23

Le build actif Bridget est toujours source `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` ; le commit de métadonnéesc57 est distinct et ne prétend pas une nouvelle compilation. Source T3 `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée au fork/local/main-20261009, application164 signée sur disque. T3PID704/969 reste chargé en162, sans redémarrage ni activation différée.
