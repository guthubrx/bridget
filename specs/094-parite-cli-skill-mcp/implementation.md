# Journal d'implémentation 094

Statut : In Progress. Début : 2026-09-07 05:41 CEST.
ETA initiale : 85–150 min ; après tâches : reste 75–130 min (~05:51).
Branche : session-094-parite-cli-skill-mcp. Aucun commit/stage.

## Préparation validée

- Synchronisation utilisateur : à jour, aucune régénération des fichiers officiels.
- Checklist de spécification : 10/10, sans gate ouvert.
- Scripts/templates officiels de setup absents : instructions des primitives
  lues entièrement, génération directe des artefacts. Chemins mémoire projet
  ~/.Codex/projects/bridget et 64.bridget absents, aucune mémoire inventée.
- Audit de réutilisation : neuf items, neuf réutilisations, zéro duplication.
- Analyze : 11 exigences couvertes par 14 tâches ; cinq corrections de plan,
  dont trois problèmes de garde et le faux zéro du compteur inbox.
- Revue indépendante Codex sol high : Domain/Availability/Runtime Declared
  sans garde conn/instance ; vérification live_connection_identity retenue.
- Aucun fournisseur différent joignable ; pas de contre-revue interprovider.

## Isolation et rôles

Source : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux.
Ce worktree existant et ses WIP sont préservés ; nouvelle branche094 sans
déplacer le cwd autorisé de l'agent développeur. Snapshots des fichiers du lot
avant modification : /tmp/b94.LcIxfV. Les changements 092/093 ne sont pas rejoués.

- Agent23 : Rust/tests, mandat mcp-47688-6a9e349e-5, modèle sol high.
- docs_parity_094 : README/skill/référence exclusivement, sol high.
- review_authority_094 : lecture seule, sol high.
- Pilote : artefacts, tests/build, contre-validation et installation.

## Tests et livraison

À 06:01 CEST, rouge observé après compilation réussie : quatre tests unitaires
échouent sur catalogue absent, autorisation fournisseur incomplète, mutation
brute acceptée et faux zéro inbox. Deux intégrations échouent causalement :
fixture Claude (allowlist) et vrai daemon/MCP privé (outil inconnu).
Logs : /tmp/b94.LcIxfV/red.log, /tmp/b94.LcIxfV/red-integration.log et
/tmp/b94.LcIxfV/red-mcp.log. Commandes avec `--features test-support`, filtre
`spec094_`, tests d'intégration relancés séparément car Cargo s'arrête au premier
binaire de test rouge. GO production envoyé par mcp-47688-6a9e3720-9.
Ces six oracles ne constituent pas encore toute la couverture T003/T007/T012.
Documentation 43 entrées figée et en contre-revue indépendante.
Aucun nouveau binaire installé ni daemon/MCP/fournisseur utilisateur redémarré.

À 06:25 CEST, candidat1 : 11/11 tests unitaires spec094 et 3/3 intégrations
réussis (fixture Claude permissions, vrai daemon/MCP, vraie reconnexion wrapper).
Logs : /tmp/b94.LcIxfV/candidate1.log et
/tmp/b94.LcIxfV/candidate1-integration.log. Pas de suite globale à ce stade.
Revue documentaire indépendante APPROVE après deux clarifications (catalogue
ancien, reçu rename normalisé). Revue code encore ouverte : validation domaine
différente du canon CLI ; concurrence ACK/persistance et Register concurrent.
Correction étudiée : verrou interprocessus local par UUID, partagé par client
et wrapper avant lecture du domaine, sans déplacement du fichier vers le daemon
(préservation de la topologie SSH). Les processus anciens non coopératifs ne
peuvent pas être déclarés compatibles sans rechargement explicite.

