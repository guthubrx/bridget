# Tests et revues — session 154

Date du point : 2026-10-12. Statut : validation finale en cours.
T025 et T026 sont acquises après GO du principal. Aucun GREEN global, aucune
livraison ni activation déclarée. Les commandes ont été exécutées par le principal.

## Deux états de code distincts

| État | Périmètre éprouvé | Empreinte du code natif |
|---|---|---|
| A — avant correctif CAS | Une suite workspace complète, exécution 3313 | `311037208c1c79b71e4bf1450d9e7b4365418cf62916cc6e98b5ef0098a1e1f9` |
| B — après correctif CAS | Cinq tests de rapport ciblés | `c93160372783b316167c8f22038b90aafaa10a02b4ed5ec9e75def99a1bcb304` |

La suite complète de l’état A ne prouve pas l’état B. Aucun nouveau passage
workspace complet n’est inventé. Chaque reçu comprend les 207 empreintes de
sources avant/après, identiques pendant son propre passage.

## Suite workspace unique — état A

94 cibles Cargo. Finales : 2194 PASS, 9 FAIL, 72 ignorés. Durée 499,95 s, RC101.
Le parseur conserve la dernière finale de chaque cible Running/Doc-tests.
Trois résumés de helpers sont exclus. Toutes les cibles ont une finale ; les
neuf noms d’échec sont extraits sans lacune de parsing.

Comparaison à la référence 152 : les huit échecs historiques restent présents.
Un échec supplémentaire est observé :
`source_brute_et_rotation_traversent_snapshot_puis_live_et_curseur_perime_annonce_gap`
dans `Running tests/core_089_attach_test.rs`.
Le diagnostic du principal et de son pair est en cours. Ce constat ne tranche
pas encore si cet écart est une régression introduite par 154.

Référence ciblée T001 : 194 cas retrouvés, aucun manquant. Les 193 cas auparavant
PASS n’ont aucune régression de statut ; le cas ignoré reste identifié comme
tel. Les 48 noms native154 sont PASS dans cette suite avant le correctif CAS.
Ces données ne rendent pas la suite globale verte.

Reçu archivé : /Users/moi/.cache/bridget-delivery154.l50umk90/workspace154-final-r1.json
Copie de validation : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/workspace154-final-r1.json
Journal : /Users/moi/.cache/bridget-delivery154.l50umk90/workspace154-final-r1.log
SHA256 du journal : `90f325c3da85499f3e107f555de8daac1e25338558132ac7c66b4b089d25916b`.
Comparaison : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/workspace154-final-r1-comparison.json

## Rejeu du seul cas coreAttachGap

Un seul cas, rejoué exactement sur le binaire compilé : 0 PASS, 1 FAIL,
1,71 s, RC101. Il échoue encore sur la lacune attendue d’un curseur périmé.
Aucun rejeu complet de la suite. Ce passage garde l’empreinte native de
l’état A ; il ne valide pas le correctif CAS de l’état B. Diagnostic en cours.

Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/core-attach154-targeted-r1.json
Journal : /Users/moi/.cache/bridget-delivery154.l50umk90/core-attach154-targeted-r1.log
SHA256 : `a373cdaf8a0ce4cbbe1a0fa0536d451d93588a9044c3331d3e899bfd63bfb063`.

## Course de publication — RED puis correction ciblée

RED : 0 PASS, 1 FAIL, 6,96 s, RC101. Le test existant
`native154_rapport_cas_et_premier_rapport_gagnant` a été étendu avec un vrai
descendant admis pendant la lecture du rapport. Le résultat devenait
result_available au lieu de rester waiting_for_children. Ce défaut de
production est distinct du diagnostic coreAttachGap.

Correctif natif relu : sous le verrou court de sauvegarde, recontrôler
`descendants_busy` avant de ranger le rapport et de publier. Si un descendant
est actif ou si ce contrôle échoue, différer. Les faits et le marqueur restent
inchangés. Aucun helper, outil, table ou migration ajouté pour ce correctif.

GREEN ciblé de l’état B : cinq tests de rapport PASS, 0 FAIL, 13,68 s, RC0.
L’empreinte du test natif est
`1af23658f16958d4fa22b83dc4fd891c7b740f7ab1ea86f615b818772dfcbd21`.
Les autres modules n’ont pas reçu de validation workspace sur cette nouvelle
empreinte dans ce passage. Les limites de preuve restent explicites.

RED reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/cas154-final-red-r1.json
RED journal : /Users/moi/.cache/bridget-delivery154.l50umk90/cas154-final-red-r1.log
RED SHA256 : `904e741a36f16fa0a36df576b8a6a32dc0eec0edbb3c6943afa49519631ab344`.
GREEN reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/rapport154-final-green-r1.json
GREEN journal : /Users/moi/.cache/bridget-delivery154.l50umk90/rapport154-final-green-r1.log
GREEN SHA256 : `7cb869558b4543997a095c35452851ee71d2d8aa997b7cf2d7e8897ff96d6ac4`.

