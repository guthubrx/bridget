# Archives techniques de l’adaptateur T3

Document d’origine : [specs/135-controle-missions-silencieux/correction-observations.md](../../../../../../specs/135-controle-missions-silencieux/correction-observations.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Commit code : `9403489a1aff79d4263c258fa8f8d1e86f6feb54`, fusion rapide dans
  main et push github/main effectués.
- Build release isolé : PASS, 54,63 s. Binaire actif et copie de build ont le
  même SHA256 : `26b74e21074cff084db6048c5e6d5046096a46c0279adb71e9f5d9c9e5723f8a`.
- Daemon et pont T3 relancés ; build actif `9403489a1aff`, daemon en ligne et
  46 agents revenus. Les identifiants des agents avant/après sont identiques.
- Les sept abonnements persistants de Politique sont conservés à l'identique.
- Sonde ponctuelle en production, abonnement sans filtre d'agent : reçu
  `notification_mode=journal_only`, puis `suppressed_total=1`,
  `notifications_lost=0`. Un vrai fait est donc observé et rendu silencieux,
  sans perte de remise invoquée pour expliquer le silence. Une lacune est
  comptée séparément (`observation_gaps=1`), jamais reconstruite en fait.
- Depuis le relevé de livraison à 15:01:33 UTC, les journaux locaux de Politique
  et du contrôleur de recette ne montrent aucun nouveau message portant
  l'identifiant `bridget-observation:`. Ce constat est borné à cette recette ;
  il n'est pas une revendication de couverture totale des sources.
- Politique a accepté le message de nouvelle règle `mcp-96911-6ac3bc06-1`.
  L'information vers cx-coordinator `mcp-96911-6ac3bc06-2` reste en vol au dernier
  relevé, sans revendication de réception. Aucun accusé demandé. La politique
  est appliquée par le daemon et ne dépend pas de ces messages.
- Les deux sondes de recette sont désabonnées. Le contrôleur Politique demeure
  activé : passages 364 puis 366, dernier code de sortie 0. Aucun registre de
  mission ni verdict modifié par cette correction.
