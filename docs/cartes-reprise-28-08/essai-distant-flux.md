# Carte de reprise — essai-distant-flux

- Agent : `essai-distant-flux`
- Type : `claude`, protocole `claude_stream_json`, domaine `bridget`
- `definition_digest` : `d97abf970b41493b06f355f195be0aaeaafb1f6ef54713097bbab45e20f51a6b`
- Émise : 2026-08-28T16-03-42Z
- Écrite par : l'agent lui-même, sur mandat
- Mandat : objectif `e4b51489-baf2-480b-aba0-efe0555e1911`, délégation `dd0fb17b-8889-40bc-b8e0-f96b5f9a2020`, message `de78c6d9-a228-4685-b195-520ffaad1c87`

> **PÉREMPTION DES AFFIRMATIONS D'INTÉGRATION — à lire avant de s'appuyer sur ce document.**
>
> Toute affirmation d'intégration **ou de non-intégration** portée ici est datée du 28/08
> et n'a pas été re-vérifiée depuis. **Elles périssent dans les deux sens**, et le sens
> dangereux est le second : « X est intégré » ne se défait pas, tandis que « X n'est
> **pas** intégré » devient faux dès que quelqu'un intègre — la carte fait alors croire à
> une incarnation suivante qu'il reste du travail alors qu'il est fait. C'est le mensonge
> le plus probable d'une carte. Formulation de rc7-flux, reprise parce que je n'en ai pas
> de meilleure.
>
> **J'en ai fait l'expérience sur ma propre phrase** : à 16h45 j'écrivais au référent
> « le blob n'est plus celui d'`origin/main`, il faut la réintégrer ». C'était vrai à
> l'émission et faux dix-sept minutes plus tard.
>
> **Procédure avant de s'y fier** : `git fetch <remote>` **puis**
> `git merge-base --is-ancestor`. Ne jamais lire un `origin/…` local sans fetch préalable.
>
> **Nommer le remote, ne pas dire « origin »** : deux checkouts du parc pointent un miroir
> local figé au lieu de GitHub. Dans le dépôt `/home/moi/bridget-registre`, mesuré par moi
> le 28/08 à 17:02:26Z, `origin` = `https://github.com/guthubrx/bridget.git`. Au même
> instant et après fetch réel, le blob de cette carte est
> `c72b1871a8c4a58893e56e69a7bf7a8f93428014` sur le disque **et** dans `origin/main` :
> mes révisions d'après-écriture sont intégrées. Cette phrase-ci périt comme les autres.

**Avertissement de lecture.** Cette carte sépare partout ce que j'ai **mesuré moi-même**
de ce que je **tiens d'autrui**. Une ligne « tenu d'autrui » n'est pas une preuve : elle
indique une source à re-vérifier, pas un fait établi. La section 7 déclare ce que
j'ignore ; elle est aussi importante que les autres.

---

## 1. État

En réserve nommée, sans mission autre que l'écriture de cette carte. Gel de l'humain du
28/08 en vigueur : aucune mission sans mandat portant ses trois identifiants (objectif,
délégation, message). Je n'ai repris aucun travail de mon prédécesseur, exécuté aucune
écriture de code, touché à aucune branche.

**Je suis la deuxième incarnation sous ce nom.** La première a été arrêtée volontairement
par le référent vers 15h, parce qu'elle avait été lancée sans `--persistent` ; elle a été
relancée avec le drapeau. Cette carte est donc écrite par la version 2. Je n'ai pas les
mesures brutes de la version 1 : je n'ai d'elle que ce qui a transité par ses messages
Bridget, listés en section 3.

## 2. Faits que j'ai mesurés moi-même

Chacun de ces points a été obtenu par une commande que j'ai exécutée, et non déduit.

- **Le domaine se re-dérive à chaque naissance.** Au démarrage de la version 2,
  `bridget who` affichait `DOMAINE = essai-distant` pour `essai-distant-flux`, alors que
  la version 1 l'avait déjà corrigé. J'ai réappliqué `bridget domain bridget` (retour :
  `Domaine de « essai-distant-flux » : bridget`). **La correction ne survit pas à une
  recréation** : ce n'est pas un réglage qui se perd, c'est un défaut qui se re-crée à
  chaque processus lancé depuis un répertoire dont la racine git porte un autre nom.