## Prélecture T026 et prochaines preuves

Le principal a relu tous les hunks des 31 fichiers du diff, puis le correctif
natif ci-dessus. Aucun blocage des Articles XIX/XX n’est signalé par cette
prélecture. Le documentaire a relu le bloc modifié : contrôle des descendants
avant mutation du workspace, du marqueur et du résultat.

FR014 est désormais limité aux parcours 154 et aux deux lectures du wrapper
T022, conformément au plan/H04 et à l’ADR. communication.rs et derive_domain_at
historiques restent hors périmètre. Aucun correctif hors périmètre appliqué.
SIGTERM reste limité au PID Git lancé ; retour et récolte bornés, pas de
SIGKILL ni garantie d’arrêt d’un programme hostile qui ignore SIGTERM.

Le principal termine le diagnostic coreAttachGap, la revue finale, Converge
et les contrôles de livraison applicables. T025/T026 ne sont pas cochées ici.
Le diff cumulé final n’est pas déclaré exporté par le documentaire.

## Vérification documentaire

Les quatre hashes de journaux ont été recalculés. Les copies JSON de validation
sont identiques aux reçus archivés. La comparaison T001, les 94 cibles et les 48 cas
native154 ont été relus dans le JSON. L’empreinte native courante a été vérifiée
contre le reçu GREEN ciblé. Aucun Git, Cargo, test, service ou source modifié
par l’owner documentaire.

## Diagnostic coreAttachGap — fixture de date, résultat corrigé en attente

Le principal a isolé une différence de calendrier entre fixture locale et
journal UTC. Les données runtime relevées sont : date locale de la veille
2026-10-11 ; date UTC de la veille 2026-10-10 ; date du journal courant UTC
2026-10-11. Après minuit en CEST, la fixture effaçait donc le fichier courant.
Le code de production concerné est inchangé depuis 153, selon la comparaison
effectuée par le principal. Ce diagnostic porte sur la fixture, pas sur une
régression de production 154.

Le correctif de la fixture existante ajoute uniquement l’option -u de date et
le commentaire UTC. Aucun nouveau scénario ni helper. Son résultat GREEN est
en attente à ce point. Le nettoyage d’anciens fichiers privés a été fait, mais
la première frame n’a pas été observée : ce nettoyage ne prouve aucune frame.

## Contrat FR001 — null explicite refusé, helper existant réutilisé

Un autre défaut réel du contrat a été trouvé : null explicite sur workspace,
base_ref ou branch était assimilé à une absence. Le test existant d’objet fermé
a été étendu à ces entrées. RED : 0 PASS/1 FAIL, 10,82 s, RC101.

Correctif : trois attributs serde réutilisent deserialize_present_optional déjà
présent dans protocol.rs:2753. Il désérialise une valeur présente en Some et
refuse null quand le type attendu ne l’accepte pas. L’absence conserve default
et les octets legacy. Aucun nouveau helper, type, migration ou outil.

GREEN ciblé : 3 PASS/0 FAIL, 7,82 s, RC0, sources stables. Protocole courant :
`3b990872e38b610a1566bbb603554eb41526376b676380f3c8c102fcf27057b8`.
RED reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/contrat154-final-red-r1.json
RED journal : /Users/moi/.cache/bridget-delivery154.l50umk90/contrat154-final-red-r1.log
RED SHA256 : `23f241977818db41fc88ff67ceef4e4fd91ad2f55b356d9bf9d7d08a6364cf2c`.
GREEN reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/contrat154-final-green-r1.json
GREEN journal : /Users/moi/.cache/bridget-delivery154.l50umk90/contrat154-final-green-r1.log
GREEN SHA256 : `a23bd2c9dd53ca2cc73bd5100f5520e906514b737cd6423e4ca4219054541e5e`.

Ce passage est un troisième état ciblé, après le correctif CAS. La suite
workspace complète antérieure ne porte pas cette empreinte de protocole.
Aucune preuve après null n’est attribuée rétroactivement à la suite globale.

T025/T026 restent ouvertes jusqu’au GREEN de la fixture coreAttach et à l’export
par le principal du diff final incluant les nouveaux fichiers. Converge formel
n’est pas encore déclaré. Les hashes des deux nouveaux logs et le protocole
courant ont été vérifiés ici ; les copies JSON sont identiques aux archives.

## Clôture de Phase5 — preuves acquises, état Implemented

