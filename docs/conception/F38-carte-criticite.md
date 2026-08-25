# F38 — Carte de criticité auto-élue : note de conception

**Auteur** : relec6 — **Date** : 2026-08-25 — **Statut** : proposition, non implémentée
**Mandat** : bridget, message `mcp-85802-*` (sans objective_id/delegation_id : conception préalable à l'ouverture d'objectif, par décision du référent)
**Sources lues** : `specs/011-maicie-orchestration/exigences-coordination-v2.md` §F38 (l.1694) et §F38-b (l.1718) ; `docs/regles-chantier.md` ; `docs/lentilles/manche-004-routines.md` ; `docs/decisions/011-approbation-exposee-puis-second-facteur.md` ; `plugins/maicie/src/catalogue.rs` ; `plugins/maicie/src/store.rs` ; `docs/catalogue-du-du.md` (lu en place, non modifié).
**Périmètre** : conception seule. Aucun code, aucune compilation, aucune écriture dans le checkout principal.

---

## 0. Le fait qui commande toute la note

**Le registre du dû ne contient aucune donnée de zone.** Ce n'est pas une difficulté d'implémentation, c'est une absence de matière première.

Schéma réel d'un constat (`catalogue.rs`, `AddEntry`) :

```
{ v, kind, id, date, mission_source: { kind, id, failed }, severity, recurrence_of, text }
```

`mission_source.id` est un slug de mission (`socle-ponts`, `G1504-merge`, `reaper-garde-descendant`) — pas une zone de code. Il reste `text`, qui est de la prose.

**Mesure sur le registre réel** (111 entrées au 2026-08-25 03:04, dont **40 constats `add`** : 11 `blocker`, 26 `major`, 3 `minor` ; le reste est 40 `pending_qualification` et 31 `transition`) :

*Note de fraîcheur : le catalogue a gagné 2 entrées pendant la rédaction de cette note. Les deux sont des `transition`. J'ai rejoué la mesure après coup — les 40 `add` et les 11 citations de chemin sont inchangés, donc les chiffres ci-dessous tiennent.*

| Mesure | Valeur |
|---|---|
| Constats citant un fichier dans leur texte libre | **11 / 40 = 27 %** |
| Constats n'élisant donc rien | **73 %** |
| Zones élues en appliquant le seuil F38 au texte libre | **6** |

Les six : `daemon.rs` (1 blocker, 3 majors), `wrapper.rs` (2 blockers, 2 majors), `fleet.rs`, `disk_hygiene.rs`, `main.rs`, `idempotency.rs` (1 blocker chacun). Quatre autres zones apparaissent sous le seuil.

Deux remarques qui aggravent le chiffre :

1. **Les noms cités sont nus.** `main.rs` et `app.rs` existent en plusieurs exemplaires dans le dépôt ; élire « `main.rs` » n'a pas de sens opérationnel. Une carte alimentée ainsi déclencherait un jury sur le mauvais fichier.
2. **Ces 6 zones sont un artefact de mon extraction**, pas une donnée du registre. Personne n'a écrit ces textes en pensant qu'une machine y chercherait des chemins.

**Conséquence de conception** : la voie 2 de F38, telle qu'elle est rédigée, n'est pas implémentable aujourd'hui — non parce qu'elle est mal conçue, mais parce que la donnée qu'elle interroge n'est pas collectée. C'est le premier travail, et il n'est pas dans F38.

---

## 1. Où vit la carte

**Réponse : les deux, mais pas pour le même contenu — et la ligne de partage est le régime d'écriture.**

| Contenu | Où | Pourquoi |
|---|---|---|
| **Le germe générique** (F38-b) | Code, constante versionnée | Identique pour tout projet, jamais élu, jamais approuvé. Le mettre en config, c'est le rendre désactivable — donc rouvrir la boucle morte qu'il ferme. |
| **Les entrées élues** (voies 1-2-3) | **Table SQLite** du greffe Maicie | Écrites par la machine à chaque battement, horodatées, avec leur cause. C'est un état dérivé et reconstructible, pas une décision humaine. |
| **Les sorties approuvées** | **Table SQLite**, régime propose/approve | Ce sont des actes humains : ils exigent la même transaction d'approbation que les profils (ADR 011). |
| **Les surcharges de projet** | `config.json`, section `criticite` | Un dépôt peut déclarer une zone critique par décision, hors élection. Rare, explicite, relu. |

