# Tâches 120
Statut: Implemented - 5/5.
- [x] T001 Diagnostic de l'incident du 25/09 05:18Z : doublon Alpha ouvert par `getApp`, tours marqués en échec, fichier d'état effacé ; workers vérifiés actifs par processus et transcrits.
- [x] T002 `restore_declaration`, `server_alive` : déclaration de notre serveur rétablie si absente ou morte, jamais sur une déclaration valide (`t3code.rs`).
- [x] T003 `app_bundle`, `notify_user` : notification macOS nommant le paquet `.app` du doublon, jamais en test (`BRIDGET_T3_NO_NOTIFY` pour les recettes) (`t3code.rs`).
- [x] T004 Consigne « Ne jamais ouvrir une seconde application T3 » dans `skills/bridget/SKILL.md`.
- [x] T005 Test `spec120_declaration_retablie_seulement_si_absente_et_serveur_vivant` ; `app_bundle` vérifié sur les processus réels (T3 Code (Local).app, Finder.app) ; t3code 87/87 ; fmt, clippy ; recette 1550 réussis, 3 échecs `search_104_test` (instabilité préexistante mesurée en 119).
