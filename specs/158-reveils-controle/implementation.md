# Implémentation158A — contrôle humain et portées MCP

Base Git : bd37e927eee0b28f058df9d6a6992e38665a6ad0. Périmètre158A uniquement. 158B, budgets de tours et tokens, réveils et secrets restent hors implémentation.

La CLI propose une vue globale bornée, l'annulation d'une tâche et un arrêt persistant des nouvelles délégations. Elle reprend les gardes humaines et le transport de contrôle existants. La liste active utilise50 entrées par défaut et100 au plus. Les mutations exigent un terminal interactif.

Delegate et Continue composent arrêt et pause après le rejeu. Une nouvelle clé est refusée pendant l'arrêt. Le rejeu expose la vue actuelle de la tâche, avec suspended dynamique. Les reçus halt rendent au contraire leur réponse historique, sans callback ni nouvelle annulation. État et reçu sont committés ensemble. Annuler après commit reste au plus une fois ; cancelled compte les arbres visés, sans confirmer leur arrêt physique. StoreUnavailable peut précéder ou suivre le commit : la CLI indique l'incertitude et recommande de relire puis d'annuler les tâches restantes.

Le catalogue MCP couvre30 outils par portée et action. Le plafond discovery vient de CallerScope authentifié côté daemon, sans DB cliente ni repli permissif. Il reste actif avant cleanup_done, même entre rondes ou après une tâche terminale. Les écritures interdites restent fermées. ChildProof queue est refusé avant toute connexion. L'aperçu pur handoff103 réemploie le parseur et le rendu locaux avant RPC ; il refuse aussi les paramètres d'envoi nuls.

La vue globale exclut missions, résultats et chemins. Elle présente les métadonnées courantes d'arrêt dans un même snapshot. Status sort avec le code1 si ces données sont indisponibles, même après un affichage partiel.

## Vérification et relecture

ROOT a exécuté une seule suite complète : 2440 PASS,12 FAIL,74 ignorés, RC101. Les255 noms de référence conservent leurs résultats : 253 PASS et2 ignorés. Les47 tests prescrits158 sont PASS dans cette suite. Quatre nouveaux échecs ont ensuite été traités par correction ou adaptation des fixtures existantes et validés par rejeux ciblés. Les huit échecs historiques restent déclarés. Aucun succès global ni seconde suite complète sur le dernier hash n'est affirmé.

Reçus ROOT :

- /Users/moi/.cache/bridget-plan158-20261012-r1/T021-full-comparison158.json
- /Users/moi/.cache/bridget-plan158-20261012-r1/T021-review-fixes158-lib.json : 22 PASS.
- /Users/moi/.cache/bridget-plan158-20261012-r1/T021-preview103-replay.json : 1 PASS.
- /Users/moi/.cache/bridget-plan158-20261012-r1/T021-concurrency089-replay.json : 1 PASS.
- /Users/moi/.cache/bridget-plan158-20261012-r1/T021-security089-replay.json : 1 PASS.
- /Users/moi/.cache/bridget-plan158-20261012-r1/T021-native152-replay.json : 1 PASS.

La relecture couvre les152 blocs du diff final de27 fichiers : 126 corps de blocs sont réutilisés à l'identique depuis la lecture initiale,26 sont relus après correctifs. Cette couverture concerne le diff ; elle ne vaut pas lecture complète de toute l'architecture.

Diff final : /Users/moi/.cache/bridget-audit158-final-20261012/diff158A-final.patch

SHA256 :67001c94315d17eeac8968df0ce7c48d653792df149b5d9e93ac6e4d748dfe76

Preuve de lecture : /Users/moi/.cache/bridget-finalread158-20261012/final-read_manifest.json

Rapport : /Users/moi/.cache/bridget-finalread158-20261012/final-report.md

## Limites conservées

La borne ADR011 du même compte système reste inchangée. Les256 reçus ne sont pas un historique d'audit. Un reçu purgé peut être appliqué à nouveau. La purge normale retire atomiquement un arbre entièrement soldé ; une lignée corrompue ferme le snapshot avant commit. Les tests ne prouvent ni un fournisseur réel, ni une activation du runtime158, ni l'absence de toute course ou panne. Le lecteur n'a lancé aucun test, build, fournisseur, service ou mutation Git. ROOT conserve le cochage et la livraison.
