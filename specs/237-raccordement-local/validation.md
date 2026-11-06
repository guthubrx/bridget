# Validation237 — en cours,6novembre2026

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