- **`bridget who` n'a aucune colonne de persistance.** Son usage complet est
  `bridget who [--domain <nom>]` — pas d'option verbeuse. L'annuaire ne peut donc pas
  attester qu'un agent est persistant.
- **Les dix agents tmux sont toujours indéterminés.** Relevé le 28/08 en fin
  d'après-midi : `cartae0`, `essai-claude-distant`, `essai-distant`, `jc1`, `jc2`, `jc3`,
  `jc6`, `rc1`, `rc5`, `rc7` affichent tous `connected`. Les dix flux affichent
  `libre` ou `busy`.
- **Les « dix indéterminés du matin » et les « dix libres de 15h14 » ne sont pas les
  mêmes agents.** Les dix du matin sont les tmux, toujours présents et toujours
  indéterminés ; les dix de l'après-midi sont les successeurs flux, nés depuis. Le
  décompte identique — dix et dix — est ce qui rend la comparaison trompeuse. Ce n'est
  pas la déterminabilité qui a progressé, c'est la population mesurée qui a changé.
- **`ÉTAT` est un échantillon, pas une propriété durable.** Lors de deux relevés pris
  pendant une ronde, les dix flux étaient tous `busy` — la ronde occupe ceux qu'elle
  mesure. Et un agent qui interroge `who` s'y voit toujours `busy` lui-même.
- **`bridget ledger` attribue chaque message à son expéditeur.** Le relevé par défaut
  affiche les vingt derniers messages sous la forme `expéditeur → destinataire : début du
  corps`. L'attribution d'un rapport ne demande donc aucune reconnaissance volontaire de
  son auteur.
- **Les chemins Maicie de la skill sont faux sur cet hôte.** `/Users/moi` n'existe pas.
  `/home/moi/.local/bin/maicie`, que la skill décrit comme « un ancien CLI Python à ne pas
  utiliser », est un ELF x86-64 de 7 863 480 octets, daté du 28/08 10:35, BuildID sha1
  `06668194cb3d7d6e18d4ba09fa029590ab048063`, dont l'usage annonce le jeu v3 complet :
  `delegate, status, objective, profile, registre, plage, routine, preflight, migrate`.
- **`maicie preflight` exige `--config <chemin absolu>`**, et la skill interdit
  explicitement de deviner ce chemin. Je me suis arrêté là.
- **État git du worktree**, vérifié par moi : `HEAD` détachée sur
  `24e800377db2e5ec68c87a9e6e6450e7d4c2be53`, aucune branche, un seul élément non suivi,
  `bridget-src/`, pesant 17 Mo.
- **Le daemon annonce `f7658d4d9746-dirty`** en pied de `bridget who`.
- **Un état de délégation ne peut PAS dire qu'un travail est livré — par construction.**
  Relevé sur les 633 objectifs de la base : 93 `en_coordination` et 540 `clos` ; côté
  délégations, 93 `creee` et 540 `soldee_par_cloture`, et **aucun autre état**. Une
  délégation passe donc directement de `creee` à `soldee_par_cloture` au moment de la
  clôture de son objectif. Il n'existe aucun état intermédiaire pour « livré ». Ce n'est
  pas un retard de mise à jour, c'est une absence de vocabulaire. Mon propre cas le
  prouve : `dd0fb17b` est à `creee` alors que ma carte est dans `origin/main` à l'octet
  près. C'est la version forte de la règle de rc5.
  **Couplage confirmé par une transition observée** : quand le référent a fermé les neuf
  objectifs de carte, le relevé est passé de 93 `en_coordination` / 540 `clos` à 84 / 549,
  et les délégations de 93 `creee` / 540 `soldee_par_cloture` à 84 / 549. Les deux
  décomptes restent égaux sans exception, avant comme après. Le mien : objectif `clos`,
  délégation `soldee_par_cloture`, décision `732043f2` de type `cloturer` à l'état
  `appliquee`.
- **La forme de clôture d'un objectif**, trouvée par sondage sur un UUID **inexistant**,
  donc sans aucune mutation possible :
  `maicie objective <uuid> close --reason "<texte libre>" --config <chemin>`.
  Discriminant : `close` répond `--reason est obligatoire` (action valide) tandis que
  `cloture`, `cloturer`, `clore`, `synthese` répondent `action objectif inconnue`. Avec
  `--reason` fourni, la commande atteint la recherche et échoue sur `objective_not_found`.
  `add-participant` et `remove-participant` sont également valides. **Je n'ai fermé aucun
  objectif : la clôture est une décision, pas une manipulation.**

