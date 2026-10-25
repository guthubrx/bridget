# Archives techniques de l’adaptateur T3

Document d’origine : [specs/099-fiabilite-communications/implementation.md](../../../../../../specs/099-fiabilite-communications/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T009 terminée : la quatrième régression daemon passe, module 4/4 PASS en 1,15 s.
La notification est structurée pour mode ACP OU transport t3code ; aucun autre
transport CLI n'est modifié. La compilation initiale avait croisé des champs
de tests encore en migration, pas un échec de l'oracle.

<a id="extrait-2"></a>

## Extrait 2

Recettes après correction (vrai binaire debug, aucun compte fournisseur) :
- reproduce_daemon.py --expect-fixed : PASS, /tmp/bg-audit-behy875h. Identité
  MCP déclarative refusée ; auxiliaire avec preuve réelle accepté. Annuaire
  0,22 ms au repos et 0,78 ms pendant cible saturée ; Nack sur écriture échouée.
  Le processus enfant s'est terminé, code 0.
- reproduce_t3.py --expect-fixed : PASS, /tmp/bg-t3-audit-kwgrk5r3. Annulation
  confirmée, zéro démarrage du travail annulé ; réponse sauvegardée pendant
  déconnexion, remise après redémarrage et demande answered. Une seule
  exécution fournisseur. Les quatre processus possédés sont arrêtés et vérifiés.
- Protocole transport : 67/67 PASS ; cœur : 39/39 PASS après correction.

<a id="extrait-3"></a>

## Extrait 3

Relecture indépendante de même fournisseur : deux cas limites supplémentaires
retenus avant clôture US4/US5 : conserver le besoin de publication après refus
enqueue sur fil inchangé ; ne pas effacer Pending sur réponse HTTP 2xx malformée
ou erreur ambiguë après acceptation possible. Deux tests rouges avant correction,
puis verts ; module t3code 25/25 PASS dont 13 nouvelles régressions.
Le refus enqueue est simulé par un writer arrêté, pas une saturation réelle ;
la branche d'erreur et sa reprise à clé inchangée sont vérifiées. Le journal
arrête le lot au premier refus pour préserver l'ordre.

<a id="extrait-4"></a>

## Extrait 4

Recette enrichie /tmp/bg-t3-audit-1dewkaac : demande expirée et perte du daemon
pendant une vraie attente busy n'ouvrent pas le travail original. Cependant,
l'oracle initial était trop faible : dispatch.jsonl contient un tour de rappel
créé par le daemon après expiration (commandId 1ae8b0e562684). Le PASS de ce
premier script enrichi ne vaut donc PAS validation complète SC03. Oracle renforcé
à zéro tour de rappel ; la correction daemon ciblée et le nouveau test rouge/vert
ont ensuite résolu ce défaut, sans changer les rappels des autres transports.

<a id="extrait-5"></a>

## Extrait 5

T004–T006 : les trois voies d'emprunt d'identité sont refusées, la preuve valide
fonctionne et la rotation invalide les connexions déjà admises. Réutilisation des
contrôles de connexion, du stockage privé et des clients ; seul le type secret et
la trame d'attestation sont nouveaux. Tests supplémentaires : rôles Attach/Service,
preuve d'une autre instance, secret absent de Debug/annuaire, fichier 0600 borné et
sans symlink. Les mocks de protocole ont une preuve synthétique vérifiée ; tout
test parlant au vrai daemon obtient sa preuve par Registered. Aucun bypass de
production. Les tests ont été écrits avant correction, mais leur première
compilation a croisé le travail t3 ; la preuve rouge initiale est aussi la recette
réelle d'usurpation. Non vérifié : tunnel SSH réel, compte OS hostile.

<a id="extrait-6"></a>

## Extrait 6

Emplacement des recettes T007/T010 ajusté avant convergence : unités dans
t3code.rs et vrais processus dans reproduce_t3.py du dossier de preuves, au lieu
du harnais t3code_098_test.rs qui réutilise les arrêts SIGKILL. Même faux serveur
HTTP et mêmes scénarios fonctionnels, aucune réduction de périmètre. Ces recettes
sont conservées comme artefacts d'audit locaux ; les régressions Rust font partie
du diff versionnable et ne dépendent pas du dossier d'audit.

<a id="extrait-7"></a>

## Extrait 7

Recettes finales sur le dernier binaire :
- reproduce_daemon.py --expect-fixed : PASS, /tmp/bg-audit-4j93yaxz ; annuaire
  0,24 ms au repos et 1,97 ms sous pression, identité illégitime refusée,
  rattachement valide accepté, Nack sur remise échouée ; enfant terminé code 0.
- reproduce_t3.py --expect-fixed : PASS, /tmp/bg-t3-audit-fz7xnjmr ; zéro tour
  après annulation, expiration et perte du daemon durant une attente busy.
  Aucun tour de rappel parasite. Une réponse durable remise après redémarrage,
  demande audit-lost-79ab2f37 answered, exactement un dispatch fournisseur.
  Les quatre enfants possédés ont tous été arrêtés et leur sortie vérifiée.

<a id="extrait-8"></a>

## Extrait 8

| Exigence | Réalisation | Preuve |
|---|---|---|
| FR01 | crates/bridget-daemon/src/daemon.rs:1518 et :11856 | socket saturée/writer occupé :8882/:8887 ; recette daemon |
| FR02 | crates/bridget-daemon/src/daemon.rs:12108 | Nack attesté sur vraie écriture échouée |
| FR03 | crates/bridget-daemon/src/daemon.rs:7419, appel avant push | test writer occupé vérifie tracked_requests avant remise ; réponse liée :8927 |
| FR04 | crates/bridget-daemon/src/daemon.rs:7232/:7244/:7465 | cinq tests d'autorisation :25003 et suivants ; recette auxiliaire réelle |
| FR05 | crates/bridget-daemon/src/daemon.rs:2957 ; mcp_identity.rs:79/:113 | rotation/même instance :25047 ; fichier :618 ; protocole :3880 |
| FR06 | crates/bridget-daemon/src/communication/client.rs:773 ; wrapper.rs:1632 | MCP43, client13, reconnexion2, canon CLI réel ; README bilingue |
| FR07 | crates/bridget-daemon/src/t3code.rs:1452/:1575 ; daemon.rs:6389 | contrôles :2283/:2315/:2483 ; rappel :20963 ; recette busy complète |
| FR08 | crates/bridget-daemon/src/t3code.rs:1892/:1936 | reprise :2437 ; HTTP ambigu :2643 ; recette redémarrage |
| FR09 | crates/bridget-daemon/src/t3code.rs:186/:220 | deux attentes :2192 ; ancien état :2473 |
| FR10 | crates/bridget-daemon/src/t3code.rs:1719/:1814 | Unicode :2237 ; refus :2268 ; panne :2364 ; reprise :2397/:2602 ; gap :2511 |
| FR11 | reuse-audit.md et diff des manifests vide | aucune dépendance, table ou service ; types/maps existants étendus |
| FR12 | fixtures socketpair et recettes Python dans audits/2026-09-16/session-2026-09-16-bridget-global-01 | états privés, faux HTTP, SIGTERM individuel, aucun déploiement |
