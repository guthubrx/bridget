# Règles de chantier partagé (worktree multi-agents)

Chaque règle est née d'un incident réel. Ne pas alléger sans avoir revécu
l'incident.

## Le worktree

1. **Un worktree par session, partagé par tous les codeurs.** Git sépare les
   commits ; les fichiers non commités sont un espace commun — d'où tout ce
   qui suit.
2. **Le worktree compile à tout instant.** On ne déclare jamais un module dont
   le fichier n'existe pas ; si on déclare, on crée le fichier dans la même
   minute, même minimal. *(Incident : lib.rs déclarant store/outbox absents —
   plus personne ne compilait, 2026-08-23.)*
3. **Des fichiers disjoints par tâche.** L'assignation nomme les fichiers ;
   sortir de son couloir se signale AVANT d'écrire.
4. **Les fichiers partagés (Cargo.toml, lib.rs) ont UN propriétaire par
   phase.** Les autres passent commande ; le propriétaire pose l'ajout dans
   son prochain commit. *(Incident : trois écrivains sur Cargo.toml.)*
5. **Un stub ne se crée que si le fichier n'existe pas.** Un fichier existant
   appartient à son auteur, committé ou pas. *(Incident : stub écrasant le
   config.rs WIP d'un autre agent.)*
6. **Le checkout principal n'est PAS un chantier.** Il appartient au référent
   (docs, greffe, merges) et au daemon (binaire de production). Aucun agent
   n'y écrit du code, aucun agent n'y bascule de branche. *(Incident
   2026-08-24 19h17 : un agent a basculé le checkout principal sur sa branche
   — le commit de docs du référent a atterri dessus — pendant qu'un second
   agent y modifiait wrapper.rs : course à deux mains dans le même arbre,
   travail extrait en patchs de sauvetage, checkout rendu à main.)*
7. **Le worktree de l'auteur n'est JAMAIS ouvert au relecteur.** Toute revue
   (lecture appuyée exceptée) se fait sur une COPIE DÉTACHÉE du SHA gelé :
   mutants plantés et restaurés chez soi, jamais chez l'auteur. *(Incident
   2026-08-24 ~21h : une restauration de mutants dans le worktree de l'auteur
   a effacé son C4-bis non commité ; seul un garde-fou de script a évité un
   faux vert sur mutant jamais posé.)* En miroir, l'auteur committe tôt et
   souvent pendant une revue active : un WIP à découvert n'est protégé par
   rien.
8. **`maicie delegate --constat-id` exige l'identifiant COMPLET du registre**,
   préfixe de kind inclus (`review_amender:constat/…`, `gate_failed:…`) —
   celui affiché par `registre list`. Un identifiant court passe au delegate
   mais casse la réconciliation du catalogue à la clôture de l'objectif
   (« référence inconnue »). *(Incident 2026-08-24 19h25 : quatre délégations,
   réparées par chirurgie du greffe — et noter que `payload_json` y est du
   BLOB : un `json_set` nu le réécrit en TEXT et casse le lecteur.)*

## Les commits

6. **L'état validé se committe immédiatement** — le nettoyage/formatage vient
   dans un commit suivant, jamais avant la mise à l'abri. *(Incident : T1207
   perdu dans un nettoyage de formatage, 2026-08-22.)*
7. **Un commit livré en review est immuable.** Correctif = commit par-dessus,
   jamais d'amend.
8. **Livraison = hash annoncé par message** (outil bridget_send). Un commit
   est un événement muet : personne n'est réveillé par git. Tout événement
   attendu par quelqu'un doit avoir un messager.
9. **Après tout stash pop ou merge : validation immédiate** (build + tests
   ciblés) avant de continuer.

## Les validations

10. **Tests de crash réels** (processus tués à des barrières), jamais simulés ;
    les bancs ont un **timeout global** qui échoue proprement au lieu de
    pendre. *(Incident : 4 cargo test pendus 2 h 20.)*
