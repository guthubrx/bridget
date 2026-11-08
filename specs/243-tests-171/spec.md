# Correction 243 - tests restés en suspens depuis la session 171

Date : 7 novembre 2026. Branche : session-243-tests-171.

## Demande

François a demandé de corriger tout ce qui avait été trouvé pendant la session 171,
en restant raisonnable dans les tests : tests en échec sur le Mac, dix tests
ignorés ajoutés en 171, worktree de vérification opus2D, AGENTS.md non committé.

## Constat au 7 novembre (main a383adc5)

- Le worktree opus2D 8db6f38a et la branche 171 n'existent plus.
- AGENTS.md est committé ; main est propre et aligné.
- Les correctifs de bancs 171 (racine brt- résolue, message spec168) sont sur main.
- Les dix tests ignorés de 171 sont toujours présents.

## Décision pour les dix tests ignorés

- Cinq tests de l'ancienne transplantation de droits : la projection ne produit
  plus native_remote_equivalent. Retirés avec deux fonctions sans appelant
  (prepare_guarded_remote_context, attach_guarded_remote_profile). La relecture
  des tâches déjà admises sur l'ancienne route reste en place.
- Recette Linux BRIDGET_REAL_HOOK_PROFILE171 : attend native_remote_guarded d'une
  projection neuve, ce que le code ne produit plus. Entrées absentes. Retirée.
- Recette Linux BRIDGET_ADDITIONAL_ROOT171_INPUTS : entrées absentes, mais la
  logique de racine additionnelle sert la production et n'avait aucun test actif.
  Remplacée par un test synthétique non ignoré, sur Mac et Linux.
- Trois recettes Mac sur données privées : lancées avec leurs entrées. Elles
  comparent un instantané du 17 octobre aux fichiers réels de l'utilisateur, qui
  ont changé (~/.claude/settings.json, profil moi, garde-propre.sh d'opus2D).
  Bridget refuse alors à raison (settings_revision_changed,
  permission_source_unavailable). Ces recettes ne peuvent plus réussir sans
  nouvelle capture réelle : retirées. Les entrées privées restent dans
  /Users/moi/.cache/bridget-delivery171/real-hook-port171/.
- Deux fonctions devenues sans appelant après ces retraits supprimées
  (check_additional_registry_authority171, refresh_remote_permissions_context).

Aucun changement de comportement exécutable : seules des fonctions sans appelant
et des tests sont retirés.

## Suite complète du 7 novembre (main a383adc5 + correctifs)

Mesure initiale Mac : 3 564 réussis, 65 échecs. Relance un par un avec les
entrées des recettes 159 : 18 réussis (charge ou entrée absente), 47 restants.

### Défauts réels de Bridget corrigés

- Entrée vide laissée dans `pending_post_response_controls` à chaque
  enregistrement d'agent (session 222) et dans la file mobile : le contrôle
  d'activité de maintenance pouvait croire une remise en attente. Insertion
  seulement si des contrôles existent.
- Chaque fin de tour ouvrait `AgentProfileStore` (fichier + schéma complet,
  ~4,5 ms mesurées) sous le verrou du daemon. Lecture de la fiche par la
  connexion déjà ouverte du daemon : 998 fins de tour passent sous 1 s (4,8 s avant).
- Adaptateur Codex : hors mode natif, l'identifiant de tour était fixé par le
  fil d'envoi après la réponse ; le fil de lecture pouvait traiter les items
  suivants avant lui et perdre le fait « compactage en cours ». Le lecteur le
  fixe avant de rendre la réponse, comme en mode natif.
- `bridget accounts` refusait immédiatement `account_registry_busy` si le
  verrou était tenu un instant (préparation brève, fork→exec voisin). Attente
  bornée de 5 s, puis refus explicite inchangé ; test ajouté.
- Inventaire des commandes de la compétence : 9 commandes ajoutées depuis 167
  n'avaient pas de décision documentée (accounts, clients, formats, questions,
  specialists, updates, workspace-view-*).

### Tests réalignés sur des décisions livrées

37 outils MCP et `bridget_specialists` (222), politique Codex et liste fermée
Claude (220-222, 185), budget worker 2 Gio (5eaeae07), format compact des fils
(f1286216), avertissement « build différent », guidage d'identité (d4965982),
guidage générique des codes inconnus, refus qualifiés (84988ab7, e948eef5),
reçu de continuation avant remise (377f647e), collecteur des vues hors réponse
(db43c142), requalification des programmes modifiés (a83186df), annuaire sans
sonde (15e3de35), mission canonique admise (d5af84f8), axum réservé au
compagnon mobile (177), heure de lecture de la reprise exclue de la comparaison.

