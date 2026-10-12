# Baseline fraîche158A — T001

Exécution du principal le 2026-10-12, début 08:46:00 UTC : **253 PASS, 0 FAIL, 2 ignored**, 255 noms distincts. RC0 en 351,95 s. Cette exécution fraîche ne remplace pas l’histoire FULL157 par un faux lancement récent.

Base de préparation : bd37e927eee0b28f058df9d6a6992e38665a6ad0, sources157 livrées8f21dfd7b553e822fcabe0a0e7eabc6741a8e45b. Référence exacte des noms : /Users/moi/.cache/bridget-plan158-baseline-preparation-20261012/baseline158A-from-full157.json, SHA256 400940deddeb58b3e44435fbcd625cc8c523d8542464c87a28d042088bd34544. Cette référence historique contient les noms tirés de FULL157 ; les résultats frais ci-dessous viennent uniquement du runT001158.

| Ensemble | Noms | PASS | FAIL | ignored |
|---|---:|---:|---:|---:|
| native148 | 28 | 27 | 0 | 1 |
| native149 | 78 | 78 | 0 | 0 |
| native150 | 51 | 51 | 0 | 0 |
| delegation149 | 10 | 10 | 0 | 0 |
| spec_087 | 11 | 11 | 0 | 0 |
| mcp.rs | 77 | 76 | 0 | 1 |
| Total distinct | 255 | 253 | 0 | 2 |

Les six ensembles sont disjoints. Comparaison par nom exact et statut : aucun nom manquant, aucun nom ajouté, aucun statut changé sur 255/255. Les tests de mcp.rs sont les noms mcp:: du libdaemon, pas delegation_mcp ou mcp_identity. Ignorés conservés :

- mcp::tests::spec138_real_agent_loop_cli_daemon_scoped_replay
- native148_parent_codex_fixture_delegates_once_to_real_glm

## Commande, stabilité et comptage

Commande Cargo -p bridget-daemon, features test-support, --locked/--offline, sélection des 255 noms exacts, tous les targets (pas --lib seulement). La commande complète figure dans le JSON. Charge : jobs2, threads2, nice10. Namespace privé :/private/tmp/b.9s3568yc. Aucun test, service, modèle ou Git exécuté par l’owner documentaire.

82 cibles Running/Doc-tests dans le log. Seule la dernière ligne test result de chaque cible est comptée. Ce run ne contient aucun résultat de sous-processus embarqué à retrancher ;les 3 résultats enfants exclus lors de la préparation FULL157 ne sont pas soustraits de nouveau. Libdaemon : 243 PASS / 0 FAIL / 1 ignored. Quatre targets supplémentaires : 1 PASS, 3 PASS / 1 ignored, 2 PASS, 4 PASS ; autres cibles : 0 test. Somme finale : 253 PASS / 0 FAIL / 2 ignored, 255 noms. Aucun ignored converti en PASS, aucun GREEN global de workspace déduit.

Le JSON du runner ROOT atteste sources_stable:true. Les 217 empreintes avant/après sont strictement identiques, vérification effectuée sur leurs deux maps. Le SHA du log correspond au reçu. JSON : /Users/moi/.cache/bridget-plan158-20261012-r1/T001-baseline158.json, SHA256 971f156c0b534fb1a510ebf6c4d0c6003bcb97e1dc11f0fa0e66fedca83a8b31. Log : /Users/moi/.cache/bridget-plan158-20261012-r1/T001-baseline158.log, SHA256 65dbc5dc2e8c0f940774e94230b5f39504a940c56864c26835eb7c873199d717.

Cette baseline ne prouve ni l’implémentation158A, ni158B, ni une recette fournisseur, ni une livraison158. La validation et le cochage T001 restent au principal. Aucun artefact du worktree n’est modifié par cette préparation.
