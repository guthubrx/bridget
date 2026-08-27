# Journal d'implémentation — Session 047

## Métadonnées

- Branche : `session-047-verdict-tete-reecrite`
- Base contractuelle : `bc745335530985ce305e82fea4007071c752d5b0`
- Statut : en cours

## Tranche 1 — Continuité du verdict

- Verdict typé relu depuis les octets terminaux de `guichet_receptions`, sans
  nouvelle colonne ni migration.
- Observation distante par `ls-remote`, puis ancêtralité dans un dépôt nu
  temporaire utilisant les objets locaux en lecture seule.
- États fermés : cible absente, verdict absent, ancêtre, réécrit et
  inobservable.
- Projection ajoutée à `maicie status` et à la carte de reprise.
- Les lectures SQLite sont groupées et les cibles Git identiques mises en
  cache pendant une campagne de statut.

## Preuves ciblées

- Empilement puis réécriture : 1 passé / 0 échec.
- Mutant égalité de SHA : 0 passé / 1 échec sur le cas empilé ; restauration
  attestée par SHA-256 identique
  `4705b6ef6c05b32fc4d05da1c5d816ee965338aca252fca8cc0a8589539f7d05`.
- Statut réel après réécriture : 1 passé / 0 échec.
- Carte de reprise, verdict absent puis alerte réécrite : 1 passé / 0 échec
  pour chacun des deux témoins.

## Reste à faire

- Propager atomiquement `ReviewTarget` sur le chemin MCP de délégation.
- Mesurer la compatibilité avec un daemon antérieur.
- Rebaser après séquençage des sessions 045 et 048, puis exécuter les gates et
  comptes base/tête complets.

## Tranche 2 — Contrat métier avant raccord filaire

- Maicie distingue désormais la délégation historique v1 de la délégation de
  revue v2 : v1 interdit une cible, v2 exige une cible valide, et aucune autre
  opération n'accepte v2.
- La canonisation conserve la version effectivement lue ; elle ne peut donc
  pas rabattre silencieusement une demande v2 sur v1.
- Le service de greffe transmet la cible au constructeur métier avant la
  transaction. Le témoin central relit ensuite la même cible depuis la
  délégation durable.
- La carte de reprise choisit le dépôt attesté par
  `review_project.repository_root`, jamais le worktree de l'agent comme source
  implicite du fait Git.

## Preuves ciblées de la tranche 2

- Matrice v1/v2 du parseur : 1 passé / 0 échec.
- Effet central avec politique chargée, faux daemon réel et relecture SQLite :
  1 passé / 0 échec.
- Mutant `review_target: None` au point d'effet : 0 passé / 1 échec sur
  l'assertion durable (`left: None`, cible exacte attendue), après mise en
  place réussie.
- Restauration du mutant : 1 passé / 0 échec et SHA-256 identique
  `afceb013b052e9c2275c32c2b1fccb8aa1c54e17ee991561a20f9b5dc7f1a414`.
- Carte de reprise existante après sélection de la racine configurée :
  1 passé / 0 échec.