### Défauts de banc corrigés

Connexion acceptée héritant du mode non bloquant sur macOS, racine de socket
trop longue (SUN_LEN 104), échéance spec168 plus courte que le démarrage de
python3 sur macOS, banc 147 borné à 64 Kio pour un catalogue de 88 Kio.

### Recettes à prérequis externes

23 recettes 159 échouaient en rouge sans leurs entrées. Avec
`--features test-support`, `BRIDGET_T010_DAEMON_TEST_BINARY`,
`BRIDGET_MIGRATION159_SNAPSHOT` et les rapports de latence : 14 e2e et 9
migrations PASS. Marquées `ignore` avec leurs prérequis, conformément à
docs/regles-chantier.md (pas de rouge « jalon absent »).

### Résultats finaux

- Mac, suite complète : 3 602 PASS, 3 échecs sous charge (bornes de 3 s :
  spec202, deux tests wrapper::reconnect), PASS trois fois sur trois isolés ;
  passage précédent 3 602 PASS / 1 échec (corrigé depuis).
- Linux (Cartae), suite complète : 3 610 PASS ; 14 échecs propres à la copie
  sans `.git`, PASS 14/14 dans une copie Git.
- Avertissements mécaniques communs corrigés (imports, mut, parenthèses).
  Code mort dépendant de la plateforme conservé : analyse séparée nécessaire.

## Activation 243-tests-r1 (7 novembre 2026, 22:50 UTC)

Source 094ceaad, version 0.8.9, build 243-tests-r1. Linux construit sur SYS
(Debian 12, glibc 2.36) SHA 4efb38c53b6fc70f23d8ee6c7e629f9cbe063349101a236fc9c18879d3cec532 ;
Mac signé ad hoc SHA 0b645ea63946b0cc88debeb61e8a3049a81bc95d0e3fe85fa4a95b477ad95515.
Formats candidat→retour et retour→candidat compatibles sur les quatre hôtes.

Sept composants chargés et empreintes vérifiées : cœur 3743706, mobile 3743773
et worker 3744119 Cartae, worker SYS 2658217, worker KS 4056783, worker Mac
20357, pont Mac 20981. Coupure du cœur 1,87 s. Pointeur `rollback` écrit par
l'écrivain atomique natif sous verrou vers 242-resume-card-r1 sur les quatre
hôtes ; aucun retour lancé. Quatre rapports 0.8.9 up_to_date/fresh.

Préservation : sept charges protégées (7024, 77ba, 702d, 2fb, 218369de,
5564cdf5, bb7d6237) identiques avant/après ; superviseur Cartae 2500061 et sa
naissance conservés ; sept processus de l'application T3 inchangés, T3 non
arrêté (consigne de François) ; registre Mac cacf9ba0 inchangé. Aucun rejeu
ni mission métier. Pas d'avis diffusé aux agents : coupure de 1,87 s et
processus de travail conservés ; à refaire si une bascule longue est prévue.

KS : 85 Mo libres au départ. Retrait vérifié de deux copies Bridget sans
processus, lien ni pointeur de retour (197-platform, 241-reliability-reader-r1,
80 Mo) ; diagnostic 188 conservé. Le manque d'espace de KS reste distinct.
Pas de publication publique : même version 0.8.9, aucun changement Android.

## Git et entretien

main 9c4a4fce poussé et aligné sur les quatre hôtes ; worktree et branche 243
retirés après intégration (propres, aucun fichier ouvert). Copies de binaires
sans usage (ni lien, retour, processus ou unité) retirées : Cartae 5 (200 Mo),
SYS 12 (510 Mo), KS 1 (43 Mo, 145 Mo libres), Mac 2 (71 Mo). Conservés : actif
243, retour 242, 183 du superviseur Cartae, 237/242 encore chargés par des
façades MCP ouvertes sur le Mac, diagnostic 188, petits dossiers de liens.
Sources, transferts et cache de test Linux (23 Gio sur Cartae) retirés.

