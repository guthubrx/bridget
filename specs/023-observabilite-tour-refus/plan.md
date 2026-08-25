# Plan — Session 023

## Décision P1

Le journal local `/home/moi/.cache/bridget/sessions/<agent>/*.jsonl` est la
source existante. Le lecteur ne retient que les enveloppes v1 et les bornes de
tour corrélées par `message_id`. Il refuse de conclure sur une ligne invalide,
une séquence régressive, une version inconnue ou un conflit entre une borne
terminale et l'état instantané `busy`.

Le transport borne la force de la preuve : `codex_app_server` et `acp`
écrivent succès et échec, tandis que les wrappers interactifs n'écrivent que
l'ouverture. Une fin réussie est l'événement fermé `turn_end`. Une fin en
échec est un `error` portant le code fixe `terminal_kind=turn_failed`, ajouté
par le worker après l'issue du prompt. Les anomalies ACP restent des `error`
sans ce code. Pour les wrappers incomplets, seul `state=busy` confirme un tour
actif ; une ouverture ancienne isolée reste indéterminée.

Une `error` historique sans marqueur ne prouve ni la fin ni la continuation :
elle devient indéterminée. Une `update` corrélée plus récente est une preuve
positive que le tour a continué. Cette règle évite à la fois le faux BLOQUÉ et
le tour artificiellement immortel.

La catégorie `BLOQUES` reste une projection de ronde, pas un état du daemon.
Sa condition devient « dernier tour terminé sans reprise ». L'ancienneté d'une
remise reste affichée comme pièce contextuelle, mais ne décide plus.

Le détail fournisseur est conservé comme donnée, jamais comme code. Le rendu
texte remplace tout caractère Unicode de catégorie de contrôle par une
séquence visible ; le JSON emploie une sérialisation ASCII échappée, qui garde
la même valeur après parsing sans exécuter ESC ou un contrôle bidirectionnel.

## Décision P2

La frontière CLI possède les trois faits nécessaires sans nouvelle autorité :
la cible demandée, l'annuaire `AgentInfo` et les profils chargés. Elle traduit
le `TargetUnavailable` métier en un diagnostic fermé et exhaustif. La fonction
de sélection n'est pas modifiée.

Un cas impossible — refus alors que cible, profil et état satisfont toutes les
gardes — devient une divergence d'invariant explicitement nommée. Le domaine
reste observable, mais il n'est pas présenté comme une condition puisque le
code de sélection ne le lit pas.

## Paquets atomiques

1. Projection P1 : producteurs de bornes, `scripts/bridget-idle.py` et son
   harnais.
2. Diagnostic P2 : frontière CLI Maicie et tests unitaires de rendu.
3. Spécification et preuves mesurées.

Les deux paquets appartiennent à une seule session parce qu'ils appliquent la
même règle d'observabilité — une conclusion cite le fait qui la fonde — tout en
restant séparables par commit et par gate.

## Validation

- Exécuter le harnais P1 avant correctif, puis après correctif.
- Exécuter les tests Maicie ciblés avant correctif, puis après correctif.
- Muter la reconnaissance d'une borne terminale : le témoin P1 doit rougir.
- Retirer ou altérer `terminal_kind` : le faux ACP doit rester sans terminal et
  le témoin doit rougir sur le code fermé manquant.
- Injecter ESC, retour chariot et contrôles bidirectionnels dans les deux
  détails libres ; attester leur présence en entrée et leur inertie en sortie.
- Muter le diagnostic `busy` en message générique : le témoin P2 doit rougir.
- Lancer `cargo test --no-run` avant tout comptage Rust.
- Mesurer les suites ciblées puis le workspace complet, avec passés, rouges et
  ignorés, et imputer chaque rouge.
- Mesurer la composition avec les têtes gelées 019, 021 et 022.
