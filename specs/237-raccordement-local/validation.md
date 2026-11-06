# Validation237 — services activés, client à recharger,6novembre2026

## Faits de départ

Mac03 :0reçu/0tâche au diagnostic. Placementopus2d révision3 maître/worker
identique, registreMacSHA bdee09c363292d1be730eecffad779a942cd75a0d45b69725472867ff3f3587f.
Le sélecteurRust for_origin adapte déjà une correspondance depuis sa machine
cible. Le helper Python refusait son source_host cartae/racineLinux depuisMac.
Le même test échoue sur l’ancienne source236 avecproject_mapping_mismatch, puis
passe avec la projection :retour du mapping provisionné brut, aucun octet de
registre/HEAD/index changé, aucunSSH ni checkout. La publication conserve le
placement brut ; les vérifications portent sur sa lecture locale effective.

SYS05 :reçu chosen/révision2, task_idnull,0native_delegation. Sélectionc001/
codex-sysle2, observation époque93abb428-8431-49b1-a2cc-f25969e3c7ca.
Le workerPID2318448 tourne depuis05h47UTC. Àpartir06h08,308refus lus sur le
canal maître pour la même observationTerminal ; chaque refus ferme la lecture
worker. Commandee3f4498a-ae39-4676-9d23-24fa91184112, instancec63a24a3-0c7b-4464-b00e-6f3b49326ba7,
génération1464, saga de spawn connected, tâche native absente. Le worker conserve
la même identité en phaseterminal. Aucun nettoyage/réécriture de ces reçus.

Le contrôle require une délégation pour toutes les observations, y compris
les commandes gérées hors délégation. Correctif :remote_worker_has_command
reste obligatoire (placement/machine/instance/génération durables). Lorsqu’une
délégation existe, remote_command_identity reste exactement requis. En son
absence, seultask_id==command_id est admis. Une erreur du store reste refusée
StoreUnavailable. Époque, authentification, forme et corrélation inchangées.
Le test ajouté échoue sur le contrôle236 puis passe avec le correctif.

## Contrôles ciblés

Suite Python de préparation :50testsPASS29,766s. Le premier passage global
avait2erreurs sur des chemins temporaires symboliques Mac ; les47autres plus
le nouveau cas local passaient. Passage final avecTMPDIR privé canonique,
aucune garde production affaiblie. Tests de projection :origine étrangère et
cwd hors cible refusés ; racine additionnelle adaptée sans modifier l’original ;
publication sur mapping existant octet pour octet inchangée.

Trois testsRust :observation gérée sans délégation, refus tardif172, réponse
Release tardive184. Le premier vérifieReleased/Terminal/Refused, attente corrélée
encore livrée, aucune tâche créée, refus instance/génération/task_id/command_id/
époque étrangers. Total53tests distincts ; répétitions et baselines non comptées.

Lecture physique Mac sans admission ni écriture :remote_inspect nouveau helper
sur opus2d/base5e0ab699e64c1dbee5b21636dbb0abec25e13a22, projetopus2d résolu,
needs_bundlefalse, placement et registre inchangés. Pas une mission réussie
inventée ; les autres gardes d’admission restent applicables.

Cache commun réutilisé via build.py. Le test baseline a déclenché l’entretien
10,57Gio logiques ; cache régénéré avec CARGO_PROFILE_TEST_DEBUG=0 pour limiter
les symboles de test, mêmes contrôles et profilrelease inchangé. Pas de target
par test ni désactivation du budget10Gio.

Diagnostic propriétaire :mcp-88804-6aed70df-15, remise63fb6e69-132f-4b61-9d24-96ca3147560b,
in_flight ; lecture non attestée. Aucun rejeu03/05, aucunStart/Cancel métier.
SYS02 à ne plus reprendre. Préserver7024/77ba/702d et anciennes revues.

Revue, activation, publication et bilan final restent à faire.

## Revue indépendante et complémentP3

Revue native1390358e-9d27-4a3e-9f98-6b744a63d87f, source2475d88d5d89ef2f1b0aa737a9769af88306b5f0,
sélectionclaude-sysle2-haiku213/claude-haiku-5-5 sans axe imposé. Terminée, résultat
intégral lu et ACK acquis. AucunP1/P2 trouvé par lecture. Relecteur sansBash :
pas de diff shell ni test indépendant ; le rapport natif confirmeHEADexact,
0commit, arbre propre. La revue a réussi entre deux fermetures du canal ; cette
indisponibilité est intermittente, pas un arrêt global.

P3-1 :la conversion historique d’une erreur d’idempotence enfalse est confirmée
sur la base151cb960, donc préexistante. Corrigée dans le parcours d’observation :
remote_worker_command_known conserve son erreurStoreUnavailable ; la projection
bool utilisée par les autres parcours garde sa sémantique. Une erreur de lecture
d’identité d’une délégation existante est aussi rendueStoreUnavailable.

P3-2 :environnement de test expliqué etTMPDIR canonique conservé, aucune
modification de la garde des liens symboliques en production.

P3-3 :deux contrôlesRust supplémentaires acquis. Une délégation existante réelle
dans la fixture accepte sa bonne identité ; le changement d’instance dans la
fiche est refusé. Deux pannes de schéma provoquées seulement dans SQLite
temporaire (spawn_commands puis native_delegations) donnentStoreUnavailable,
sans conversion enCommandMismatch. Les trois tests237 passent ensemble0,42s.
Total55tests distincts :50Python et5Rust. Les répétitions ne sont pas ajoutées.
Dernier delta de traitementP3 auto-relu, pas de seconde mission ni nouvelle ronde.
APK8005 déjà compilé ; aucun changement Android entre la source revue et ce
deltaRust, donc pas de seconde compilation Android. Nouveau build237-r2 requis
pourMac/Linux ; aucune imageR1 installée ou activée.