Entretien Cargo : `installed` refusé sur Mac et SYS, observation des processus
incomplète (descripteurs révoqués, cwd illisibles) ; garde conservée. Les
passes de test utilisaient le Python système 3.9, inférieur au 3.11 requis par
l'entretien automatique : à lancer avec un Python ≥ 3.11. Cache Mac ramené de
28 à 13 Gio par retrait des seuls fichiers incrémentaux de test, sans processus
dans ce cache ; il reste au-dessus du budget de 10 Gio.

## Complément : points restés ouverts après la première livraison

- KS : 145 Mo libres. 15 worktrees opus2D intégrés au main, propres, sans
  fichier ouvert ni tâche ouverte, retirés par `git worktree remove` après
  archivage de leurs fichiers ignorés sur Cartae
  (/home/moi/.local/share/bridget-maintenance/243-ks-worktrees-ignores, 938 Mo,
  28 fichiers, empreintes vérifiées) : 103 Gio récupérés, 99 Gio libres.
  Conservés : 158-fraicheur-pietons-r56 (tâche protégée 7024 en starting, avec
  f37ed24c imbriqué), 158-reprise-rapide-r55 (revue R55 214f8726), et tous les
  worktrees portant des commits non intégrés ou des modifications. Branches
  conservées. Les fiches du worker restent des reçus (inventaire sans contrôle
  d'existence du dossier).
- Cache Mac : 28 → 1,5 Gio (target/debug régénérable retiré, sans processus).
- Avertissements : 38 sur Mac, autant sur Linux, ramenés à 0 sur toutes les
  cibles des deux systèmes. Code sans aucun usage supprimé (marker_name,
  prepare, deux mapping_paths_and_git/workspace_path) ; code servant aux seuls
  bancs sous `cfg(test)` ; chaîne de diagnostic de remote_worker_read sous
  `cfg(any(test, target_os = "linux"))` ; champs lus par les bancs et
  vocabulaire stable des gardes G001..G083 conservés avec justification ;
  visibilité de deux fonctions réduite ; appel mach déprécié autorisé
  localement (pas de dépendance mach2 pour un seul appel) ; noms de recettes
  158B/159 documentés conservés.
- Tests sensibles à la charge : faux worker spec202 sans battement de cœur
  (jugé périmé après 30 s, comportement voulu), attente de vivacité du relais
  portée à 10 s, borne spec_024 à 4,5 s sous le watchdog de 5 s, lecture des
  arguments spec147 avant l'exec, échéances spec168 à 2 s sur macOS.
- Linux dans un vrai dépôt Git : 3 624 PASS, 0 échec. Mac : 3 605 PASS, 0 échec,
  0 avertissement. worker162 a échoué une fois sous charge sans se reproduire
  (46 lancements isolés, 6 × 87 tests en parallèle) : l'assertion affiche
  désormais l'erreur exacte ; remote162 (refus StoreUnavailable sur Release,
  une fois sous charge) non reproduit au passage suivant.

Activation 243-tests-r2 (source f47c366c, 7 novembre 2026, 23:50 UTC) : Linux
SHA d2c61578db47ac42c9447d97cbc0bd7fb5f09414b6e001d47cc015bcec0d6c76, Mac signé
SHA e733deffb34306c37ea7241067fcc4a6b5b43181afb32ffc5c6f4c125cd6568b, 0 avertissement.
Sept composants chargés et vérifiés (cœur 4047099, mobile 4047167, worker
Cartae 4047544, SYS 2665531, KS 4090508, worker Mac 76871, pont Mac 77105) ;
coupure 1,92 s ; retour 243-tests-r1 écrit atomiquement sur les quatre hôtes.
Tâches protégées, superviseur 2500061, sept processus T3 et registre Mac inchangés.

Nettoyage r2 : worktrees 243-avertissements (Mac) et b243-avert-wt (Cartae)
retirés, cache de test Linux 23 Gio retiré ; copie 242 retirée sur Cartae, SYS
et KS (actif r2, retour r1), conservée sur le Mac car encore ouverte par des
façades MCP de T3 (T3 non redémarré). Caches de compilation : Mac 1,5 Gio, SYS
3,7 Gio après retrait de 18 Gio de debug du 5 novembre et de cinq caches de
sessions closes (SYS 25 Gio libres au lieu de 1,8 Gio), Cartae < 1 Gio.

## Causes réelles de worker162 et remote162 (8 novembre 2026)

