# Voie de lecture des sondes worker — session221

Statut : correctif chargé et vérifié sur les sept services. Programme52/182 In Progress.

## Incident et limites de causalité

Signalement `opus201-incident-worker-sys-trois-lots-20261101-01`,05:43–05:48UTC.
Les trois clés et leurs enveloppes sont conservées ; aucune relance par ce fil.
- maisons8 : `c2240df5-047c-437e-9365-28a17b613272`, admise puis failed,
  `remote_start_failed:remote_start:WorkerUnavailable`. Le reçu worker est
  terminal, marqueur1222713/naissance68184727, PID absent au contrôle ciblé.
- cultes-nord8 : aucun reçu dans le registre maître à cette lecture ; refus
  rapporté not_recorded.
- cultes-sud8 : reçu chosen, task_id nul ; préserver comme pending, pas comme
  refus terminal ni comme absence d’admission.
- palais7100defb-cc51-4ac0-babf-dbe7f9925f46 : résultat disponible, erreur nulle.

Le worker SYS1209579 est actif depuis05:18:11UTC, NRestarts=0, disque et
mémoire disponibles. Ses journaux corrélés montrent trois sondes de12candidats
en33,425s,33,952s et32,472s et une sonde d’un candidat en6,993s. Le chemin
exécutait ces lectures dans la même boucle que Start/Cancel et les retours.
Le délai de confirmation Start est30s. Le défaut de disponibilité est
reproductible dans la fixture de vrai worker ; le détail temporel de chaque
étape historique de maisons n’est pas reconstitué. Aucun manque de compte,
mémoire ou disque n’est déduit du code WorkerUnavailable.

Écart distinct actuel confirmé : CodexSYS0.166.0, binaire319346888octets,
SHA b8a11fedf24b2dfb59a70c9e52b030f9a508dcd49cd2dd0d41956f4c785156b8 ;
les profils codex-sysle2/codex-sysle2-terra213 attendent encore0.164.0 et
SHA b93f6811d3f653f0fb83be4050d857fb9abbb111dfa2ce160563327daff7f306.
Ce fait explique un refus de préflight de ces profils ; il ne démontre pas la
cause de tous les refus de définition ni de l’échec Claude. La garde reste
fermée. Aucune empreinte, installation, connexion ou offre modifiée pour la
faire passer. Le guidage DefinitionUnavailable nomme désormais la vérification
de version/empreinte et la décision de qualification ou de retour à prendre.

## Correction et garde-fous

Lectures existantes déplacées vers WorkerReadView et ProbeReadContext. Aucun
store écrivain, Child, token Prepared ou runtime dans ce contexte. Un permis
RAII limite à une lecture physique à travers les reconnexions. Deux files
indépendantes : huit commandes ; sept sondes et une tête datée. Le maître,
les trames, les canons, les droits et l’autorité d’admission sont inchangés.

Budget45s de réception à fin : contrôle entre candidats et refus des sondes
expirées en attente, même si la lecture précédente continue. Le délai propre
d’un contrôle Git déjà engagé reste applicable ; aucun arrêt forcé ni abandon
de son permis. Les réponses d’un ancien lien restent sur son ancien socket
et portent son époque ; les gardes maître de connexion/époque/corrélation
restent obligatoires. Une panique de lecture produit un refus fermé.

Les racines prepared sont copiées en mémoire. Les commandes ne sont lues que
pour Shared ayant besoin de refresh : métadonnées seules, bornées4096, mêmes
validations de reçu et d’enrollment, aucun parcours des blobs, aucune
quarantaine écrite. Une erreur ne touche que les candidats qui ont besoin de
ce contrôle. Le validateur de commande est partagé sans changement de logique
avec WorkerStore. La garde d’occupation est commune à Prepare et à la sonde.
Prepare relit toujours les gardes vivantes avant toute mutation.

Le relevé hôte est propre à la sonde ; sa moyenne CPU peut couvrir jusqu’à10s.
ResourceLimit candidat ne prouve plus une file pleine : probe_capacity utilise
worker_probe_capacity_unavailable/unavailable_at_probe/current_state unknown.
Le guidage distingue file/délai non départagés, conserve detail.admission et
interdit toute nouvelle clé d’une issue inconnue.

## Vérifications utiles

Treize tests bibliothèque ciblés PASS, dont sonde retenue+commande native,
saturation corrélée+annulation, créneau inter-époques, erreur de store paresseuse
sans quarantaine et sonde périmée refusée sans exécution. Neuf tests spec189
PASS. Régression Shared incident171 réexécutée après le complément R2,
PASS0,50s ; sa logique de garde reste commune et le recontrôle est conservé.

Vrai processus worker : une sonde bloquée dans Git et une vraie commande Start
refusée par son contrôle avant libération de Git, PASS1,12s ; aucun fournisseur
démarré (bootstrap de fixture seul). La sonde128candidats, révocation et
préparation épinglée PASS6,02s,29appels Git pour deux tuples, borne40.
Le premier essai d’intégration a échoué avant Hello : relais ancien annonçant
des capacités qu’il ne savait pas négocier. Fixture corrigée pour n’annoncer
que ses capacités prises en charge. Cinq champs service_tier=None manquants
dans cette ancienne fixture ont également été ajoutés. Pas de qualification
fournisseur ni de mission métier, pas de campagne générale.

