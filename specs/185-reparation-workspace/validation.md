# Validation — Session 185
Statut : implémentation et qualification en cours. Aucune activation ni réparation de la tâche métier réelle.

## Autorisation
L'utilisateur a demandé le 27 octobre « fais ce qui est nécessaire », après explication du parcours natif. Le travail conserve la même tâche et ne permet ni forçage, ni édition directe d'état métier, ni redémarrage de service déduit.

## Source
Base : fe776eda5f65e07571bdf28776bf0ee21dde97ba.
Branche : session-185-reparation-workspace.
Checkout : /home/moi/10.Scripts/64.bridget/.worktrees/185-reparation-workspace.
Cache Cargo commun : /home/moi/10.Scripts/64.bridget/target.
Les commandes Cargo passent par python3 scripts/build.py cargo.

## Revue de conception
- b3bad360-3746-47e4-afc1-21b669b567fe : aucun verdict, le brief interdisait le seul lecteur disponible (shell).
- 543a123c-619d-4cb1-b4d9-54b73f817c07 : lecture effective, avis favorable au parcours dédié, avec exigences sur ancrage, quiescence, arrêt borné, même fil, durabilité, négociation et ancien wrapper.
- Ces exigences structurent l'implémentation. La contre-revue du premier diff a imposé huit corrections, détaillées ci-dessous.

## Contrôles déjà observés
- cargo check -p bridget-daemon --lib : PASS avant les derniers compléments.
- 5 tests spec185_ bridget-transport : PASS. File/steer/tour externe/approbation/événement non lu ; reprise non attestée ; arrêt de l'unique fournisseur ; arrêt incertain borné ; schéma fermé.
- 8 tests spec185_ bridget-daemon : PASS, avec publication worker et attestations après réparation. Trois contrôles Git ; parent étranger sans mutation ; continuation fermée ; guidage ; wrapper réel avec cwd supprimé, reprise du même fil et aucun tour métier.
- Worker : comparaison corrigée après observation réelle des définitions de fixture. Le contrôle réapplique désormais modèle, effort et politique héritée avant comparaison avec la définition admise ; la nouvelle projection est vérifiée séparément. Le canon et les slots restent identiques. Le refus de la première fixture sous répertoire protégé a été conservé.
- 4 tests mcp_scopes : PASS (chaque outil a une portée, y compris la réparation).
- Aucun test de panne maître/worker à toutes les frontières ni test réel de la tâche SYS n'est encore attesté.

## Runtime
Le binaire actif reste celui de la session183. Le candidat184 attend déjà une maintenance native admise. Installer un fichier ne remplace pas les programmes chargés. L'ancien wrapper de la tâche SYS ne possède pas la capacité185 ; aucun succès rétroactif n'est promis.

## Prochaines actions
Finir les contrôles du worker et de coordination ; revoir le diff ; intégrer et pousser après validation ; préparer le binaire ; respecter le refus de maintenance s'il persiste. Traiter explicitement le préalable de migration de l'ancien wrapper, sans mission de remplacement.

Aucune interface graphique modifiée : contrôles techniques, pas de diaporama requis.

## Contre-revue du candidat 2164346a
Tâche478e2ba5-4595-466a-8803-5963e7788dce ; clé bridget-spec185-code-review-20261027-03 ; verdict défavorable.
Les huit objections ont été reprises dans la source suivante, qui attend une nouvelle revue :
1. Verrous fournisseur : try_lock, ordre pending puis queue, délai établi avant les verrous.
2. Reçu repris : lecture et réconciliation avant contrôle d'expiration autorisant un nouvel effet.
3. Annulation : garde relue après préparation, sous le verrou d'écriture du contrôle et à chaque publication ; signal natif observé entre les effets du wrapper.
4. Admission : aucune entrée de socket fournisseur pendant la barrière. Préparation, commit worker, décision maître durable puis publication worker. Aucun événement système ne passe avant cette publication.
5. Reprise ratée : arrêt TERM/INT borné, ressource de lancement conservée si arrêt incertain, aucune attente de lecteur non bornée dans ce parcours.
6. FIFO : ouverture non bloquante puis contrôle du type.
7. HEAD : égalité pendant la migration ; ensuite même branche et ascendance du HEAD de migration, avec conservation des identités de dossiers.
8. Refus : terminalité explicite avant effets ; refus wrapper repris durablement par le worker, puis nouvelle clé autorisée sans écraser l'historique.
Les répétitions de lecture ont été retirées ; la négociation utilise une structure nommée et des capacités indépendantes ; le contexte d'échec reste réservé au parent propriétaire.

Diagnostic de qualification : le cas workspaceWrite a révélé un séparateur final ajouté par join sur un suffixe vide. Les politiques observées différaient seulement entre /disk185/ et /disk185. La traduction conserve désormais exactement la racine canonique ; aucune restriction n'a été assouplie.
Le test d'échec de reprise comporte une sonde de compte sans réponse, bornée à2s, puis un nettoyage borné à4s. Il conserve le propriétaire tant que le processus ou son lecteur n'est pas arrêté. La fixture finit seule, sans SIGKILL.