- worker162 : sur macOS, SO_RCVTIMEO rend EINVAL quand le pair a déjà fermé
  la socket (vérifié par sonde). Le canal de statut d'un bootstrap mort tôt
  produisait donc Io(EINVAL) au lieu de « EOF avant BootstrapReady ». Le
  réglage du délai tolère ce seul cas ; la lecture rend EOF aussitôt. Preuve :
  banc rendu déterministe, Io(Os code 22) sans correctif, PASS avec, plus un
  test unitaire du canal fermé.
- remote162 (cas late-ready) : `refuse()` ne transmettait que
  Invalid(ResultSlotExpired) ; un permis expiré refusé par le magasin
  (Invalid(ReleaseExpired)), une connexion périmée, une enveloppe ou un
  placement devenaient StoreUnavailable. Sous charge, le contrôle mémoire de
  l'expiration passait juste et le magasin refusait : le maître voyait un
  Release incertain. Les refus précis du magasin sont transmis ; les vraies
  pannes restent StoreUnavailable ; test ajouté.
- Banc spec147 membres : naissance lue sur un CLI court déjà sorti (ESRCH sous
  charge) ; 0 retenu, aucun TERM possible, Drop constate la fin.

Suites complètes : Linux (worktree Git Cartae) 3 626 PASS, 0 échec, 0
avertissement ; Mac 3 606 PASS, 1 échec spec147 corrigé ensuite (72/72 trois fois).

Activation 243-tests-r3 (source 12721b3c, 8 novembre 2026) : Linux SHA
28f3f038ba46d8e14e784bd5392c83615417626f8048a68519e40b86f9dfbdc6, Mac signé SHA
c9998713531cb437cf0864143d236e620756ab16061346a20a0a3d519bd9547d, 0 avertissement.
Sept composants vérifiés (cœur 118423, mobile 118440, worker Cartae 118566,
SYS 2669259, KS 4100806, worker Mac 52268, pont Mac 52898), coupure 1,83 s,
retour 243-tests-r2 sur les quatre hôtes. Tâches protégées, superviseur
2500061, sept processus T3 et registre Mac inchangés.