À 06:32 CEST, contre-oracles rouges de revue : 9 tests lib passent, 3 échouent
(validation domaine MCP/daemon, ordre concurrent). La recette crash/reset échoue
avec `reconnexion-094` au lieu du vrai dérivé `bridget-daemon`. Une erreur de
compilation du support test (Receiver non Sync) a été corrigée avant ces runs,
sans changement de production. Logs : /tmp/b94.LcIxfV/review-red-lib2.log et
/tmp/b94.LcIxfV/review-red-crash.log. GO correctifs mcp-47688-6a9e3e68-1a.
Oracles finaux à compléter : contention réellement rencontrée (pas fenêtre
250 ms), DND échéance expirée, runtime blanc, messages différés pré-ready.

Demande additionnelle humaine à 06:29 : réinstaller sur cartae.app port2222.
SSH vérifié en lecture seule : Linux x86_64, ancien build4efa0549, services
daemon/UI actifs et timers Maicie/ronde actifs, deux gérés visibles. Aucun
changement distant. Arbitrage asynchrone demandé : remplacer ancien ensemble
par noyau ou installer à côté. Agent ssh_install_inventory effectue uniquement
l'inventaire de déploiement et de retour arrière ; l'implémentation094 continue.

## Self-review initiale XIX/XX

À 06:53 CEST, candidat3/candidat4 : 14/14 unitaires ciblés et 3/3 intégrations
passent. Le test crash corrige un oracle qui confondait basename du cwd et
racine Git : il compare désormais à la valeur initiale attestée et attend la
projection de l'override après redémarrage. Aucun changement de production pour
cette correction de test. Revue indépendante de lecture finale APPROVE.
Logs : /tmp/b94.LcIxfV/candidate3.log et
/tmp/b94.LcIxfV/candidate4-integration.log. Fmt et clippy ont ensuite relevé
des ajustements de présentation/types ; correction ciblée en cours avant suite
globale. Le modèle commercial réel n'a pas encore exécuté les six nouveaux
outils ; les preuves fournisseur citées restent les recettes de configuration
Codex et la fixture de permissions Claude, sans revendication de service réel.

Jalon 06:44 CEST : candidat2, 13/13 tests `spec094_` passent. Le canon domaine,
le refus DND expiré/runtime blanc, la sérialisation réelle de deux mutations,
le reset et l'auxiliaire de reconnexion sont implémentés. Log :
/tmp/b94.LcIxfV/candidate2.log. La revue a demandé un témoin supplémentaire
de contention entre mutation et reconnexion ; ajout test-only en cours.
Pas de nouvelle installation locale ni de redémarrage utilisateur.

SSH : bundle de sources et 110 dépendances vendored préparé dans
/private/tmp/b94-linux.n1Zo5s. Compilation Linux isolée déléguée, sans service,
base existante ni binaire global modifié. Activation ancienne flotte toujours
suspendue à l'arbitrage humain ; paquet release complet, tests 091 non inclus
par le script historique de packaging (limite distincte du build release).

Nécessité : capacités présentes mais inaccessibles depuis MCP et documentation
incomplète. Solution : mêmes commandes/états, frontières d'identité renforcées,
pas de nouvelle autorité ni de protocole. Complexité : parseurs bornés et
projections finies. Non vérifié : implémentation et recettes, à venir. Évité :
six commandes shell proxy, nouveau service, table de droits, duplication reply
et requests, exposition de scripts machine sous le nom de lecture.

## Clôture technique — 2026-09-07, 07:13 CEST

Six nouveaux outils MCP sont construits : rename, dnd, domain, runtime,
status et control_status. Inventaire documentaire : 43 entrées. Autorisation
fournisseur fermée : 12 outils Bridget ; les quatre outils Maicie du catalogue
restent distincts, sans autorisation générale ajoutée.

Validation composée, et non revendication d'une unique passe globale verte :

