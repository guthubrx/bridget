# Contrôles thématiques — SPEC136

## Sécurité et fiabilité
Deux lectures des accès et migration. L'auteur vient de la connexion active,
le fil et son membre sont contrôlés avant la relation. Audience ancienne
persistée, cibles nouvelles normalisées ; différence = refus uniforme.
La validation précède ACK/INSERT dans une transaction ; index unique interdit
les branches. Erreurs de stockage bornées, aucun corps ajouté aux logs.
SQL nouveau paramétré ; aucune commande shell construite à partir du corps.
Les membres autorisés lisent l'histoire entière : notify n'est pas une ACL.
Aucun rejet ne réécrit un verdict de mission. Un reçu ne l'accepte pas.

Migration additive : anciens nouveaux champs NULL, index vide sur V1,
ALTER gardés, transaction IMMEDIATE et reprise idempotente. Pas de purge.
Rollback : conserver la DB migrée et les messages récents ; ne jamais remettre
une ancienne DB en place. Un retour à l'ancien binaire exige de suspendre la
coordination structurée : son read montrerait de nouveau les anciens corps.
La rétention normale du ledger reste indépendante ; sauvegardes SQLite prévues.

## Complexité, mémoire et performance
Huit anti-patterns recherchés dans les ajouts. Aucun N+1, récursion, tri répété,
concaténation en boucle, copie en cascade ou I/O fournisseur ajouté.
Comparaison de cibles ≤16 ; colonnes ≤10 ; boucle de page ≤200 et60Kio.
read_range fait une jointure indexée, O(log E + P log E). Post référence :
O(log E + M). Le budget sérialisé reste conservé, entrée jamais coupée.
EXPLAIN réellement asserté dans le test de migration, pas une indexation supposée.
Pas de benchmark p99 ni gain de temps de travail déduit des données.

## Qualité et duplication
Un warning MEDIUM QUAL-001 de longueur sur read_range (648–735).
post/migration/parse/tools restent des fonctions historiques longues ; pas de
refonte hors scope. Aucun nouveau stub, catch-all, TODO ou fichier utilitaire.
JSCPD :25932 lignes,87 clones,927 lignes dupliquées,3.5747339194817216%.
Top blocs :29/25/25/23/23 lignes, dans les anciens tests MCP hors hunks136.
Ajout136 repéré :9 lignes de préparation de tests distincts, classification
Type3/garder : même forme de fixture, scénarios évoluant indépendamment.
Les seuils globaux5/10% et blocs100/200 ne sont pas atteints.
Ne pas transformer les fixtures en framework pour satisfaire un indicateur.

## Tests et preuve d'exécution
RED cinq refus de contrat puis quatre échecs fonctionnels réels avant store.
Neuf tests ciblés PASS avant les renforcements. Première suite complète interrompue
sur attenteV1 ; deuxième sur ancien scénario socketDarwin. Cette dernière suite
de quatre tests relancée seule :4PASS, sans modifier son fichier.
Suite fraîche --no-fail-fast en cours ; aucune réussite globale présumée.
Outil cargo-llvm-cov absent ; pas de pourcentage de couverture de lignes inventé.
Scénarios Gherkin documentent le contrat ; assertions Rust l'exécutent.

## Minimalisme et vertus
Checklists1–6 exécutées sur les ajouts et les call-sites. Aucun état dérivé
persisté en plus de la référence source, aucun wrapper passthrough ni option
fictive. L'abstraction Option non-null porte une règle de compatibilité.
Enum publique partagé réel, pas de moteur de règles. Diff relu entièrement.
Potentiel minimalisme : ~0 lignes suppressibles à comportement constant.
Le coût futur baisse pour les lecteurs via corps courants et références exactes.
Le volume est expliqué par migration, snapshot, refus et interfaces communes.
Les invariants et limites sont assumables : explicite, sans modèle de résumé.
Aucune glue nouvelle non expliquée ; le protocole de classification est documenté.

## Résultat final
La suite fraîche --no-fail-fast est terminée : sortie0,0 échec,10 nouveaux tests136 PASS.
Source/test validés par empreintes, aucune correction de code pendant audit.