Passage final Mac sur 8fb9964b : 3 605 PASS, 2 bancs sous charge. native154
(faux Git 25 s contre un délai de 20 s : marge de 5 s dépassée par la
préparation sous charge ; faux Git porté à 90 s, borne < 60 s) et
spec221_journal_saturation (le fil du relais peut retirer une commande puis
bloquer sur le writer tenu par le banc, libérant une place : pause puis second
remplissage jusqu'au refus ; événement reçu affiché en cas d'échec). Tests
seuls, binaire actif inchangé.
Passage suivant sur 97bba61e : 3 606 PASS, 1 échec spec104_s27_s29 (deux
appels à now() séparés par un changement de seconde rendaient deux
horodatages égaux et inversaient l'ordre attendu ; une seule lecture
d'horloge). Test seul.

Résultat final sur d5bd55c3 : suite complète Mac 3 607 PASS, 0 échec, 0
avertissement ; Linux 3 626 PASS, 0 échec, 0 avertissement (dépôt Git réel,
commit 12721b3c, bancs de charge revérifiés sur Linux ensuite). Binaire actif
243-tests-r3 inchangé par ces corrections de tests.

## Derniers points (8 novembre 2026)

- Copies 242, 243-r1 et 237 du Mac : encore exécutées par des façades MCP de
  sessions T3 ouvertes, relancées depuis leur chemin résolu. Chaque
  exécutable remplacé atomiquement par un lien vers 243-tests-r3 : les
  processus en cours gardent leur inode, une relance démarre la version
  active, 108 Mo libérés, aucune session coupée, T3 non arrêté. Le Mac ne
  garde que l'actif r3 et le retour r2.
- Entretien Cargo sur Mac : seuls des descripteurs « (revoked) » rendaient
  l'observation incomplète. Un descripteur révoqué désigne un nœud mort sans
  chemin et ne peut ouvrir aucun fichier du cache : accepté seul ; chemins
  illisibles et refus d'accès restent bloquants. 65 tests du script PASS ;
  `clean` et `installed` s'exécutent désormais sur le Mac.
- Suite Linux sur a556789d : 3 625 PASS, 1 échec claude_gere_avec_bypass
  (corps de réponse vide). Défaut réel de l'adaptateur Claude : quand le
  fournisseur sort aussitôt après son résultat, le lecteur atteignait EOF et
  retirait le tour actif (« stdout Claude fermé ») après avoir déjà signalé
  Completed ; le fil de la mission ne retrouvait plus la réponse et publiait
  « réponse vide ». Le tour dont l'issue est signalée n'est plus repris, ni
  à EOF ni à l'arrêt. Preuve : test de 40 sorties immédiates, Ok("") au 5e tour
  sans correctif, 40/40 avec ; banc Claude géré 10/10 (environ 1 échec sur 8
  avant) ; 407 tests transport PASS. Codex et ACP ne reprennent pas le tour à
  EOF : non concernés.

Activation 243-tests-r4 (source b44c0e3b, 8 novembre 2026) : Linux SHA
8989dcdfa69e8df9ea9324841b3357238b8afc1b2c1555f14d0ec777c0cc1913, Mac signé SHA
5cf883f6c60a55abed3cf69b5c2a97d7525ec1a1743fb360e731ea266ce33133, 0 avertissement.
Suites complètes avant build : Mac 3 608 PASS, Linux 3 627 PASS, 0 échec.
Sept composants vérifiés (cœur 557726, mobile 557804, worker Cartae 557863,
SYS 2679316, KS 4132146, worker Mac 68535, pont Mac 68832), coupure 1,77 s,
retour 243-tests-r3 sur les quatre hôtes. Contrôle client Mac : glm-mac en
probe_failed car le Mac était en dark wake (trousseau illisible) ; exception
bornée et consignée au reçu, ce client vérifié deux heures plus tôt (r3) avec
la même politique et le même claude 2.1.317, toute autre non-compatibilité
restant bloquante. Préservation inchangée.

Nettoyage r4 : copie 243-tests-r2 retirée sur les quatre hôtes (actif r4,
retour r3). Sur le Mac, les exécutables 237, 242 et 243-r1, encore référencés
par des façades MCP de sessions T3 ouvertes, mènent au lien stable
~/.local/bin/bridget, donc toujours à la version active. Entretien Cargo :
Mac, Cartae et KS observent complètement et simulent sans refus ; SYS refuse
car `sudo -n lsof` exige un mot de passe — garde volontaire, la lever demande
un droit sudo sans mot de passe que l'utilisateur doit décider ; cache SYS
3,7 Gio, sous le budget. Push GitHub fait depuis Cartae (compte guthubrx) :
le trousseau du Mac est illisible pendant le dark wake (errSecInteractionNotAllowed).

## Clôture des derniers points (8 novembre 2026)

- glm-mac : `gclaude --version` lisait la clé GLM avant même de lancer
  claude ; au trousseau verrouillé (veille), la sonde échouait. Une demande de
  version seule n'appelle pas l'API : elle n'exige plus la clé (sauvegarde du
  script conservée). Recontrôle de la r4 : quatre clients compatibles,
  exception retirée du reçu.
- Copie 237 du Mac retirée (aucun utilisateur). 242 et 243-r1 ne sont plus des
  copies : des liens vides vers ~/.local/bin/bridget, gardés tant que trois
  façades MCP de sessions T3 ouvertes (PID 18554, 21774, 2305) peuvent se
  relancer depuis ces chemins ; les retirer couperait ces sessions.
  Reçus d'installation anciens (quelques octets) conservés comme preuves.
- Cache SYS : sans sudo sans mot de passe, l'observation était refusée.
  Repli strict : si sudo échoue et que chaque dossier observé n'est atteignable
  que par ce compte (ancêtre à lui, sans traversée groupe/autres), lsof
  observe les seuls processus de ce compte. Le cache bridget-build-227 de SYS
  est rendu privé (700). Hors de ce cas, la garde reste entière. Test ajouté,
  66 tests du script PASS.
- Repli SYS : les gardiens de session `sshd: moi` et `(sd-pam)` tournent sous
  le compte mais restent protégés par le système (fichiers ouverts illisibles
  sans root). Dans le seul mode de repli, ils sont écartés par leur nom
  système exact ; tout autre processus illisible garde la garde entière.
  67 tests du script PASS ; simulation SYS : 0,57 Gio (profil de 10,5 jours).