Forme proposée pour la table :

```
zones_critiques(
    zone            TEXT PRIMARY KEY,   -- chemin ou glob, normalisé depuis la racine du dépôt
    origine         TEXT NOT NULL,      -- germe | diff | registre | contrat | config
    cause           TEXT NOT NULL,      -- constat_id, chemin de contrat, ou marqueur détecté
    entree_le       INTEGER NOT NULL,
    dernier_signal  INTEGER NOT NULL,   -- dernière fois où la zone a été touchée OU citée
    etat            TEXT NOT NULL       -- active | sortie_proposee | sortie_approuvee
)
```

`origine` et `cause` ne sont pas décoratifs : sans eux, une zone dans la carte est un verdict sans pièce — exactement ce que la nuit du 24/08 a passé son temps à refuser.

### La circularité, assumée et nommée

Le schéma de persistance est dans le germe. Donc **la table `zones_critiques` est elle-même une zone critique, et le premier lot qui la crée déclenchera le jury qu'il définit.**

Je ne cherche pas à l'éviter, pour une raison simple : l'éviter demanderait une exemption, et une exemption est exactement le geste humain que F38 supprime. Le coût est borné et connu — un jury 1+1 à polarités croisées, ~2 h (manche 4). Le bénéfice est qu'aucune migration de la carte ne pourra jamais échapper au régime qu'elle institue.

**Point de vigilance** : la migration qui crée la table est une ressource globale ordonnée (règle de chantier n°17). Elle se déclare au greffe avant commit, comme toute migration.

---

## 2. Les trois voies, concrètement

### Voie 1 — Auto-détection dans le contenu du diff

C'est la seule voie qui fonctionne **sans donnée nouvelle**, et c'est pour cela que je propose de la livrer en premier.

Marqueurs, par catégorie du germe :

| Catégorie | Marqueur dans le diff ajouté/modifié | Faux positif à borner |
|---|---|---|
| Schéma de persistance | `CREATE TABLE`, `ALTER TABLE`, `DROP TABLE`, `CREATE INDEX`, `PRAGMA user_version`, `schema_migrations`, `current_version <` | Une chaîne SQL dans un **test** ou un commentaire. Borne : ignorer `tests/`, `benches/`, et les lignes dont le `+` est dans un bloc de commentaire. |
| Format de protocole | Modification d'un `enum`/`struct` dérivant `Serialize`/`Deserialize` dans `protocol.rs`, ou tout fichier cité par un `contracts/` | Ajout d'un champ `#[serde(default)]` — rétrocompatible. Borne : le signaler quand même, mais en `major`, pas en `blocker`. |
| Auth / permissions | `permission`, `approval`, `capability`, `second_facteur`, `0600`, `forbidden_env`, `chmod` | Le mot dans de la prose. Borne : ne compter que dans les fichiers `.rs`. |

**Ce que je refuse de promettre** : une détection sans faux positifs. Un marqueur textuel dans un diff est une heuristique. Ce qu'on peut garantir, c'est la **direction de l'erreur** : les bornes ci-dessus sont toutes choisies pour rater vers le jury (faux positif) plutôt que vers le silence (faux négatif). Un jury de trop coûte 2 h ; un jury manquant coûte la nuit du 24/08.

**Le vrai risque de la voie 1**, et il n'est pas dans F38 : le diff est analysé **avant** merge, mais la carte est un état **persistant**. Si un lot touche `CREATE TABLE` et est ensuite abandonné, la zone reste-t-elle dans la carte ? Je propose que **non** : la voie 1 déclenche un jury sans écrire dans la carte. Elle est *évaluée à chaque lot*, pas *mémorisée*. Seules les voies 2 et 3 alimentent la table. Cela évite qu'une carte se remplisse de zones jamais mergées.

### Voie 2 — Élection par le registre du dû

**Elle n'est pas implémentable en l'état** (§0). Voici ce qu'il faut d'abord, et c'est un travail à part entière :

**Prérequis P1 — le constat doit porter ses zones.** Ajouter au schéma :

```
zones: [ "crates/bridget-daemon/src/daemon.rs", "plugins/maicie/src/store.rs" ]   // optionnel, chemins depuis la racine
```

Champ **optionnel dans le schéma** (les entrées existantes restent valides — le catalogue est un journal append-only, on ne réécrit pas le passé) mais **obligatoire dans le protocole de revue**. *(Amendé au §9.2 : ma première rédaction disait « optionnel » tout court, et elle était fausse.)* Rempli par l'émetteur du constat, c'est-à-dire le relecteur. C'est une ligne de plus dans un mandat de revue, pas un travail.

