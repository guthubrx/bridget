# Journal d'implémentation 135

## 2026-10-05 — Préparation

- Worktree isolé créé depuis `main` au commit `e5d48591`.
- Cause confirmée : `notify_signature()` inclut `age_bucket`. Le même problème
  devient donc une nouvelle signature à chaque intervalle.
- Le heartbeat actuel n'a pas d'état distinct pour ACK, progrès ou escalade.
- La boucle Politique possède 42 tâches `pass`, 59 `review`, 36 `pending`, une
  `running` et une `dispatched`. Son LaunchAgent appelle le script officiel via
  un wrapper `runpy` et peut donc prendre la correction en direct.
- Le dernier passage réel termine avec `orchestrator_missing`.

## 2026-10-05 — Contre-revue du plan

- Verdict externe : `APPROVE_WITH_CHANGES`.
- La clé d'anomalie est maintenant définie par `event + task_id`.
- Un destinataire injoignable provoque une escalade immédiate et durable.
- La stratégie de test couvre la séparation des pauses par nature de travail.

## Implémentation et preuves

Statut : Implemented, 11/11 tâches vérifiées. La livraison concerne le
contrôleur de la boucle, pas l'acceptation des travaux métier Politique.

- Métadonnées nouvelles : responsable, échéance, résultat attendu et nature
  du travail. ACK 120 s ; progrès vérifiable 300 s ; contrôle toutes les 60 s.
- Registre d'anomalies event+task_id, digest unique par destinataire et silence
  jusqu'au changement d'étape ou de preuve. Worker, coordinateur puis ROOT.
- Preuves réelles et non rejouables, y compris A/B/A. Empreintes par tentative.
- `disposition` conserve le verdict. `close-run` refuse toute obligation ouverte.
- Lecture sous flock et fusion atomique des champs : tests RED puis GREEN pour
  le résultat concurrent, l'ACK périmé et la migration pendant un résultat.
- Outbox stable, identifiant idempotent Bridget et reçu accepté avant la
  persistance finale : crash après remise couvert sans nouvel envoi.
- Sélection de rôle pure en dry-run ; digest multi-rôles ; reprise legacy datée.

## Validation finale

- 99 tests unittest PASS ; compilation Python PASS ; skill validée ; diff-check
  PASS. Tests sur fichiers temporaires réels et transports externes simulés.
- Cinq contre-revues externes : plan APPROVE_WITH_CHANGES, puis quatre APPROVE
  ciblés après vérification. Aucun défaut confirmé ouvert.
- Analyze manuel : le script de prérequis SpecKit est absent dans ce dépôt.
  Comparaison et relecture 14/14 exigences ; détails dans analyze-report.md.
- Audit v14 final ciblé et readonly : 0 erreur, 0 warning au validateur.
  Le mode fix du protocole exige un dépôt propre ; les changements étrangers
  de dotfiles sont préservés. Les corrections ont été faites dans l'implémentation.
- Couverture de lignes non mesurée : module coverage absent. Pas de qualification
  générale des fournisseurs LLM ou du transport tmux historique.

## Mise en service Politique — 10:48 CEST

Sauvegarde complète et comparaison avant mutation :
/Users/moi/Documents/opus2D/gouvernance/agent-loop-backups/spec135-20261005-ItTInI

Run actif :
/Users/moi/Documents/opus2D/gouvernance/agent-loops/politique-20261004

LaunchAgent :
/Users/moi/Library/LaunchAgents/local.agent-loop.politique-20261004.plist

- Ancien lancement suspendu alors qu'il ne tournait pas. ROOT rattaché et run
  migré à 08:48:17 UTC. Appel direct au script officiel, intervalle 60 s au lieu
  de 600 s. Anciens wrappers conservés, mais plus appelés.
- Premier passage réel : exit0, 1 digest pour 2 actions. Deuxième : exit0,
  aucun nouvel envoi. Le coordinateur a ensuite enregistré dix blocages avec
  responsable et prochain contrôle ; aucun verdict transformé en succès.
- Un refus de transport à 08:50 UTC a révélé le fallback d'une alerte démarrant
  au coordinateur : elle retombait sur lui et attendait un passage supplémentaire.
  Cause du refus de transport initial non identifiée. Le défaut d'escalade est
  confirmé par test RED, puis corrigé : destinataire déjà failed sauté jusqu'à
  ROOT dans le même passage. Les diagnostics gardent uniquement type fermé et
  code numérique, pas stderr ou commande libre.
- LaunchAgent suspendu pendant ce correctif, puis relancé à 10:54:40 CEST.
  Quatre passages réels contrôlés ensuite : exit0 et aucun nouvel envoi sur
  l'état inchangé. La dernière contre-revue approuve cette correction.
- Résultats comparés à la sauvegarde : identiques. Projections task_id/status/
  attempts : identiques. 142 tâches conservées : 42 pass, 65 review, 35 pending.
  Le chantier reste ouvert. L'automate ne déclare pas ces missions acceptées.
