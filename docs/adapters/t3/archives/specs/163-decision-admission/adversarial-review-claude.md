# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/adversarial-review-claude.md](../../../../../../specs/163-decision-admission/adversarial-review-claude.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Objection | Preuve vérifiée par ROOT / choix retenu | Limite conservée |
|---|---|---|
| O1 — fin réelle du travail minreq | Source /Users/moi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/minreq-2.14.1/src/connection.rs:72, :111, :197–207, :351–380. Exclure with_timeout ; la présence de MINREQ_TIMEOUT même vide/invalide interdit activation et réseau du seul service, au démarrage et avant appel. Le daemon et le Delegate explicite continuent. Raison visible ; aucune suppression globale de variable. | DNS, connexion, TLS, écriture ou lecture serveur peuvent retenir le permis indéfiniment. Demandeur deux secondes, suivants repli/refus immédiat, aucun thread nouveau. Une IP littérale ne prouve pas qu’un thread détaché est terminé. HTTPS via même minreq. |
| O2 — reçus orphelins et refus ultérieurs | ROOT retient l’entretien existant comme propriétaire après réconciliation du redémarrage, indépendamment du demandeur : tous reçus d’époque précédente traités par lots/CAS/E/S hors mutex. pending probes figés puis unique/repli autorisé ou decision_interrupted ; decision_started repli/refus ; chosen admission idempotente + lien atomique. | Aucun HTTP de reprise ni nouveau scheduler. Tout refus après chosen terminal auto, même task_limit/pause/retention_saturated. Parent/droits indisponibles : aucun lancement. Erreurs techniques/SQL restent visibles, non résolues et entretenues ; aucune fausse clôture ni purge d’état ambigu. |
| O3 — rétention sous pression | ROOT retient UNION4 096 incluant refus auto. Seuls refus auto terminaux sans Task purgeables sous pression, du plus ancien au plus récent ; tombstone trente jours dans même transaction. Natif et auto tentent purge sûre avant refus au plafond. | Rejeu exact jusqu’à purge, request_expired après. Aucun pending/chosen/ambigu/vivant touché. Collecte existante réemployée ; cinq refus natifs hors compteur et inchangés. |
| O4 — compatibilité Lineage | ROOT confirme v3 → v2 seulement unsupported_version structuré et maintien historique v2 → v1 sur unsupported_version ou invalid_request structurés. | invalid_request en v3 n’autorise aucun repli. Pages de version homogène ; journal et cancel sans fallback. |
| O5 — collision native/auto | ROOT retient vérification du reçu auto avant by_agent_request par l’entrée native externe sous Immediate. Le chemin interne chosen écrit le refus uniquement dans le reçu auto, sans native_refusal sous la même clé. | Clé owner stable, autorité interne non forgeable et deux unicités historiques conservées. Les cinq codes natifs ne sont pas élargis. |
| O6 — orientation Cartae | Nouvelle orientation utilisateur intégrée : Rust/compilation/tests Linux sur Cartae dans namespace privé umask077/TMPDIR court/fixtures isolées ; Mac pour T3/tests OS/signature/installation. | Worker actif 471321/socket/state intacts. Recette décideur depuis maître Mac ; mêmes sources par commit SHA, preuves Linux et Mac distinctes. Une compilation active dans toute la flotte, jobs2/nice10/tests2 ; FULL unique seulement si gate prévu. Connecteur défaillant hors feature mais bloquant pour lancement distant. |

<a id="extrait-2"></a>

## Extrait 2

Le reviewer n’a pas relu T3 Reader/tooltip, CLI/MCP, remote_worker/fixture, les 27 tests Linux ou journal R2, ni la correction DTO santé. Ses conclusions ne sont donc pas des preuves d’exécution ou de compatibilité de ces surfaces. Il a lu les transitions SQL/CAS, la réservation/purge, l’admission/réseau/reprise, le probe maître et la projection v3 ; les garanties vérifiées statiquement restent distinctes des oracles à exécuter.

<a id="extrait-3"></a>

## Extrait 3

O3 : l’arbitrage ROOT du 14 octobre est inscrit dans le plan et le contrat. AutoDelegate avec deux axes désactivés et tuple fixe réel validé conserve Explicit ; Fallback reste prioritaire ; hors repli, un axe automatique garde Automatic même avec candidat unique. Delegate162 historique garde decision:null si son profil est inconnu. Le rapport T3 en lecture seule confirme les trois modes consommés et la neutralité de null, sans attester à lui seul la projection Rust finale. O2 demeure conforme au refus sûr sans grâce worker nouvelle. O5 demeure un contrôle du diagnostic, pas une assimilation de CAS à une panne SQL.

<a id="extrait-4"></a>

## Extrait 4

Limites du reviewer : tests T025/T3, fixtures MessagesTimeline/CLI/worker et documents spec/plan/tasks largement survolés ; parties hors 163 et advisories non relues. Les gates futurs T040/build/installation/Git/nettoyage ne sont pas comptés comme défauts. F1 doit être corrigée avant clôture ; F2/F3 restent réserves, sans prétendre les avoir corrigées. Audit R6 readonly en cours puis delta R7 frais avant grade. Total courant 57 tâches/quatre cochées ; P3 NONCONVERGED historique conservé, correction de contre-revue en phase 5, pas P4 formel ou approbation finale anticipée.

<a id="extrait-5"></a>

## Extrait 5

Reçu /Users/moi/.cache/bridget-delivery163-20261013/code-review163-r4-final.json, SHA-256 5cb0c98f361e7079eb59085ec2988b39636c4378a1fc5f4cab8f1544b7f8a5f0. Reviewer Claude Sonnet 5.5/claudeAgent ; taskId node:delegated-task:command%3Amcp%3Aa3163845-67c8-4e7f-bea0-f779216e5c09%3Adelegate-task%3A163-code-review-20261014-r4. Deux hashes source, manifeste et delta R9 vérifiés ; 21 sources inchangées, T3, logs et résultats de tests non relus par le reviewer. R1/R2 non relus exactement ; aucun test exécuté par cette revue. Audit32 encore en cours, aucun grade/cochage/livraison déduit ; 4/57, P3 NONCONVERGED historique et exception phase5 conservés, aucun P4 formel.

<a id="extrait-6"></a>

## Extrait 6

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

<a id="extrait-7"></a>

## Extrait 7

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

<a id="extrait-8"></a>

## Extrait 8

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

<a id="extrait-9"></a>

## Extrait 9

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

<a id="extrait-10"></a>

## Extrait 10

T058 reste ouverte. Reconstruction, réinstallation du correctif et recette G8 réelle restent à faire. Compteurs inchangés : SPEC163 53/58, SPEC164 19/22. La source5105378bf4f03376e9bc93e52065cb022f81dee5 reste chargée au dernier checkpoint de runtime ; aucun nouveau commit, binaire ou succès G8/G9 n’est anticipé. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18 reste inchangée. P3 NONCONVERGED historique et poursuite corrective autorisée sont conservés.

<a id="extrait-11"></a>

## Extrait 11

P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 restent conservés, sans P4 formel inventé. Les anciennes FULL avec échecs, l’audit163 partiel 3M/2L, le delta T058 partiel sans finding et le delta T3 R6 sans finding restent distincts ; aucun GREEN global déduit.