**Prérequis P2 — normalisation.** Chemin depuis la racine du dépôt, jamais un nom nu. Un constat qui cite `main.rs` sans chemin est un constat qu'aucune machine ne peut exploiter, et il faut le dire à celui qui l'écrit au moment où il l'écrit.

Une fois P1 et P2 acquis, la requête est triviale :

```sql
SELECT zone FROM constats_zones
GROUP BY zone
HAVING SUM(severity = 'blocker') >= 1 OR SUM(severity = 'major') >= 2
```

**Granularité : le fichier, pas le répertoire ni le symbole.**
- Le répertoire élirait trop large : un blocker dans `daemon.rs` rendrait tout `crates/bridget-daemon/src/` critique, soit un jury permanent — précisément ce que la doctrine à deux étages refuse (« le jury permanent transformerait la flotte en tribunal »).
- Le symbole élirait trop fin : il exigerait une analyse syntaxique, et un renommage viderait la carte en silence.
- Le fichier est ce que le diff donne gratuitement, et c'est déjà la granularité des cinq critères de la doctrine (« vérifiables par les CHEMINS des fichiers modifiés »).

**Seuil : celui de F38 (1 blocker ou 2 majors), sans le modifier.** Sur les données actuelles il élirait 6 zones — un ordre de grandeur soutenable. Je recommande de **mesurer à nouveau après 3 mois** de constats correctement zonés, et de ne toucher au seuil que sur cette mesure.

### Voie 3 — Ancrage contractuel

La plus simple et la plus sûre : **12 répertoires `specs/*/contracts/` existent déjà.**

Règle : tout fichier cité dans un fichier sous `specs/*/contracts/` entre dans la carte, `origine = contrat`.

Deux précisions que F38 ne donne pas :
- **Cité comment ?** Par chemin littéral dans le texte du contrat. Pas d'inférence.
- **Que se passe-t-il si le contrat est supprimé ?** La zone passe en `sortie_proposee`, elle ne sort pas d'elle-même. Un contrat retiré est une décision qui mérite un regard humain, pas un effacement silencieux.

---

## 3. Le germe, reconnu sans liste écrite à la main

C'est la question la plus difficile de la note, parce que **la reconnaissance par catégorie et l'application par chemin ne sont pas la même opération**, et F38-b les confond.

- Le germe est défini par **catégorie** : schéma, protocole, auth.
- La carte s'applique par **chemin** : `crates/bridget-daemon/src/store.rs`.
- Traduire l'une dans l'autre, c'est écrire une liste — exactement ce qu'on veut supprimer.

**Ma proposition : ne pas traduire.** Le germe ne peuple pas la table `zones_critiques`. Il s'exprime **uniquement comme prédicats sur le contenu du diff** (voie 1), qui sont génériques par construction et ne nomment aucun fichier :

| Catégorie du germe | Prédicat générique, sans liste |
|---|---|
| Schéma de persistance | le diff contient du DDL, **ou** touche un fichier qui contient déjà du DDL |
| Format de protocole | le diff modifie un type sérialisé **exporté** par un crate, **ou** un fichier cité par un `contracts/` |
| Auth / permissions | le diff touche un chemin d'approbation, un mode de fichier, ou une liste de capacités |

La clause « **ou touche un fichier qui contient déjà du DDL** » est la clé : elle rend le germe auto-localisant. Personne n'écrit « `store.rs` est critique » ; la machine constate que `store.rs` contient `CREATE TABLE` et en tire la conséquence. Le jour où le schéma déménage, le germe suit sans qu'on touche à une liste.

**Coût honnête** : ce prédicat exige de lire le fichier *entier*, pas seulement le diff. C'est peu cher (grep sur les fichiers modifiés), mais ce n'est plus « analyser un diff » — il faut le dire.

---

## 4. La sortie de carte

**Signal de calme proposé : `dernier_signal`, qui est le plus récent de deux événements** — la zone a été *touchée par un lot mergé*, ou la zone a été *citée dans un constat*.

Prendre les deux, et pas seulement « touchée », évite un piège : une zone gelée parce que tout le monde a peur d'y toucher paraîtrait calme alors qu'elle est dangereuse. Une zone qui continue de produire des constats n'est jamais calme, même si personne n'y écrit.