- Aucun octet ajouté au journal stderr depuis la sauvegarde. Ses anciennes
  erreurs de wrapper ne sont pas effacées ni présentées comme des erreurs neuves.
- Règles transmises au coordinateur et à ROOT via Bridget, messages
  0bf7939ab3df4 et 6725b08861ca4. Les workers seront instruits lors de leur prochaine
  attribution, sans réveil général.

## Convergence et livraison

Convergence manuelle finale : code réel, tests et 14 exigences relus. CONVERGED,
un passage formel, aucune tâche ajoutée. tasks.md byte-identique avant/après,
SHA256 ec06cbd1a29c24290213f3aaedd8280e9a701c5fcb88105175fa0d7a5bc356f8.

La source de skill vit dans /Users/moi/dotfiles et les artefacts dans le worktree
Bridget. Les modifications restent non committées. Aucun commit, merge ou push
automatique dans ce pipeline ; aucun changement étranger ajouté ou supprimé.

## ETA et limites

ETA initiale 53–90 min. Après Tasks, fin recalibrée 10:30–10:50 CEST. La clôture
se termine vers 11:00 CEST, soit environ dix minutes après la borne recalibrée.
Cause : corrections prouvées de concurrence, de rejeu et cas réel de transport.
La durée totale depuis le tout premier échange n'est pas mesurable avec les
traces chargées ; aucun pourcentage d'écart n'est inventé. Première spec sur
disque à 09:38:51 CEST : cette borne ne représente pas le début du travail.

Le contrôleur signale et escalade ; il ne force pas un modèle à travailler ni
ne juge la qualité métier. Le délai observé peut ajouter jusqu'à un intervalle
de contrôle au seuil configuré, hors panne ou ralentissement du transport.
Le retour arrière ne remplace jamais le registre actif par une copie devenue
ancienne : restaurer lancement/politiques en conservant résultats et événements.

## 2026-10-06 — Reprise systémique des producteurs de rappels

Demande utilisateur : corriger les chemins réels, pas seulement les messages
signalés. Contrainte ajoutée : préserver les relances utiles.

Causes prouvées :

- Politique utilise le contrôleur canonique ; Psychologie utilisait un détecteur
  ET un émetteur séparés. Celui-ci renvoyait chaque anomalie toutes les 600 s.
- Le mode legacy et la copie Claude conservaient des chemins de notification
  distincts. La documentation Claude recommandait encore le renvoi par âge.
- Le transport direct sans clé idempotente créait une identité CLI temporaire
  à chaque envoi ; busy était refusé comme s'il s'agissait d'une déconnexion.
- Le détecteur Psychologie ignorait les progrès vérifiés inscrits par la commande
  canonique progress, pourtant demandée dans les rappels communs.

Correction :

- notify_mission_events constitue le seul moteur de remise pour v2, legacy et
  l'adaptateur Psychologie. Celui-ci garde ses contrôles de hash ROOT, de budget
  total, de dépendances et d'inventaire. Aucun verdict métier n'est modifié.
- Digest unique par destinataire ; worker, coordinateur puis ROOT ; preuve
  différente, nouvelle tentative et nouvelle occurrence réactivent le suivi.
- Génération d'occurrence durable et reprise du token/corps/portée d'une ancienne
  outbox en vol, même si issues était vide au crash.
- Une migration ne déclare un fait déjà remis que si l'ancien dépôt, son hash,
  sa tentative et le propriétaire concordent. L'ambiguïté produit une demande
  groupée, jamais une fausse acceptation.
- Les commandes et l'installateur Claude chargent l'implémentation Codex, sans
  copie du moteur. Les instructions et les tests suivent aussi cette source.
- busy reste joignable ; le transport utilise le client idempotent sans emprunter
  d'identité T3. Le nom du programme et du run restent visibles dans le digest.
  Ce correctif ne crée pas un nouveau profil humain pour un programme autonome.
- Psychologie lit aussi les preuves progress canoniques ; répéter une preuve
  avec un nouvel horodatage ne remet pas son délai à zéro.

Validation : 115 tests canoniques PASS (99 existants et 16 systémiques), 22 tests
Psychologie PASS. Les 99 tests canoniques passent aussi via l'entrée Claude.
Validation des deux skills PASS ; plist PASS ; diff-check PASS. Les simulations
utilisent fichiers temporaires et transports simulés, pas d'appel fournisseur.
Revue indépendante : deux défauts RED corrigés (réapparition et nouvelle
tentative), puis reprise d'une ancienne outbox en vol corrigée et testée RED/GREEN.
Pas de revendication d'un audit v14 complet ou de livraison Git de cette reprise.

Mise en service :

- Les deux LaunchAgents ont été suspendus pendant la validation puis réactivés.
- Contrôle 60 s ; ACK 120 s et progrès 300 s configurés pour Psychologie.
- Passage manuel réel Politique : REFRESHED changed=0, NO_OP, exit0.
- Passage réel Psychologie : exit0, aucun envoi. Les deux faits T057 déjà remis
  à ROOT sont conservés comme obligations, sans être acceptés ni rejoués.
