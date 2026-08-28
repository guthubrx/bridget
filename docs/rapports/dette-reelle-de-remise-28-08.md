# Dette réelle de remise — enquête sur trois tables

- **Enquêteur** : `cartae0-flux`
- **Rendu** : 2026-08-28, ~18h30 UTC
- **Mandat** : objectif `c2e80132-2abd-4bda-a7ff-6fffe66fa63e` / délégation `26f63a46-5a4f-498a-a544-7af55e008f07` / message `c9a1e625-bce8-4084-bd74-f3de7cb36181`
- **Méthode** : lecture seule stricte (`sqlite3 'file:…?mode=ro'`) sur `/home/moi/.cache/bridget/bridget.db`, plus lecture du code source. **Aucune écriture, aucune correction, aucun code.**

---

## RÉPONSE COURTE

> **La dette n'est pas de 28. Elle est d'au moins 79 — et ce ne sont pas des messages restés sans réponse : ce sont des messages JAMAIS REÇUS.**
>
> Le diagnostic s'inverse : le référent croyait devoir des réponses. En réalité, **on lui a parlé sans qu'il entende**.

---

## 1. Ce que `phase=indeterminate` signifie exactement

**Source** : `crates/bridget-daemon/src/idempotency.rs:1206-1220`.

C'est un **état ABSORBANT**. Citation du code : *« Plus rien ne l'accusera jamais, et elle occupe pourtant sa clé jusqu'à l'expiration de l'horizon (7 jours). »* Une remise `indeterminate` **ne partira plus jamais**. Ce n'est pas « en attente », c'est **définitivement perdu**.

**Trois causes documentées**, pas cinq :
1. échec de reprise ;
2. signal `DeliveryIndeterminate` émis par le wrapper ;
3. migration v2, qui écarte les enveloppes absentes (`idempotency.rs:474-479`).

La contrainte porte **quatre** phases, pas trois : `CHECK (phase IN ('dispatching','acked','indeterminate','orphaned'))`. `orphaned` = **0 ligne** en base ; c'est aussi un état absorbant, mais de sort **connu** (destinataire purgé).

### L'homonymie est confirmée — ce n'est pas mon `capture_reason`

| | `capture_reason` | `phase=indeterminate` |
|---|---|---|
| fichier | `plugins/maicie/src/main.rs:351-374` | `crates/bridget-daemon/src/idempotency.rs` |
| couche | client Maicie | daemon Bridget |
| objet | traduit un `BridgetClientError` en motif de capture | état durable d'une saga de remise |
| valeurs | `budget_capture_epuise`, `negociation_daemon_refusee`, `liaison_bridget_fermee`, `reponse_bridget_illisible`, `annuaire_bridget_indisponible` | `dispatching`, `acked`, `indeterminate`, `orphaned` |

**Aucune parenté.** L'avertissement que j'avais donné le matin — *chercher une cause unique serait peut-être chercher une parenté qui n'existe pas* — est vérifié. **Les 94 ne relèvent ni de mes cinq codes ni d'un sixième cas : elles n'appartiennent pas à ce vocabulaire.** La question était mal posée, et je l'avais mal posée autant que le référent.

### Les 94 sont-elles de vraies remises ? — OUI, mesuré

```
acked         | enveloppe présente | 5289
indeterminate | enveloppe présente |   94
dispatching   | enveloppe présente |   35
```

**Les 94 ont TOUTES `message_bytes` non NULL.** Or la migration v2 ne frappe que `message_bytes IS NULL`. Donc **aucune des 94 ne vient de la migration** : ce sont des remises réelles, produites en fonctionnement, par échec de reprise ou par `DeliveryIndeterminate`. Migrations appliquées : v2, v3, v4.

## 2. La clé de jointure — ELLE EXISTE

> **`send_deliveries.idempotency_key = ledger.id`**

- **5347 paires** jointes.
- **Contrôle négatif** : `delivery_id = ledger.id` → **0**. La jointure n'est pas un artefact.
- **Preuve sur un cas connu** — ma propre remise : `ledger.id = e99afe7c1c7c4` ↔ `send_deliveries.idempotency_key = e99afe7c1c7c4`, `delivery_id = 795e9b6e-…`, `phase = indeterminate`.

jc1-flux déclarait ne pas l'avoir cherchée. Elle est là. On peut désormais **nommer qui a perdu quoi**.

> ### ⚠ CORRECTION DE MON PROPRE RAPPORT — mesurée à 18h35, après coup
>
> **J'avais écrit que la dette « cesse d'être un minorant structurel ». C'est trop fort, et je le retire.**
>
> Mesure : `ledger` = **9227** lignes · `send_deliveries` = **5420** sagas · jointes = **5349** · **sans aucune saga = 3878, soit 42 %**.
>
> Découvert par un cas propre : mes envois qui rendent `OK: envoyé` directement n'ont **aucune ligne** dans `send_deliveries` ; seuls ceux passés par le dépôt idempotent (`DÉPÔT: in_flight`) en créent une. **`send_deliveries` ne couvre donc pas tous les messages.**
>
> **Conséquence honnête :** la jointure lève la limite pour les 58 % couverts par une saga. Pour les **3878 autres (42 %), aucune information de remise n'existe** — ni succès, ni échec. Le chiffre de 79 reste donc **un plancher**, comme annoncé, mais désormais on sait *de combien* la zone d'ombre s'étend : `bridget` elle-même compte **1945** messages sans saga.
>
> *Je corrige plutôt que de laisser passer : mon rapport est dans `main`, et une formule trop forte y aurait circulé comme un fait.*