**N proposé : 90 jours**, et je donne mon raisonnement plutôt qu'un chiffre nu.

- N doit être **long devant le rythme des incidents**. Le registre porte 40 constats en quelques jours de chantier intense ; sur un régime de croisière, un trimestre est l'échelle à laquelle une zone peut être dite stabilisée.
- N doit être **court devant la durée de vie du projet**, sinon la carte ne dégonfle jamais et le jury redevient permanent.
- **N n'est pas mesurable aujourd'hui** : il n'existe aucun historique de zones. Je propose donc 90 jours **comme valeur initiale déclarée provisoire**, avec l'obligation de la remesurer au premier trimestre de données réelles. Un chiffre inventé qu'on annonce comme inventé vaut mieux qu'un chiffre inventé qu'on présente comme mesuré.

**Le régime d'approbation, et le piège à ne pas reproduire.**

ADR 011 pose que l'approbation valide une proposition **déjà persistée**, dont l'empreinte est comparée **dans la transaction d'écriture**. La manche 4 a montré ce qui arrive quand ce chaînage est mal fait : la garde d'approbation des routines comparait le hash lu en base à lui-même (tautologie, `main.rs:1043`), et l'écran affichait **4 des 6 champs scellés** — donc une altération de trois champs était doublement invisible.

Trois exigences qui en découlent, et qui doivent être dans les oracles avant d'être dans le code :

1. **L'empreinte est recalculée depuis les champs relus**, jamais relue depuis la base.
2. **L'écran affiche tout ce que l'empreinte scelle** — ou affiche le hash recalculé à côté du stocké. *Un écran qui montre une empreinte sans montrer ce qu'elle scelle est trompeur même après correction du câblage.*
3. **L'oracle de la garde doit mourir sous mutation.** Retirer la garde doit rendre un test rouge. Sinon c'est une fausse assurance, qui est pire qu'une absence.

**Ce qui n'exige aucune approbation** : les entrées. Une zone entre seule.

> ⚠️ **Ce paragraphe est amendé au §10.** J'y écrivais que l'asymétrie entrée-automatique / sortie-approuvée est « voulue » parce que « se protéger est gratuit, se déprotéger se paie ». **C'est faux, et c'est le défaut central de F38** : se protéger n'est pas gratuit, et une structure qui croît seule sans décroître seule finit par tout contenir. Le §10 instruit la question et propose un autre régime. Le « N = 90 jours » ci-dessus survit, mais comme demi-vie d'un score, pas comme délai avant une demande d'approbation.

---

## 5. Le battement : tenable ?

**Non, pas tel qu'écrit — et le problème n'est pas la charge.**

F38 dit : « la relève qui évalue les routines peut évaluer la carte au même battement ». Trois obstacles, du plus dur au plus mou :

**(a) Le lot routines n'est pas mergé.** `plugins/maicie/src/routines.rs` n'existe pas sur `main` (vérifié sur `200185f`). Le lot est BLOCKED — par la mesure 1/2/2 que j'ai rendue cette nuit. **F38 est donc chaîné derrière un lot bloqué par son propre auteur de conception.** Ce n'est pas une objection à F38, c'est une dépendance à déclarer dans le plan : soit F38 attend routines, soit F38 se donne son propre déclencheur.

**(b) Le battement des routines s'exécute à chaque commande `maicie`** (manche 4, §« Aggravant »). Y greffer l'évaluation de la carte reviendrait à analyser des diffs à chaque invocation CLI, y compris `maicie status`. C'est le mauvais rythme : la carte n'a pas besoin d'être fraîche à la seconde, elle a besoin d'être fraîche **au moment où un lot est jugé**.

**(c) Les deux voies n'ont pas le même rythme naturel.**

| Voie | Rythme correct | Pourquoi |
|---|---|---|
| Voie 1 (diff) | **à la demande de revue** | elle porte sur un diff, qui n'existe qu'à ce moment |
| Voies 2 et 3 (registre, contrats) | **au battement**, ou à l'écriture d'un constat | état lentement variable |

**Proposition** : découpler. Les voies 2 et 3 s'évaluent au battement (ou, mieux, à l'écriture d'un constat — c'est le seul instant où le registre change). La voie 1 s'évalue à la demande de revue, sur le diff du lot. Le geste « ce lot déclenche-t-il un jury ? » consulte alors la table **plus** le résultat de la voie 1.

---