## Activation des services et publication

Source exécutable1719ab969c10ec09344d938bbd0a69a95ceca299, version0.8.5/build237-local-project-r2.
Sept services/SHA vérifiés :Cartae core1605276/mobile1605395/worker1608385 ;
SYS2354435 ; KS3439620 ; Macworker57995/pont58232. Coupure maître1,95s.
MacSHA6701a5fb7e1484bc0aef8f33e9b7b668bae161a1d37d9157167969f03c646f7a ;
LinuxSHA2cb1fdc5764f56c4bca9598afabacf994efd238e035ba2d61300544f4c713066.
Retour236R2 compatible dans les deux sens ; pointeursrollback privés atomiques
sous verrous natifs sur4hôtes, watchers relancés. SuperviseurCartae2500061/
birth270585460 et registreMacbdee09c... conservés. Aucune imageR1 installée.

Après chargement :aucun refus dans le journal du maître depuis06h40m49UTC au
contrôle. Deux sockets SYS7012323/7012329 et PID2354435 identiques pendant111,91s.
Cette stabilité qualifie le canal observé ; pas une admission métier future.
Les reçus03/05 sont strictement conservés :Mac03 sans reçu ni tâche ; SYS05
chosen/révision2/tasknull, canonSHA54327e07b741198f5ff7e1404d1dbb41dc1e196269206bdb5c35933e9946be6c
et metadataSHA3d2f2b536bb653aebf6afc9f95035b0fd8e035f37b284be4c59443b548da3a97
inchangés. Payloads/canons7024/77ba/702d inchangés, aucune clôture inventée de7024.

Releasepubliquev0.8.5 id436184033, cible1719, sixactifs téléchargés et SHA/tailles
vérifiés. APK8005 SHA77740a099c5288a29e937833ff3fb57613c41377f2f1b65fd75a521606e06175,
certificat0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588
et lignée existants conservés. Quatre rapports0.8.5/up_to_date/fresh, sans erreur.
Correction technique, pas de changementUI ni diaporama requis.

Diagnostic serveur au propriétaire :mcp-88804-6aed77c3-2a,
remisecbae611c-9a48-4b87-8911-f1bcc2b550a8,in_flight ; lecture non attestée.
Vingtavis avant passage,19autres avis après services, reçus conservés, aucune
lecture inventée. Les refus de reprise éventuels devront être lus avec leur
clé existante ; SYS02 ne doit plus être reprise.

## Client — nouvelle consigne utilisateur

François demande explicitement le6novembre :ne pas relancerT3 automatiquement.
Terminer les étapes indépendantes puis afficher une question utilisateur visible.
Aucune fermeture/réouverture ni continuation automatique237 lancée. Mêmeapp231
conservée. Vingt-et-une anciennes façades236 étaient observées avant passage ;
leur rechargement237 reste à qualifier après intervention manuelle.
Le lien natif courant est vivant, projetBridget nonnul et aucun avertissement,
mais cette lecture passe encore par une ancienne façade ; ce n’est pas une
preuve du chargement237 du client.

## Entretien et conservations

AndroidCartae :940fichiers/141198392octets logiques retirés après comparaison
des62sources à l’archive et observation privilégiée des usages. Cartae etKS :
deuxcopies chacun (transfert237 et ancien235 sans usage)88445760octets parhôte.
Mac :ancien235 sans usage36427424octets retirés. Une première observation Mac
était incomplète ; lecture suivante complète et sans usage, retrait qualifié.
Actif237 et retour236 conservés, ce dernier encore utilisé par les anciennes
façades. CacheMac2,72Gio aprèsinstalled, sous budget ; entretienCargo10,57Gio
logiques compté séparément. SYSinstalled refuse l’entretien sans lecture
privilégiée complète :cache, sources et anciennes images gardés sans forcer.

Revues139/05ac arrêtées/ACK mais worktrees natifskept/review_required conservés ;
ancienne2fb, source227sale, lecteur234R0Mac/branche et preuves indépendantes
préservés. Aucun processus d’un autre travail arrêté pour nettoyer.
Les opérationsGit et le retrait final de nos transferts sont consignés dans
/Users/moi/.cache/bridget237-local-mapping/completion.json.
La livraison client reste explicitement en attente de la question et du
rechargement manuel, sans masquer ce travail restant.

## BilanGit et nettoyage avant la question utilisateur

Notre branche237 locale et distante ainsi que le worktree Mac propre ont été
retirés après preuve d’intégration et observation native complète des usages.
Worktree :4947fichiers/111720381octets logiques ; trois transferts (copieLinux
et deuxbundles vérifiés)44227349octets. Total des retraits237 :5895fichiers et
510465066octets logiques, hors entretienCargo déjà compté séparément.
Ce total n’est pas un gain physique APFS mesuré. Actif237, retour236, sources
et cachesSYS sous observation incomplète, worktrees de revue et autres preuves
conservés. Le bilan exactGit/alignement est danscompletion.json.

Aucune relanceT3 effectuée. Tout le passage serveur, publication, sources et
entretien est terminé ; seulT006 (question visible, rechargement manuel puis
contrôle des nouvelles façades) reste ouvert conformément à la consigne humaine.
