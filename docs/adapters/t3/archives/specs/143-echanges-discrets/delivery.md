# Archives techniques de l’adaptateur T3

Document d’origine : [specs/143-echanges-discrets/delivery.md](../../../../../../specs/143-echanges-discrets/delivery.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Après la clôture technique, l'utilisateur autorise commit, fusion, push, installation et redémarrage de T3 pour143. Cette autorisation remplace la restriction initiale pour la livraison143 seulement. Les résultats historiques « non installé/non activé » restent exacts à leur date. Préserver142, les autres travaux, missions et programmes Bridget/Agent Loop. Aucun cleanup supplémentaire autorisé ici.

<a id="extrait-2"></a>

## Extrait 2

| Étape | Preuve | État |
|---|---|---|
| Validation | Gel392 PASS, contrôles/recette/revue, convergence11:39:54 | Terminé avant livraison |
| Commit T3 | `a1a4f2ef12e2e4c4f2b64cd6d4ac162bceb22c39`, quatre fichiers source/tests | Effectué |
| Fusion T3 | Fast-forward de `local/v0.0.45` | Effectuée |
| Push T3 | `local/v0.0.45` et `session-143-bridget-discreet` au même commit sur `https://github.com/guthubrx/t3code.git` | Réussi, confirmé |
| Capitalisation | `local-patch/bridget-headings-v0.0.45`, cherry-pick `b8ffa6f2806fe94b7bba8c15e97ab4f674aac2ec`, poussé au même fork | Réussie ; diff des quatre fichiers vide |
| Rejeu avant livraison |392 PASS en8,15s à12:07:12 CEST | Réussi |
| Paquet complet | Build223s, exit0 ; ZIP et SHA ci-dessous | Construit, pas encore installé |
| Candidat | Version/commit embarqué, signature stricte et données frontend confirmés | Validé, non installé |
| Installation | Job de promotion préparé après publication documentaire | Encore attendue |
| Activation | Redémarrage autorisé, contrôle durable prévu | Non effectuée ou non prouvée à ce stade |
| Docs Bridget143 | Commit `19296c48`, fusion fast-forward `main`, push `github` de `main` et `session-143-echanges-discrets` | Confirmés |

<a id="extrait-3"></a>

## Extrait 3

Paquet complet construit : `/Users/moi/.cache/t3-spec143-package.a0aX0E/T3-Code-0.0.45-local.143-arm64.zip`.
SHA256 : `109def5349098706175a5f51938541bbfcc2e49a45df401d39bcb2f9cce0934b`.

<a id="extrait-4"></a>

## Extrait 4

Sauvegarde de l'application existante effectuée : `/Users/moi/.cache/t3-adoptions/spec143-20261007.KfjhE4/T3 Code (Local before SPEC143).app`.
Ancien ASAR SHA256 : `dab141939a4171c3b4e7169fc68346b498ce179c414f9e2e8ee6acdb42f89ba1`.

<a id="extrait-5"></a>

## Extrait 5

Snapshot SQLite créé en lecture seule par `VACUUM INTO`, exit0 : `/Users/moi/.cache/t3-adoptions/spec143-20261007.KfjhE4/state-snapshot.sqlite`,12522332160octets, permissions0600. Contrôle `quick_check` terminé exit0, résultat `ok`. Marqueur de validation créé : `/Users/moi/.cache/t3-adoptions/spec143-20261007.KfjhE4/backup-validated.json`. Ces preuves de sauvegarde ne sont pas une activation de143.

<a id="extrait-6"></a>

## Extrait 6

Bundle candidat : `/Users/moi/.cache/t3-spec143-package.a0aX0E/staging/T3 Code (Alpha).app`.
Version : `0.0.45-local.143`. Commit embarqué : `a1a4f2ef12e2`.
Signature `codesign` stricte/deep : PASS.
ASAR SHA256 : `99cacfda54554e6b9f1500ac88db5ed4038efdec49850b9a3cfaacd66e174f68`.
Frontend chat SHA256 : `636252587511e6fe3ea1b492eb81700589ca1f644d05647ae25ad6547f1b7e9b`, identique octet par octet au build validé.

<a id="extrait-7"></a>

## Extrait 7

Reçu durable exact : `/Users/moi/.cache/t3-adoptions/spec143-20261007.KfjhE4/result.json`. Il fait autorité pour le résultat réel après le redémarrage. Sa réussite n'est pas anticipée ici. Activation supervisée autorisée et préparée, non lancée à cet instant. Candidat validé ; installation et restart encore attendus.

<a id="extrait-8"></a>

## Extrait 8

Le reçu /Users/moi/.cache/t3-adoptions/spec143-20261007.KfjhE4/result.json confirme deployed_health_ok à2026-10-07T10:36:54Z. /Applications/T3 Code (Local).app contient0.0.45-local.143 ; codesign strict/deep PASS ; ASAR99cacfda54554e6b9f1500ac88db5ed4038efdec49850b9a3cfaacd66e174f68 ; nouveau backend10491/parent10378 et environnement conservé3b1ba3d1-3df7-4d49-a17f-45154959c3a9 ; asset /assets/index-DgA9zaWk.js SHA2d8df3e24e9ab1394cda50a1a4eaa4917ad856c67d9890f813495a2f52b08602. La première tentative s'était arrêtée avant remplacement : comparaison des dates ps sans retrait des espaces finaux. Fix et précontrôle réel ont précédé le second lancement ; script final2bb6b40b127764d83695704076a1bfc40757ab11a64daab16483423277e3656a. Aucun résultat anticipé ni restauration de SQLite. Suivi préactivation983e40ea committé, fusionné et poussé avant le lancement.