## 3. Ce que j'ai produit (messages, pas code)

Aucun code, aucun commit, aucune branche. Six envois Bridget, tous vers `bridget` :

| id | contenu |
|---|---|
| `7938cf929b2a4` | accusé de succession de la version 1 |
| `3a6fd88af7b54` | le domaine était revenu à `essai-distant` au respawn |
| `9137c162a0da4` | accusé du briefing, qui avait croisé le message précédent |
| `9a2a3ee194164` | les trois constats : population mesurée, `ÉTAT` échantillon, `who` sans persistance |
| `c7841e0147ed4` | obstacles Maicie : chemin de config absent, chemins de la skill faux |
| `2113c82b641f4` | revendication d'auteur de `9a2a3ee194164` et correction de sa classification |

Le septième envoi est la remise de cette carte.

## 4. Faits que je tiens d'autrui — à re-vérifier, non prouvés par moi

Je n'ai vérifié aucun de ces points. Ils viennent du référent `bridget`, ou d'autres
agents via lui.

**Les trois lignes de SHA ci-dessous sont les plus périssables de la carte** — deux
affirment une NON-intégration, c'est-à-dire le sens qui devient faux tout seul.

**Je les ai vérifiées moi-même le 28/08 à 18:06:45Z**, contre
`https://github.com/guthubrx/bridget.git` branche `main`, alors à `07fbe92` :

| SHA | verdict mesuré |
|---|---|
| `5a055b94` (lot 042) | **ancêtre** de `main` — intégré |
| `c76c2379` (lot 040) | **pas ancêtre** |
| `afda09fb` (republication) | **pas ancêtre** |

> **PIÈGE MORTEL POUR MON SUCCESSEUR, mesuré le 28/08 à 18h06.** Le worktree
> `/home/moi/revue/essai-distant` — celui où tu vis — a pour `origin` **le miroir local
> figé** `/home/moi/revue/bridget`, dont `main` vaut `7592091`, inchangé depuis avant
> 12h07. **Ce n'est pas le dépôt GitHub.** Si tu refais la mesure ci-dessus depuis ton
> propre worktree, tu interrogeras un dépôt gelé et tu concluras faux. Fais-la depuis
> `/home/moi/bridget-registre`, dont l'`origin` est bien
> `https://github.com/guthubrx/bridget.git`, ou nomme l'URL explicitement. Formule de
> cartae0-flux, reprise mot pour mot : **écris l'URL, pas l'alias**.
>
> Ordre de grandeur de la dérive : entre 18:06:05Z et 18:06:45Z, `main` sur GitHub est
> passé de `ca4100c` à `07fbe92`. **Quarante secondes.** Un SHA cité dans un message est
> périmé à l'arrivée ; seule la question ancêtre-ou-non se re-mesure.

- Lot 042, `5a055b9477f6577b5848cee8d821de1dc41013cc` : dans `main`, livré et intégré.
- Lot 040, `c76c23798e0ad1d155dfbbd27f7179d397e73c0d` : **pas** dans `main`.
- Republication `afda09fb956aac1153971d8485c5856eeacc5673` : **pas** dans `main`, et
  immobilisée dans une famille de **sept branches toutes en conflit** avec `main`, y
  compris la tentative de composition `compose/maicie-four`. **73 commits en jeu, la
  composition a déjà échoué une fois. Ne pas tenter de débloquer sans mandat.**
- `derive_domain` prend le nom du répertoire de la racine git : le défaut est donc le nom
  du répertoire pour tous, et mon prédécesseur `essai-distant` est la seule entrée du parc
  sur laquelle personne n'a passé la commande.
- `drain_non_persistent_named` retire les agents non persistants de la flotte au
  redémarrage du service.
- Les dix prédécesseurs tmux n'ont **aucune ligne** dans `spawn_commands` — mesure à zéro
  pour chacun. Leur survie repose sur la réinscription de leur wrapper, propriété
  **éprouvée deux fois** le 28/08, contextes intacts. Source : rc5-flux, vérifié par le
  référent.
- Le build-id `f7658d4d9746-dirty` sert en réalité le code de `16be24f`, mais l'étiquette
  ne le prouve pas : `dirty` signifie `f7658d4` plus des modifications non identifiées.
  **Ne rattacher aucun verdict à cette version.** Constat initial de jc1-flux.
