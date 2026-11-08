# Validation248 — comptes rendus pendant un tour

## Cause et correction

Le parent Parix était dans le tourCodex39 depuis15:03:30UTC. La remise sans réponse attendue était exclue du parcours pendant un tour par un filtre limité àClaude. La relation logique d’une mission distante ne couvrait que parent→enfant. Le helper248 reconnaît aussi enfant→parent, avec les deux instances vivantes, la tâche active et les révocations. Il ne donne à l’enfant aucun droit de piloter ou interrompre son parent.

Sous le contratV2, un compte rendu groupable et autorisé utilise le tourCodex actif avec le même identifiant. Les demandes avec réponse attendue conservent leur propre corrélation et leur attente. L’expiration est nommée comme telle.

## Sort réel des deux messages

- Message `mcp-2878019-6af09455-2`, remise545efd04 : ACK à16:06:11UTC, après2461secondes en file. Cette réception précède le chargement248 et ne prouve pas son effet. Aucun nouvel envoi.
- Message `mcp-2878019-6af0976b-3`, remise1abfc9bc : demande expirée à15:48:20UTC, completed_at1794152900 ; sagaindeterminate, aucunACK. Le pont l’avait retirée avant démarrage. Aucun rejeu ni remplacement.

## Défaut découvert pendant le passageR1

La tâcheParix a3a311f9 était working dans les deux sauvegardes avant et après arrêt du maître. Elle est passée failed à16:08:59UTC, sortieCodex fermée ; terminaison native exit0, fichiers conservés. Le stderr du wrapper donne `spool_envelope_changed`. Le spool contenait notre avis de maintenance de06a6e0, avant injection ; au raccordement le maître recalculait son échéance de tour. Même remise, même instance, même génération et même expiration ; l’ancienne comparaison des octets prenait cette décoration pour un changement et arrêtait le fournisseur.

R2 compare l’enveloppe entière sauf l’échéance de tour et les deux libellés de présentation. Il garde la première trame, sans modifier son délai ni son autorité. Contenu, identité, génération, exécution et autorisation pendant le tour restent stricts. Un refus de dépôt ne produit niACK ni injection et ne ferme plus le fournisseur. Aucune relance de la tâcheParix échouée. Une erreur d’ouverture ou de lecture du spool peut encore arrêter le wrapper : limite distincte, non rencontrée ici.

## Contrôles utiles

- Mac :2tests248 initiaux,4tests247,1test129,1test180 ; puis test248spool et test244spool. Dix contrôles distincts. Tous verts.
- Linux : les mêmes tests248 et247, soit7contrôles distincts aprèsR2. Pas de campagne générale ni d’essaiClaude.
- La première exécution du testRPC utilisait une fixtureV1 : corrigée en fixtureV2 existante. Aucun assouplissement de production pour faire passer le test.
- RevueR1 task3a1408fb : aucunP1/P2 confirmé ; terminée, lue etACK.
- RevueR2 task1b7ce7bd : APPROVE, aucunP1/P2 confirmé ; terminée, lue etACK. Revues statiques sans tests indépendants. La limite d’ouverture/lecture du spool est explicitement conservée.
- `git diff --check` acquis. Aucun nouveau format, table ou dépendance.

## Activation et conservation

Source exécutable finale :d96e1ec4b6e1ab1272229288c0f77b88af266ab0. Version0.8.14, build248-messages-en-cours-r2. Retour qualifié247-coordination-r1/0.8.13 ; formats avant, retour et inverse compatibles, ancien245 refusé comme lecteur. Pointeurs atomiques sous verrous natifs, aucune activation concurrente.

Le passageR1 a chargé les sept composants sur quatre hôtes ; coupure du maître1,93s. R2 remplace ces images. L’applicationT3 n’a été ni arrêtée ni relancée. Les treize processus T3 initialement observés gardent leurs PID et dates de naissance ; deux renderers supplémentaires sont apparus, sans disparition des précédents. Le registreMac est inchangé. Les sept payloads protégés sont identiques avant/après. Les wrappers étrangers vivants et leurs fichiers restent conservés.

Aucune preuve de réception d’un nouveau compte rendu enfant→parent dans un tour réel aprèsR2 : la correction est contrôlée par les tests ciblés et par les images effectivement chargées. Aucun lancement métier de qualification.

## Avis et preuves

L’avis de diagnostic initial a rencontré binding_transport_unavailable sans identifiant rendu ; il n’est pas annoncé livré ni rejoué. L’avis avant passage et deux avis factuels au parent ont des reçus conservés ; une remise en vol ne prouve pas leur lecture. Après disparition de l’enfant, le diagnostic utile est porté à son propriétaire vivant, sans nouveau destinataire inventé.

Preuves privées, journaux de tests, sauvegardes, reçus et empreintes : `/Users/moi/.cache/bridget248-messages-en-cours/`. Aucun corps métier ni secret publié.

## Bilan final

Les sept composantsR2 sont chargés : maître, mobile et workerCartae ; workersSYS etKS ; workerMac et lienBridgetT3. SHA Mac33a12f9874a3c7bce0620587dbb20f32505bcbbe865f95551635085868ee71ca ; Linuxb9a8bbda83ff8da6621b9252a16f7ddb99146d683edaa37bcb489f38059fd1aa. Coupure du maîtreR2 :1,79s. Les pointeurs de retour247 et leurs images sont vérifiés sur les quatre hôtes. Session nativeV2 et annuaire same_project réussis après passage.

Release publiquev0.8.14 : six actifs téléchargés sans authentification, tailles etSHA vérifiés. APK8014, certificat0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588 et lignée8013 vérifiés. Le deltaR2 purementRust ne change pas les sourcesAndroid de l’APK déjà construit.

Source exécutable intégrée àmain et poussée, quatre hôtes alignés surd96e1ec4 avant nettoyage. Bilan documentaire final poussé ensuite et alignement relu. Le Mac ne conserve que main : branche248 et worktree retirés. Worktree de compilationSYS et worktreeAndroidCartae retirés après intégration, absence d’usage et contrôle des fichiers ignorés ;870fichiers de buildAndroid identifiés. Caches partagés conservés sous budget :4,15GioMac et7,89GioSYS au dernier contrôleCargo.

Cinq transferts de binaires/bundles et deux bundlesSYS, soit134306426octets, retirés ; imageR1Mac obsolète de36780384octets retirée. L’imageR1SYS reste chargée par des processus étrangers2855715/2855820 ; imagesR1Cartae etKS conservées car certains usages ne sont pas observables. Aucun processus étranger arrêté pour le ménage. Versions actives et retours247 conservés. Les deux vues de revue sont conservées selon leur garde native ; aucun retrait forcé de mission, reçu, preuve ou travail Parix.

Les anciennes façadesMCP ouvertes et les wrappers étrangers ne sont pas remplacés par arrêt de leur fournisseur. Les nouveaux démarrages emploientR2 ; aucun redémarrageT3 demandé ou exécuté.