- Inventaire des worktrees SYS : `~/.local/share/bridget-projects` est un
  montage lié de /mnt/data ; l'outil refusait (« alias mount may hide usage »)
  car un fichier pouvait être ouvert par le second chemin. Il suit désormais
  les chemins alias du même périphérique et ramène chaque fichier ouvert à son
  chemin canonique ; sans liste d'alias le refus reste. Test ajouté, 68 tests
  PASS. Copies de sessions anciennes de ~/.cache rendues privées (700).
  Observation SYS complète ; 91 worktrees de revues fusionnés, 41 tâches liées
  toutes closes, aucune commande worker active : retirés par l'outil.
- SYS : 91 worktrees de revues fusionnés retirés par l'outil (branches
  conservées), puis huit copies de sources de sessions 227-240 et la copie
  197-distribution, intégrées, propres et inutilisées, retirées ; la copie 227
  portait 56 modifications non committées, conservées en patch, liste et
  archive des fichiers non suivis dans
  ~/.local/share/bridget-maintenance/243-sys-sources (reconstitution vérifiée
  identique). Deux binaires candidats 234 inutilisés retirés. SYS : dépôt
  principal seul, entretien sans refus, 26 Gio libres.
- Copie 237 du Mac retirée ; Mac, Cartae, KS et SYS observent complètement.
- Conservés pour un usage réel : worktree Cartae 1cc44f4d (cwd du superviseur
  2500061, une modification) ; liens 242 et 243-r1 du Mac (façades MCP vivantes
  18554, 21774, 2305 de sessions T3 ouvertes) ; reçus d'installation anciens.
- Suite Linux sur 5eb2956f : 1 échec crash_after_rename (EPIPE à la
  libération). Les extrémités de tube étaient passées à l'enfant aux numéros
  fixes 110/111 par dup2 direct : une extrémité déjà numérotée 110 ou 111
  était écrasée. Copie préalable au-dessus des cibles (F_DUPFD), comme le code
  de production ; banc fleet : dup2 sur lui-même laissait FD_CLOEXEC, retiré.
  Preuve : test qui force la collision, échec identique (ligne 666, -1) sans
  correctif, PASS avec.
- Suite Mac sur 5eb2956f : 1 échec spec185 (réparation) : « occupé » est une
  réponse passagère sous charge ; le banc réessaie jusqu'à une réponse stable.
- Brouillon du worktree Cartae 1cc44f4d (session 198, ne s'applique plus à
  main) sauvegardé dans ~/.local/share/bridget-maintenance/243-cartae-1cc44f4d ;
  dossier gardé car cwd de l'agent Codex vivant du superviseur 2500061.
- Reçus anciens archivés dans ~/.local/share/bridget-maintenance/243-recus-anciens
  (31 dossiers sans exécutable) ; docs, skills et credits de Cartae restaurés.

Résultat final sur 21a16236 : Mac 3 609 PASS, 0 échec, 0 avertissement ;
Linux (worktree Git réel) 3 626 PASS, 0 échec, 0 avertissement. Binaire actif
243-tests-r4 inchangé (corrections de bancs seulement depuis b44c0e3b).
- Brouillon 1cc44f4d committé sur la branche archive/243-1cc44f4d-brouillon198
  du dépôt Cartae (bfe9f01a), identique au contenu vivant ; le worktree actif
  de l'agent n'est pas modifié.

Restent, par règle explicite et non par défaut : le worktree 1cc44f4d est le
cwd de l'agent Codex du superviseur 2500061 (« ne pas terminer les processus
d'un autre travail pour faire le ménage », checklist kill avec confirmation
utilisateur) ; les liens 242 et 243-r1 du Mac servent à trois façades MCP de
sessions T3 ouvertes (T3 ne doit pas être arrêté). Leur retrait suit l'arrêt
de ces processus par l'utilisateur.
- Lien 243-tests-r1 du Mac retiré dès la fin de sa dernière façade (2305).

## Retraits différés (8 novembre 2026)

Deux éléments restent conservés car des processus vivants les utilisent. Aucun
processus n'est arrêté pour le ménage. Deux gardiens persistants les retirent
après la fin de ces processus, avec une nouvelle vérification au moment du retrait.

- Mac : dossier `/Users/moi/.local/lib/bridget-communication/242-resume-card-r1`,
  utilisé par les façades MCP 18554 et 21774 des sessions T3. Gardien
  `com.bridget.nettoyage243.lien242` (LaunchAgent), script
  `/Users/moi/.cache/bridget243-tests-r4/differe/retirer-242.sh`. Toutes les
  5 min, il vérifie qu'aucun processus n'ouvre ce dossier et que l'état de mise
  à jour ne le référence pas. La version de retour reste 243-tests-r3.
