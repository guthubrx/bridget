# Convergence SPEC140 — Passage1

Date :2026-10-07. Issue :CONVERGED. Relecture réelle par le principal après la dernière correction :spec, plan, modèle, contrat, checklist, code et tests.11 exigences couvertes. Aucun manque ajouté ni code/tâche modifié pendant cette phase.

SHA256 tasks avant et après :cc8a04c49f743637360a96daa59478920056c751121520d97a9039ca8ad65238. À ce passage,13/14 cases étaient cochées ; T014 attendait le gate strict final de l'audit. Ce résultat de convergence n'est pas une installation ou un commit.

## Racines des références

Bridget : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes.
T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact.
Les chemins de la matrice sont relatifs à la racine indiquée, avec positions relues après correction.

## Matrice exigence → code → vérification

| Exigence | Réalisation | Test ou preuve |
|---|---|---|
| FR14001 cinq familles, compacte droite | T3 apps/web/src/components/chat/MessagesTimeline.logic.ts:73,109 ; MessagesTimeline.tsx:4034 | Logic.test.ts:115 ; MessagesTimeline.test.tsx:295, vrais cartes repliées44px |
| FR14002 logo/thèmes/320 | T3 MessagesTimeline.tsx:2124,4056 ; apps/web/src/assets/bridget-logo.svg | cmp officiel identique ; mobile320×800,12 cartes44px, débordement cartes et document0 ; clair/sombre observés |
| FR14003 clavier/focus/ARIA | T3 MessagesTimeline.tsx:4056 bouton natif, aria-expanded/controls | Tests UI:295 ; Enter/Space Browser T3, focus conservé |
| FR14004 corps/brut/copie inchangés | T3 MessagesTimeline.tsx:4090 renderer existant,4109 détails texte,2270 copie | UI:295 texte brut exact, copie source ; Browser carte fermée7357 exact au GET, mock clipboard restauré |
| FR14005 libellés fiables | T3 MessagesTimeline.logic.ts:113,150 | Logic.test.ts:166,190 ;12 cas délégués RED/GREEN et six suffixes inconnus préservés ; aucune lecture du corps comme type mission |
| FR14006 titre à remise et audience | Bridget crates/bridget-core/src/message.rs:95 ; daemon.rs:3964,7515,14348 ; store/threads.rs:1216 | daemon.rs:16336 membre/non-membre/direct ;16454 erreur DB non bloquante ; browser Coordination politique/Fil partagé |
| FR14007 JSON/borne/canon | Bridget t3code.rs:3095 ; message.rs:241 compatibilité ; communication.rs:449 canon | t3code.rs:3472 hostile/borne ; daemon.rs:16366 replay durable,16502 titre client absent des octets persistés |
| FR14008 parsing visuel borné/fallback | T3 MessagesTimeline.logic.ts:100,109 | Logic.test.ts:130 négatifs,135 CRLF ; préfixes cités/incomplets et>1024 rejetés, pas preuve d'identité |
| FR14009 état isolé/ancrage | T3 MessagesTimeline.tsx:2239 key fil+message,4049 callback ; rowSize existant conservé | MessagesTimeline.test.tsx:361 changement fil/recyclage ; browser second fil seule carte Psychologie deux repliée ; long corps top623stable et scroll1795stable |
| FR14010 pas nouveaux services/dépendances | Diff et manifeste des deux worktrees ; composants, Store.thread_show et logo existants | Reuse-audit PASS ; aucune API, migration SQL ou dépendance nouvelle ; champ racine additif seul |
| FR14011 essais isolés | BRIDGET_HOME=/tmp/b140.uk4wWV ; aperçu /Users/moi/.cache/t3-spec140-preview.9yd2HW | Serveur arrêté/sauvegarde avant SQL ; suppression de seule clé IndexedDB synthétique identifiée ; aucun provider/process active userdata/production utilisé |

## Limites conservées

269 tests frontend et67 Rust distincts ciblés, pas suite globale ni mesure instrumentée de couverture. Lint22 warnings baseline identiques ; build web chunks>500ko non bloquant. API32 messages mais DOM16 desktop et12 mobile :virtualisation active, pas32simultanées ni benchmark recyclage complet. Corps10ko automatique et7ko réel DOM. Le clic détails7ko a d'abord échoué côté client Browser ; après réouverture, brutDOM7357 exact au GET confirmé. Snapshot/savePNG finale a échoué deux fois côté outil :aucune capture finale revendiquée. Sources gelées après contrôles, manifesteSHA256450f753a86d0df510cc7ee002d575d4043d95556018ef18a43c1df9f5ce76594.

Revue inter-fournisseurs indisponible, revue locale finale APPROVE. Audit final et validation stricte clôturés par le principal :exit0, zéro erreur/zéro warning. Audit réellement readonly, Phase09 non exécutée car gate clean non rempli et aucun finding actuel ; l'historique du défaut appartient à Implement/contre-revue. Aucun desktop build, installation, commit ou déploiement.

## Passage2 final — 06:36 CEST

Issue :CONVERGED. Le principal a relu les artefacts, la checklist et le code gelé après validation du gate audit et clôture de T014. Neuf sources inchangées (manifeste450f753a86d0df510cc7ee002d575d4043d95556018ef18a43c1df9f5ce76594). Les11FR restent couvertes ; aucun manque à ajouter.

Tasks avant/après strictement identique :ee8836218d9d227d79700dab06e89ef59afc19689edd636114437585bd04056a. Aucune écriture de code ou tasks pendant ce passage. L'ancien hash du passage1 reste consigné plus haut ; le changement entre passages correspond à la clôture explicite de T014, pas à Converge.

Statut final :Implemented,14/14 tâches, première tâche non cochée :aucune. Deux passages CONVERGED. Ce statut n'est ni un commit ni une livraison production.
