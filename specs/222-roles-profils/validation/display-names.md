# Noms humains dans les fiches — T035

Constat utilisateur : bridget specialists list présente le rôle et l'UUID, sans bridge1016. Lecture CLI réelle avant correction : trois spécialistes dont bc833746-3f24-41cd-bc64-46d60f4c0769, rôle mainteneur-bridget ; aucun champ display_name.

Correction : champ additif display_name dans chaque entrée list/search et dans get. Source : AgentProfileStore.profile_for_agent_id, le profil humain persistant ; aucun nom figé ni déduit du rôle. Un renommage est relu à la prochaine consultation, même hors ligne. L'identifiant et le rôle restent séparés. Les filtres de visibilité sont appliqués avant enrichissement ; aucun nouveau droit, changement de format persistant ou déclaration de rôle.

Vérifications et livraison à compléter. L'application Android ne change pas de parcours pour cette correction de réponse CLI/API.

Test ciblé unique PASS (0,02s) : list/search/get renvoient bridge1016 puis Gentil après renommage, même UUID et rôle patate ; fiche spécialiste strictement inchangée par les lectures, agent hors périmètre masqué. Première compilation de test corrigée pour comparer les sérialisations (SpecialistRecord n'implémente pas PartialEq), aucun code produit supplémentaire. Pas de campagne générale.

## Résultat réel et livraison
Source7fa71bffa2b84c975c0d4be062514b1ebd233a80 intégrée/poussée. Version0.7.2/build222-specialists-names-r1 active sur sept services. Sources Rust/manifests identiques (277fichiers, SHA6f381fe78f4b61253b0c6e9280586a862d37d12a09bd141f83b3af0ec17308a1).
- LinuxSHAeab9694b4dac416c2da494dfd7685dda3aee295c7b70723ee5cab78dc995313f : maître Cartae1619459/mobile1619495/worker1620170, SYS1419358, KS1718871.
- MacSHA5f297caca01bcf20cefadcf033e635159dbc45ed2cdc42dbf70c0cb7e9b17547 : worker82514/pont82673 ; updater83373. Images PID/chemin/SHA vérifiées.
- Coupure maître2,26s, superviseur Cartae2500061 préservé ; aucune commande non terminale sur SYS/KS au passage. Sauvegardes et formats aller/retour vérifiés, rollback natif vers0.7.1/recovery-r1 sous verrou sur Cartae/Mac.
- CLI réelle après activation : bridget specialists list retourne display_name=bridge1016, UUIDbc833746-3f24-41cd-bc64-46d60f4c0769 et rôle mainteneur-bridget. Aucune déclaration, désignation ou activation de spécialiste. Preuves HTML deux étapes texte vérifiées dans html_preview, navigation2/2 puis1/2 contrôlée ; sources brutes privées /Users/moi/.cache/bridget222-names.

Vérification allégée demandée explicitement : un seul test ciblé (0,02s) et circuit CLI réel, pas de campagne générale ni nouvelle mission/revue déléguée. Auto-relecture du delta ; aucune nouvelle revue indépendante revendiquée. Preuves ajoutées après activation sans réécrire le commit source déjà poussé.

Publicationv0.7.2 complète : six actifs cohérents, quatre natifs et APK/checksum. APK0.7.2/code7002, SHA61fa27a3f79c3af844821c162c70e2d6d894059eb1348fbc053f204e5e051dab, même certificat/lignée, configuration Firebase publique conservée. Garde publish-android.py PASS avant publication et --verify-published PASS après ; l'upload initial en brouillon rendait le tag non lisible par l'API tag, actifs relus par ID avant publication, aucun nouvel upload APK. Assemblage initial lancé avec ancien Gradle8.11 refusé par les verrous ; wrapper8.14.6 prévu utilisé, aucun changement des dépendances. Pas de nouveau circuit Android ni de test FCM, code UI inchangé. Installateur public sur7fa71bff, contenu vérifié, Caddy rechargé sans changement de PID.

Entretien installed Cartae/SYS terminé, caches3,44/1,05Gio conservés ; Mac conserve staging non enregistré sans forcer. Huit transferts redondants/195478474octets logiques retirés sur Mac/Cartae/KS, paquets publics et APK signés conservés sur Cartae. Aucun gain disque physique revendiqué. Actif/retour/proofs/tâches et worktree222 conservés pour les portes antérieures encore ouvertes et observations d'usage incomplètes. Aucun nouveau worktree créé pour cette correction.
