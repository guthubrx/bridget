# Historique des échecs FULL158B pour la préparation159A

Ce relevé est historique. Il ne constitue aucune baseline159A fraîche. L’unique FULL exécutée reste **2456 PASS / 12 FAIL / 74 ignored**, 95 cibles, 1356,44 s, RC101. Sept résumés enfants sont exclus. Les 30 références comparées sont PASS. Les 221 empreintes source sont stables pendant ce run.

Les 12 noms ci-dessous forment une liste fermée. Deux régressions DND ont été corrigées et passent dans leur replay. Les dix autres noms correspondent à huit FAIL hérités de158A et deux oracles du défaut historique deadline099. Parmi ces deux derniers, un passe au replay sans correctif. « Dix défauts hérités » ne signifie donc pas dix FAIL d’une suite finale fraîche.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/159-edition-file/validation/historique-full158B-pour159A.md#extrait-1).

## Deux régressions DND corrigées

Le replay T111–T113/T116 dure44,08 s et rend RC101 : **4 PASS / 1 FAIL**. Les deux tests spec100/spec101 sont PASS (journal lignes48–49). L’autre FAIL est G6, sur une clé de fixture accentuée. Ce lot ne constitue pas un GREEN global.

SHA daemon FULL : `cbd072a2d3f407ae609021c070dde2a7c54023b6ff33dd31d816263f525efdb1`.
SHA daemon replay DND : `67c51d36dba7d48c4e844a3547e2d8303b8f594a87578a6b9fe8285129537f60`.

## Échéance099 conservée

Le replay dure15,85 s, RC101 : **1 PASS / 1 FAIL**.
- PASS : `recovery_acked_wrapper_finalise_accepted_apres_crash_daemon`.
- FAIL : `reprise_daemon_redelivre_les_octets_immuables_a_la_meme_instance`.

Le journal lignes35–39 montre uniquement deadline_at **1791811178 →1791811179**. L’écart d’une seconde reste un échec réel. Aucun seuil, assertion d’égalité, test ou correctif hors158B n’a été modifié pour rendre ce replay vert. Le fichier de tests est byte-identique à158A, au FULL et au replay : `a2fabf4e2bb58b82502357ab43b716b8558139950b9a64f2e16fee38a10ce321`.

La sérialisation durable précède l’ajout de deadline. La reprise relit les octets sans deadline et appelle la même fonction avec une nouvelle seconde civile. Les fonctions defer_idempotent_delivery et stamp_turn_deadline_for_delivery sont byte-identiques à158A selon la preuve diagnostique. Le PASS du premier oracle au replay ne prouve pas la correction du défaut.

## Provenance et limites

Source livré158B : `e52a96240b6168bf4466712e1bda8852b4ecb43f`. Cette empreinte ne remplace pas les empreintes du FULL antérieur. Aucun test, accès DB ou service n’a été exécuté pour ce relevé. Les lignes du tableau sont celles du journal FULL exact. Les fichiers lus sont identifiés ci-dessous et dans le JSON. Les copies archivées ont été comparées à leurs SHA du manifeste fermé.

Archive : /Users/moi/.cache/bridget-delivery158B-preparation-20261012/proofs158B-preinstallation-closed.tgz
SHA256 : `e2b355e9d70ac774e4edf3cd56619bcf20e00023aac79ca71cfe4db2c3bbe935`.

| Source absolue | SHA256 |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/livraison158B.md | `1a2006d82c991d487e420a8e2c269152930bf93ff24be3582cd49c4be2993664` |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/tests-158B.md | `18b11fcb6ff17d3b675d63684b7da5624f713e883aab16ced0b08da44f8e9e26` |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/T021-full-comparison158.json | `50ad3e3462987ef9c8f83e57523eadc2a1afdb30243421ae1ebb61fadabb230d` |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/099-recovery-deadline-diagnostic.json | `29ba539fb0f87331a6a2c72805c49323266099871a65eaf32a56649525986400` |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/099-recovery-deadline-diagnostic.md | `2ad4f616d05918264556911385300a839eae3d2e0164d5689fb4e8c6242b18dd` |
| /Users/moi/.cache/bridget-tests158B-20261012/T121-FULL158B-compiled-final.json | `ccac449dadc9ee930a863a0e8877bc3a2ae86146960def230c827d569178f491` |
| /Users/moi/.cache/bridget-tests158B-20261012/T121-FULL158B-summary.json | `21a2d955cdf9c14f1d3ea2bc52df7ce379d5722b52ddd7c9f3f71844043ad50d` |
| /Users/moi/.cache/bridget-tests158B-20261012/T121-FULL158B-compiled-final.log | `fd5ba6c30e89954db7d40725589d0b368d4ded5569cffd1ef66dcc1be2b7a274` |
| /Users/moi/.cache/bridget-tests158B-20261012/T111-T113-G6-G7-attested-T116-diagnostic-GREEN.json | `dd53b24bb8c27b2bc4a094c543ac1e40d0b89f215419221c96d929b603c2d057` |
| /Users/moi/.cache/bridget-tests158B-20261012/T111-T113-G6-G7-attested-T116-diagnostic-GREEN.log | `61a1ca494c6dc9e2f8eec6237edd21e898ed734252ebd331d73961deb9b57b0e` |
| /Users/moi/.cache/bridget-tests158B-20261012/T121-099-recovery-timing-replay.json | `ddccaa56cded02ca8d958daea96cf8e55b712ef99e1eb2d9324ac7208df1142c` |
| /Users/moi/.cache/bridget-tests158B-20261012/T121-099-recovery-timing-replay.log | `f23d4365907c20d18f71fa56ed906cfa84d304f91cc5009412295a03c89a6ef3` |
