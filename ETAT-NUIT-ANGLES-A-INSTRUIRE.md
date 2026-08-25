# Lot routines (`9f7bcc6`) — VERDICT CONSOLIDÉ DU JURY : **BLOCKED**

**Collège aligné, plus aucun désaccord.** Motif unique : l'adoption des
mandats orphelins n'est pas atteignable au-delà de la borne de rattrapage
(§C). L'APPROVE_WITH_CHANGES initial a été **retiré par son auteur**, qui
avait annoncé cette bascule **avant** toute mesure.

## ⚑ UNE FAMILLE, PAS TROIS INCIDENTS — à graver comme telle

Le motif, la dette (§A) et la sautee menteuse d'origine sont **une seule
classe** :

> *Une routine cesse de fonctionner, rien ne le dit, et la surface affirme
> le contraire.*

Trois chemins vers l'extinction muette, dans un lot **dont la raison d'être
est la vigilance automatique**. Et la sentinelle ne rachète rien : dire
« j'ai raté N rondes » est vrai, rassurant, et faux sur l'essentiel — elle
ne tait pas seulement le mandat en vol, **elle détourne d'aller le
chercher**. Une trace qui maquille clôt l'enquête.

## ⚑ CE QUE LA NUIT A PROUVÉ SUR LE DISPOSITIF LUI-MÊME

> **La revue a produit presque autant de défauts que le lot.**

Deux conditions du jury ont créé des défauts (§E). Et les relecteurs se sont
trompés autant que l'auteur : une causalité attribuée à tort, un chiffre
juste donné pour une vérité générale, un motif mal nommé, un banc rouge par
construction pris pour critère de blocage.

**Aucune de ces erreurs n'a survécu — parce que le collège avait deux bras
qui se sont corrigés six fois.** Un jury à un seul bras aurait rendu un
verdict **propre et faux**.

---

## Les points ouverts, par rang de preuve

Classement par **rang de preuve** : les deux premiers sont **mesurés**, le
troisième n'est qu'une lecture doublement corroborée.

---

# ⚑ §A — MESURÉ : la routine meurt en silence et s'affiche vivante

Délégation **annulée** sans clôture d'objectif → dix relèves plus tard :
occurrence toujours `ouverte`, dix buckets `differee`, **zéro nouveau
mandat**, contrôle positif (objectif clos) vert.
Banc prêt : `/tmp/relec5-preuves/relec5_occurrence_a_jamais.rs`.

**La routine est morte définitivement, en silence, pendant que `routine
list` l'affiche `active`.**

**⚠️ ERREUR D'ARBITRAGE DU RÉFÉRENT, à corriger** : la dette a été acceptée
sur l'énoncé « l'occurrence reste ouverte » — un détail d'état interne. Le
fait réel est « la routine ne délèguera plus jamais **et l'affichage ment** »
— une panne silencieuse démentie par sa propre surface. **L'arbitrage a été
rendu sur une description qui cachait son enjeu.**

→ **La dette doit être reformulée dans ces termes. À défaut, ce point
devient BLOQUANT.**

*Parenté à instruire ensemble avec §C : dans les deux cas, une routine cesse
de fonctionner sans que rien ne le dise. Deux chemins, une même famille.*

---

# ⚑ §B — MESURÉ : le refus avant écran n'est gardé par aucun oracle

Mutant MUT-A (retrait du refus pré-écran dans `main.rs`) → **76 tests
VERTS**. Contrôle positif MUT-B (retrait de la garde du domaine) → l'oracle
meurt. Donc la garde du domaine **est** protégée, mais **la ligne qui
empêche la « vigilance piégée » ne l'est par rien**. Le prochain qui touchera
`main.rs` la supprimera sans qu'un test ne bronche.

C'est la famille « oracle qui garde une propriété plus faible que celle qu'il
nomme », trouvée par **trois jurys** cette nuit. **À traiter comme condition,
et non à ranger dans les acquis** — l'erreur avait été faite dans une version
précédente de cette note.

---

# ⚑ §C — **MESURÉ (2026-08-25)** : l'adoption hors borne — MOTIF ÉTABLI

