# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/validation/tests164.md](../../../../../../../specs/164-conversation-t3-distante/validation/tests164.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Avant code, inspecter seulement les formes de journal nécessaires. Réduire toute copie de données réelles et vérifier l'absence de secrets avant une revue externe. Privilégier fixtures synthétiques des formes observées.
2. Une campagne finale des trois groupes ciblés ; avant cela seuls retours courts nécessaires à l'implémentation. Après correction, rejouer le groupe affecté, pas les suites inchangées.
3. TypeScript/format/lint du périmètre modifié selon scripts du dépôt, Git diffcheck et self-review. Les contrôles Mac autorisés par ROOT peuvent compléter une toolchain Linux absente. Ne pas utiliser des binairesMac sur Linux.
4. ROOT inspecte un client isolé après intégration. Aucun sous-agent ne lance serveur/browser. La demande UI autorise cette vérification réversible. T3 vivant et ses données restent protégés.
5. Contre-revue code et audit : bornes, confidentialité, lecture seule, complexité, réemploi, surfaces et aucune dépendance nouvelle.

<a id="extrait-2"></a>

## Extrait 2

| Contrôle | Résultat réel | Limite restante |
|---|---|---|
| Vérification initiale du format sur11 chemins | RC1, neuf fichiers à formater | Ce contrôle initial reste historique |
| Format appliqué sur les neuf fichiers | RC0 ; log99ms, commande252ms | Aucun recheck du format ni rejeu des tests après formatage au checkpoint |
| Garde AST normalisée | RC1 ; six fichiers égaux, trois différents ; zéro erreur parseur avant/après | Caractériser les trois différences avant copie des sources ; ne pas les qualifier de simples espaces sans preuve |
| Lint du périmètre | RC1, chargement configuration bloqué | Plugin workspace @oxlint/plugins non exposé à oxlint-plugin-t3code ; ce résultat n’est pas un lint du code achevé |
| TypeScript client-runtime | RC1, une erreurTS2322 de fixture à la ligne22 : unknown n’est pas Json ; suggestions Effect également présentes | Annotation de payload corrigée en type wire BridgetJournalEvent["payload"] selon ROOT ; vingt charges de fixture validées JSON par gardeRC0, mais TypeScript non rejoué |
| TypeScript web | RC1, dix diagnostics | Log entier byte-identique au log163r1 ; correction des deux fixtures protégées déjà prouvée par163r2. Aucun PASS TypeScript164 intégré n’en découle |
| TypeScript mobile | RC0, log vide,6.676s | Sur snapshot contrôlé seulement ; ne prouve pas thème, sélection et rendu natif réel |

<a id="extrait-3"></a>

## Extrait 3

- /Users/moi/.cache/bridget-delivery163-20261013/t3-typecheck-web163-r1.log — SHA25609b78c3a62f183902acc04b49f086fac4f3ba5fafefa7cb26290a6c3f9c5845e ; identique byte par byte à types-web164-r1.log.
- /Users/moi/.cache/bridget-delivery163-20261013/t3-fixtures163-r2.json — SHA256ce697573dd3866f5e736e847c97e7cdce4e56d79d570c3814029cd6b908c5a36 ; typecheck web163r2 RC0 après correction de providerThreadId:null dans les deux fixtures MessagesTimeline. Ce reçu ne valide pas les nouvelles sources164.
- /Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-attestation.json — autorité des copies et limites. Les fixtures MessagesTimeline protégées ne doivent pas être modifiées par164.

<a id="extrait-4"></a>

## Extrait 4

La liste interop de `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts` ajoute uniquement le conteneur mobile, car les props Markdown natives ne consomment pas `className`. La règle garde `error` avec `allowUniwindTheme: true` dans la liste existante. Aucun changement global de règle.

<a id="extrait-5"></a>

## Extrait 5

**Converge passage1 NON CONVERGED, T021 ajoutée, In Progress2/21.** Les détails originaux et témoins masquent des métadonnées v/ts sur web/mobile ; les36PASS existants ne couvrent pas cette ouverture. Une correction ciblée et les deux bancs Journal149 existants sont requis. Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p1.md`. Aucun contrôle relancé par le responsable documentaire ni source modifiée. Contre-revue/audit, S164-12/T017, Git/paquet/install/cleanup/runtime restent futurs. Aucune case cochée.

<a id="extrait-6"></a>

## Extrait 6

Rapport `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p2.md`. Prochaine gate T014 contre-revue externe/audit, puis T015–T020 source163/Git/paquet/recette/install/cleanup. Recette web/mobile/native, livraison et runtime restent futurs. Aucun test/service/Git/code relancé ou modifié par cette lecture documentaire.

<a id="extrait-7"></a>

## Extrait 7

Mission corrective `3b3905be-3214-44bd-b160-37ab4e5edc9f`, ronde6, six fichiers déjà dans le scope. Code seulement, sans contrôles du développeur tant que163R8 réserve le permis. Aucun retour stable, hash final, test, P3 ou auditR6 présumé. Trois bancs existants affectés après permis ROOT ; pas de suite globale ou outil nouveau. Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r1.md`. T014 complète, recette et livraison restent ouvertes ; aucune case cochée.