- Première comparaison : toutes les tâches et tous les résultats des deux runs
  sont byte-identiques aux sauvegardes. Annuaire : 45 agents avant/après.

Sauvegarde complète :
/Users/moi/Documents/opus2D/orchestration/systemic135-XCEisN/backup

Inventaire contrôlé : LaunchAgents utilisateur, cron utilisateur, variantes
Politique et copies Codex/Claude. La ronde et la relève Maicie sont désactivées ;
le runner de fédération chargé n'émet pas de rappels. Pas de contrôle exhaustif
des services distants, LaunchDaemons système ni de chaque programme indirect.
Les sources déjà modifiées dans dotfiles sont conservées ; pas de reset, pas de
commit global, pas de push ou de redémarrage T3.

Contrôle final après remise en service : Psychologie quatre passages et Politique
trois passages planifiés, tous exit0, cadence60s. Aucun nouveau digest ; les tâches
et résultats des deux runs restent byte-identiques aux sauvegardes. stderr reste
315octets pour Psychologie et742octets pour Politique, exactement comme avant.
Annuaire inchangé à45agents. La copie /Users/moi/.agents/skills/agent-loop est
un lien vers Codex : elle bénéficie aussi du moteur corrigé. Compilation Python
des cinq points d'entrée PASS. Reprise corrective :15/15tâches, code actif sur la
machine ; état Git conservé noncommitté. T057 demeure review et attend ROOT.

## 2026-10-06 — Livraison et nettoyage demandés

Sauvegarde des sources et des artefacts avant livraison :
/Users/moi/Documents/bridget-cleanup-20261006-hdwG80

Les corrections sont isolées depuis origin/main dans un worktree dotfiles
dédié. Les changements étrangers déjà présents dans les autres dossiers restent
hors livraison. Revue de reprise : nouveau fait ajouté après crash pouvait
remplacer le digest en vol ; correction et régression ajoutées avant fusion.
La preuve initiale results.json reste historique. Une preuve de livraison
distincte consigne les nouvelles sources, les tests et les commits.

Cache debug inutilisé nettoyé par cargo clean --profile dev. Les sept anciennes
sauvegardes d'installation sont déplacées dans retired-builds, avec une archive
tar vérifiée. Le binaire release conserve son SHA256
02cfdf77ad5b6971572e328bae551dbc1028e74a993abc39778a5263611211e6.
Le daemon et l'adaptateur T3 ne sont pas arrêtés. Les données des missions,
les historiques et les sauvegardes de reprise ne sont pas supprimés.

SPEC137 : 200 tests PASS frais ; code 3d30a4836b fusionné et poussé dans le fork
T3 local/v0.0.45. Documents fusionnés dans Bridget au commit 79a3c523.
Ses deux worktrees terminés sont retirés après vérification d'ascendance.
L'application T3 installée reste inchangée pour préserver les agents ouverts.

Validation de livraison : 124 tests canoniques PASS (99 existants et 25
systémiques), 22 tests Psychologie PASS. Les 99 tests passent aussi par l'entrée
Claude. Les cinq points d'entrée compilent ; les deux skills sont valides.
Une fixture de l'adaptateur rend les tests systémiques reproductibles sans le
dossier personnel de Psychologie. Revue ciblée indépendante : APPROVE.

Sources fusionnées et poussées dans dotfiles/main : 5fc64e37. La branche active
104 conserve le même contenu par le commit ciblé 2c5280f2. Aucun changement
étranger n'a été inclus. Le correctif final est installé dans le moteur commun.

Deux anciens lots Politique possèdent un reçu accepted et un événement durable
digest_sent concordants. Migration opérateur explicite : leurs corps et preuves
sont archivés dans validation/completed-outboxes.json et dans la sauvegarde,
puis seuls les deux champs de l'outbox sont retirés. Ni tâches, ni verdicts,
ni compteurs de rappel, ni anomalies ne sont acceptés par cette opération.
Pour les autres anciens lots réellement ambigus, le destinataire est bloqué et
ROOT doit arbitrer ; ce blocage n'est pas levé par une clôture automatique.

Après réactivation : deux passages Politique et trois Psychologie, exit0,
cadence60s. Les 406 fichiers de tâches et résultats sont byte-identiques à la
sauvegarde faite juste avant installation. Le daemon et l'adaptateur Bridget
gardent leurs PID et leur binaire release. La preuve historique results.json
n'est pas remplacée par la preuve fraîche systemic-delivery.json.

Clôture de livraison : 17/17 tâches. Documents135 fusionnés au commit4415a162
et poussés dans Bridget/main. Les worktrees135 Bridget et dotfiles, ainsi que
leurs branches locales, sont retirés après contrôle de propreté et d'ascendance.
Avec les deux worktrees137, quatre worktrees terminés sont nettoyés. Les autres
branches, builds actifs et travaux non fusionnés restent en place. L'empreinte
du patch déjà staged dans le worktree dotfiles/main reste byte-identique avant
et après fusion :6588fe2ba33bdec78a96fa2cb48d52a419df3c1e3f8ab31347030aeead9521d5.
