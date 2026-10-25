# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/plan.md](../../../../../../specs/163-decision-admission/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Au point commun finalise_auto_choice, une nouvelle admission AutoDelegate avec auto_engine=false et auto_placement=false, dont le tuple de profil fixe réel est connu et validé, conserve le mode Explicit. Un choix de repli reste Fallback. Hors repli, si au moins un axe est automatique, le mode reste Automatic, même avec un seul candidat. Le parcours Delegate162 historique conserve decision:null lorsque son profil n’est pas connu. Le parcours configuré R3 met en œuvre cette règle selon le journal et les contrôles ciblés indépendamment attestés. La lecture du consommateur T3 confirme les modes ; aucune recette du runtime T3 ni validation de livraison déduite.

<a id="extrait-2"></a>

## Extrait 2

| Fichier ou surface | Travail prévu |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/registry.rs | Politique et services optionnels stricts, bornes et références. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation.rs | Reçu auto et transaction d’idempotence dans le store existant. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs | Préfiltrage, admission auto hors mutex, raccord au puits existant. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/remote_worker.rs | Lecture groupée sans effet, extraction de gardes déjà exécutées par prepare. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs | Opération/capacités négociées et probe fermé. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation_mcp.rs et CLI natif existant | Outil bridget_auto_delegate et nouvelle sous-commande auto-delegate, même contrat natif, validation stricte et refus legacy. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/decision_service.rs | Nouveau module proposé, limité aux bornes HTTP et à la normalisation du serveur ; frontière réseau justifiant l’isolation. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/lineage_protocol.rs et lecteur T3 existant | Projection négociée du seul résumé de décision. |

<a id="extrait-3"></a>

## Extrait 3

Les chemins T3 finaux seront repris du worktree de livraison162 canonisé ; aucun dossier étranger modifié. La liste précise de tests reprend les harnesses existants après audit, sans création d’un banc autonome.

<a id="extrait-4"></a>

## Extrait 4

1. Canonisation effectuée après livraison complète 162 ; vérifier gouvernance et configuration réelle sans secrets. Audit-existing PASS et contre-revue R1 traités ; GO ROOT acquis avant génération des tâches.
2. Configuration et contrats fermés ; tests de combinaisons, refus legacy, corps et listes aux bornes.
3. Candidats local/distant en lecture seule ; vrais chemins et droits, preuve zéro worktree, processus fournisseur/bootstrap ou slot pendant probe, révocation avant admission.
4. Reçu idempotent, crashs et concurrence ; observer appels décision, Task et lancement, pas uniquement des mocks.
5. Adaptateur et service indisponible ; serveur HTTP contrôlé pour timeout, santé, permis occupé, réponse tardive, corps excessif et réponse invalide.
6. Raccord explicite162 et projections : profil fixe/repli, reprises sans décision, anciens readers et pages homogènes.
7. Une validation regroupée des tests prévus ; au plus une FULL finale si requise par le gate applicable, jamais par tâche. Nice10, jobs2, threads2, une seule compilation active à la fois dans toute la flotte ; replays des seuls tests affectés après correction. Budget de cache utile repris du reçu162, pas de purge réflexe.
8. Recette réelle du service retenu et de ses bornes ; revue, audit, commit/intégration/push, installation applicable, reçu, nettoyage. G8/G9 séparés des tâches de préparation. T3 non relancé.

<a id="extrait-5"></a>

## Extrait 5

Le développement Rust, les compilations et les tests Linux de 163 sont réalisés sur Cartae. Utiliser un checkout de session et un namespace privé avec umask 077, TMPDIR court et fixtures isolées. Ne jamais toucher le worker actif 471321, son socket ni son état pour ces essais. Le défaut de rattachement du connecteur est hors fonctionnalité 163. ROOT rapporte son rétablissement opérationnel : limite du seul pont passée de 4096 à 8192 et pont seul relancé ; daemon et T3 protégés. Le checkout distant de session existe, branche session-163-decision-admission, base dcfd9bc29140457b1fa016c276f5a48933415a5c propre selon ROOT. Le rédacteur n’a exécuté aucun SSH ; ce checkpoint ne prouve pas encore le lancement de DEV_CARTAE163.

<a id="extrait-6"></a>

## Extrait 6

Les modifications T3, les tests propres à macOS, la signature et l’installation Mac restent sur le Mac. La recette du serveur de décision part du maître Mac : un succès Linux ne vaut pas validation du chemin réseau ni du comportement macOS. Les builds Linux et Mac doivent attester le même commit source et les empreintes des sources utilisées. Aucun redémarrage de T3.

<a id="extrait-7"></a>

## Extrait 7

Checkpoint historique du transfert du 13 octobre 2026 : la préparation du consommateur T3 est achevée ; le gate T030 et l’interopérabilité finaux restent ouverts. Aucun cochage ajouté : 45 tâches, 4/45 prouvées à cette date, G8/G9 ouverts. Aucune nouvelle fonctionnalité, campagne ou pipeline ajouté par ce transfert.

<a id="extrait-8"></a>

## Extrait 8

31/31 couverture documentaire et 4/57 inchangés, aucun cochage. T033 quatre contrôles nominativement prouvés, sans extrapolation vers nouveau runtime T3. P3 NONCONVERGED historique/phase 5 corrective autorisée conservés, aucun P4 formel. Audit final/delta R9, builds, recette T040, Git, G8/G9 et livraison restent ouverts. T039 prépare quatre profils off/on moteur/placement sans appliquer la configuration ; quota CLI à 100 % et nouveaux children ne prouvent pas l’usage opérationnel de ces profils en production.

<a id="extrait-9"></a>

## Extrait 9

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

<a id="extrait-10"></a>

## Extrait 10

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

<a id="extrait-11"></a>

## Extrait 11

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

<a id="extrait-12"></a>

## Extrait 12

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

<a id="extrait-13"></a>

## Extrait 13

L’audit 163 reste A/99,047619 partiel : 0C/0H/3M/2L, couverture 4,655325 % du snapshot et 0,355837 % du contexte global. Le delta T058 reste A/100 partiel 6,749687 %, 0 finding actif, validateur RC0/0 erreur/0 avertissement. Le delta T3 R6 conserve 0 finding actif sur son seul périmètre. Ces audits ne deviennent pas des audits globaux du produit.

<a id="extrait-14"></a>

## Extrait 14

P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 restent conservés, sans P4 formel inventé. Les anciennes FULL avec échecs, l’audit163 partiel 3M/2L, le delta T058 partiel sans finding et le delta T3 R6 sans finding restent distincts ; aucun GREEN global déduit.
