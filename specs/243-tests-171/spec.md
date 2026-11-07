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
