# Journal d’implémentation — Session 048

**Spec** : 048-identite-instance-daemon
**Branche** : session-048-identite-instance-daemon
**Base** : b00d417182622152bd9cc191ff98a7e1dca45ce4

## T1 — Oracle de remplacement local-vers-local

- **Statut** : terminé.
- **Preuve initiale** : vraie socket Unix, listener remplacé, même hôte,
  binaire et chemin de base ; le champ `instance_id` manque et l’oracle échoue
  à `0 passed; 1 failed`.
- **Preuve finale** : le même oracle ferme à `1 passed; 0 failed`.
- **Protection du harnais** : le nettoyage RAII retire socket et SQLite même
  lorsque l’assertion rouge panique.

## T2 — Contrat filaire

- **Statut** : terminé.
- **Modification** : `DaemonIdentityReport` contient désormais
  `host`, `db_path` et `instance_id`.
- **Preuve** : `protocol::tests::le_daemon_atteste_sa_machine_sa_base_et_son_instance`
  ferme à `1 passed; 0 failed`.

## T3 — État du daemon et rôles

- **Statut** : terminé.
- **Modification** : `DaemonState::new` produit `Uuid::new_v4()` une fois ;
  chaque rapport lit cette valeur conservée. La sonde interne de statut la
  décode également et la marque absente pour un daemon antérieur.
- **Preuve** : Client et Service reçoivent la valeur de l’état à
  `2 passed; 0 failed` dans l’oracle ciblé.

## T4 — Mutant et validations

- **Statut** : terminé.
- **Mutant** : remplacer l’UUID par `BUILD_ID` réutilise
  `b00d41718262-dirty` après redémarrage ; l’oracle échoue à
  `0 passed; 1 failed` sur l’inégalité exigée.
- **Restauration** : l’UUID rétabli repasse à `1 passed; 0 failed`.
- **Transport complet** : `181 passed; 1 ignored; 0 failed`.
- **Compatibilité Maicie ciblée** : `13 passed; 0 failed` ; les 405 tests
  Maicie ont compilé lors de l’inventaire.
- **Limite déclarée** : le banc complet daemon (univers 566) a affiché deux
  rouges hors périmètre puis s’est bloqué ; il a été arrêté après une attente
  bornée. Il ne sert pas de verdict de cette session.