- `spec094_` : 14/14 unités et 3/3 intégrations passent.
- Première reprise globale : 71 suites, 1295 succès, 2 échecs, 54 ignorés.
  Les deux échecs CLI provenaient d'un test historique manipulant le drapeau
  d'arrêt GLOBAL pendant leurs daemons in-process. Dette prouvée aussi dans
  les snapshots antérieurs à 094, pas nouveau comportement de production.
- Correction exclusivement `cfg(test)` : l'oracle du vrai adaptateur d'arrêt
  s'exécute dans un enfant isolé. Son témoin passe ; le binaire lib complet
  rejoué en parallèle passe 793/793, 10 ignorés. Les autres cibles globales,
  inchangées depuis leur passage vert, ne sont pas rejouées inutilement.
- Codex local 0.153.4, vrai `config/read` : 1/1. Aucun appel de modèle ; la
  fixture Claude atteste les arguments de permissions, pas le service distant.
- `fmt --all --check` et clippy workspace/all-targets/test-support `-D warnings`
  passent après l'isolation. Revue indépendante de lecture : APPROVE.

Preuves : /tmp/b94.LcIxfV/{workspace-final,lib-final,shutdown-isolation,
native-mcp-policy,fmt-post-isolation,clippy-post-isolation,release-final}.log.

Installation locale atomique effectuée, sans arrêt de processus :

- Binaire : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget
- SHA-256 : b9eccd5bd80f9c5fd1ece4ac4d6de6a7fb7e120c20ff1f693fb8d444ac0be8e1
- Retour arrière : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/.install-094.iUCGie/bridget.previous
- Skill : /Users/moi/Nextcloud/10.Scripts/64.bridget/skills/bridget/SKILL.md
- Référence : /Users/moi/Nextcloud/10.Scripts/64.bridget/skills/bridget/references/commandes.md

Les publications Codex/Claude/Agents pointent vers cette source et sa référence.
Un nouveau processus MCP du binaire installé a publié les 16 outils attendus
(12 Bridget + 4 Maicie), sans appeler le daemon ni un fournisseur.
Le daemon vivant PID73764 et les sessions MCP déjà ouvertes restent anciens :
aucune activation transparente revendiquée. Le redémarrage est soumis à accord
humain puisqu'il termine les wrappers gérés avant leur recréation.

## SSH Cartae — candidat prêt, activation suspendue

Le candidat Linux x86_64 a été compilé hors ligne avec Rust1.92 dans un préfixe
isolé, puis republié atomiquement après le dernier delta test-only :
/home/moi/.local/bridget-communication-candidate-094/bin/bridget

SHA-256 : 8b41fa7ad83cd33ba2158dcc347dc43aaca971021305ba62604dfd371475e976.
Build-id : ea52cd048915-dirty ; le manifeste de contenu complète cette identité.

Recette via cartae.app:2222, daemon privé et vrai tunnel Unix SSH : deux clients,
message b94-ssh-probe-1788757027, livraison accusée, rejeu identique sans doublon.
Après coupure, erreur Connection refused sans repli vers un daemon de production.
Preuves conservées : /private/tmp/b94ssh.MrkwtD. Les processus et sockets de la
recette ont été arrêtés/nettoyés ; le candidat ne tourne pas en service.

L'ancien binaire global, les services daemon/UI, timers et bases distants sont
intacts. Aucun remplacement de l'ancien ensemble ni modification de service
sans arbitrage humain. Limite du paquet : fixture de tests 091 non embarquée,
sans effet sur le build release ; pas de prétention de tests exhaustifs Linux.

Audit final borné au delta 094 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux/audits/2026-09-07/session-2026-09-07-spec-094-01

Validation déterministe v14 : exit0, zéro erreur/zéro avertissement. Aucun
finding ouvert dans ce périmètre ; ni audit CVE ni qualité globale du dépôt
revendiqués. Les artefacts audits sont ignorés par Git ; aucun stage/commit
forcé. T001–T014 achevées pour les fichiers ; activation opérationnelle explicite
toujours en attente des choix humains indiqués ci-dessus.
