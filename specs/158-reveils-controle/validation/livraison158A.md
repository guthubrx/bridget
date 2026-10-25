# Livraison158A — contrôle humain et portées MCP

**Livrée et activée le 2026-10-12. 22/22 tâches de 158A.** La partie 158B reste
à faire ; ses réveils, déclencheur local, faits manqués et secrets ne sont pas
livrés ici. Ce reçu ne clôt pas la série 152–160.

Sources intégrées et poussées vers github/main par ROOT :
80b43407936110526b28f2c38ea72e1b34055154.
Ce SHA désigne le code livré. Le commit ultérieur du reçu et des métadonnées
reste distinct ; aucun SHA futur n’est supposé.

Convergence : passage 3 CONVERGED, 33/33 exigences 158A, aucune tâche ajoutée.
Tâches byte-identiques pendant le passage final. Haiku R2 :
APPROVE_WITH_CHANGES, sans blocage ; réponses et limites conservées.
Audit A100,  aucun finding actif confirmé, couverture partielle 129/151 hunks
(85,43 %). Validateur RC0, 0 erreur, 0 warning. Ce score ne vaut ni lecture de
l’ensemble du dépôt ni couverture de test instrumentée.

Tests : FULL unique historique : 2440 PASS, 12 FAIL, 74 ignored, 95 cibles,
804,17 s, RC101. Les 255 statuts de référence sont identiques et les 47/47 oracles
prescrits 158A passent. Les quatre nouveaux échecs passent ensuite en quatre
replays ciblés distincts. Le lot lib de 22 retests passe aussi après correctifs.
Les huit échecs historiques restent déclarés. Ces lots ne sont pas additionnés
comme tests uniques ; aucune seconde FULL ni GREENglobal n’est revendiqué.
Le dernier changement C1 est un texte CLI compilé dans le build de livraison,
sans nouvel oracle de panne forcée ni test de modèle fournisseur.

Build release : 113,53 s, RC0, jobs 2, nice 10. Signature vérifiée. Binaire installé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget
SHA256 : 2d71f7d35a6a63b19bc064abb5ae22a699b13d3ba74849f9e220190967b96501.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/158-reveils-controle/validation/livraison158A.md#extrait-1).

Nettoyage terminé le 12 octobre à 12:28:42 CEST. Worktree et branche retirés :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle
session-158-reveils-controle
Le stage a libéré 0,29 Gio logiques ; 34,92 Gio de caches protégés restent conservés
(tailles logiques, clones APFS compris ; aucun gain physique prétendu).
Stage nettoyé : /Users/moi/.cache/bridget-build-158A.0x5vzwte/release
Sauvegarde 157 conservée :
/Users/moi/.cache/bridget-delivery158A-preparation-20261012/rollback157-bridget
Archive d’installation vérifiée et conservée :
/Users/moi/.cache/bridget-delivery158A-preparation-20261012/proofs158A-installation.tgz
SHA256 : 8852f006186d671242bad4f1c990bdca3a2192af378d56a336a7bcbf312d2e5f.

Preuves :
- /Users/moi/.cache/bridget-delivery158A-preparation-20261012/git-source158A.json
- /Users/moi/.cache/bridget-delivery158A-preparation-20261012/build158A.json
- /Users/moi/.cache/bridget-delivery158A-preparation-20261012/install-activation158A.json
- /Users/moi/.cache/bridget-delivery158A-preparation-20261012/cleanup158A.json
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/converge158A-final.json
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/closure158A-root.json
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/audit158A-read_manifest.json
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/audit-validator158A.json
