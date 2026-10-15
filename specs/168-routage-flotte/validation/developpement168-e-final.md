# SPEC168 — correctif final du stockage, T028

Ce reçu décrit les modifications source. Aucun oracle Rust n’a été exécuté par ce responsable. Les assertions décrites restent à vérifier par ROOT.

## Périmètre et propriété

Source modifiée exclusivement :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs

Le développement antérieur E est conservé. Les signatures CAS, les contraintes SQL et les formats historiques 163/v2 restent identiques. Aucune dépendance, table métier ou colonne n’est ajoutée par ce complément. Aucun fichier des autres responsables n’a été modifié.

SHA-256 avant ce complément :
`a4362c78770ac8c26301114098ec3f68b9212944730538956b0989af888a969c`

SHA-256 final après formatage :
`374210826fa0f09e5ca411855a753537905594875b6acf3a29a268e89799fd53`

Le diff Git cumulé comprend aussi le développement E antérieur. Ses compteurs ne mesurent donc pas seulement ce complément.

## Deux corrections

1. Après renommage SQLite, le nom connu apparaît entre guillemets dans CREATE TABLE. Le normaliseur accepte les deux écritures exactes de ce seul nom, nu ou cité. Le corps et ses contraintes restent comparés exactement. Aucun parseur SQL général ni assouplissement des contraintes.
2. La détection des clés étrangères entrantes utilise eq_ignore_ascii_case pour leur table cible. Une référence AUTO_ADMISSION_RECEIPTS ne contourne plus la garde.

## Scénarios source T028

- `delegation::tests::spec168_migre_163_sans_alterer_les_blobs_et_borne_v2` : constate le nom cité produit par le renommage ; compare canonical, metadata et delegate_canonical pour les cinq états migrés ; ajoute des octets non textuels aux BLOB ; ferme puis rouvre un reçu v2 dont les métadonnées dépassent 16 Kio.
- `delegation::tests::spec168_migration_faute_copie_rollback_conserve_legacy` : construit une ligne legacy dépassant sa borne via PRAGMA ignore_check_constraints réservé à la fixture ; réactive les contraintes avant ouverture du store. L’erreur attendue pendant la copie porte CASE metadata_version. Vérifie après rollback : ancien SQL, 17 colonnes, ligne et trois BLOB intacts ; aucune table intermédiaire ni aucun des trois index créés dans la transaction.
- `delegation::tests::spec168_reouverture_refuse_trigger_et_fk_entrante` : vérifie les refus avant remplacement du schéma, avec trigger, FK en minuscules et FK REFERENCES AUTO_ADMISSION_RECEIPTS(...) ON DELETE CASCADE. Vérifie les sentinelles parent/enfant et l’ancien SQL après refus.
- `delegation::tests::spec168_v2_persiste_decision_fermee_et_admission_atomique` : injecte un trigger de fixture sur la transition vers admitted. Il ne produit le message attendu que si la Task a déjà été insérée dans la transaction. Vérifie le rollback du reçu v2, l’absence de Task et de projection ; retire le trigger ; rouvre le fichier ; vérifie la même admission sans faute et sa persistance.
- `delegation::tests::spec168_cas_niveau_epoch_revision_et_choix_sont_fermes` : oracle existant conservé pour les gardes CAS, avec signature stable.
- `delegation::tests::spec168_refuse_schema_index_fk_sortante_et_preserve_la_base` : oracle existant conservé pour les schémas, index et clés étrangères sortantes.

Les injections restent dans le module de tests et réemploient SQLite et les fixtures existantes. Aucun failpoint de production.

## Contrôles réellement exécutés

Commandes terminées avec RC 0 et sans diagnostic :

```sh
/home/moi/.cargo/bin/rustfmt --edition 2024 --config skip_children=true /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs
git -C /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte diff --check -- crates/bridget-daemon/src/delegation.rs
```

La source a été relue après formatage. Ces contrôles ne prouvent ni la compilation ni le résultat des oracles.

## Commande future réservée à ROOT

Après restitution du permis de compilation et intégration des autres surfaces :

```sh
cd /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte
CARGO_TARGET_DIR=/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target CARGO_BUILD_JOBS=2 /home/moi/.cargo/bin/cargo test -p bridget-daemon --lib delegation::tests::spec168_ -- --test-threads=2
```

Le filtre doit exécuter les six oracles nommés ci-dessus. Un filtre ne sélectionnant aucun test ne constitue pas une preuve.

## Limites

Aucun Cargo, test, build, contrôle de production, changement Git ou redémarrage. Aucune tâche cochée. Aucun PASS de test ni succès global déduit.

Une base contenant des lignes metadata_version=2 interdit un retour à un ancien binaire incompatible. Ce complément ne modifie aucune procédure de restauration.
