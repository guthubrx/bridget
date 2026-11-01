# Validation225 — en cours
Revue indépendante de conception : e32e917c-9f3c-4941-8343-239539712369, clé bridget225-cli-table-design-review-20261101-01, enfant84a40d4d-9799-4350-ba06-f25c32f05ee7, SYS/ClaudeHaiku choisi par Bridget, discovery/commit_view, base08e51c8d. Starting, aucun résultat ni avis de validation encore acquis. Conserver/relire/clôturer une fois, aucun remplacement.

## Contrôles ciblés et lectures réelles
Cinq tests225 PASS : cinq scénarios groupés, parseurs natifs inclus ; pas de
campagne générale. Assemblage Mac PASS. Lecture réelle de douze variantes :
spécialistes deux positions, mises à jour, contrôle/budget, agents, comptes,
tâches, demandes, formats et politique clients réussis. Mobile devices en shell
non-TTY : human_terminal_required, même garde conservée ; pas de pseudo-terminal
pour simuler l'humain. L'inventaire fédération s'appelle status, pas list :
documentation et garde --table alignées sur la grammaire réelle.
JSON formats policy byte-identique à l'actif0.7.3 ; conflit --json/--table refusé2.

La lecture réelle a montré les listes d'options réservées au formulaire mobile
dans specialists list : retirées de la présentation tabulaire, disponibles dans
le JSON original. Colonnes du spécialiste effectivement lues : bridge1016,
Mainteneur Bridget, bridget, monordinateur, Occupé. Aucun nom inventé.

Revue code refusée project_preparation_link_missing, not_recorded/tasknull,
clé bridget225-cli-table-code-review-20261101-01. Cause vérifiée : worker Mac
avec registre dans .config/bridget ; has_link compare sa parent à root des
liaisons dans .cache/bridget-core. Signalement et complément factuel transmis
à bridge1016 ; pas de remplacement ou contournement. Auto-relecture du delta :
toutes mutations/TT Y/portées initiales conservées, flush borné au thread CLI,
seul helper fédération readonly capturé ; code de retour inchangé.

Auto-relecture : quatre références command_id de cycle de vie sont vidées avant
la remise RPC, pour garder leur visibilité en cas de Ctrl+C. Sans --table le
flush est neutre. Aucun envoi ni requête supplémentaire.

## Revue indépendante de code
4a0ad91d / Claude Sonnet KS : original2f30f028 lu, verdict pas prêt.
P1command_id déjà corrigé par b78d9641 avant le verdict.
P1who/discover : suppression de la réécriture en agents ; mêmes commandes,
gardes et pieds de contrôle. Annuaire JSON interne uniquement au point de rendu.
P1faux vide : chemin daemon arrêté conserve sa cause stderr en mode table.
P2revoke ajouté aux booléens ; send/reply ne consomment plus de --table dans
leur corps ; texte libre rendu comme ligne, pas découpé à un deux-points ;
largeur minimale2 ou fiche sur terminal trop étroit.
P2federate réfuté : stdout est effectivement capturé seulement pour status,
qui ne teste pas le TTY. Les seuls tests -t du script sont les confirmations
de connexion/retrait, hors mode table ; garde humaine directe inchangée.
Échec stdout rend1 ; not_reported reste Non signalé, distinct de Non observé.
Le delta de guidage a444af64 est inclus dans l'image finale225r4.

## Contrôles finaux R4 et activation
Six tests groupés225 PASS, 2738 autres tests non relancés. Le test224 du guidage
est réutilisé sans changement du delta a444af64. Deux assemblages finaux PASS,
Mac arm64 et Linux x86_64 sur Debian12/glibc2.36. Neuf lectures/contre-épreuves
CLI R4 : deux positions spécialistes, who/discover et leurs pieds, updates,
federate status, corps send refusé2 sans envoi, serveur absent refusé1 sans faux
vide, stdout fermé refusé1. Mobile reste human_terminal_required hors TTY.
JSON formats policy identique à0.7.3 ; chemin par défaut non modifié.

Sept images0.7.4/build225-cli-tables-r4 chargées et empreintes vérifiées :
Cartae core1950987/mobile1951222/worker1954787 ; SYSworker1445745 ;
KSworker1767457 ; Macworker12673/pont12903. WatcherMac13307 actif.
Linux SHA66b878a32d1f4a2cf82260fb48b1f39f7ea940e7699cb0996c27d2b9478b932c.
Mac SHA47877a7d0544deae3d9b0b33f7a53a97b31736b5f4d392362975e0db46470417.
Coupure du maître2,20s. Superviseur métierCartae2500061/naissance270585460
conservé ; mêmes marqueurs KS conservés. Aucun Start ou rejeu métier.
Mac R2 avait été activé avant le verdict ; remplacé parR4 après corrections,
sans masquer cet intermédiaire. Les paquets publiés sont exclusivementR4.
Retours0.7.3 installés et compatibles vérifiés par helpers natifs ; rollback
atomique sous verrou sur les quatre hôtes, aucune activation en cours.
Le registre worker Mac reste dans .cache/bridget-core, jamais rétabli sous
.config. Le correctif de guidage du mainteneur est chargé avec ce lot.
Rapports installés réels : quatre hôtes0.7.4/fraîcheur récente.

Limite distincte : capabilities MCP de ce fil refuse encore l'attestation
après une seule bridget_session réussie(révision14). Cause exacte non observée,
verification origin_session_reattest ; pas de boucle. Annuaire/messages natifs
et CLI humain répondent. Signalement au mainteneur b2541703-c93c-42f6-b8c3-84c1e79f4ac5
in_flight, réception non attestée. Ne pas déclarer cette attestation rétablie.

## Preuves et paquet Android
Parcours CLI réel en cinq étapes, texte original et extraits signalés.
Diaporama autonome13Kio inclus ; html_preview sans erreur, navigation
suivant/dernier/précédent/premier contrôlée. Pas de clic inventé pour la CLI.
APK0.7.4/code7004 signé par le certificat existant, garde223 PASS ;
SHA1677cdc3f09ee86740ec032cfe37a5e4e2a36db53e2682948866f18607b0826e.
Android identique fonctionnellement à224 : seuls numéros du paquet changent,
Firebase publique réutilisée. Pas de nouvelle recette téléphone ni test FCM.
Les six actifs sont préparés ; publication et entretien ci-dessous.