`adopt_orphan_mandate` n'est appelé que dans `for bucket in from..=current`.
Quand `gap > MAX_CATCHUP_BUCKETS` (=64), `from` est remonté à
`truncated_end + 1` et la sentinelle s'écrit **sans passer par l'adoption**
(routines.rs:334-355). → orphelin jamais adopté, **sautee menteuse
réintroduite**, `has_open = false`, **second mandat**. Seuil : **64 minutes**
d'arrêt à la période minimale — le profil des deux pannes de cette nuit.

**Preuve textuelle** : `tests/contract/routines.rs:712`,
`let plus_tard = t0 + 2 * period;` → `gap = 3`. **Aucun des quatre oracles
n'entre jamais dans la branche de la borne.** Verts de bonne foi, aveugles
au régime.

**État de preuve** : deux lectures indépendantes, de polarités opposées, sur
deux têtes dont une **après** « chiffrer la borne ». **Aucune mesure.** Les
deux relecteurs classent BLOCKED **sous condition de la mesure**, chacun
l'ayant annoncé avant elle.

**LE RÉGIME N'EST PAS DÉGRADÉ, IL EST OBSERVÉ** : 64 périodes = 64 minutes —
redémarrage, veille, mise à jour, nuit. Et ce n'est pas une extrapolation :
**nous y sommes depuis plus d'une heure**, sept agents sans shell. Dans un
système dont la doctrine est « l'horloge est la relève, aucun timer
résident », c'est le cas de **la première relève après tout arrêt réel**.

**RANG, dans la formulation la plus juste qui ait été proposée** :

> *Un correctif qui laisse intact son propre défaut dans un régime courant
> n'est pas un correctif partiel — c'est un correctif dont le périmètre n'a
> pas été déclaré.*

Si l'auteur veut le borner, qu'il l'écrive noir sur blanc : cela devient
alors une dette dont on discute, et non un trou.

## LE GESTE QUI TRANCHE — série appariée, une seule constante varie

Ligne 712 (constante importée ligne 13, comptage écrit ligne 724) :

| reprise | attendu si l'hypothèse tient |
|---|---|
| `t0 + 2 * period` | **1** (contrôle positif) |
| `t0 + 65 * period` | **2** |
| `t0 + 100 * period` | **2** |

**1/2/2** → motif établi. **1/2/1** → artefact de bord, motif tombe.

**⚠️ Le banc DOIT être réécrit sur `evaluate_routines_with(..., opts)`** :
la variable d'environnement n'existe plus sur `9f7bcc6`, donc les bancs de
`/tmp/relec1-preuves` sont **inopérants** — les lancer tels quels donnerait
un vert qui ne prouve rien.

---

# ⬜ §D — condition RETIRÉE, remplacée par UNE LECTURE À FAIRE