Le cas coreAttach corrigé en UTC a été rejoué seul : 1 PASS/0 FAIL, 13,99 s, RC0,
sources stables. Cette preuve clôt le défaut de fixture ; les fichiers de
production concernés restent inchangés depuis 153.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/core-attach154-targeted-r2.json
Journal : /Users/moi/.cache/bridget-delivery154.l50umk90/core-attach154-targeted-r2.log
SHA256 : `dcd25fa1fcecc2cdc74a12af3eaf3ecb50dbd9d97a7c9599102b881364e3470e`.
Empreinte de la fixture core : `021ad4c62de547d802b2ffd4b4c1bdf8d8d299f07994b03fe4ebcbff1af53d63`.

Le principal a vérifié à 22:49 les worktrees : principal et 154 seulement, aucun
reste créé par les tests. Il a relu les 31 fichiers du diff complet, puis les
incréments CoreUTC, trois attributs null et recontrôle CAS. Aucun helper neuf
pour FR001 ; générique existant réutilisé. Aucun effacement de code de production.

Le patch complet, nouveaux fichiers compris, existe : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/diff-154.patch
SHA256 constaté AVANT le cochage documentaire : `7e7d8136fe2ef7bff519c30c2aa851af9ce24f49d1b97e89438c9d846d3877fd`.
Le principal le réexportera après le gel documentaire ; cette empreinte ne
prétend pas couvrir les cases et notes ajoutées après son export.

GO principal reçu : T025/T026 cochées. 28/28 tâches acquises. Statut Implemented,
qui signifie implémentation terminée et ne signifie pas livraison terminée.
La suite unique demeure historiquement 2194 PASS/9 FAIL/72 ignorés, RC101. Huit
échecs historiques restent ; le neuvième défaut de fixture UTC est corrigé et
prouvé par son seul rejeu. CAS corrigé 5 PASS et contrat null corrigé 3 PASS restent
des preuves ciblées distinctes sur leurs propres empreintes. Aucun faux
GREEN global ou nouveau passage workspace déclaré. Aucune Converge formelle,
validation de livraison ou activation déduite de ce cochage.

## Revue R1 et audit C1 — aucun nouveau PASS ajouté

Après Converge P1, F1/F2 sont retenus et désormais corrigés avec preuve ciblée : quota des chemins retirés
seulement sur NotFound ; tolérance des voisins illisibles du registre, cible
encore fermée en erreur. Les tests union32 et ancre existants sont étendus.
Le principal a fourni les reçus finaux RED0/2 et GREEN2/0, cités ci-dessous. Les nombres de cette note
ne reprennent pas un ancien passage comme preuve des nouvelles corrections.

C1 archivé : 0 critique, 0 élevé, 2 moyens, 2 faibles ; couverture 64,75 %.
Les sauvegardes de nettoyage sont clarifiées : rapport d’abord après arrêt,
cleanup_done ensuite dans une seconde sauvegarde, seulement si rapport prêt.
Tasks reste byte-identique, SHA 18e6f48b96ec1568fab624c46e08ed85c9a8edfe3a8f546433b9595d7a9ccc54.
Aucun hash de nouvelle source ou patch final déclaré avant capture fraîche.

## F1/F2 corrigés — preuves ciblées finales avant P2

RED : 0 PASS, 2 FAIL, 13,03 s, RC101. GREEN : 2 PASS, 0 FAIL, 0 ignoré,
10,68 s, RC0. Les extensions des tests union32 et ancre existants ont été
exécutées sur 207 sources stables pendant chaque passage. F1/F2 sont corrigés,
avec preuve ciblée. La cible illisible reste en erreur ; seule une absence
NotFound de chemin Task libère la place du quota. Aucun helper nouveau.

RED reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/review154-final-red-r1.json
RED journal : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/review154-final-red-r1.log
RED SHA256 : `8696fe79a32405b3b29830029f0c8d1cd53b43045c9668c477bbe3af66e1f4bd`.
GREEN reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/review154-final-green-r1.json
GREEN journal : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/review154-final-green-r1.log
GREEN SHA256 : `c8caa4f6a9bd63a4024830c02e4f22254ce081a82d872658924b1811306c770e`.

Empreintes vérifiées contre les sources courantes et le reçu GREEN :
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/daemon/native_delegation.rs : `75e3713d50a0ee60492e5cdaf2cf7f6f5960fb814550ee95d100d2d36d4979a4`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/workspace_git.rs : `e0e4f6dcf385d8644b777572dcf23aae4f8d54743fb95ee31d82165770f79ca4`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/daemon/native_workspace154_tests.rs : `150bfbbdac8ece378bf45a3c6883d23abd9aa14a6a2b58df4fafeb9fcca8b0e8`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/workspace_git_tests.rs : `2208dc67f2d6fb6aa6dfb9bfc28c50be8b945ce1566d75f7a0e7c3b509bd8472`.

Les copies JSON et logs de validation sont identiques aux archives. Ce passage
ne remplace pas la suite workspace historique avant les correctifs. Aucun
GREEN global, verdict P2, capture d’audit fraîche ou nouveau patch final déclaré.
Docs gelées pour permettre la reconvergence P2 du principal. Tasks inchangé.
