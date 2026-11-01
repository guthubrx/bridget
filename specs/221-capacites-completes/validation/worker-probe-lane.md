# Voie de lecture des sondes worker — session221

Statut : vérifié, livraison en préparation. Programme52/182 In Progress.

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
