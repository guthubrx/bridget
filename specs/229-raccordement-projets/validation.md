# Validation en cours

## Cause constatée

Les admissions `mails-015-ontology-review-20261103-01` et
`mails-016-references-review-20261103-01` ont échoué à `candidate_mapping`.
Le registre source contient les trois correspondances Linux du projet.
Le registre du maître ne contient aucune de ces correspondances. Le résultat
`ProjectPrepared` perdait le résultat structuré déjà vérifié par le helper.
Identité native et projet de communication présents. Les deux clés restent
absentes des tâches et des reçus d'admission au contrôle en lecture seule.
Aucune demande métier rejouée, aucun fichier métier changé.

## Correction

La capacité `origin_project_context_v1` négocie la transmission du contexte.
Sans cette capacité, requête et réponse conservent leur trame historique.
Le maître compare l'autorité cible observée avant la préparation avec le reçu
du worker d'origine authentifié. Il valide la forme du mapping, puis publie
sous le verrou natif et par l'écrivain atomique existant. Le writer relit
l'autorité et le registre avant publication. Aucun écrasement divergent.
Une racine additionnelle ne peut être importée sans approbation existante
sur cette cible dans le registre du maître. Les droits du parent restent
bornés par son cwd attesté, y compris dans un worktree lié.

Un ancien worker conserve les projets déjà connus du maître. Son succès
sans contexte ne prouve pas le raccordement d'un nouveau projet : ce cas
nomme la correction à charger, sans déclarer identité ou projet inconnus.
La liste des capacités reste fermée ; sa borne passe de 16 à 32 pour porter
17 capacités connues. Aucune condition sur un numéro de version fournisseur.

## Contrôles acquis

- Quatre nouveaux cas Python ciblés : préparation et publication sur deux
  registres distincts, reprise identique, worktree, autorité/racine/droit
  additionnel divergents et absence d'écrasement.
- Module de préparation complet : 47 cas PASS en 14,714 s.
  Premier passage Mac : deux fixtures de verrou refusaient le lien symbolique
  du dossier temporaire système. Reprise avec TMPDIR physique `/private/tmp` :
  47 PASS, aucune garde du produit affaiblie.
- Première chaîne Rust : reçu sérialisé, validateur natif, helper réel et
  résolution du mapping dans un registre maître initialement vide PASS.
  Une compilation initiale a signalé un import Path manquant, corrigé.

Assemblages, revue et activation restent à consigner. Preuves privées :
/Users/moi/.cache/bridget229-project-relay.
