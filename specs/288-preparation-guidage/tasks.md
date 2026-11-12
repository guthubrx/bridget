# Correction 288 : diagnostic de préparation

- [x] Corréler le refus 97d5b971 avec le journal worker : held_bootstrap, held=1, prepared=0/128.
- [x] Préciser le guidage sans inventer de sous-cause dans le reçu.
- [x] Tests ciblés, Clippy et release Mac/Linux ; APK 8052 signé et publication vérifiée.
- [x] Activer le maître, Cartae, KSSTORE et les deux Mac ; vérifier le retour et les processus conservés.
- [ ] Activer SYSLE2 : retenu par client_check_unavailable / probe_timeout, contrôle de version Claude à 6,69 s pour une borne de 2 s. Worker 287 préservé.
- [x] Sources de correction alignées sur les trois checkouts Linux ; ménage gardé et reçus conservés.
- [ ] Corriger la course entre redémarrage du connecteur et remise initiale : incident a716ff89 documenté, parcours ponctuel suspendu.