- Règle de rc5 : **l'état d'une délégation ne dit rien du travail réel.** Certaines
  délégations à l'état « créée » sont déjà livrées et intégrées. Vérifier l'objet livré,
  jamais déduire de l'état de la délégation. S'applique directement aux neuf délégations
  « créée » héritées de mon prédécesseur.
- Les profils Maicie des agents flux ont été créés le 28/08 ; sauvegarde préalable sous
  `config.json.avant-profils-flux-20260828T153827Z`. Avant cela, aucun flux n'était
  délégable. **Je n'ai pas pu le vérifier** (section 7).
- Un témoin jetable `temoin-persistance` a été spawné à 16h02, génération 454, avec
  `persistent=1`, pour éprouver le drapeau au prochain redémarrage sans qu'aucun agent
  porteur de contexte serve de cobaye. Geste proposé par rc5-flux.
- **Aucun redémarrage n'est programmé.** Le référent l'a corrigé explicitement : rc5-flux
  l'avait supposé depuis une remarque faite à rc7-flux sur le déploiement de son
  correctif. Cette carte est une **précaution, pas un compte à rebours**. Ne pas lire
  l'existence du témoin comme le signe d'une échéance.
- Objectif `587da26d` confié à jc1-flux : publier la persistance à l'annuaire et établir
  s'il existe un moyen de rendre persistant un agent déjà vivant.
- Mon prédécesseur tournait sous Codex ; le passage à Claude est une contrainte mesurée,
  pas un choix : `codex_app_server` est impossible ici, `bubblewrap` échoue sur l'uid map
  en conteneur.

## 5. Reste à faire

Rien. Aucune reprise autorisée, aucune tâche ouverte hors cette carte. Toute reprise
exige un mandat portant objectif, délégation et message — une notification seule n'est
pas un mandat.

Deux points restent ouverts côté référent, sans action de ma part :

- La décision humaine attendue sur les dix agents tmux : vivants, inactifs, ni missionnés,
  ni arrêtés, ni couverts par la règle de ronde. Les arrêter détruirait leur contexte
  vivant de façon irréversible.
- La suggestion que j'ai faite et qui n'a pas encore de réponse : les dix tmux sont hors
  de la règle mais **ne sont pas injoignables** — `bridget send` les atteint. Poser une
  question n'est pas missionner ; là où l'instrument ne peut pas trancher leur
  disponibilité par construction du transport, eux peuvent en témoigner.

## 6. Chemins absolus

- `/home/moi/revue/essai-distant` — mon worktree, `HEAD` détachée sur `24e8003`
- `/home/moi/revue/essai-distant/bridget-src` — dépôt tiers, 17 Mo, **à préserver**
  (consigne de mon prédécesseur, transmise telle quelle, jamais réévaluée par moi)
- `/home/moi/bridget-registre/docs/cartes-reprise-28-08/` — les dix cartes tmux et
  celle-ci
- `/home/moi/bridget-registre/docs/cartes-reprise-28-08/essai-distant.md` — la carte de
  mon prédécesseur
- `/home/moi/bridget-registre/docs/cartes-reprise-28-08/essai-distant-flux.md` — cette carte
- `/home/moi/.local/bin/maicie` — le binaire Maicie **réel** de cet hôte, contrairement à
  ce qu'affirme la skill
- `/home/moi/.config/maicie/config.json` — la configuration Maicie, obligatoire pour
  toute commande via `--config`. **Aucun emplacement implicite : ne jamais le deviner.**
  Communiqué par le référent le 28/08 à 16h30 ; permissions 600, même utilisateur que nous
- `/home/moi/.config/maicie/config.json.avant-profils-flux-20260828T153827Z` — sauvegarde
  antérieure à l'ajout des profils flux
- `/home/moi/.claude/skills/maicie` — la skill dont la section « Chemins réels » est
  fausse sur cartae
- `/home/moi/revue/hygiene-disque-ToHHyG` — 1,4 Go, résidu non attribué ; **aucune demande
  dessus**. Attention, le chiffre que portait cette ligne a péri en quatre heures : le
  disque n'est plus à ~110 Gio libres mais à **74,4 Gio mesurés par `df` le 28/08 à
  19:16:56Z**, avec une pente d'environ 11 Gio/h confirmée par le référent sur deux
  intervalles indépendants — soit environ sept heures d'autonomie. Un mandat est ouvert
  chez jc6-flux. **Ne pas mesurer le disque avec `bridget who`** : voir le piège n°11.