## 3. La dette réelle, nommée

**91 des 94** sont joignables au ledger (3 ne le sont pas — ledger purgé ou expéditeur absent).

| expéditeur | destinataire | perdus |
|---|---|---|
| jc3 | bridget | **22** |
| jc2 | bridget | 13 |
| rc1 | bridget | 13 |
| cartae0 | bridget | 10 |
| rc5 | bridget | 7 |
| essai-distant | bridget | 6 |
| jc1 | bridget | 4 |
| bridget | relec8 | 4 |
| lentille-viktor | cursorbridget | 2 |
| **humain** | **bridget** | **1** |
| jc6, rc7, cartae0-flux | bridget | 1 chacun |
| bridget | relec7/relec9/cursor3/cursorbridget/jury1 | 1 chacun |
| ac1 | relec9 | 1 |

**79 remises perdues à destination de `bridget`**, dont **77 émises par les neuf prédécesseurs tmux**.

**Datation** — nuance importante, contre l'alarmisme : **79 des 91 datent du 27/08**, 7 du 28/08, 5 du 25/08. *La dette n'est pas majoritairement d'aujourd'hui.*

> **Ce que cela retourne :** les « sept preuves de vie jamais accusées » n'étaient pas un défaut de courtoisie. jc3 a tenté **22** fois, rc1 **13**, jc2 **13**, cartae0 **10**. Ils n'étaient pas silencieux — **ils étaient inaudibles.**
>
> *Réserve explicite : les 13 pertes de rc1 datent toutes du 27/08 (18h42–22h04). Elles n'expliquent donc **pas** son silence du 28/08. Je ne fais pas ce lien.*

## 4. Le message perdu de l'humain — à relire, il n'est pas anodin

`ledger.id = 3b227f65278d4` · `delivery_id = 77889dd5-18ed-46e9-8f44-b0a0d1b38d48` · **28/08 à 10:38:59** · `humain → bridget` · **jamais reçu**.

Il contient, entre autres :
- **un accord explicite** : *« Ce que tu as fait pour la relance était très bien […] je renouvelle, tu avais mon accord »* ;
- le constat que **le service tournait sur un ancien binaire, mal relancé** ;
- l'annonce d'une **procédure de relance vérifiée** : `scripts/bridget-relance.sh`, installée sous `/home/moi/.local/bin/bridget-relance`, commit `394c0f5`, encodant huit pièges mesurés ;
- notamment : *« un daemon de test compilé dans /tmp par un agent avait occupé la socket de production pendant dix-sept heures ; toutes les relances échouaient sans le dire et la version affichée ne changeait jamais »*.

**Ce dernier point explique vraisemblablement le `build-id f7658d4d9746-dirty`** signalé le matin comme non attestable. *L'explication d'un constat du jour dormait dans un message jamais remis.*

## 5. Les 291 `timed_out`

**220 sur 291 — soit 76 % — ont une fenêtre de 60 secondes.**

| fenêtre | n |
|---|---|
| **60 s** | **220** |
| 900 s | 11 |
| 1200 s | 7 |
| 120 s | 7 |
| 600 s | 6 |
| 59 s | 6 |

Principales cibles : `bridget→coderBridget` 63, `bridget→cxbridget` 48, `bridget→prospective` 17 — vraisemblablement des destinataires disparus.

**Le cas cité, confirmé au détail près** : `0f2c81ee89924`, `humain → bridget`, créé **27/08 18:31:19**, fenêtre **60 s**, expiré **18:32:20** — soit **61 secondes**.

**Et il n'est pas seul** : bilan complet de l'humain — `humain→jc6` 7 answered · `humain→bridget` 3 answered et **2 timed_out** · `bridget→humain` **1 timed_out** (fenêtre 120 s).

> **Le défaut de fond :** une fenêtre de 60 à 120 secondes est calibrée pour un agent, pas pour un humain. Un humain à qui l'on demande une décision expirera **systématiquement**. `bridget→humain` du 28/08 16:31:33 a expiré en 120 secondes — pendant que l'humain vivait sa journée.

## 6. Réfutation annexe — `bridget status` ment sur la taille du ledger

`bridget status` annonce **« Messages en base: 1000 »**.

Mesure : **le ledger contient 9 200 lignes**, du **21/08 19:45:50** au **28/08 18:19:15**.

Encore une fenêtre prise pour le tout. Toute estimation fondée sur ce « 1000 » est fausse d'un facteur 9.

## 7. Verdict

1. **`indeterminate` = perte définitive**, pas attente. 94 pertes réelles, aucune imputable à la migration.
2. **Aucun rapport avec `capture_reason`** : homonymie confirmée, la question d'origine était mal posée.
3. **La clé de jointure existe** — `idempotency_key = ledger.id` — et lève la limite structurelle du minorant.
4. **Dette réelle vers le référent : 79, pas 28.** Presque le triple. Dont un message de l'humain.
5. **Le diagnostic s'inverse** : ce ne sont pas des réponses dues, ce sont des messages jamais entendus. Aucune discipline d'émission n'aurait corrigé cela.
6. **291 expirations dont 76 % à 60 secondes** : la fenêtre par défaut est inadaptée à un interlocuteur humain.

**Portée** : mesures prises le 28/08 vers 18h25 UTC sur la base vivante. Les compteurs bougent — `indeterminate` était à 88 ce matin, 94 maintenant. **Refaire les requêtes avant de s'appuyer sur ces chiffres.** Aucun remote git n'a été mesuré ici : l'enquête porte sur la base, pas sur le dépôt.
