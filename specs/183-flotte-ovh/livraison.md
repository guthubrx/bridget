# Livraison partielle — flotte OVH — 25 octobre 2026

## État réel

| Machine | Système | Worker installé | Exécution native | Routage |
|---|---|---|---|---|
| SYS-LE-2 (`sysle2`) | Debian 12, glibc 2.36 | 0.6.3, build `183-b6c045cd` | Deux missions terminées avec résultat | Disponible |
| KS-STOR (`ksstore`) | Ubuntu 24.04 | 0.6.3, build `183-b6c045cd` | Non qualifiée ; admission Claude en attente | Connexions désactivées |

Les liaisons Unix sur SSH et les services utilisateur sont persistants.
Le maître reste Cartae. Les workers, comptes et connexions sont propres à chaque
machine. Aucun secret fournisseur n’a été copié. Le dépôt Bridget et le dépôt
Opus2D sont préparés sur chaque cible. Le contrôle officiel de préparation
Opus2D a réussi pour le commit `91d4eccd1f45048da2a6b4d05f46c9b78e3e5545`,
sans transfert supplémentaire après amorçage du miroir depuis Cartae.
Cela ne qualifie ni les dépendances de chaque métier, ni une capacité GPU.

## Sources et contrôles

- `706a70abc1138e3d4e93503e4c01a9935b141f9a` : le helper identifie d’abord le
  registre du worker local, et non un registre global contenant toute la flotte.
  Deux reproductions RED, puis 35 tests Python PASS. Contre-revue favorable.
- `9709ab20` : une capture/relecture ne rehache pas deux fois le même chemin
  d’exécutable. Aucun cache ne survit à l’appel. Les deux révisions attendues
  restent comparées, et un wrapper distinct garde sa propre lecture.
- `b6c045cd1787f465e9727a6a1b7f7bd6d42820e0` : source commune intégrée et poussée,
  contenant aussi les derniers parcours mobiles `1d122d31`.
- Contrôles Rust ciblés : 32 PASS / 7 ignored dans le daemon ; 2 PASS dans le
  transport. Les nouveaux tests vérifient le nombre de lectures, les attentes
  distinctes, puis le refus d’un fichier modifié ou supprimé entre deux appels.
- Revue indépendante `5828e46c-f43d-4c45-8da4-e476c1cd6ec0` favorable,
  lecture seule. Aucun défaut concret relevé ; aucune FULL revendiquée.
- Builds optimisées : Ubuntu, 3 min 19 s ; Debian, 3 min 02 s. Les binaires
  démarrent et les deux services workers sont actifs. Les 21 warnings existants
  de compilation ne sont pas présentés comme corrigés.

## Reçus natifs

- SYS-LE-2 : `b854e7b4-0dd8-472b-8965-dfdad2ed18ae`, résultat
  `QUALIFICATION_SYSLE2_OK`, hostname `backup`, Git 2.56.0, x86_64 et SHA attendu.
- SYS-LE-2 : `cc3feb4f-57d7-418f-955c-29f97f63fcce`, diagnostic du coût des
  lectures CLI terminé. Aucun fichier modifié par ces deux missions.
- KS-STOR : `c1a0f917-795b-4341-83a2-89e70e96d30c`, échec fournisseur avant les
  commandes : connexion Codex expirée, HTTP 401. Connexion retirée du routage.
  Une reconnexion native au compte sur cette machine reste nécessaire ; aucun
  changement de compte, transfert de jeton ou achat n’a été fait.
- KS-STOR : clé `bridget-ovh183-ksstore-qualification-20261025-02`, état durable
  `chosen`, task_id absent, après `remote_prepare:WorkerUnavailable`.
  Mesures avant correction : sondes Claude 21 062, 19 233 et 19 188 ms.
  Le Prepare partagé relit deux fois ce contexte et dépasse son budget de 30 s.
  Le fix de déduplication est installé. Son gain réel sur ce worker reste à
  mesurer : le rejeu strict lit encore l’admission en attente et ne recommence
  pas la préparation. Aucune nouvelle clé n’a été créée pour contourner ce sort.

## Point bloquant conservé

La reprise automatique actuelle sélectionne uniquement les reçus d’une époque
antérieure du coordinateur (`auto_admission_receipts_before_epoch`). Un échec
technique `chosen` de l’époque courante reste donc en attente jusqu’à une reprise
appropriée du coordinateur. La clé ci-dessus est conservée, sans tâche connue.

Le garde d’activation du core a refusé `ovh183-activation-706a70ab` avec
`update_missions_active_or_unconfirmed`. Aucun redémarrage forcé du core/mobile
ou du fournisseur partagé n’a été fait. Les workers neufs ont été remplacés
après vérification que leurs ordres étaient terminaux. Le worker d’origine Mac
utilise `183-706a70ab` ; son store était vide lors de son remplacement.

Suite autorisée à effectuer quand cette condition est levée : reprendre le
sort de la même clé, fermer son refus durable s’il est constaté, puis réactiver
la seule connexion réellement authentifiée et refaire une qualification.
Ne pas annoncer KS-STOR disponible avant un résultat natif réussi. Ne pas
rejouer de fabrication Opus2D pour cette recette.