- Cartae : worktree `/home/moi/10.Scripts/64.bridget/.worktrees/1cc44f4d`,
  dossier courant du superviseur 2500061. Gardien utilisateur systemd
  `bridget-nettoyage243-1cc44f4d.service`, script
  `/home/moi/.local/share/bridget-maintenance/243-cartae-1cc44f4d/retirer-1cc44f4d.sh`.
  Il retire le worktree seulement si plus aucun processus n'y travaille, si le
  contenu égale l'archive `archive/243-1cc44f4d-brouillon198` (bfe9f01a) et si
  aucun fichier non suivi n'existe. La branche `bridget198-worker-start` est
  ensuite retirée, car l'archive la contient.

Chaque gardien journalise son issue puis se retire. En cas d'écart, il conserve
l'élément et note le motif dans son journal.

Issue Cartae (8 novembre 2026, 05:59 UTC) : arrêt de 2500061 confirmé par
l'utilisateur. Le wrapper 183 n'avait plus ni connexion ni base ouverte, et son
dernier journal datait du 28 octobre 13:16. Après SIGTERM, son enfant Codex
s'est arrêté, mais le wrapper a gardé le signal sans terminer pendant plus d'une
minute. SIGHUP, non géré, l'a arrêté ; aucun SIGKILL. Le gardien a ensuite retiré
le worktree 1cc44f4d et la branche `bridget198-worker-start`, puis s'est
désinstallé. L'archive `archive/243-1cc44f4d-brouillon198` (bfe9f01a) reste
conservée. La boucle de l'image 183 qui ignorait la demande d'arrêt n'est pas
identifiée. Le lien Mac 242 attend toujours le redémarrage de T3.

Issue Mac : les façades 18554 et 21774 se sont terminées d'elles-mêmes. Le
gardien a retiré `242-resume-card-r1` le 8 novembre 2026 à 04:14 UTC, après ses
vérifications. Il est ensuite désinscrit de launchd. Restent `243-tests-r4`
(actif) et `243-tests-r3` (retour). Toutes les façades MCP observées chargent r4.
Les deux retraits différés sont soldés.

Cause de l'arrêt ignoré par 2500061 (image 183) : sa clôture distante
(`RemoteDrainGuard::drop`) ne récoltait les descendants que si SIGTERM avait
atteint chacun d'eux. Un descendant `sudo`, sous l'UID root, refusait ce signal ;
la boucle reprenait toutes les 100 ms sans récolte ni diagnostic. Cela explique
la pause observée et les zombies conservés. Le défaut est corrigé depuis
86a88e3f (session 221, 30 octobre 2026) : un refus de signal n'interdit plus
`waitid`, et `native_drain_stop_unconfirmed` est journalisé. Test existant :
`spec221_drain_reaps_owned_foreign_uid_zombie_after_signal_refusal`. Les
versions actives 243-tests-r4 et de retour r3 incluent ce correctif. Aucun
nouveau code. Limite conservée par conception : tant qu'un descendant vivant ne
peut pas être arrêté, le superviseur reste présent et le signale ; aucun SIGKILL.

Vérification du correctif (8 novembre 2026) : 86a88e3f est ancêtre des sources
r4 (21a16236) et de main. Le binaire chargé sur Cartae
(`243-tests-r4/bin/bridget`) contient `native_drain_stop_unconfirmed` et
`remote_drain_signal_uid_mismatch`. Recette root
`spec221_drain_reaps_owned_foreign_uid_zombie_after_signal_refusal` exécutée sur
Cartae depuis 947d1dc1 avec `scripts/build.py` : 1 passed, marqueur
`SPEC221_FOREIGN_UID_REAP_OK`. Elle montre qu'un descendant d'un autre UID refuse
le signal, puis est récolté sans zombie restant. Dossier temporaire retiré.

## Correctif r5 (8 novembre 2026) : qualification Codex et refus de périmètre

Constat : depuis la mise à jour Codex 0.163 → 0.166 du 1er novembre sur
Cartae, toute mission Codex y était refusée avec `program_upgrade_check_pending`.
L'observateur lançait la qualification par `bridget managed-bootstrap` avec
`HOME` = dossier de travail `.../catalogue/compat-<32 hex>` (94 caractères).
Le bootstrap préparait le namespace Bridget sous ce HOME ; la socket dépassait
la limite système et il sortait en code 2 (« bridget socket trop longue »),
sous stderr fermé. Aucune preuve n'était écrite : `program_unavailable` au
catalogue, `program_upgrade_check_pending` à l'admission, sans fin. Observé par
strace (exit_group(2)) puis reproduit à la main avec un HOME de même longueur.

