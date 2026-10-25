# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-network-proofs-r3.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-network-proofs-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Binaire release r8 immuable : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb`, SHA-256 `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6`, égal au reçu `native149-release-receipt-r8.json` au début et à la fin de chaque recette (`native-network-recipes/results-r3/shacheck.log`) et à la fin de la ronde (19:51:47).
- Production Rust : empreinte `3943009ca82db59d...` (égale au reçu r8) au début. **À 19:19:03, `crates/bridget-daemon/src/wrapper.rs` a été modifié par un autre propriétaire** (digest `49f3a304d9504fe6`). L'empreinte devient `4fa56fcac7b37220f4d27cf15a8cdd94c8e434d92d86261b32a508a901f3adfa`. Aucun autre fichier. Mes recettes ne touchent aucun fichier de production. Toutes les preuves valent pour le binaire r8 seulement. Un correctif de wrapper (par exemple pour O4) demandera un binaire neuf et une nouvelle ronde : la preuve actuelle ne ferme pas ce code.
- Sources T3 : HEAD `33f6d04e11`, `git diff HEAD` `59a54267ce7c1d2a` avant et après (inchangé).
- Détail : `native-network-recipes/results-r3/fingerprints.json`.

<a id="extrait-2"></a>

## Extrait 2

Total de contrôles : 143 = 131 RÉEL PASS + 2 RÉEL FAIL + 2 SIMULÉ PASS + 1 SIMULÉ FAIL + 7 OBS (les OBS ne sont ni PASS ni FAIL au sens des trois colonnes ; ils sont comptés PASS dans les tableaux de recette parce qu'ils n'ont pas de critère). La recette UI (T040) est mesurée avec les outils de la preview T3 : voir son rapport, pas de compteur automatique.
Des échecs de passes antérieures (recovery passe 1 : 5, passe 2 : 2) étaient des défauts de mes prédicats, corrigés (détail dans `recovery149.md`).

<a id="extrait-3"></a>

## Extrait 3

- T036 : sessions T3 scopées -> daemon réel, union v1/v2, rotation, révocation, identité étrangère, projet croisé, un seul serveur T3 qui émet le credential et sert Lineage.
- F1 (reprise sans doublon) : R9.2 a à f. 1 tour, 1 PID, lancements 12 -> 12, tâche `failed/unreachable` dès le démarrage, exécution durable `running` -> `unreachable/daemon_restart`, 1 notice, rejeu sans lancement.
- **F2 corrigé** (R9.4.a à i, sans aucune mutation de base) : parent hors ligne, la racine retenue sort de l'attente en 4,7 s et garde son résultat ; au retour du parent : UNE remise exacte ; 2e redémarrage : la remise accusée n'est pas rejouée ; exécutions avant/après cohérentes (16/16, aucune rouverte ni active).
- Annulation, retry, refus de fournisseur, parent extérieur, identité inconnue, droits réduits, panne de T3 indépendante (six scénarios US6) : PASS.
- O1/G-P-07(b) : le comportement est conforme au contrat clarifié (R8.4).
- O5/G8 : `journal --follow` de la CLI réelle (7/7). Panne T3 : zéro tour T3 (4 PASS).
- UI web native : journal réel en direct, nesting, modèles, statuts, arrêt qui termine le PID réel, reconnexion après redémarrage de T3, zéro champ de saisie.