## Contrôles après résolution des huit objections
-12 tests daemon spec185_ : PASS. Incluent annulation/expiration avant contrôle, refus durable, accès parent étranger, FIFO, ancrage Git, équivalence workspaceWrite, commit après publication, reprise réelle du même fil, rejeu après expiration sans nouvel effet.
-8 tests transport spec185_ : PASS. Incluent verrou fournisseur déjà pris et échec de reprise avec processus/lecteur résistants, conservés sous borne.
-13 tests de module MCP : PASS ;1 test CLI du cycle natif : PASS ;4 tests de portées MCP : PASS ;10 tests d'attestation d'outils : PASS.
Les journaux locaux de ces passages sont conservés sous /home/moi/.cache/bridget184/spec185-*.log, hors Git.
La compatibilité MCP complète et la revue du commit corrigé restent en cours. Les tests ne constituent pas une qualification exhaustive de toutes les coupures à toutes les frontières. Aucun fournisseur réel ni tâche métier réelle n'a été lancé par ces contrôles.

## Contre-revue57018bf6 et seconde correction
Source8bdbfab0 : verdict défavorable. Le candidat compilé8bdbfab0 n'est pas éligible à l'activation.
Cinq objections :
- Publication worker : le verrou partagé des annulations couvre désormais la relecture fraîche et la persistance de la publication.
- Réponse de publication perdue : une opération Observe sans effet consulte le reçu avant les gardes d'admission. Le maître peut réconcilier sa décision publishing après expiration ou annulation, sans réactiver la tâche.
- Écriture du refus wrapper : aucun terminal n'est annoncé si le reçu n'est pas écrit. Le wrapper conserve son résultat en mémoire, ferme les admissions et ne retente que l'écriture. Le résultat repris attesté utilise cette même écriture finale.
- Autorité du parent : le dossier et les droits proviennent de la projection publiée. Les preuves worker ParentProof et ParentSourceProof utilisent le placement effectif ; leurs canons restent inchangés. Le contrôle du parent distant utilise l'identité de commande attestée par la flotte, et non l'hypothèse command_id=task_id. Le journal de test a confirmé cette divergence avant correction.
- Collecte : la capacité et chaque reçu de réparation sont retirés avec le reçu de commande, sous ses conditions de terminalité, rétention et absence de slot référent. Une commande active les conserve.

14tests daemon185 : PASS après ces corrections. Un aller-retour réel de protocole vérifie Observe après expiration et annulation ; la tâche reste annulée. Le test de délégation vérifie le nouveau fait parent et les preuves worker. La fixture d'écriture défaillante ne retourne aucun refus terminal puis persiste le même reçu dès rétablissement. La collecte conserve les marqueurs actifs puis retire les marqueurs avec la commande terminale hors rétention.

## Troisième revue et échéance finale
Tâche0d1fd469-d80d-4650-afbd-440c7081747d ; clébridget-spec185-code-review-20261027-06 ; source177d0a5e : un bloquant.
L'échéance est désormais relue au point atomique de commit/publication, sous le verrou d'annulation et après Git/droits. Un dépassement conserve la phase et la barrière. Les lectures idempotentes et Observe restent possibles sans nouvel effet. Le test worker refuse Commit puis Publish avec un délai expiré et vérifie l'absence de mutation.
15tests daemon185 PASS dans la source corrigée.

## Guidage des briefs refusés
La demande de revue05 a été refusée sans tâche : référence de fichier mal formée (source_label interdit pour kind=file, chemin relatif et host absent). Le brief a été corrigé avant la demande06 ; aucun doublon de mission.
Le diagnostic source a confirmé que l'admission durable supprimait le suffixe de brief_invalid. Le champ et la cause sont maintenant conservés pour les nouveaux refus et leur rejeu, dans la borne historique128octets et sans caractères de contrôle. Aucun changement du schéma ni assouplissement de validation. Un test réserve puis refuse un vrai reçu SQL, relit le code durable et sa projection publique, et constate l'absence de tâche/lancement.
Les anciens reçus dont le diagnostic n'a jamais été enregistré ne sont pas réécrits.

## Régression élargie
La suite de droits a passé128tests avec un échec d'attente JSON ancien et1test explicitement ignoré. Le refus permission_not_inherited était correct ; seul son nouveau guidage manquait dans l'attente. Le test conserve ses contrôles d'absence de tâche/lancement et vérifie désormais le code plus le guidage.
La régression du collecteur a aussi révélé le cas ancien sans dossier markers : cette absence est désormais une collecte vide, et un dossier existant doit rester privé avant tout retrait.
Les passages finaux de ces régressions restent à consigner.
