# Complément147 — accès MCP et pagination

Mandat du 2026-10-09 : corriger, committer, fusionner et pousser. Ne redémarrer ni T3 ni Bridget. Ce complément reste dans la session147.

## Cause vérifiée

L'agent Relance utilise Claude/GLM. L'application installée porte encore la version0.0.45-local.146. Le montage MCP Claude/GLM est déjà corrigé dans les sources147 de T3 ; aucun second montage ni changement global des configurations fournisseur n'est nécessaire.

L'activation147 précédente a échoué avant le remplacement de l'application, à la phase staging_path. Le reçu conserve aborted_before_replacement ; aucune base T3 n'a été restaurée ou copiée. Le job com.bridget.install147, arrêté en échec, a été retiré pour empêcher une activation ultérieure non demandée.

La pagination est déjà disponible : lire toute la page, confirmer son reçu avec ack, puis refaire le même read sur le même fil. Le serveur reprend après la dernière page confirmée. history sert à relire des preuves exactes et ne déplace pas ce repère. Une page est limitée à200 entrées et60Kio ; atteindre la borne d'octets peut produire moins de200 entrées.

## Correction

- CLI : l'aide décrit read → ack → même read. Le refus de --from-seq/--to-seq dans read explique cette suite et réserve les bornes à history.
- MCP : la description de bridget_thread et celles des bornes donnent la même procédure.
- Alertes : le texte reçu indique la répétition du même read après ACK.
- Skill et référence : découvrir aussi les outils différés du harnais, distinguer l'annuaire Bridget des outils internes Claude, ne pas déduire le catalogue du modèle GLM. Ne jamais ACK une page dont les corps ont été filtrés ou tronqués.
- Aucun changement de contrat, de permissions, de notifications ciblées, de données, d'Agent Loop ou d'acquittement automatique.

## Preuves initiales

Le nouveau test CLI a échoué sur l'ancien message : il ne contenait pas la suite ack → même read. Après correction, ce test passe.

Les tests T3 ClaudeMcp.test.ts et ClaudeAdapter.test.ts passent :194 réussites,1 ignoré. Ils vérifient le montage dans les sources147, pas la présence du MCP dans un agent actif sur146.

Le candidat T3 est reconstitué et sa signature est valide. Version0.0.45-local.147 ; SHA256 app.asar a1ae5da6e46a3f8dca1f77dc6eefaf9fae2e9695e1b06850770500773808cee2. Il reste en staging. L'application active146 n'a pas été remplacée ou relancée.

Les résultats Rust finaux et la publication des compétences sont consignés après exécution. Aucun succès de livraison active n'est déduit d'un paquet préparé.

## Suite globale : limite conservée

La première exécution Rust sans TMPDIR court a donné894 réussites,210 échecs et12 ignorés. De nombreux échecs signalent explicitement une socket Unix trop longue. Après correction du dossier temporaire du harnais, la commande `TMPDIR=/tmp CARGO_TARGET_DIR=/Volumes/SD1TO/bridget-build-147-rust cargo test -p bridget-daemon --lib -- --test-threads=4` donne1094 réussites,10 échecs et12 ignorés.

Les dix échecs concernent des tests de spawn, de supervision, de reprise et d'arrêt. Les erreurs comprennent auxiliary_credential_required, Operation not permitted sur des fichiers fleet privés et des délais de superviseur. Ils ne sont ni corrigés ni masqués par ce complément. La suite globale n'est pas déclarée verte. Les tests des zones modifiées, du montage MCP et de la pagination sont distingués de ce résultat.

## Vérification ciblée

- CLI de fils :11 tests passent.
- Moteur de fils :19 tests passent.
- MCP :54 tests passent,1 test explicite Agent Loop ignoré.
- Intégration réelle sur daemon privé :11 tests passent avec le filtre spec102_v1. Lecture incrémentale, page201, réponse perdue, lectures simultanées, matrice ACK, budget UTF-8 et history sans déplacement de repère sont couverts.
- Total Rust ciblé :95 réussites, aucun échec.
- Validation de la skill et format Rust : succès ; git diff --check sans erreur.

Les suites unitaires ciblées utilisent le binaire de test compilé par Cargo, avec TMPDIR=/tmp. L'intégration utilise cargo test -p bridget-daemon --test spec102_threads_test spec102_v1, avec le même CARGO_TARGET_DIR externe. Aucun daemon de production n'est lancé ou arrêté.