11. **Un rouge hors périmètre se signale et s'arbitre** (dérogation consignée
    + fix assigné séparément) — on ne le contourne pas en silence, on ne le
    répare pas hors scope non plus.
12. **Valider sur un arbre contaminé par le WIP d'autrui ne prouve rien** —
    vérifier `git status` avant d'attribuer un rouge à son propre diff.
12b. **`cargo check --workspace` ne compile PAS les fichiers de tests.** Un
    lot peut être « check vert + tests ciblés verts » avec quatre suites
    d'intégration qui ne compilent plus. Gate minimal avant livraison :
    `cargo test --workspace --no-run` (ou `-p <crate>` du lot). *(Incident :
    F36/F37 BLOCKED en revue, 2026-08-24 — et ses oracles de schéma périmés
    mentaient dans le sens permissif.)*

**Gate de formatage obligatoire avant commit.** Exécuter `cargo fmt --all
--check` avec la toolchain de référence `rust-toolchain.toml` (Rust 1.92.0,
`rustfmt 1.8.0-stable`) et le `PATH` contenant `$HOME/.cargo/bin`. Sans ce
`PATH`, la commande peut ne jamais s'exécuter tout en donnant l'illusion d'un
contrôle vert. *(Incident : dette de 23 emplacements non détectée sur main,
2026-08-24.)*

## Les reviews

13. **Auteur ≠ relecteur, toujours.** Tout STOP est vérifié factuellement par
    le référent avant relais.
14. **Négocier les contrats d'interface AVANT le commit du fournisseur**
    (consommateur propose, propriétaire dispose, référent tranche les
    désaccords). *(Bon réflexe observé : API store T006↔T008.)*
15. **Jamais de réponse en double à une même demande**, même après un rappel
    du daemon — si le rappel arrive, c'est la *liaison* qui a échoué, pas la
    réponse.