Revue5478888b : aucune permission omise ; F1 (store trop large) et F2 (âge non
borné) corrigés ; F3 couvert dans le vrai processus worker. F4 helper commun,
F5 limite CPU documentée, F6 refus de panique, F7 garde maître inchangée.
Ronde finale3692016c refusée au démarrage WorkerUnavailable, sans résultat ;
ACK et annulation demandés une fois, aucun remplacement. Auto-audit du delta
R2 déclaré : lecture paresseuse et bornée, validateur partagé sans écriture,
expiration incluant la file, permis conservé jusqu’au retour physique. Les
gardes maître connexion/époque/identité/corrélation/admission/révision et liste
de candidats sont relues dans daemon.rs:19043–19130 ; aucun changement.
Cette passe ne constitue pas une revue indépendante réussie du delta R2. Les erreurs initiales de
compilation de fixtures (type BufReader puis helper absent) sont corrigées ;
seuls les PASS finaux sont comptés, sans addition des répétitions.


## Livraison active

Source06ba1606c29f08a0932d156ec5da0efa2eebd866 intégrée à main et poussée.
Les356fichiers du manifeste d’assemblage correspondent à cette source ;
stagingsMac et SYS identiques au manifeste. Assemblages Linux2min et Mac1min18s
réussis. Pas de nouveau test Mac ; les25tests distincts finaux sont Linux
(13bibliothèque+9spec189+vrai worker+128candidats+Shared).

Build221-worker-probe-lane-r2, version0.6.5 :
- Linux41525776octets, SHA256
  `7cd9c1a8957dd350a9ec9d41d02119bfe104b933c8656328cb725183f6d49396`.
- Mac34318528octets, SHA256
  `d9a6b0987034496a4ff2d14a68a57da74477ba0bf40a77189d57569d7d5852aa`.

Sept images réellement chargées, chemins et SHA relus : maîtreCartae552506,
mobile552580, workerCartae553042, workerSYS1240730, workerKS1612683,
workerMac44587 et pontMac44891. Coupure maître2,27s. WrapperCartae2500061,
naissance270585460, conservé. Aucun autre commandement actif dans les reçus
worker au précontrôle ; cela ne vaut pas inventaire universel des processus.
Le journal système mentionne Codex1240655 survivant au stopSYS ; PID absent
au contrôle suivant, sans signal envoyé par ce lot ni conclusion métier.

Retour221-continuation-receipt-r1 qualifié dans les deux sens par le lecteur
natif de formats. Pointeurs rollbackCartae/Mac écrits atomiquement sous verrou
updater, puis images et SHA relus ; aucun retour exécuté. Sauvegardes privées
avant passage conservées.

Une session native réussie. Première capabilities pendant republication :
primary_absent. Après exposition attestée de bridge1016 à06:37:27UTC dans le
journal du pont, nouvelle lecture réussie :45options, delegations_halted=false.
Aucun nouveau schéma MCP dans ce lot : les anciens clients bénéficient du
correctif de boucle worker. Aucun redémarrage de l’application hôte demandé.
Les limites de client du lot continuation-receipt restent décrites dans son reçu.

## Sort des demandes originales après passage

Lecture administrative seule du registre maître à06:37:28UTC, puis détail nord :
- maisons8 : même tâchec2240df5, failed/WorkerUnavailable, ronde1,
  mission_issued_at=null. Aucun travail métier remis selon ce reçu.
- nord8 : désormais admise sous be3d8445-e6f4-4b5f-9b83-518a43d72c54,
  result_available, erreur nulle, ronde1. Mise à jour du reçu à1793512763.
- sud8 : désormais refused, task_id=null,
  routing_connection_changed:selected_probe:worker_epoch, mise à jour1793512811.
  Cette fin de reçu précède la maintenance de ce lot ; ne pas lui attribuer
  notre coupure. Aucune nouvelle tâche créée par le présent responsable.
- palais7100defb : result_available, erreur nulle, ronde1.

Le premier instantané nord absent/sud chosen est conservé comme historique,
pas présenté comme état courant. Aucun résultat métier lu, aucune enveloppe
rejouée, aucun remplacement ni changement de droits/compte/modèle. Message de
résultat envoyé à monuments sous bridget221-worker-probe-lane-result-20261101-01,
issued_at1793515104, delivery17f98ca8-fa27-4476-94f1-c644ab617f1b,
in_flight ; la lecture du destinataire n’est pas attestée.

## Git et entretien

MainMac et mainCartae propres et poussés. Les deux stores de chaque miroir
SYS/KS reçoivent main par bundle incrémental ; les HEAD détachés actifs restent
inchangés, aucun checkout/reset ni changement d’origin. La preuve finale inclut
le commit documentaire qui complète cette source fonctionnelle.

Entretien installed exécuté après contrôle des images. Mac refuse le propriétaire
du staging ; SYS indique une observation processus incomplète. Caches conservés
sans forcer. Trois copies binaires de transfert identifiées retirées après
vérification de l’image installée :124577328octets. Actif, retour qualifié,
preuves, snapshots sources et anciennes images encore utilisées conservés.
Worktrees de revue5478888b et3692016c conservés : rapport final ou observation
d’usage incomplets ; aucune suppression fondée sur une absence supposée de
processus. Annulation R2 demandée une fois, aucun rejeu.

Correction technique de boucle et de reçu MCP ; aucun circuit d’interface
visuelle modifié, donc pas de diaporama de clics. Programme global toujours
52/182 In Progress, aucune parité globale annoncée.
