# Validation 170

## Résultats ciblés

- `remote170_ecriture_suivi_echoue_puis_reprise_entretien` : PASS — panne SQL
  de clôture, tâche déjà nettoyée, tick réel, reprise sans rejeu de travail,
  `pending_replies` vidé.
- `remote162_resultat_canonique_notifie_parent_sans_recapture_legacy` : PASS —
  résultat accepté avant deadline, `answered`, `delivery_acked=false`, zéro
  action Gentle/Firm/Deferred/Timeout après deadline synthétique.
- `remote162_capture_dispositions_terminales_prouvees` : PASS — annulation,
  ronde obsolète, exécution obsolète et vraie expiration restent non clôturées.
- `remote162_capture_commit_rejeu_parent_et_wrapper_absents` : PASS — rejeu
  exact et récupération du reçu sans nouvelle remise de travail.
- `remote170_recu_accepte_etranger_ne_cloture_pas_le_guichet` : PASS — un reçu
  `Accepted` dont la mission ou la ronde ne correspond pas reste sans effet.
- `remote170_demande_sans_resultat_expire_vraiment` : PASS — un suivi distant
  réellement sans résultat produit bien son timeout.

La garde R2 est évaluée avant le collecteur : pendant l’échec de `mark_answered`,
le résultat accepté durable ne produit aucun `Gentle`, `Firm`, `Deferred` ou
`Timeout`; le tick suivant clôture puis retire le pending après commit.

Warnings de compilation : warnings hérités du banc (interfaces privées,
fonctions historiques non utilisées/noms non snake case) ; aucun finding lié au
diff 170.

## Commande commune

```text
CARGO_TARGET_DIR=/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target CARGO_BUILD_JOBS=2 cargo test --locked --offline -p bridget-daemon --lib <filtre> -- --test-threads=2
```

## Cas non qualifié

Un résultat distant vide est refusé par la validation existante du protocole
avant `capture_remote_result`. Ce comportement est attesté et reste hors du
périmètre de la correction 170 ; aucune nouvelle capacité vide n’est requise.