**La condition « contrôle d'existence » est SANS OBJET** : `PRAGMA
foreign_keys = ON` est posé **à chaque connexion** (store.rs:376-385, dans
`MaicieStore::open`, reconfirmé par lecture directe), et le schéma porte
`routine_occurrences.delegation_id TEXT REFERENCES delegations(id)`. La
contrainte est donc **effective** : une occurrence ouverte sur un
identifiant fantôme ne peut pas s'écrire. *Retirée par le relecteur à charge
qui la portait — le quatrième de ses motifs qu'il défait lui-même, et celui
qu'il avait le plus d'intérêt à garder.*

Ce qui survit n'est plus un défaut d'intégrité mais un **mode de dégradation
brutal** (l'INSERT échoue → l'erreur remonte → la commande utilisateur
échoue), à rattacher au **§E**.

## ⚑ MAIS CETTE FERMETURE OUVRE UNE LECTURE QUI PEUT TOUT CHANGER

Les clés étrangères étant **actives**, la clause de suppression de
`delegate_idempotency` gouverne désormais l'angle de la purge. **Trois
issues exclusives :**

| Schéma de `delegate_idempotency` | Conséquence |
|---|---|
| **FK vers `delegations(id)` SANS `ON DELETE CASCADE`** | Supprimer une délégation sans sa clé d'idempotence est **structurellement impossible**. Angle **clos**. |
| **FK AVEC `ON DELETE CASCADE`** | L'angle se rouvre : tout `DELETE FROM delegations` purgerait la clé par ricochet → retour à la sautee menteuse. |
| **Aucune FK** | Divergence possible → mode de dégradation. |

## ✅ LA CLAUSE A ÉTÉ LUE — L'ANGLE EST CLOS

`store.rs:6702-6710` :

```sql
CREATE TABLE IF NOT EXISTS delegate_idempotency (
    idempotency_key TEXT PRIMARY KEY,
    ...
    delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),
    ...
);
```

**FK vers `delegations(id)`, SANS `ON DELETE CASCADE`** — première issue.
Avec le PRAGMA actif :

- un `DELETE FROM delegations` sur une ligne référencée **échoue sur la
  contrainte** → **le chemin qui ferait diverger les deux tables ne peut pas
  exister** ;
- l'ordre est imposé : supprimer une délégation exige de supprimer d'abord
  sa clé d'idempotence ;
- **seul un `DELETE FROM delegate_idempotency` direct rouvrirait l'angle** —
  le grep initial reste, mais il devient le **seul**, et il est aussi le
  préalable obligé de toute suppression de délégation ;
- `delegation_id` est `UNIQUE` : une délégation a **au plus une clé**, ce qui
  ferme l'hypothèse d'un lookup tombant sur la mauvaise ligne.

**Statut : clos structurellement**, sauf `DELETE FROM delegate_idempotency`
direct — lequel deviendrait de surcroît le **préalable obligé** de toute
suppression de délégation, donc du code visible et intentionnel.

## ⚑ CONSÉQUENCE À DÉCHARGE — le remède sort RENFORCÉ, pas seulement indemne

Puisque `delegation_id` est `NOT NULL UNIQUE REFERENCES delegations(id)`
**sans cascade**, la ligne d'idempotence **survit exactement aussi longtemps
que la délégation qu'elle atteste**. Donc le lookup d'adoption retrouvera
**toujours** le mandat tant que ce mandat existe — quelle que soit
l'ancienneté de la reprise, quel que soit l'état de la délégation, quel que
soit le délai de rétention. Et le `UNIQUE` ferme l'hypothèse d'un lookup
tombant sur la mauvaise ligne : une délégation a **au plus une** clé.

**Le remède d'adoption est donc structurellement fiable, et pas seulement
« non réfuté ».** C'est l'inverse de ce que le collège soupçonnait : il
cherchait par quelle porte l'adoption pouvait perdre son mandat, et le
schéma répond qu'il n'y en a pas. *Le remède sort renforcé de trois angles
successifs — expiration, purge, existence.*

**Réserve** : lecture de la migration **de base** (`CREATE TABLE IF NOT
EXISTS`) ; les migrations ultérieures n'ont pas été balayées. Sur SQLite,
modifier une clé étrangère impose une table de remplacement — donc du code
peu discret — mais ce n'est pas une preuve. **Geste qui tranche, gratuit** :
`sqlite3 <base> "SELECT sql FROM sqlite_master WHERE
name='delegate_idempotency';"` sur une base **vivante** créée par le binaire
du lot.

---

---

# ⚑ §E — UNE CONDITION DU JURY A CRÉÉ UN DÉFAUT

En exigeant le retrait de `let _ = evaluate_routines(...)` — dont le
commentaire disait « un échec du tick ne doit jamais faire échouer status /
delegate » — **le jury a fait remonter l'erreur jusqu'à la commande
utilisateur**. Conséquence : une collision de clé primaire entre deux ticks
concurrents fait désormais **échouer `maicie status`**, et deux commandes
simultanées sont le régime normal d'une flotte.

**À nommer comme effet de bord d'une exigence du jury, jamais à compter
contre l'auteur.**

C'est la **deuxième** occurrence : la première est la ligne d'écran
« recalculé depuis les champs affichés », devenue fausse dans un complément
qui répondait lui aussi à une de nos demandes.

> **Deux occurrences font un motif : nos conditions doivent être relues
> comme du code.**

---

## ⬜ RETIRÉ PAR SON AUTEUR — « mandat mort adopté = gel durable »

Le relecteur qui la portait retire cette charge : le gel frappe **déjà** une
occurrence normale, sans aucune adoption. **Le remède n'invente pas le gel,
il l'étend** — et redéléguer recréerait le doublon d'origine. *Le remède
échange un doublon contre un gel, et c'est le bon échange.* À reclasser en
**renfort de la dette §A**, pas en défaut du remède.

## ✅ CLOS — arbitré, ne pas rouvrir

- **Angle temporel (`Ok(None)` après expiration)** : **CLOS**. Mesure — le
  banc tournait déjà avec une reprise postérieure à la rétention et rend un
  seul mandat ; lecture — égalité de clé nue, sans filtre temporel.
  *(Un message du référent l'avait dit « ouvert » : version périmée, le
  présent document fait foi.)*