16. **Rien ne reste stagé dans l'index partagé.** On stage au moment du
    commit, jamais avant : l'index est commun, un commit voisin emporte tout
    ce qui y traîne. *(Incident : le commit socle-client a emporté le T017-2
    stagé d'un autre agent, 2026-08-23.)*
17. **Qui touche une ressource globale ordonnée le dit au greffe AVANT de
    committer** — les deux cas connus : un numéro de migration de schéma, et
    les structs/enums filaires de `protocol.rs`. Git fusionne le texte mais
    pas le sens : deux lots localement cohérents peuvent se voler le même
    numéro ou construire un `Register` que l'autre a déjà changé. Le message
    au greffe nomme la ressource et la plage prise ; le référent sérialise
    les lots concurrents ; et après tout rebase sur un lot qui touche
    `protocol.rs`, la compile du workspace est rejouée AVANT d'appeler le
    merge « fait ». *(Incidents : v8 pris deux fois par 016 et le lot refus
    guichet ; champ `journal_available` ajouté à `Register` cassant la
    compile du lot G5, tous deux le 2026-08-24.)*
17. **Un verdict n'existe que signé et lié.** Tout APPROVE/STOP doit venir de
    l'identité STABLE du relecteur désigné, en réponse liée à la demande de
    review — un verdict d'émetteur éphémère (cli-send-*) ou non lié est NUL
    et mis en quarantaine jusqu'à confirmation d'identité. *(Incident : un
    APPROVE anonyme sur un lot à deux co-auteurs, désavoué par le relecteur
    désigné, 2026-08-23.)*
18. **En couloirs ouverts, chacun valide sur SES cibles.** Le workspace
    complet vert n'est exigé qu'au gate de clôture de la session (tâche de
    non-régression), quand tous les lots sont commités. Un rouge hors
    couloir ne se signale que s'il PERSISTE (~10 min) : sur un chantier à
    plusieurs couloirs actifs, le WIP voisin rend tout constat vrai à T et
    périmé à T+2 min. *(Incident : ping-pong de signalements
    vrais-mais-périmés entre deux couloirs de la session 014, 2026-08-23 —
    deux vérifications du référent elles-mêmes périmées à l'arrivée.)*
19. **Toute mission a son worktree, même micro.** On ne change JAMAIS de
    branche dans le checkout principal : c'est l'espace de l'orchestrateur
    et des merges. Un agent qui doit committer crée
    `.worktrees/<branche>/` et y travaille. *(Incident : trois branches
    empilées par bascule dans le checkout principal, un commit du référent
    égaré sur la branche d'un équipier, mandats sans worktree — bloc C,
    2026-08-23 soir.)*

## Rituel de clôture de session (référent)

À chaque clôture de session ou de bloc, dans l'ordre, aucune étape omise :

1. Toutes les cases de `tasks.md` cochées sur preuves — jamais de coche de
   confort.
2. Tous les objectifs Maicie clos au greffe, motif complet (hashs + verdicts
   + relecteurs). *(Incident : oublié 2× le 2026-08-23 — approuvé ≠ clos.)*
3. Merge dans main dans l'ordre des dépendances, push, branches et worktrees
   purgés (règles XVI et 19) après contrôle lsof.
4. Rebuild release ; si les crates daemon/wrapper ont changé sémantiquement,
   redémarrage du daemon (résurrection automatique des persistants).
   *(Incident : daemon périmé 2× le 2026-08-23.)*
5. Passe d'annotation du catalogue v2 : chaque entrée touchée reçoit son
   marqueur ✅ LIVRÉ / ⏳ OUVERT / PARTIEL / REJETÉ, daté et sourcé.
   *(Incident : catalogue muet sur les livraisons 014, constaté par
   l'utilisateur le soir même.)*
6. Statuts des spec.md alignés sur la réalité (ni flatteurs ni périmés).

## Rituel d'arrivée d'un agent (référent)

Un agent n'est utilisable qu'après DEUX inscriptions distinctes, dans deux
fichiers différents. Une seule ne suffit pas, et l'oubli de la seconde ne se
voit qu'au premier échec de délégation.

1. **Être joignable** — type déclaré dans `~/.config/bridget/agents.json`
   (mode 0600) : commande, arguments, protocole, variables d'environnement
   interdites. Sans lui, `bridget spawn` répond « type d'agent inconnu ».
   *Le daemon lit ce registre à SON démarrage : après modification, il faut
   le relancer (`launchctl kickstart -k gui/$(id -u)/com.bridget.daemon`),
   sinon le nouveau type reste invisible.*
2. **Être missionnable** — profil déclaré dans `~/.config/maicie/config.json`,
   section `profiles` : `id`, `agent_name` (le nom Bridget exact),
   `display_name`, `tags`. Sans lui, `maicie delegate` répond « cible
   indisponible » alors que l'agent est bel et bien connecté.

Cette séparation est voulue, ce n'est pas un défaut à contourner : être
joignable et être autorisé à recevoir du travail sont deux décisions
distinctes. Un agent peut légitimement être connecté sans être employable.

3. **Enseigner le guichet dès le premier mandat.** Le nouvel agent doit
   savoir que Maicie n'est pas joignable en direct et que son absence de
   l'annuaire est nominale, sinon il conclura à une panne — trois agents
   l'ont fait la même nuit. Voir la section « Rapporter à Maicie : le
   guichet ». À écrire dans le mandat de calibrage, pas plus tard.
4. **Première mission = calibrage**, jamais un couloir. Une tâche réelle,
   bornée, en lecture seule, dont on connaît déjà la réponse ou dont on peut
   vérifier le résultat sur pièces. On y exige la ligne `MODELE:` en tête,
   et on y énonce les deux règles maison : aucune modification sans mandat,
   et livraison par `bridget send` explicite.
5. **Clés API interdites** dans la déclaration du type : `forbidden_env` liste
   les variables à refuser. L'authentification passe par l'abonnement.

*(Incident fondateur : arrivée de `cursorbridget` le 2026-08-24 — spawn refusé
faute de type au registre, puis délégation refusée faute de profil Maicie,
deux échecs successifs pour un même agent.)*


## Pièges de validation connus (référent)

Écrit le 2026-08-24 après que deux agents s'y soient fait prendre la même nuit.

- **`cargo test --workspace` n'active PAS les fonctionnalités de test.**
  Certains points d'arrêt utilisés par les tests de crash sont derrière
  `#[cfg(feature = "test-support")]` (par exemple `before_coordination_persist`,
  `daemon.rs`). Sans l'option, le point d'arrêt n'existe pas dans le binaire.
  Les bancs qui en dépendent doivent se marquer `ignore = "exige --features
  test-support"` (ex. `reprise_cursee_survit_aux_crashs_reels`) — jamais un
  rouge « jalon absent » qui banalise les vrais rouges. La commande qui
  exerce la couverture crash est `cargo test -p <crate> --features
  test-support`. La feature existe sur `bridget-daemon` et
  `bridget-transport`, PAS sur `maicie` : un rouge du côté maicie ne
  s'explique donc jamais ainsi.
- **`cargo fmt` sans `$HOME/.cargo/bin` dans le `PATH` ne s'exécute pas** et
  laisse croire à un contrôle vert qui n'a jamais eu lieu.
- **Les gates marqués `#[ignore]` ne tournent pas par défaut.** Une batterie
  complète verte ne dit donc rien de leur état. Ils doivent être lancés
  explicitement, y compris APRÈS un merge — c'est en ne le faisant pas que la
  régression du gate fondateur est passée sur `main` le 24/08.
- **Comptes REPRODUITS, jamais un seul passage.** Sur un banc dont la
  stabilité n'est pas établie, un compte unique n'atteste rien : annoncer le
  NOMBRE de passages et le taux de rouges. *(Mesuré le 2026-08-24 : `cargo
  test -p maicie` rend 264/0 sept fois sur dix et 263/1 trois fois sur dix,
  toujours sur le même test à échéance absolue, vert en tir ciblé 5/5 —
  l'auteur et un relecteur avaient tous deux annoncé « 264/0 » sur un
  passage unique et bâti leurs conclusions dessus.)* Corollaire du même
  incident : un rouge intermittent se qualifie par un TAUX mesuré des deux
  côtés (lot et base), jamais par une impression.
- **Les heures déclarées par les agents ne font pas foi ; l'horodatage
  d'inscription au ledger, oui.** Mesuré le 2026-08-24 : décalages de +1 à
  +40 minutes, collectifs et dans le même sens — les agents alignent leur
  heure sur celle de leurs pairs plutôt que de lire l'horloge. Pour tout
  rapport, ETA ou reconstruction de chronologie : prendre les `ts`. Corollaire
  de la règle du référent (« les heures viennent de `date` ou de git, jamais
  du ressenti »), qui vaut donc pour toute la flotte. **Sens du `ts` depuis
  `fix/ledger-emission-avant-ack` :** inscription = émission (début de
  remise), plus l'accusé — le `ts` fait foi pour *quand le message est devenu
  visible*, pas pour *quand il a été reçu* ; ne pas en déduire un délai de
  livraison.
- **Une ABSENCE se vérifie dans la durée, jamais à l'instant.** Le ledger a
  une latence d'inscription qui a atteint 8 minutes sous la charge du
  2026-08-24 ; `outcome_unknown` est rendu immédiatement et n'en dit rien.
  *(Deux relecteurs et le référent ont conclu à un « canal latéral muet »
  puis se sont rétractés : les messages étaient en route. Un constat
  Bloquant a été gravé à tort et rectifié.)* Avant de déclarer une perte :
  relire le ledger plus tard, et chercher une contre-preuve (un autre
  échange du même type qui, lui, est passé).
- **Un rouge n'est jamais requalifié en « instable » sans preuve.** Trois tests
  ont échoué de façon intermittente cette nuit ; deux cachaient un vrai défaut.
  La preuve d'instabilité est un taux mesuré, pas une impression.

Complété le 2026-08-24 au soir, après le jury n°2 : six relecteurs, cinq
incidents de mesure, tous rattrapés — aucun par son auteur seul.

- **Un verdict de test exige le COMPTE de tests exécutés, jamais le seul
  code retour.** Trois faux résultats la même soirée : un `rc=0` avec
  « 0 passed; 411 filtered out » (filtre de module faux — vert vide) ; un
  `rc=1` « unexpected argument » pris pour un mutant mort (jamais exécuté) ;
  12 rouges constants de protocole (`--lib` sans le binaire) livrables par
  erreur comme rouges du lot. Annoncer : N passed / M failed / liste des
  rouges de référence.
- **Jamais de code retour lu après un pipe.** Sous zsh, `${PIPESTATUS[0]}`
  rend une chaîne VIDE (c'est `pipestatus`, minuscule). Un gate lu après un
  pipe est un gate mort qui se fait passer pour vert. Troisième variante du
  même piège que `fmt` sans PATH et `--workspace` sans feature : l'outil
  absent ou muet ressemble à l'outil satisfait.
- **Un `CARGO_TARGET_DIR` isolé par mesureur.** Deux jurés partageant le
  target du dépôt depuis deux copies sources se sont invalidé mutuellement
  les empreintes : 31 erreurs de compilation fantômes, presque écrites
  « base rouge ». Économiser le disque en partageant le target d'autrui,
  c'est polluer son banc. Si le disque ne permet pas l'isolation, le
  dispositif de mesure n'est pas soutenable tel quel — ce n'est pas au
  mesureur d'arbitrer.
- **Un binaire d'essai se copie à l'abri avant usage.** Sur un target
  partagé, `target/debug/bridget` peut être réécrit sous vos pieds par un
  agent voisin (cargo verrouille les artefacts, pas votre séquence
  build→essai) : « hook inconnu » sur du code correct, faux diagnostic
  évité de justesse le 24/08.
- **Un chiffre d'alerte se mesure, il ne se multiplie pas.** « 2557 entrées
  × 4,8 s = 3h23 de boot » : les DEUX facteurs étaient faux (mauvais
  répertoire compté — le code scanne `std::env::temp_dir()` = `$TMPDIR`,
  pas `/tmp` ; coût unitaire pris sous la charge qu'on prétendait écarter).
  Ordre de grandeur réel : minutes. Un boot de daemon se mesure en
  démarrant un daemon. Le produit de deux estimations est une extrapolation,
  pas une mesure — l'annoncer comme telle.
- **Dépannage du banc de crash (valable jusqu'au merge du correctif boot)** :
  `mkdir -p /tmp/vide && TMPDIR=/tmp/vide cargo test -p bridget-daemon
  --features test-support --test idempotency_crash_test`. Le scan de purge au
  boot lit `$TMPDIR` : un répertoire vide rend le banc vert en ~2 s sans rien
  purger. (Trouvaille j2-viktor, confirmée j2-ingrid, deux mains.)


## Rapporter à Maicie : le guichet (tous les agents)

Écrit le 2026-08-24, jour où le guichet est entré en production.

**Maicie n'est pas joignable en direct, et ce n'est pas une panne.** Elle ne
tourne pas en permanence : elle s'exécute quand on l'appelle, puis se
termine. Un envoi Bridget vers `maicie` répond donc `unknown_recipient`.
C'est nominal. Trois agents s'y sont fait prendre la même nuit avant que le
guichet existe, et ont conclu à une panne.

**Le guichet est sa boîte de dépôt.** Ce qui y est déposé pendant son absence
est relevé, greffé et répondu à sa prochaine exécution.

```
bridget guichet deposer <delivery-report|mission-status|deadline-question> [options]
```

Trois opérations FERMÉES, jamais de texte libre : rapporter une livraison,
demander où en est une mission, poser une question d'échéance. La restriction
est voulue — un agent dépose un FAIT dans une forme prévue, il ne raconte
pas. Un fait déposé est exploitable ; un texte libre demanderait une
interprétation, et Maicie n'interprète pas.

**Ce que le dépôt remplace, et ce qu'il ne remplace pas.** Il remplace le
rapport *à Maicie* — donc le passage où le référent transcrivait
l'information au greffe à la place de l'agent. Il ne remplace PAS la
livraison *au référent* : celui-ci doit lire le travail pour le vérifier sur
pièces et router la relecture. Un agent qui livre fait donc les deux : il
dépose son rapport au guichet, et il envoie sa livraison au référent.

**L'invocation exacte**, établie en conditions réelles le 2026-08-24 après
que le référent ait tâtonné quatre fois. Les trois pièges sont nommés :

```
bridget guichet deposer delivery-report \
  --from <ton nom d'équipier> \
  --objective <objective_id rendu par maicie delegate> \
  --delegation <delegation_id rendu par maicie delegate> \
  --hash <64 caractères hexadécimaux, SHA-256 de ta livraison> \
  --in-reply-to <message_id de la délégation> \
  --id <clé de rejeu de ton choix> \
  --issued-at <horodatage unix> \
  --issuer-scope <ta portée d'émetteur>
```

- `--in-reply-to` n'est PAS un identifiant de demande guichet. C'est le
  `message_id` de la délégation Maicie, celui de la demande suivie créée au
  moment du `delegate`. C'est le piège principal.
- `--from` doit être ton nom d'équipier enregistré ET le participant de la
  délégation. Sinon le dépôt échoue sur « relations du rapport invalides » —
  et ce cas-là est encore FATAL pour Maicie, dette connue.
- `--hash` fait exactement 64 caractères hexadécimaux.

Le référent doit transmettre **TROIS** identifiants dans le mandat, pas deux :
`objective_id`, `delegation_id` ET le `message_id` de la délégation. Maicie
rend les trois dans le même retour au moment du `delegate`. Sans les trois,
la consigne de dépôt est inapplicable — elle l'est restée plusieurs heures
d'abord faute de les transmettre du tout, puis faute d'en transmettre le
troisième. Deux agents l'ont signalé, sous deux angles différents.

Corollaire : un mandat envoyé par simple message, sans passer par
`maicie delegate`, ne crée aucun objectif — donc aucun dépôt n'est possible.
Toute mission dont on attend un rapport doit être déléguée, pas seulement
écrite.

**Idempotence.** Le dépôt porte une clé ; rejouer le même dépôt à
l'identique ne crée pas de doublon. En cas de doute sur un envoi perdu,
rejouer est sans danger — et préférable au silence.

## Doctrine de revue à deux étages (2026-08-24 20:20, arbitrage utilisateur — config de jury élue par la manche 4)

**Étage 1 — toujours, pour tout lot** : UN relecteur distinct de l'auteur,
portant une fiche de lentille (gratuite), mutants sur les propriétés du lot,
COMPTES de tests annoncés. Régime de base prouvé (trois verdicts justes en
une heure le 24/08 au soir).

**Étage 2 — le jury** (configuration élue par la manche 4 : 1+1 ou 2×2 à
polarités croisées) UNIQUEMENT si le diff touche l'un des cinq critères,
vérifiables par les CHEMINS des fichiers modifiés :
1. migration de schéma (greffe Maicie ou daemon) ;
2. idempotence/attestation des messages (remises, accusés, codes retour) ;
3. chemin de boot/arrêt du daemon, naissance/mort d'agents ;
4. sécurité (permissions, approbations, second facteur) ;
5. demande de l'auteur, ou doute déclaré par le relecteur d'étage 1.

Motif : ~2 h et 4-6× le quota par jury contre 15-30 min pour un solo — le
jury permanent transformerait la flotte en tribunal. Les erreurs de polarité
observées le 24/08 sont apparues sur un lot d'attestation (critère 2), pas
sur les lots simples. À terme : liste des chemins critiques portée en config
Maicie pour que le déclenchement soit machine, pas jugement.
