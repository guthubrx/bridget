# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/correctif-limite-fichiers149.md](../../../../../../../specs/149-sous-agents-lineage/validation/correctif-limite-fichiers149.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Code testé : commit `0d5107afcf186bacc57aefd1925cd98f076806b8`, clone propre
`/Users/moi/.cache/bridget-fd149.eksbzxke/source`. Le helper est
`/Users/moi/.cache/bridget-fd149.eksbzxke/source/crates/bridget-daemon/src/service_limits.rs` (49 lignes).
Il est appelé à `t3code.rs:1080` (pont `t3 serve`) et à `daemon.rs:4482` (daemon).

<a id="extrait-2"></a>

## Extrait 2

Environnement : `CARGO_TARGET_DIR=/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-fd-target`,
`CARGO_HOME=/Users/moi/.cargo`, `CARGO_BUILD_JOBS=2`, `CARGO_INCREMENTAL=0`,
`TMPDIR=/private/tmp/b149fd-tests-l8Gv` (droits 0700), `BRIDGET_BUILD_ID` et `BRIDGET_T3_*`/`T3CODE_*` retirés.
Compilation : 1 min 16. Exécution : 0,81 s.

<a id="extrait-3"></a>

## Extrait 3

- **Le pont `bridget t3 serve` n'est pas couvert.** Il exige un runtime T3 réel et un jeton. Le helper partagé
  est testé via le daemon seulement. Le branchement du pont (`t3code.rs:1080`) est vérifié par lecture, pas par exécution.
- **Pas de RED mesuré sur l'ancien binaire.** Aucun build de l'ancien code n'a été fait, comme convenu.
  La preuve est logique : un daemon limité à 256 ne peut pas tenir plus de 256 descripteurs.
  Le test en observe plus de 256 avec 120 clients (3 descripteurs par client).
- Les cas 4096 et 8192 supposent un hard du test d'au moins 8192. Ici il est illimité. Sinon le test échoue avec un message clair.
- La vérification de 256 vers plus de 1000 descripteurs sur le pont réel reste à faire après activation (côté principal).
- Warning de forme du snapshot `threadId` secondaire : non traité, hors périmètre.

<a id="extrait-4"></a>

## Extrait 4

Le binaire signé build `0d5107afcf18` est installé. Le daemon et le pont ont été relancés individuellement.
Le pont PID14501 expose322fils, contre63 avant. Il tient1295–1296descripteurs ; le daemon PID10624 en tient983.
Les deux annoncent `soft=4096` avec le plafond dur inchangé. Deux observations successives confirment un état `running` frais.
Aucune erreur `Too many open files` après leur ligne de démarrage corrigé. T3 PID60116/60228 est inchangé.
Le pont réel est donc désormais vérifié pour ce budget de fichiers. La limite du test isolé reste historique.
Le contrôle MCP hérité répond `identity_not_found` : ses variables d’attestation sont absentes. Les warnings `threadId` subsistent.
Preuve : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/limite-fichiers149-apres.json`.
Les chemins du clone, de la cible Cargo et des TMPDIR de test/build deviennent historiques après le nettoyage du principal.