<a id="extrait-8"></a>

## Extrait 8

Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md`. P1/P2 ettasks byte-identiques pendantP3 ; casesROOTtoujours2/22. Contre-revueR2/deltaaudit/T014complète puis Git/QA/paquet/installG8G9 futurs. Aucun succès produit/runtime déduit.

<a id="extrait-9"></a>

## Extrait 9

**P3 CONVERGED sur la source ; R2 APPROVE.** F1/F2/F3 sont fermées. Le compteur représente des occurrences, pas des versions uniques. F4 est accepté et non relu. Rapport et limites : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`.

<a id="extrait-10"></a>

## Extrait 10

Audit incrémental de six sources,309lignes lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01`. GradeA100 ciblé,0finding actif ; validateur canonique RC0,0erreur/0warning. AuditR5b, P1/P2 et tasks conservés byte-identiques. Aucun nouveau test ni contrôle source relancé. Les28PASS et trois TypeScript RC0 proviennent du manifeste finalR6, pas de la lecture documentaire.

<a id="extrait-11"></a>

## Extrait 11

ROOT accepte la recette du serveur et du client web réellement construits sur source511190beb307. Le daemon est remplacé par la fixture CLI synthétique existante. Le serveur isolé utilise son propre dossier de données ; aucune base active T3 ni session utilisateur n'est modifiée. Le reçu final ci-dessous complète l'observation historique de la page suivante ; cette première observation incomplète reste conservée.

<a id="extrait-12"></a>

## Extrait 12

T3 : source `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée vers le fork/local/main-20261009. Application signée `/Applications/T3 Code (Local).app` installée en0.0.45-local.164 ; PID704/969 et naissances inchangés. Code162 encore chargé, aucun redémarrage ni activation différée. Bridget demeure natif Bridget.

<a id="extrait-13"></a>

## Extrait 13

Nettoyage T3 WT164 et branche attesté : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt164-fresh-stable-retirement.json`, SHA-256 `b0661c19c7243969d90035dfd8a9de062e2a823e5ae2183c78ac8fa6553f6b1b` ; retrait sans force après source intégrée/poussée et garde lsof complète,321991571octets logiques. Les anciens chemins WT164 sont historiques. G9 final : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json`, SHA-256 `08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3`. Onze ressources retirées : trois worktrees, deux stages Mac et six namespaces TMP ; trois branches retirées. Le WT T3 de163 et sa branche sont conservés : inventaire complet282139fichiers réussi, puis inventaire exact changé après archive. La garde refuse le retrait ; aucune cause de changement inventée, aucun contournement.

<a id="extrait-14"></a>

## Extrait 14

Tests sources :28PASSR6 distincts des10réducteurR4 et8MarkdownR1 conservés, soit46preuves de tests qualifiées par provenance, pas une nouvelle campagne46 ni un secondFULL. L'échec historique de rejeu/réalignement reste historique ; sa correction est couverte par l'oracleR6, aucun replay supplémentaire déduit de ce texte. Paquet :2574PASS/2ZIPFAILR1 conservés puis deux seuls oracles ZIP PASSR2 après réarchivage ; pas de nouveauGREEN2576. P1/P2/P3, contre-revues et audits ciblés restent inchangés ; aucun succès global/runtime de T3 inventé.

<a id="extrait-15"></a>

## Extrait 15

Les caches `/Users/moi/.cache/t3-final162.2nosz6zd` et `/Users/moi/.cache/t3-final163.zsb7fmfu` sont conservés pour leurs bases SQLite persistantes réelles,20480octets chacune, aux chemins `/Users/moi/.cache/t3-final162.2nosz6zd/source/node_modules/.vite/task-cache/v18/cache.db` et `/Users/moi/.cache/t3-final163.zsb7fmfu/source/node_modules/.vite/task-cache/v18/cache.db`. Ces dépendances ne sont pas conservées sous prétexte d'un futur build163. L'ancien échange `/Applications/.T3 Code (Local).app.new-163` reste utilisé par les processus T3 protégés ; le rollback et les preuves restent conservés. Cache Cargo partagé `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug` conservé, permis lourd rendu ; aucun nouveau FULL ou build163 en attente prétendu. Les trois namespaces SQLite `/tmp/b163.uqd3443p`, `/tmp/b163.dxmfq_la` et `/tmp/b163.07di472q` sont protégés par leurs attestations historiques ; leur existence actuelle n'est pas revendiquée. Aucun gain physique ni modification DB de production revendiqué.

<a id="extrait-16"></a>

## Extrait 16

Le build actif Bridget est toujours source `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` ; le commit de métadonnéesc57 est distinct et ne prétend pas une nouvelle compilation. Source T3 `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée au fork/local/main-20261009, application164 signée sur disque. T3PID704/969 reste chargé en162, sans redémarrage ni activation différée.