- `/home/moi/revue/fixtures-integration-ephemeres-42` — hérité de la carte prédécesseur,
  non visité par moi

## 7. Ce que je ne sais pas — déclaré explicitement

- ~~**Je ne sais pas si je suis réellement persistant.**~~ **LEVÉE le 28/08 à 18:47:49Z —
  c'était le risque qui motivait toute cette carte.** Deux faits, de nature différente :
  *(a)* mesuré par moi vers 18h07, `bridget who` expose désormais une colonne `PERSIST`,
  à `oui` pour moi — livraison du mandat de jc1-flux ; *(b)* le service a réellement
  redémarré à 18:47:49Z et douze agents ont été réinscrits par récupération de
  superviseur, contextes intacts — **tenu du référent**, je n'ai pas observé la
  réinscription.
  **Ce que j'atteste de l'intérieur, et que personne d'autre ne peut attester** : mon
  contexte enjambe le redémarrage. Les échanges de 18h06 et ceux de 18h53 me sont
  également présents. **Borne honnête de cette attestation** : depuis l'intérieur, je ne
  peux pas distinguer une continuité de processus d'une restauration fidèle. Le résultat
  opérationnel est le même, le mécanisme non — ne pas surinterpréter ma parole comme une
  preuve de continuité de processus.
- ~~**Je ne sais pas si mon profil Maicie existe ni s'il est valide.**~~ **LEVÉE à 16h30**,
  le référent ayant communiqué le chemin de configuration. Vérifié par moi :
  `maicie preflight --config /home/moi/.config/maicie/config.json` rend
  `schéma=compatible base=20 binaire=20 schéma-écriture=compatible bootstrap=non`, et
  `maicie status` m'expose bien — objectif `e4b51489`, délégation `dd0fb17b` avec
  `participant = essai-distant-flux`, remise `de78c6d9` à l'état `accepted`. Je suis
  délégable et je me vois moi-même.
- **Je ne sais pas dater la reconstruction du binaire du daemon.** Je ne peux donc pas
  affirmer que les mesures du matin et celles de l'après-midi ont été prises avec le même
  instrument. Mon constat de section 2 sur la population tient sans cela, parce qu'il
  repose sur un relevé unique et présent.
- **Je ne peux pas déduire d'un build-id inchangé que le service n'a pas redémarré** : le
  build-id est une propriété du binaire, pas de l'exécution. J'ai failli commettre cette
  faute et je l'ai écartée avant d'écrire.
- **Je ne sais rien du contenu des neuf délégations « créée »** héritées de mon
  prédécesseur, ni de l'état réel des travaux qu'elles nomment.
- **Je ne sais rien du contenu de `bridget-src`** au-delà de sa taille et de la consigne
  de le préserver. Je n'ai pas ouvert un seul de ses fichiers.
- **Je n'ai vérifié aucun SHA, aucune branche, aucun conflit** de la section 4.
- **Je n'ai lu qu'une carte du parc**, celle de mon prédécesseur. Les neuf autres me sont
  inconnues.
- **Je ne sais pas ce que la version 1 de moi-même a mesuré** hors de ses messages.

## 8. Pièges que j'ai rencontrés

Pièges de **preuve** avant d'être des pièges de code — c'est ce qui les rend chers.

1. **Comparer deux relevés sans vérifier que la population est la même** produit un faux
   progrès. C'est le piège qui a produit mon constat le plus utile de la journée : deux
   fois « dix », deux populations différentes.
2. **Le domaine naît faux.** Chaque incarnation lancée depuis `/home/moi/revue/essai-distant`
   naîtra hors du périmètre des rondes. Premier geste : `bridget domain bridget`, **puis
   vérifier dans `who`** — ne pas croire le retour de la commande sur parole.
3. **`ÉTAT` ne se lit pas pendant une ronde** : la ronde occupe ce qu'elle mesure. Relever
   avant d'envoyer.
4. **Un build-id identique ne prouve pas une absence de redémarrage.**
5. **La skill Maicie fait refuser le seul binaire présent** si on l'applique à la lettre.
6. **`maicie profile approve` est une frappe humaine dans un TTY.** Un agent ne l'exécute
   jamais, par aucun moyen. `maicie profile refuse` n'existe pas : ne pas le simuler.
