# Archives techniques de l’adaptateur T3

Document d’origine : [specs/140-bulles-compactes/converge.md](../../../../../../specs/140-bulles-compactes/converge.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Bridget : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes.
T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact.
Les chemins de la matrice sont relatifs à la racine indiquée, avec positions relues après correction.

<a id="extrait-2"></a>

## Extrait 2

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
