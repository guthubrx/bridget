# Recherche — Session 056

## Sources locales mesurées

- L'ouverture immédiate et l'ouverture différée utilisent deux transactions
  distinctes mais convergent sur `upsert_objective`.
- Une routine construit directement `DelegateRequest` puis appelle `delegate` :
  une garde seulement MCP ou CLI serait contournable.
- Les objectifs sont stockés comme payload JSON sans colonne de provenance.
- Le ledger Bridget et le store Maicie sont deux bases SQLite distinctes ; un
  check dans l'une suivi d'un commit dans l'autre laisse une fenêtre TOCTOU.

## Sources primaires

- SQLite documente qu'une seule transaction d'écriture peut agir à la fois
  sur une base. Le futur gate doit donc partager la transaction d'insertion
  Maicie : <https://www.sqlite.org/lang_transaction.html>.
- SQLite décrit le commit atomique par journalisation ; cela ne transforme pas
  deux fichiers de bases indépendants en transaction distribuée :
  <https://www.sqlite.org/atomiccommit.html>.
- Serde permet un défaut de champ à la désérialisation. Il convient ici à
  `legacy_unknown`, à condition que le constructeur neuf ne réutilise jamais
  ce défaut : <https://serde.rs/field-attrs.html>.
- OWASP recommande un refus par défaut, une validation sur chaque requête et
  une garde centralisée. Le permit et l'inventaire structurel matérialisent ces
  trois contraintes : <https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html>.

## Décision issue de la recherche

La tranche 1 ne transporte pas une déclaration `human_request` fournie par
MCP/CLI. Tant que le daemon n'émet pas l'attestation causale, ce champ serait
une auto-attribution et non une preuve. Les producteurs actuels sont donc
explicitement `auto_generated` ; la voie humaine reste fermée.

La commande DevKMS `mem` n'est pas installée sur cette machine. Ces résultats
ne peuvent pas être capturés dans DevKMS pendant cette tranche ; ils voyagent
dans la présente spec et l'ADR.

