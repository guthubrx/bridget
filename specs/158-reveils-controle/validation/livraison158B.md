# Livraison158B — livrée, activée et nettoyée

Le 12 octobre 2026 : **23/23 tâches B**, soit **45/45 pour 158A+B**.
Converge passage3 CONVERGED, puis complément R2D CONVERGED_MAINTAINED.
La revue code R2 APPROVE_WITH_CHANGES est traitée. US8 et HTTP restent reportés.

Sources intégrées en ff-only et poussées sur github/main, empreinte distante
identique : e52a96240b6168bf4466712e1bda8852b4ecb43f.
Cette empreinte est le commit des sources. Aucun commit documentaire ultérieur
n'est annoncé par ce reçu préparatoire.
Preuve : /Users/moi/.cache/bridget-delivery158B-preparation-20261012/source-delivery158B.json

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/158-reveils-controle/validation/livraison158B.md#extrait-1).

Audit final **A 99,95**, 0 CRITICAL / 0 HIGH / 0 MEDIUM / 1 LOW conservé.
Couverture partielle : **7,9 %**, 32/405 hunks entièrement lus. Ce taux décrit
la lecture du diff ; il n'est pas une couverture de tests. Le LOW concerne
deux paramètres d'horloge ignorés, sans défaut de délai revendiqué.
Preuves : /Users/moi/.cache/bridget-audit158B-20261012/session-2026-10-12-spec-158B-01/grade.json
et /Users/moi/.cache/bridget-audit158B-20261012/session-2026-10-12-spec-158B-01/cycle-scoring/aggregated-findings.json

L'unique FULL réellement exécutée reste **2456 PASS/12 FAIL/74 ignored**,
95 cibles, 1356,44 s, RC101 ; sept résumés enfants exclus, 30/30 références PASS.
Huit échecs historiques A restent. Les deux régressions DND passent dans leurs
replays ciblés. Le diagnostic de projection de l’échéance 099 explique les deux autres
échecs ; leur replay reste 1 PASS / 1 FAIL, avec un écart d'une seconde conservé.
Dernier source ciblé 2124b1d0 : **4 PASS / 0 FAIL**, 22,51 s, RC0, 221 sources stables.
Il couvre les oracles affectés après correction des deux ordres d'horloge.
Aucune seconde FULL ni GREEN global annoncé. G5 prouve Deferred/no099/noFrame,
pas la finalisation à un passage ultérieur qui relit l'occurrence.
Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/tests-158B.md
et /Users/moi/.cache/bridget-tests158B-20261012/T109-T117-return-clock-order-GREEN.json

## Nettoyage terminé

Le nettoyage sélectif du build installé est attesté RC0 à13:22:04 UTC : 0,30 Gio logiques
retirés du stage /Users/moi/.cache/bridget-build-158B.7ctiu4v3/release.
39,67 Gio logiques restent ; caches protégés conservés malgré le budget dépassé.
Aucun gain physique de disque n'est déduit des tailles logiques/APFS.
Preuve : /Users/moi/.cache/bridget-delivery158B-preparation-20261012/build158B-installed-prune.json
Sauvegarde 158A conservée : /Users/moi/.cache/bridget-delivery158B-preparation-20261012/rollback158A-bridget

Nettoyage achevé le 12 octobre à 15:23:48 CEST : worktree
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle
et branche session-158B-reveils-controle retirés. Worktree propre avant retrait,
sans fichier ouvert ; main est seul et propre après retrait.
Preuve : /Users/moi/.cache/bridget-delivery158B-preparation-20261012/cleanup158B.json

Archive de preuves préinstallation fermée :
/Users/moi/.cache/bridget-delivery158B-preparation-20261012/proofs158B-preinstallation-closed.tgz
SHA256 : e2b355e9d70ac774e4edf3cd56619bcf20e00023aac79ca71cfe4db2c3bbe935.
466 membres extraits et SHA vérifiés ; reçus build/install distincts de cette
archive. Archives, binaire de stage, caches de tests utiles et rollbacks conservés.
Aucune base de données supprimée. La publication du commit documentaire de ce
reçu reste à faire par ROOT ; aucune empreinte future n'est inventée.