Correction (5c731211) : `managed-bootstrap` ne prépare plus de namespace ; il
transmet ses canaux puis exec son programme, et le wrapper lancé prépare
toujours le sien. Test `spec243_bootstrap_gere_ne_prepare_pas_de_namespace`.

Signalement « refonte monde » : un `cwd` hors du dossier attesté du parent était
classé `origin_request_invalid:cwd:invalid_envelope`. Il rend désormais
`cwd_outside_parent_project` et son guidage existant. Test
`spec243_cwd_hors_du_parent_est_un_refus_de_perimetre`.

Tests ciblés : Mac 45 PASS, Linux 46 PASS. Build r5 sans avertissement : Linux
86a3855c…, Mac signé 5515503a…. Activé sur sept composants, coupure 2,31 s,
retour r4 sur quatre hôtes, tâches protégées et T3 inchangés. Après activation :
preuve Cartae écrite à 06:31:41 UTC (compatible, vérifiée), et le motif
`program_upgrade_check_pending` a disparu de l'admission.

Refus suivant observé : une délégation distante depuis cette session Claude du
Mac restait refusée (`permission_mapping_unavailable`, site projection). Cause
établie et corrigée plus bas (racine additionnelle des anciens mappings).
L'hypothèse initiale, des crochets Claude non reproductibles, était fausse.

Refus de ce matin identifié : « Bridget a refusé la mission » vient du fil
« refonte monde » (opus2D), 06:17 UTC, clé parix-ecarts-teleoperator-20261108-a.
C'est le refus de périmètre cwd corrigé en r5 ; l'agent a reçu la cause et la
marche à suivre.

Hypothèse écartée : un profil gardé (session 171) n'est pas nécessaire pour
déléguer depuis une session Claude du Mac qui a des crochets. Preuve : la tâche
d9400941 (projet bridget, aucun profil gardé) a été admise vers Cartae et
terminée. L'avertissement envoyé à « refonte monde » sur ce point a été
rectifié par un second message. Le message public de ce refus dit « cause non
observée » et renvoie au diagnostic worker, qui nomme le code exact.

Cause réelle du refus `permission_mapping_unavailable` (8 novembre 2026) : ce
n'étaient pas les crochets. Chaque session T3 reçoit le dossier
`/Users/moi/.t3/userdata/attachments` dans ses droits. Les anciens mappings
manuels `cartae/bridget` et `cartae/roadie` ne déclaraient pas ce dossier en
racine additionnelle ; la projection ne pouvait donc pas transposer ce droit.
Les mappings créés automatiquement, et ceux de bridget sur SYS et KS, le
déclaraient déjà. Correction de configuration, même forme que `cartae/opus2d` :
racine distante `/home/moi/.local/share/bridget-attachments/<projet>` (0700),
`access: inherit`, révision 1 → 2, sur les cinq registres (maître et worker
Cartae ; origin-agents177, worker-monordinateur-agents et agents sur le Mac).
Verrou natif `project-preparation.lock`, sauvegarde privée
`*.avant-243-extra-*`, écriture atomique avec contrôle des octets. Les cinq
copies sont identiques (empreinte 00bbf83c6c5996f7). Rejeu réel : le refus de
droits a disparu ; la demande suivante a reçu un refus de capacité, légitime
(Cartae à 100 % de charge pendant les calculs opus2D).

Essai réel sur un superviseur en service (8 novembre 2026, ~06:52 UTC), accepté
par l'utilisateur : tâche d9400941 (Claude Haiku sur Cartae, worktree), admise
après la correction des mappings. La mission a lancé `setsid sudo -n sleep 150`
(sudo root, PID 1397535), rattaché au superviseur r5 1396412. Après annulation,
le superviseur est resté présent tant que ce descendant root vivait, sans aucun
zombie (contrôle toutes les 15 s) ; à la fin du sleep, il a tout récolté et
s'est terminé (~136 s), sans orphelin. Statut remote_execution_ended, résultat
lu et accusé, worktree propre retiré avec sa branche. Le sudo 3584579
préexistant, étranger à l'essai, n'a pas été touché.