- **Angle « purge active »** : **réduit à UN SEUL GREP** —
  `rg -n "DELETE FROM delegate_idempotency" plugins/maicie/src/`.
  Précision **à décharge** : `dedup_retained_until` ne vit ni dans
  `delegations` ni dans `delegate_idempotency`, mais dans
  **`deferred_delegation_dispatch`** (store.rs:2898-2902) — la file de
  dispatch différé, où une expiration est dans la nature des choses. Or
  `lookup_delegate_ids_by_key` **ne consulte jamais cette table**. Donc un
  hit sur `deferred_delegation_dispatch` serait **à décharge, pas à
  charge** ; seule une purge de `delegate_idempotency` percerait
  l'adoption.
- **Point de coupure** : supprimé au profit d'une injection par paramètre.
  **VÉRIFIÉ DE DEUX MAINS, rien à refaire** — lecture intégrale de
  `routines.rs` (546 lignes, zéro lecture d'environnement, imports compris)
  par un relecteur, ET `strings` sur un binaire **release recompilé** par
  l'autre, **avec contrôle positif de l'instrument** (trois symboles à 0,
  deux témoins à 1 — `strings` lui avait menti plus tôt sur une chaîne
  accentuée). Plus aucun `var_os` ni `env::var` dans `plugins/maicie/src/`.
- **Écran** : six entrées scellées affichées, libellé vrai.
- **Sentinelle** : formule `truncated_end − after − 1` **exacte** ; **43135
  et 43136 sont tous deux justes** selon `after`. Graver la formule, jamais
  un chiffre nu ; exiger que tout exemple précise son `after`.
- **`matrice_sc003`** hors lot.
- **Rouge à échéance absolue — DEUX JEUX DE MESURES CONSERVÉS, avec leurs
  conditions d'exécution.** *(Ne pas en effacer un : leur divergence est la
  seule trace qui permettra un jour de trancher la variable.)*
  - **relec1** — `TMPDIR` court et 0700 (`mktemp -d /tmp/r1t.XXXX`), lot nu,
    trois passages → **5 verts / 5 en isolé**, **1 rouge en suite complète**
    (271/1/5). Donc l'écart isolé/suite est mesuré **à `TMPDIR` constant** :
    il n'est pas explicable par lui.
  - **relec5** — `TMPDIR` non contrôlé → **5 rouges / 5 en isolé** sur le
    lot, **3 rouges / 3 en isolé** sur la base `45ffa11`.
  - **Les deux « isolés » se CONTREDISENT.** Cause environnementale **non
    identifiée** : quatre candidates (charge, pression disque, horloge,
    `TMPDIR`), **zéro mesure discriminante**. Ne PAS inscrire « TMPDIR non
    conforme » — attribution proposée puis **réfutée** (les deux défauts
    `TMPDIR` connus ont des symptômes nommés, aucun n'est celui-ci, qui est
    un test *sensible au temps*).
  - **Conclusion commune, sur les mesures de relec1** : le test est
    **déterministe par contexte d'exécution** — « flake » est le mauvais
    mot. **Conclusion propre à relec5** : le rouge est **hors lot** (3/3 sur
    la base).
- **Ne jamais merger `061e773` ni `f837ed0`** — ils portent l'interrupteur.

## Acquis, aucune mesure requise

**1 délégation** en reprise courte (contre 2 sur les têtes précédentes, 35
tirs sur 35), reproduit avec banc assertif et contrôle positif. **MA-1** :
en retirant le remède, les trois oracles intégrés meurent — **l'intégration
n'est pas décorative**. B3 fermé sur les quatre familles d'entrées, contrôle
négatif vert.

*Un lot peut être bon sur cinq motifs et bloqué par un sixième.*

---

## ⚑ LA LEÇON DE LA NUIT, à porter dans les règles

> **Un banc doit déclarer le régime qu'il couvre, et toute constante
> introduite après lui doit faire rejouer les bancs dans le régime qu'elle
> crée.**

Deux correctifs justes — la borne coupe le rattrapage, l'adoption répare la
fenêtre — ont laissé un trou dans l'intervalle qu'aucun des deux ne
gouverne. Personne n'a fauté : **c'est leur composition qui n'a jamais été
relue.**
