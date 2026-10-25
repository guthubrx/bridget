# Recherche 101

Date : 2026-09-16. Exploration locale en lecture seule ; aucun fournisseur arrêté.

## Décisions et preuves

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/research.md#extrait-1).

## Sources primaires consultées

Baseline utilisateur : security-compliance et testing-quality ; principes retenus,
pas de métriques anciennes utilisées comme chiffres actuels.

- https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html :
  refus par défaut et vérification de l'autorité ; aucun élargissement de droits.
- https://www.cisa.gov/sites/default/files/2023-06/principles_approaches_for_security-by-design-default_508c.pdf :
  responsabilité produit plutôt que configuration de sécurité répétée à l'utilisateur.
- https://martinfowler.com/articles/practical-test-pyramid.html : tests aux frontières
  de sérialisation et intégrations réelles, distincts des tests unitaires.
- https://testing.googleblog.com/2008/03/tott-understanding-your-coverage-data.html :
  couverture de lignes différente de preuve du comportement.

## Outillage indisponible et substituts

Scripts/modèles `.specify/scripts` et `.specify/templates` absents (vérifiés ls),
`mem` absent (command -v), Sequential Thinking absent du catalogue : protocole
SpecKit manuel et décisions écrites ici. Aucun runtime réinstallé.
Contre-revue autre fournisseur : canal MCP refuse actuellement l'identité ;
à retenter après correction, pas de CLI usurpant un autre agent.

## Vérification du contrat réel pendant implémentation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/research.md#extrait-2).

## Preuves après adoption101 autorisée

Le16septembre à17:02CEST : CodexT3 reconnu par son MCP préexistant (abonnement
Horizon propriétaire exact), Claude natifT3 reconnu par CLI (MCP absent dans son
catalogue, repli annoncé par le témoin). Réception réelle dans filClaude bdget,
4s après observation de fin, une seule notification ponctuelle. La limitation
node/bun n'est pas levée par ce test. Aucun fournisseur existant redémarré.
Le refus lsof est sûr et journalisé précisément dans le pont ; le client voit
encore un refus d'identité générique. Limite ergonomique déclarée, pas un succès
fictif ni une raison d'affaiblir l'attestation.
