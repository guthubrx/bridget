# Converge 151 — passage 2

Date : 2026-10-11. Verdict : **CONVERGED**. Zéro tâche ajoutée.

Les 39 FR, 13 SC et 7 US sont couverts. T027 ferme le manque SC-009 du
passage 1. Aucun défaut de production n'est confirmé. Cette archive reprend
le verdict rendu après la levée du gel du passage 2. Aucun test, Cargo,
service, modèle ou opération Git n'a été exécuté pendant ce passage.

## Périmètre et intégrité

Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache.
Delta151 versus le socle150 importé : 30 chemins suivis, dont15 production,
11 tests/format,3 documents et1 sélecteur, plus les documents151 nouveaux.
Les fichiers150 importés sans changement ne produisent pas de finding151.

Le fichier /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/tasks.md
est resté byte-identique pendant Converge2 : SHA256
`d04cf9d3ab41aeed6af4a7ac1d3ec597691713b53ec9a84cb7721e7a80a408c3`.
Les74 empreintes de fichiers du manifeste r2 ont été vérifiées sans écart.
L'agrégat47 communiqué par le principal avant le passage est
`fe01a6ab5d29cc9de00c2c1750943ec200d00e419d7437e22a1e6ac95699ebf4` ;
sa vérification avant/après appartient au principal.

Captures vérifiées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation :

| Fichier | Octets | SHA256 |
|---|---:|---|
| diff-151-final-r2.patch | 524153 | 557302374671dad10d2eba50c32fdd86407cb765ffc2cd2c8212fcf48d4ce871 |
| delta151-vs150-final-r2.patch | 324332 | 7cab421ab1bc0c89e15598fda97557f7009d8ea3facf510c8c770d05165ae2c0 |
| snapshot-151-final-r2-manifest.json | 93585 | 0e8183de42c73029ccdf625b50029035496f5f12eb81be0155a3c08a45c926fa |

## Repères fichier:ligne

Chaque repère désigne le chemin absolu suivant. Les lignes sont celles du
code courant du passage2, après T027 et les corrections de format ciblées.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/validation/converge151-passage2.md#extrait-1).

## Matrice FR

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/validation/converge151-passage2.md#extrait-2).

## Matrice SC

| SC | Code et test | Preuve retenue |
|---|---|---|
| 001 | N:483 ; E:301 | Daemon réel, capture SQLite, zéro Status ; E012:971ms |
| 002 | N:546 ; E:301 | Expiration1s, enfant poursuit, résultat unique |
| 003 | N:1243 ; E:301 ; N:2364 | Message après résultat rendu ; attente pure |
| 004 | N:535 ; N:2622 | Trois tâches synthétiques, une réponse, trois entrées |
| 005 | T:25 ; N:2647 |100 publications concurrentes, zéro expiration |
| 006 | N:90 ; E:454 | Même tâche pending puis claimed puis delivered |
| 007 | N:84 ; N:2893 ; T021/W004 dans V | Clés historiques et régressions ciblées conservées |
| 008 | N:483 ; D:242 ; N:2453 ; D:419 ; N:2679 | SELECT à0/5/10s ;64 attentes, Status2,430ms, Cancel19,056ms |
| 009 | D:7979 ; N:1958 | Vrais handlers/SQLite : open avant ACK,406/357µs, deux scopes, stabilité |
| 010 | D:211 ; M:96 ; M:2858,M:2928,M:3449,M:3631 | Fermeture/annulation avant, pendant, après inscription ; contrôles disponibles |
| 011 | C:24 ; A:31 | Vrai binaire150 :44,710/54,874ms ; trame ancienne seule |
| 012 | M:190 ; E:749 | Vrai MCP, quatre attentes3600s libérées ; E012:14ms |
| 013 | N:551 ; D:315 ; N:2420 ; N:2273 | Expiration<1,5s ; guichet<2s avec filet60s |

## Matrice US

| US | Réalisation et tests | Verdict |
|---|---|---|
| 1 | N:483 ; E:301 ; N:2420 | Couvert |
| 2 | N:73 ; N:2822,N:2868,N:2893 | Couvert |
| 3 | N:90 ; N:2798,N:2922 ; E:454 | Couvert |
| 4 | N:535 ; N:2622 ; B:3632 | Couvert |
| 5 | D:242 ; R:699 ; N:1958 ; I:121 ; G:253 | Couvert, SC-009 complété |
| 6 | C:24 ; P:7938 ; N:2893 ; A:31 ; W:8130 | Couvert |
| 7 | N:796 ; T:190 ; huit tests ACK ; E:707 | Couvert |

## Nature des preuves et limites

T027 utilise une fixture unitaire, les vrais handlers du daemon et SQLite.
Il ne lance ni processus daemon ni modèle. Les E2E lancent les vrais processus
Bridget/MCP avec données privées et fournisseur enfant de fixture. Les tests
de façade utilisent aussi un daemon simulé. La compatibilité ancienne utilise
le vrai binaire150.

Le journal /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T027-ack-requestwait151.log
donne1 PASS/0 FAIL/0 ignoré, distinct de W004. Les mesures406µs/357µs sont
ancrées avant le handler DeliverAcked, après preuve de l'attente open.
Les bons résumés, l'epoch émis par le handler et la capacité libérée sont
assertés. Les deux snapshots restent corrects après remplacement du ledger.

W004 reste2046 PASS/8 FAIL hérités150/72 ignorés,94 cibles,RC101.
Aucun total workspace2047 n'est déduit. Aucune suite complète après T027.
Format globalRC1/549 hunks anciens :547 corps identiques150, deux contextes
anciens qualifiés. Les18 corrections ciblées/10 fichiers gardent une
normalisation rustfmt byte-identique. Aucun test après ces seuls retours à la
ligne. Aucun succès global de format n'est revendiqué.

Non vérifiés : modèle réel, activation de production, attente réelle d'une
heure, branche EINPROGRESS effectivement atteinte sur macOS. Le callback
avant connect est testé ; le passage précis de polling reste vérifié en
lecture seulement. Ces limites ne constituent pas un manque du contrat151.

## Six réponses Article XX

1. Nécessité : attendre sans polling, lire un résumé correct et conserver la
   remise normale. T027 prouve le réveil après ACK normal.
2. Simplicité : réutilisation des stores, handlers, routeur, compteur et
   fixtures. Aucun module, hook de production, dépendance ou table de boîte neuf.
3. Hypothèses : socle150 conservé, résultat stable après publication, identité
   attestée, résumé figé au premier answered. T027 distingue deux demandes et
   scopes malgré une clé de réponse commune.
4. Vérifié : spec, plan, tâches, checklist, code actuel et journaux existants.
   T027 contrôle l'attente ouverte, l'ACK réel, les résumés, le réveil et la
   capacité libérée. Les captures r2 et74 empreintes correspondent.
5. Non vérifié : modèle réel, production, attente3600s entière, EINPROGRESS
   atteint sur macOS. Aucune suite complète après T027 ni test après format.
6. Conservé : compteur contre les réveils perdus, résumé contre le remplacement
   du ledger, registre/callback pour fermer les sockets, grâce EOF4,5s pour
   les clients historiques. Les barrières et remises préparées de T027 prouvent
   le stimulus. Aucun ajout de confort ni abstraction superflue identifié.