## 6. La question à charge : le germe ferme-t-il vraiment la boucle morte ?

**Non. Il en ferme une sur trois.** F38-b l'affirme ; je l'instruis à charge comme demandé, et l'affirmation ne tient pas.

### Boucle 1 — le démarrage à froid : FERMÉE, et bien fermée

« Pas de revue → incidents non détectés → registre vide → pas de revue ». Le germe la coupe : la carte naît non vide, la première migration de schéma déclenche un jury sans que personne ait rien élu. **Ce point de F38-b est juste et je ne le conteste pas.**

### Boucle 2 — la boucle de donnée : OUVERTE, et c'est la plus grave

La voie 2 est censée densifier la carte après le germe. Or elle interroge une donnée qui n'est pas collectée (§0). Donc :

> registre sans zones → aucune élection → la carte ne contient que le germe → seules les zones du germe sont protégées → les incidents hors germe ne peuvent jamais faire entrer leur zone.

**C'est une boucle morte permanente, pas de démarrage.** Le germe ne la ferme pas : il la masque, en donnant à la carte un contenu initial qui fait croire qu'elle vit. Une carte qui ne contient jamais que son germe est une liste écrite à la main — celle du germe — avec plus de machinerie.

Elle se ferme par le prérequis P1 (§2), et par rien d'autre.

### Boucle 3 — l'angle mort de catégorie : OUVERTE, et non fermable

Le germe protège les « portes sans retour universelles ». Deux contre-exemples, tous deux réels :

- **L'incident fondateur cité par F38-b lui-même** (le rc menteur d'`outcome_unknown`, né avec la commande d'envoi) : il tombe dans « format de protocole », donc le germe l'aurait attrapé. Bon point pour F38-b.
- **L'incident de cette nuit** (2026-08-24 21:53) : un agent a écrasé `/opt/homebrew/bin/zsh` par un script de 32 octets et paralysé vingt agents pendant quatre heures et demie. Cette zone n'est ni schéma, ni protocole, ni auth. **Aucun germe concevable ne l'aurait couverte**, parce que le geste n'était pas dans le dépôt.

Ce que ce second cas enseigne n'est pas « il faut élargir le germe » — un germe qui couvre tout ne trie plus rien. C'est : **une carte de criticité protège les zones dangereuses connues, elle ne protège pas contre les gestes hors périmètre.** Le dire dans la spec vaut mieux que le découvrir.

### Verdict sur la question posée

F38-b ferme la boucle qu'il annonce. Il n'annonce pas les deux autres. **La plus grave est la boucle 2, et elle est invisible tant qu'on ne mesure pas le registre** — ce que j'ai fait, et le chiffre est 27 %.

---

## 7. Angles morts de cette note, déclarés