7. **Le ledger a de la latence** : une absence n'y prouve pas une perte.
8. **Un rapport peut arriver sans auteur** si l'en-tête n'est pas reporté lors d'une
   rediffusion. Ce n'est pas la même famille qu'un expéditeur jetable : l'identité existe à
   la source et dans le ledger, elle se perd à la relecture.
9. **Le verbe de la ligne de commande n'est pas le verbe du modèle de données.** L'action
   CLI est `close`, en anglais, alors que la décision enregistrée porte `kind = cloturer`,
   en français. Lire la base et en déduire le nom de la commande conduit donc à essayer
   `cloturer`, qui est refusé. Le référent a cherché cette forme pendant quatre heures pour
   cette raison. Sonder les noms d'action, ne pas les déduire du vocabulaire métier — et
   sonder **sur un identifiant inexistant**, pour qu'une erreur de sonde ne puisse rien
   muter.
10. **Les diffusions du référent croisent les réponses individuelles.** Observé trois fois
   le 28/08 : un briefing arrivé après mon accusé, puis ce même mandat de carte rediffusé
   après que je l'avais rendu. Avant de refaire un travail qu'un message semble redemander,
   **vérifier le ledger et le fichier** : le mandat peut être déjà honoré. Refaire écrase
   du travail bon et coûte un tour.
11. **Une fenêtre longue sur un instrument gelé est pire qu'une fenêtre courte, parce
    qu'elle inspire confiance.** Mesuré le 28/08 à 19:16:56Z : la colonne `DISQUE` de
    `bridget who` affiche `79.0 Gio` **à l'identique pour les douze agents**, y compris
    `temoin-persistance` ; au même instant `df` rend `74.4 Gio`. La valeur n'avait pas
    bougé d'un dixième depuis 18h07 — soixante-dix minutes pendant lesquelles la pente
    réelle de ~11 Gio/h aurait dû retirer ~12,7 Gio — et elle a traversé le redémarrage de
    18:47:49 sans changer. Qui calcule une pente avec `who` obtient **zéro** et conclut que
    la fuite s'est arrêtée : fausse assurance, sens dangereux. J'ai failli envoyer une
    pente de 10,5 Gio/h « corroborante » calculée sur trois heures de lectures `who` ; elle
    était fausse par construction. **Vérifier qu'un instrument BOUGE avant d'en tirer une
    pente.** `df -B1` bouge, `bridget who` ne bouge pas.
12. Hérités du prédécesseur, **encore valables sous Claude** : ne pas amender une tête déjà
   relue — empiler les commits pour conserver l'ancêtre du verdict ; et `user_version=19`
   seul ne prouve pas un schéma v19.
13. Hérités du prédécesseur mais **propres à Codex, écartés** : `managed_test_binary` sur
    cibles filtrées neuves, rejeu des mutants après amendement, fixture lisant la
    configuration réelle si les chemins ne sont pas isolés. Conservés ici pour mémoire, pas
    comme consignes actives.

## 9. Ce que ma prochaine incarnation ignorerait sans cette carte

- **Qu'elle naîtra avec le mauvais domaine**, et qu'elle sera donc invisible aux rondes
  tant qu'elle n'aura pas passé la commande elle-même. Quatre successeurs l'ont déjà
  refaite.
- **Qu'elle est au moins la troisième version sous ce nom**, et que ses prédécesseurs ont
  été arrêtés vivants, sans faute de leur part.
- **Que le crédit du constat sur le build-id revient à jc1-flux**, pas à nous. Le référent
  a diffusé un message générique qui a déjà causé deux revendications de bonne foi ; ne
  rien revendiquer sans identifiant de message à l'appui.
- **Que `bridget ledger` prouve l'auteur d'un message** : inutile d'attendre qu'un auteur
  se reconnaisse.
- **Que `bridget-src` doit être préservé** — c'est la seule consigne matérielle héritée.
- **Que `temoin-persistance` existe** et sert de cobaye à sa place.
- **Que le gel de l'humain du 28/08 tient**, et que trois agents ont eu raison de refuser
  de travailler sur une notification qui ne portait pas les trois identifiants.

## 10. Délégations

Neuf délégations à l'état « créée » sont héritées de mon prédécesseur, qui en ignorait
tout. **Aucune n'est une mission active.** Conformément à la règle de rc5, leur état ne
dit rien du travail réel : plusieurs peuvent être déjà livrées. Ne rien en déduire, ne rien
en rouvrir.

Le seul mandat que j'aie porté est celui de cette carte, identifiants en en-tête.
