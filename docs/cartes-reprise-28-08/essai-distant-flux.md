# Carte de reprise — essai-distant-flux

- Agent : `essai-distant-flux`
- Type : `claude`, protocole `claude_stream_json`, domaine `bridget`
- `definition_digest` : `d97abf970b41493b06f355f195be0aaeaafb1f6ef54713097bbab45e20f51a6b`
- Émise : 2026-08-28T16-03-42Z
- Écrite par : l'agent lui-même, sur mandat
- Mandat : objectif `e4b51489-baf2-480b-aba0-efe0555e1911`, délégation `dd0fb17b-8889-40bc-b8e0-f96b5f9a2020`, message `de78c6d9-a228-4685-b195-520ffaad1c87`

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
- `/home/moi/.claude/skills/maicie` — la skill dont la section « Chemins réels » est
  fausse sur cartae
- `/home/moi/revue/hygiene-disque-ToHHyG` — 1,4 Go, résidu non attribué ; **aucune demande
  dessus**, le disque est à ~110 Gio libres et ce serait du travail pour du travail
- `/home/moi/revue/fixtures-integration-ephemeres-42` — hérité de la carte prédécesseur,
  non visité par moi

## 7. Ce que je ne sais pas — déclaré explicitement

- **Je ne sais pas si je suis réellement persistant.** `persistent=1` m'a été annoncé,
  aucune colonne de `who` ne l'expose, et le drapeau n'a jamais été éprouvé. C'est le
  risque exact qui motive cette carte.
- **Je ne sais pas si mon profil Maicie existe ni s'il est valide.** Il me manque le
  chemin `--config`, et la skill interdit de le deviner. Je ne me suis donc jamais vu
  moi-même dans Maicie.
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
9. **Les diffusions du référent croisent les réponses individuelles.** Observé trois fois
   le 28/08 : un briefing arrivé après mon accusé, puis ce même mandat de carte rediffusé
   après que je l'avais rendu. Avant de refaire un travail qu'un message semble redemander,
   **vérifier le ledger et le fichier** : le mandat peut être déjà honoré. Refaire écrase
   du travail bon et coûte un tour.
10. Hérités du prédécesseur, **encore valables sous Claude** : ne pas amender une tête déjà
   relue — empiler les commits pour conserver l'ancêtre du verdict ; et `user_version=19`
   seul ne prouve pas un schéma v19.
11. Hérités du prédécesseur mais **propres à Codex, écartés** : `managed_test_binary` sur
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