- **Le seuil de 6 zones élues** est une simulation par extraction de texte libre, pas une mesure de la voie 2 réelle (qui n'existe pas). Il donne un ordre de grandeur, rien de plus.
- **N = 90 jours est inventé.** Aucune donnée ne le soutient ; il est déclaré provisoire et à remesurer.
- **Je n'ai pas instruit le coût d'exécution** de la voie 1 sur un gros diff. Il est probablement négligeable (grep sur les fichiers modifiés), mais « probablement » n'est pas une mesure — et cette nuit a montré ce que vaut un chiffre non mesuré (« 2557 × 4,8 s = 3h23 », les deux facteurs faux).
- **Je n'ai pas vérifié** si le régime propose/approve des profils est réutilisable tel quel : je n'ai pas trouvé de fonction `propose`/`approve` dans `profiles.rs` sur `main`. La réutilisation annoncée par F38 (« régime propose/approve des profils ») **reste à confirmer sur pièce** avant d'être planifiée.
- **La base de production n'a pas été ouverte**, ni en copie : aucune des questions posées n'en avait besoin. Les chiffres viennent du catalogue, lu en place.

---

## 8. Ce que je recommande, dans l'ordre

Par minimalisme : chaque étape est utilisable seule et laisse le dispositif meilleur qu'avant.

1. **P1 — zoner les constats.** Champ `zones[]` optionnel dans `AddEntry`, plus une ligne dans le mandat de revue. Sans lui, la voie 2 n'existera jamais. C'est le plus petit changement à plus fort effet de toute la note, et il ne demande aucune carte.
2. **Voie 1 + germe, sans persistance.** Prédicats sur le diff, verdict « jury / pas jury ». Aucune table, aucune migration, aucune approbation — donc aucun jury sur soi-même. Rend la doctrine mécanique dès le premier jour.
3. **Voie 3 — contrats.** Lecture de `specs/*/contracts/`, alimentation de la table. C'est là que la table apparaît, donc là que la circularité se paie : ce lot déclenche son propre jury.
4. **Voie 2 — élection**, quand le registre porte assez de constats zonés pour que le seuil soit mesuré et non supposé.
5. **Sorties de carte**, en dernier. C'est la seule partie qui touche à l'approbation, donc à l'ADR 011 et au piège B3. Elle ne doit pas être bâclée en fin de chantier — mieux vaut une carte qui ne dégonfle pas encore qu'une garde d'approbation tautologique de plus.

**Ce que je ne recommande pas** : livrer la voie 2 en extrayant les chemins du texte libre pour « démarrer sans attendre P1 ». Ça marcherait sur 27 % des constats, produirait des zones ambiguës (`main.rs`), et donnerait l'illusion d'une carte vivante. Une carte fausse est pire qu'une carte vide : elle déclenche des jurys sur les mauvais fichiers et fait taire la question.

---
---

# Amendements — réponse à deux objections du référent (2026-08-25)

## 9. Objection 1 — la voie 1 doit-elle être livrée seule, tout de suite ?

### 9.1 Oui. Et je n'ai pas trouvé de raison de s'en priver.

La question était : « y a-t-il une raison de NE PAS livrer la voie 1 seule ? Si oui, laquelle. »

J'en ai cherché quatre, et aucune ne tient :

| Raison possible | Verdict |
|---|---|
| Elle dépendrait d'une donnée manquante | **Non.** Elle ne lit que le diff. C'est la seule voie autonome. |
| Elle exigerait la table, donc une migration, donc un jury sur elle-même | **Non**, à condition de ne rien persister — voir §2, voie 1 : elle décide, elle ne mémorise pas. Aucune table, aucune migration, aucune circularité. |
| Elle produirait des faux positifs coûteux | **Marginal.** Ses bornes ratent vers le jury ; un jury de trop coûte 2 h, un jury manquant a coûté la nuit du 24/08. |
| Elle rendrait la voie 2 moins urgente | **C'est le seul vrai risque**, et il est de gouvernance, pas de technique. Voir 9.3. |

Ta mesure la justifie mieux que mon raisonnement : **3 lots sur 4 auraient été routés correctement par la voie 1 seule**, et ce sont exactement les trois que tu as routés à la main. Une règle mécanique qui reproduit les décisions d'un référent expérimenté sur 3/4 des cas, sans aucune donnée nouvelle, doit être livrée avant tout le reste. **Je révise mon §8 : la voie 1 passe en position 1, le zonage des constats en position 2.**

Le zonage reste indispensable — mais tu as raison de le qualifier d'investissement à **rendement différé** : la carte ne se remplira qu'au rythme des incidents futurs. Ce n'est pas un argument pour le repousser, c'est un argument pour ne pas *attendre* qu'il porte avant de livrer ce qui marche déjà.

### 9.2 `zones[]` : tu as raison, et ma rédaction était fausse

J'écrivais « optionnel ». Ton objection est juste et je l'adopte : *un champ facultatif n'est pas rempli, et une donnée collectée à moitié produit exactement la carte fausse que je refuse.* C'est mon propre argument retourné contre moi — la mesure des 27 % en est déjà la démonstration.

Mais la réponse n'est pas « obligatoire » tout court, parce que les deux niveaux ne sont pas le même objet :

- **Dans le schéma JSON : optionnel.** Le catalogue est un journal append-only ; rendre le champ requis invaliderait les 41 constats existants, qu'on ne réécrira pas. Un lecteur doit pouvoir relire le passé.
- **Dans le protocole de revue : obligatoire.** Un constat sans `zones[]` est un constat incomplet, et le mandat de revue l'exige au même titre que la sévérité.
- **Et le respect se mesure, sinon il n'existe pas** : exposer le **taux de constats zonés** dans `registre list` — « 38/41 zonés (93 %) ». Une règle de process sans indicateur retombe en trois semaines ; celle-ci se contrôle en une ligne.

Ajout que ta remarque m'impose : **le seuil de la voie 2 ne doit pas être appliqué tant que le taux de zonage est bas.** Élire sur 30 % des constats, c'est élire sur un échantillon biaisé — les constats les plus verbeux. Proposition : la voie 2 reste inactive sous **80 % de zonage sur les 90 derniers jours**, et le dit dans son affichage. Sinon on retombe dans la carte fausse par un autre chemin.

### 9.3 Le seul vrai risque, et comment le borner

Livrer la voie 1 seule donnera l'impression que **F38 est fait**. Elle traite le cas le plus visible (les migrations), avec un rendement démontré de 3/4 — et l'énergie pour aller chercher le zonage retombera.

Ce n'est pas une raison de ne pas livrer. C'est une raison de **livrer en le disant** : que la voie 1 soit annoncée comme *« déclenchement mécanique par marqueurs — la carte n'existe pas encore »*, et que son affichage porte la mention du 4ᵉ lot qu'elle n'a pas attrapé. Un dispositif qui nomme sa propre couverture ne se fait pas passer pour complet.

---

## 10. Objection 2 — le faux positif, et pourquoi F38 ne tient pas tel qu'il est écrit

Tu as raison sur le fond, et l'objection est plus grave que tu ne la formules. **Je n'avais instruit qu'une moitié du problème.**

### 10.1 Ce que ça donne, chiffré

Dénominateur : **59 fichiers `src/*.rs` hors tests** (141 `.rs` au total).

| | |
|---|---|
| Zones élues, **mesurées** sur les 29 % de constats zonables | **6** = 10 % du dépôt |
| Zones élues, **extrapolées** à 100 % de zonage | **~20** = **35 % du dépôt, en UNE nuit** |

*(Le second chiffre est une extrapolation et je l'annonce comme telle — la leçon de la nuit sur « le produit de deux estimations n'est pas une mesure » vaut aussi pour moi.)*

Projection du modèle F38 tel qu'écrit — entrée automatique, sortie sur approbation — en supposant que chaque nuit de chantier apporte 40 % de zones neuves :

| Nuits de chantier | Carte | Part du dépôt |
|---|---|---|
| 1 | 20,5 | **35 %** |
| 2 | 28,7 | **49 %** |
| 3 | 32,0 | **54 %** |
| 10 | 34,2 | **58 %** |

**Une seule nuit met un tiers du dépôt sous jury 1+1. Trois nuits, la moitié.**

### 10.2 Le vrai défaut n'est pas le coût — c'est l'inversion de doctrine

Le coût de revue est l'objection évidente et tu l'as nommée. Il y a pire, et c'est structurel :

Une carte qui ne décroît jamais **finit par contenir exactement le code mature, et lui seul**. Le code neuf, lui, n'a pas encore de constats — donc il n'est pas dans la carte, donc il ne déclenche pas de jury.

Or F38-b dit, textuellement : *« les erreurs de jeunesse sont les plus chères : les portes sans retour se franchissent dans du code simple »*, et cite le rc menteur d'`outcome_unknown` né avec la commande d'envoi.

**À cinq ans, le modèle monotone protège tout sauf ce que F38-b désigne comme le plus dangereux.** Il ne meurt pas seulement de tout déclencher : il déclenche là où on a déjà appris, et se tait là où on n'a pas encore payé. C'est l'inverse exact de son intention.

### 10.3 Tes quatre questions

**« Quel signal de calme, mesurable et non déclaratif ? »**
Le seul signal non déclaratif disponible est **l'absence de constat qualifiant depuis Δt**. Tout le reste (« la zone est stabilisée », « le code est propre ») est une opinion. J'ajoute une correction à mon §4 : j'y proposais de compter aussi « zone touchée par un lot mergé » comme signal de non-calme. **Je la retire** — elle rendrait critique toute zone activement développée, c'est-à-dire précisément le code neuf, mais pour la mauvaise raison, et elle gonflerait la carte d'un second facteur.

**« Une zone dont le constat a été CORRIGÉ devrait-elle sortir ? »**
**Non, et c'est le piège de la question.** Un constat corrigé prouve que la zone est *difficile*, pas qu'elle est *sûre* — et le code qui vient d'être modifié pour la correction est du code frais, donc plus risqué qu'avant. Faire sortir sur correction récompenserait la réparation par un retrait de surveillance, au moment exact où la surveillance est la plus utile. Ce qui doit valoir, c'est **le temps écoulé sans nouveau constat**, pas la réparation de l'ancien.

**« Peut-on borner la taille par construction plutôt que par décision ? »**
Oui, et c'est la bonne réponse à toute l'objection. Non pas en plafonnant à K zones — un plafond dur ferait sortir une zone dangereuse parce qu'une autre est entrée, ce qui est inacceptable — mais en remplaçant **l'appartenance permanente par un score qui décroît**.

```
score(zone) = Σ  poids(sévérité) × 2^(−Δt / T)         poids : blocker 8, major 4, minor 1
              constats de la zone                       T = demi-vie, proposition 90 j
zone critique  ⟺  score ≥ 4
```

Propriétés, et c'est là que le modèle se justifie :

- **Le seuil d'entrée de F38 est préservé à l'identique** : 1 blocker (8) ou 2 majors (4+4) franchissent 4 ; 1 major seul (4×2⁻ᵋ) ne franchit pas. Rien à renégocier.
- **La sortie est automatique et datée d'avance** : un blocker isolé protège sa zone pendant exactement T jours. Deux blockers, 2T. La récidive prolonge.
- **La taille est bornée par le rythme des constats, pas par une décision** : à l'équilibre, la carte contient les zones ayant reçu un constat qualifiant dans les ~T derniers jours. Entrées et sorties s'équilibrent seules.
- **La carte devient une photo du présent** au lieu d'un cumul du passé — donc elle suit le code neuf au lieu de sédimenter sur l'ancien.
- **Un dépassement devient un signal, pas une fatalité** : si la carte franchit, disons, 30 % du dépôt, ce n'est pas qu'il faut plus de jurys, c'est que T ou le seuil sont mal réglés. À afficher, pas à subir.

**« La sortie automatique est-elle impossible sans humain ? »**
Non. Et l'approbation humaine sur les sorties est, telle qu'écrite, **une protection contre un risque qui n'existe pas**.

L'asymétrie « entrée gratuite / sortie approuvée » ne se justifie que si une sortie erronée est **difficile à annuler**. Ici, elle est annulée par le constat suivant, gratuitement et automatiquement. Le coût de l'erreur de sortie est donc quasi nul — pendant que le coût de l'asymétrie est la croissance monotone, c'est-à-dire la mort du dispositif.

### 10.4 Donc : « non, pas comme ça » — et voici ce que je propose à la place

Tu m'as demandé de le dire si je le pensais. Je le pense sur ce point précis, et sur lui seul.

**L'exigence F38 est juste dans son intention et fausse dans son régime d'approbation.** Ce qu'il faut renverser :

| | F38 tel qu'écrit | Proposition |
|---|---|---|
| Entrée | automatique | automatique — **inchangé** |
| Sortie ordinaire | approbation humaine après N jours | **automatique**, par décroissance du score |
| Geste humain résiduel | approuver chaque sortie | **approuver les exceptions** : sortie anticipée, ou entrée permanente déclarée en config |

L'humain ne disparaît pas : il change de place. Il quitte le geste routinier et nombreux — approuver des dizaines de rétrogradations, qu'il finira par approuver en bloc sans les lire, exactement comme l'écran B3 de la manche 4 finissait par être lu sans être vu — et il garde les gestes rares et réellement décisifs.

**Ce que ça coûte à cinq ans si on ne le fait pas**, puisque tu le demandes : la carte tend vers l'ensemble du code mature ; le jury 1+1 devient le régime ordinaire ; le coût de revue passe de 15-30 min à ~2 h **par lot** ; et plus personne ne distingue une zone dangereuse d'une zone historiquement accidentée. Le dispositif ne s'effondre pas d'un coup — il devient une formalité qu'on contourne. C'est la fin la plus banale des dispositifs de contrôle, et la plus difficile à inverser une fois installée.

### 10.5 Ce que cet amendement ne résout pas

- **T = 90 jours reste inventé.** Le score le rend moins critique qu'un seuil binaire (se tromper sur T décale une sortie, il ne la supprime pas), mais il reste à remesurer sur données réelles.
- **La pondération 8 / 4 / 1 est choisie pour préserver le seuil de F38**, pas mesurée. Elle est défendable parce qu'elle ne change rien à l'entrée ; toute autre pondération devrait être justifiée par le même critère.
- **Je n'ai pas instruit la migration du modèle** : si une carte monotone est livrée d'abord, la basculer vers un score demandera de dater les constats de chaque zone — le catalogue les date, donc c'est faisable, mais ce n'est pas gratuit. Argument de plus pour ne pas livrer le régime monotone à titre transitoire.