## Compatibilité système et SSH

Le binaire Linux publié antérieurement requiert glibc 2.39 ; il ne fonctionne
pas sur Debian 12. SYS-LE-2 utilise donc une build native Debian. La glibc du
système n’a pas été remplacée. Git 2.56 est installé dans un préfixe privé.
Ne pas remplacer ce binaire Debian par un asset Linux incompatible lors d’une
mise à jour ultérieure ; ce lot ne configure pas d’auto-update sur ces hôtes.

OpenSSH 9.2 initialise les permissions de retour Unix depuis le contrôle de
forwarding TCP. Une règle limitée au compte et aux deux origines prévues permet
le retour Unix. `PermitListen none` et `PermitOpen none` maintiennent le refus
TCP. `sshd -t` PASS, rechargement sans interruption, liaison Unix réussie et
vrai essai TCP refusé (code 255, `remote port forwarding failed`).
Sources primaires : [session.c](https://raw.githubusercontent.com/openssh/openssh-portable/V_9_2_P1/session.c),
[channels.c](https://raw.githubusercontent.com/openssh/openssh-portable/V_9_2_P1/channels.c),
[sshd_config](https://man.openbsd.org/sshd_config).

## Monuments et notifications

Le dernier refus exact trouvé pour Monuments, à 07:34 UTC, vient de
`bridget_capabilities` et `bridget_status` : `identity_not_found`, ancien texte.
La configuration Codex officielle pointe désormais vers le lien installé,
mais plusieurs MCP anciens restent ouverts. Le guidage de réparation existe
déjà dans le binaire installé. Le rétablissement du fil n’est pas confirmé.
L’attestation doit provenir de son propre appel natif ; aucune identité ne peut
être copiée et aucun autre fil ne doit être interrompu pour le remonter.

Disponibilité SYS-LE-2, indisponibilité KS-STOR et procédure de signalement
transmises par la skill Bridget aux trois responsables, avec motif interprojet :

- Monuments : `mcp-31009-6ade79a1-5` ;
- Trajets bus : `mcp-31009-6ade79a3-6` ;
- Politique sol : `mcp-31009-6ade79a5-7`.

Ces envois sont `in_flight` : aucune lecture n’est revendiquée. Le précédent
diagnostic suivi a expiré sans réponse. Aucun nouveau rappel périodique ajouté.

## Nettoyage et conservation

- Worktree de compilation temporaire Mac retiré après état Git propre et
  observation ciblée lsof complète, sans fichier ouvert.
- Intermédiaires Rust Mac et SYS-LE-2 retirés après les mêmes contrôles ciblés.
  Sources et archive de compilation Git temporaires SYS-LE-2 retirées.
- Ancien binaire initial SYS-LE-2 et copies de transfert Mac/KS-STOR retirés.
  SYS-LE-2 conserve la version active et le retour `183-706a70ab`.
- La commande `installed` a retiré 0,56 Gio sur Cartae après la première build.
  Les passages suivants signalent une observation de processus incomplète.
- Worktree/branche locaux de flotte sur Cartae conservés : lsof produit des
  avertissements et ne certifie pas l’absence d’usage. Les autres worktrees
  protégés ou appartenant à un travail distinct ne sont pas forcés.
- Ancienne copie KS-STOR conservée pour la même observation incomplète.
- Logs, reçus worker et admissions conservés. Les processus MCP anciens encore
  actifs sur Mac sont conservés ; aucun arrêt d’un autre travail pour nettoyer.

Corrections techniques : pas de changement d’interface dans ce lot, donc pas de
diaporama requis. Le parcours mobile intégré conserve ses propres preuves.

## Complément : correction du rejeu strict préparée

La source `7478e9e756dced0e0f85d9519229c279644d362c` est intégrée et poussée sur `main`. Elle reprend un reçu `chosen` de l'époque courante sur rejeu explicite, avec son choix et son enveloppe d'origine. L'exclusion concurrente couvre la préparation et la clôture de l'erreur. Aucun nouveau décideur ni repli n'est ajouté. Les sources mobiles récentes sont conservées.

Validation : 42 tests ciblés réussis, aucun ignoré, et contre-revue R2 sans défaut bloquant. La build optimisée est terminée. Le candidat `183-admission-7478e9e7` est installé et son SHA-256 vérifié. Le détail et les limites figurent dans [validation/admission-replay.md](validation/admission-replay.md).

La commande d'entretien après installation a retiré 4,12 Gio ; 3,10 Gio de caches sont annoncés restants. Le worktree, sa branche, les preuves et le candidat sont conservés pour terminer la livraison. Aucun ancien binaire actif ou retour n'a été retiré.

**Activation non effectuée.** Le garde réel renvoie encore `update_missions_active_or_unconfirmed`. Le core Cartae reste en 0.6.2. La clé KS-STOR reste non réconciliée et ce serveur n'est pas annoncé disponible. La tâche historique `e5959986-e426-422d-956f-e035dbcf89be`, en arrêt non confirmé, constitue un autre point à traiter. Aucun service n'a été arrêté pour franchir ce refus.
