# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/analysis.md](../../../../../../specs/163-decision-admission/analysis.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| ID | Catégorie | Sévérité | Constat | Correction autorisée ROOT |
|---|---|---|---|---|
| I1 | Cohérence des statuts | MEDIUM | Plan Ready/gate ROOT soldé, mais des paragraphes de checklist/research/reuse/ADR/reçu R1 gardent Draft ou relecture attendue. | Corriger les seuls statuts/gates historiques contradictoires, conserver verdict externe R1 APPROVE_WITH_CHANGES. |
| I2 | Cohérence de progression | LOW | Spec annonce 0/0 et tasks absentes alors que la génération produit 45 tâches. | Mettre 0/45 et artefact tasks présent, sans cocher aucune tâche future. |
| D1 | Ordre de responsabilités | MEDIUM | Les chemins T3/Cartae sont prévus, mais non attestés ; une entrée utilisateur ne crée pas le binding de travail. | T001–T004 rendent attestations et wire figé bloquants ; T031 attend T030/binding, aucune existence supposée. |

<a id="extrait-2"></a>

## Extrait 2

Couverture sémantique : configuration/axes, profils et droits, probes réels sans effets, HTTP et permis, reçus et crash, refus/purge, collision, entretien sans demandeur, interfaces versionnées, T3 et livraison sont présents avec résultats observables. O1–O6 sont rattachés à leurs campagnes et non à des tests par minigarde. La transmission wire et les frontières de propriété sont explicites. Correction finale d’ordre relue : T036 Converge et campagnes passées → T037 contre-revue/audit → T038 checkpoint source validé/build Mac/signature → T039. Si le build Mac révèle une correction, contrôles affectés et delta de revue/audit précèdent le commit final, sans FULL répétée. Les textes de configuration et modèle détaillent le comportement ; les tâches nomment le lieu d’implémentation et sa preuve.

<a id="extrait-3"></a>

## Extrait 3

Constitution locale pont et constitution globale1.9.0 relues : français, ADR, pipeline, isolation, complexité, minimalisme et responsabilité future sont couverts. Les prescriptions de validation répétée par tâche de l’Article XV sont adaptées à l’instruction utilisateur persistante : tests prévus groupés et pas de répétition longue inutile. Chaque tâche n’est soldée que lorsque ses contrôles utiles sont prouvés ; ROOT possède commits/intégration, les développeurs n’écrivent pas Git. Les travaux simultanés n’éditent pas la même surface sans arbitrage ; compilation unique à l’échelle flotte. Les 17 worktrees T3 étrangers historiques ne sont pas un motif de purge du lot.

<a id="extrait-4"></a>

## Extrait 4

Le checkpoint P3 NONCONVERGED et l’exception autorisée de poursuite en phase 5 sont conservés : le mandat utilisateur de terminer prime sur l’arrêt borné qui laisserait l’objectif inachevé. Cette adaptation ne déclare aucun P4 formel ou CONVERGED nouveau. L’analyse documentaire présente ne ferme pas T036. La contre-revue CODE Claude R2, l’audit, les builds de livraison/paquet, la recette T040, Git/push, installation, G8/G9 et clôture restent à attester. Aucun nouveau runtime T3 annoncé ; aucune case cochée.

<a id="extrait-5"></a>

## Extrait 5

Résultat : zéro finding documentaire CRITICAL restant dans les sept documents attribués après ces corrections ; aucune case, exigence, tâche ou campagne ajoutée. La fausse clôture SQL R7 reste un défaut de code ouvert : R8 préparatoire sur deux sources, 22 sources à attester, discriminant Git et liste métier fermée à prouver avant acceptation T057. Audit final, clôture Converge, T040, builds, livraison et G8/G9 restent futurs. Aucun PASS R8, correction SQL ou nouveau runtime T3 annoncé. Contrats et checklist ROOT n’ont pas été édités.

<a id="extrait-6"></a>

## Extrait 6

31/31 couverture documentaire et 4/57 inchangés, aucun cochage. T033 quatre contrôles nominativement prouvés, sans extrapolation vers nouveau runtime T3. P3 NONCONVERGED historique/phase 5 corrective autorisée conservés, aucun P4 formel. Audit final/delta R9, builds, recette T040, Git, G8/G9 et livraison restent ouverts. T039 prépare quatre profils off/on moteur/placement sans appliquer la configuration ; quota CLI à 100 % et nouveaux children ne prouvent pas l’usage opérationnel de ces profils en production.

<a id="extrait-7"></a>

## Extrait 7

Passe 2 : 22 FR/neuf SC inchangés, cinq US, 57 IDs uniques, quatre cases initiales conservées et table45 exhaustive. T033 quatre contrôles R8 et CLI2 R2 ont chacun leurs logs/limites de provenance ; aucune nouvelle réussite de runtime T3. Trois replays R9 encore attendus, aucun PASS futur. Zéro CRITICAL documentaire restant identifié dans les sept fichiers attribués ; la réserve de code E/S demeure ouverte avant acceptation T057 et ne disparaît pas par Analyze. Contrats/checklist/audits ROOT non édités ; aucun code, test, Git ou service exécuté ici.

<a id="extrait-8"></a>

## Extrait 8

Seconde lecture : 31 exigences inchangées et toujours mappées, cinq US, 57 IDs uniques, seules T001–T004 cochées et 45 lignes d’acceptation exhaustives. Périmètre R9 final23/deux changements/21 inchangées ; oracle EIO typé via code de production qualifié sans syscall OS forcée. Zéro CRITICAL documentaire restant identifié dans les sept documents attribués. R4/audit et acceptation ROOT restent ouverts ; aucun cochage, nouveau runtime T3, Converge/P4 formel, G8/G9 ou succès de livraison déduit. Aucun source, test, build, Git ou service exécuté par cette analyse.

<a id="extrait-9"></a>

## Extrait 9

Passe 2 : 22 FR/neuf SC et cinq US inchangés, 57 IDs uniques, table45 exhaustive, seules T001–T004 cochées. Le PASS tardif/reprise ne prouve ni syscall EIO ni recette POST/tokenizer de production. 49/57 est une acceptation ROOT préparée mais encore future ; 4/57 reste courant avant grade/validator final. Aucun CRITICAL documentaire restant identifié ; aucun nouveau runtime T3, Mac PASS, GREEN global, CVE, P4 formel, build ou G8/G9 inventé.

<a id="extrait-10"></a>

## Extrait 10

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

<a id="extrait-11"></a>

## Extrait 11

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

<a id="extrait-12"></a>

## Extrait 12

Passe 1 : introductions 53/57 et quatre cochages autorisés synchronisés ; application off réelle séparée des futurs axes/G8/G9. Ordre Git avant T040 expliqué par checkpoint déterministe préactivation. Passe 2 : 57 ID uniques, exactement53 cases ; seules T039/T043/T044/T045 ouvertes, FR/SC inchangés et table45 conservée. RC1 T040 et captures de production historiques qualifiées ; T3 installé sur disque distinct de l’ancien code chargé. Aucun source Git/AGENTS/runtime/test modifié par le rédacteur.

<a id="extrait-13"></a>

## Extrait 13

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

<a id="extrait-14"></a>

## Extrait 14

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

<a id="extrait-15"></a>

## Extrait 15

P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 restent conservés, sans P4 formel inventé. Les anciennes FULL avec échecs, l’audit163 partiel 3M/2L, le delta T058 partiel sans finding et le delta T3 R6 sans finding restent distincts ; aucun GREEN global déduit.
