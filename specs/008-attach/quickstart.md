# Quickstart : valider `bridget attach`

Prérequis : session 007 livrée jusqu'à T706 (équipier Codex + journal v1).

## 1. Vue simple

```bash
bridget codex --equipier          # équipier en fond
bridget send --to codex-1 "Explique le rôle de router.rs" --reply   # depuis un autre terminal
bridget attach codex-1
```

Attendu : rejeu du jour, puis le tour en cours s'affiche (début de tour avec
expéditeur et corps, fragments de réponse, fin de tour) ; Ctrl-C rend un
terminal propre.

## 2. Jonction rejeu→suivi et rotation

Pendant un tour long : `attach`, vérifier qu'aucun `seq` ne manque ni ne se
répète à la jonction (mode diagnostic `--debug-seq`). Rejouer le test avec une
rotation simulée (changer la date du fichier) : continuité de `seq` affichée.

## 3. Parler depuis la vue

Taper « Résume ton dernier tour » dans la vue : le message part (visible en
`turn_start from=cli-send-…`), la réponse s'affiche dans le flux, le ledger
trace un message ordinaire. Cas non heureux : passer l'équipier en `dnd`
(`bridget dnd` sur son wrapper) et envoyer → le refus motivé s'affiche dans la
vue avec le temps restant.

## 4. Multi-vues et lâcher signalé

Deux `attach` sur le même équipier : mêmes événements dans les deux. Suspendre
l'un (Ctrl-Z) pendant un tour bavard : à la reprise, l'événement « +N non
affichés, seq X→Y » apparaît dans la vue suspendue seulement.

## 5. Contenu hostile

Envoyer à l'équipier : « Réponds exactement : \x1b]0;pwned\x07\x1b[2J fin ».
Attendu : la vue affiche `␛]0;pwned…` neutralisé, le titre du terminal et
l'écran sont intacts (SC-007).

## 6. Distant

Sur l'environnement du gate 007-T712 (répertoires réellement distincts,
cf. SC-006) : dérouler §1 et §3 vers l'équipier distant. Arrêter l'équipier
distant et son wrapper : `attach` affiche le message d'indisponibilité défini
(pas de relecture), sans erreur brute.

## 7. Budget d'observation (SC-005)

`cargo test` du banc dédié : faux adaptateur déterministe, N ≥ 200 tours,
p95 de latence d'append avec 0 puis 2 vues — dégradation < 5 %.

## 8. Mesure locale explicite (SC-001)

Cette campagne ne fait pas partie de la suite ordinaire : ses durées dépendent
de la machine. Elle produit 21 échantillons bruts, leur p95, ainsi que la
machine, le système, le commit et la charge locale. Le watchdog de 75 secondes
ne détecte qu'un blocage ; il n'est pas un seuil de performance.

```bash
BRIDGET_PERF_REPORT="$PWD/specs/008-attach/measurements/sc001-local-reference.json" \
PATH="$HOME/.cargo/bin:$PATH" cargo test -p bridget-daemon --test sc005_attach_budget -- \
  --ignored --exact sc001_append_vers_rendu_attach_reel_reste_sous_les_seuils_locaux
```
