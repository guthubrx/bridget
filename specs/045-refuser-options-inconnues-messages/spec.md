# Spécification 045 — Refuser les options inconnues dans les messages

**Branche** : `session-045-refuser-options-inconnues-messages`
**Base gelée** : `bc745335530985ce305e82fea4007071c752d5b0`
**Dépendances** : session 041 intégrée (validation stricte de `--timeout` et `--hops`)
**Objectif** : `30d9a283-2ca6-431b-ba1f-9b1d6d9e1c6a`

## Contexte

Les commandes `send` et `reply` traitent tout jeton non reconnu comme une
partie du corps. Une faute d'option rend donc le code 0 et altère réellement le
message sérialisé. Un usage réel a ainsi envoyé un corps commençant par
`--body`, alors que l'appelant croyait employer une option.

La base refuse déjà les valeurs invalides et les valeurs finales manquantes de
`--timeout` et `--hops`. En revanche, `--to` et `--from` sans valeur peuvent
encore tomber sur un diagnostic générique ou être ignorés.

## Scénarios utilisateur et tests

### US1 — Refuser une option inconnue avant tout envoi (P1)

Un utilisateur commet une faute d'option dans `send` ou `reply`. La commande
rend le code 2, nomme le jeton incompris et n'établit aucune connexion au
daemon. Aucun message corrompu n'est sérialisé.

**Critères d'acceptation** :

1. L'option inconnue est refusée avant, entre ou après les mots ordinaires du
   corps tant que le séparateur explicite n'a pas été rencontré.
2. Le diagnostic contient le jeton brut incompris.
3. Le faux daemon ne reçoit aucun message et aucun fichier PID ou base n'est
   créé.

### US2 — Envoyer volontairement un texte commençant par un tiret (P1)

Un utilisateur place `--` avant le corps opaque. Tous les jetons suivants sont
du texte littéral, même s'ils ressemblent à des options.

**Critères d'acceptation** :

1. `send --to X -- --body TEST` sérialise exactement `--body TEST`.
2. `reply -- --body TEST` sérialise exactement `--body TEST`.
3. Les deux commandes conservent le code 0 lorsque le faux daemon accuse
   réception.

### US3 — Nommer une valeur manquante (P1)

Une option connue privée de sa valeur doit être diagnostiquée comme telle,
avant la validation du corps ou la lecture du dernier expéditeur.

**Critères d'acceptation** :

1. `send --to` nomme `--to` et indique qu'une valeur est requise.
2. `send --to X message --from` nomme `--from`, rend le code 2 et n'envoie
   rien.
3. Les garanties déjà acquises pour `--timeout`, `--hops` et les options
   idempotentes restent inchangées.

## Décision sur le texte libre

Deux contrats ont été instruits :

- arrêter l'analyse à la première parole ordinaire préserverait des lignes
  historiques, mais continuerait à avaler une option inconnue placée après
  cette parole ;
- réserver `--` comme frontière explicite permet de refuser tout jeton inconnu
  avant la frontière et de préserver tout texte après elle.

Le second contrat est retenu. Il est non ambigu et n'ajoute aucune option de
corps artificielle.

## Exigences fonctionnelles

- FR-001 : avant `--`, `send` et `reply` refusent avec le code 2 tout jeton
  commençant par un tiret qui n'est pas une option reconnue.
- FR-002 : le refus nomme le jeton brut et précède toute connexion au daemon ou
  lecture de l'état de réponse.
- FR-003 : `--` n'appartient pas au corps et rend tous les jetons suivants
  opaques au parseur.
- FR-004 : une option connue privée de sa valeur nomme cette option et indique
  qu'une valeur est requise.
- FR-005 : les mots ordinaires restent concaténés dans leur ordre avec un
  espace, comme sur la base.
- FR-006 : le message sérialisé et le code de sortie sont tous deux vérifiés.

## Appelants et changement de comportement connu

`--no-reply` n'est ni reconnu ni documenté par la CLI. Deux appels externes
dans `agent_loop.py` l'utilisent pourtant et l'injectent actuellement dans le
corps. La session ne crée pas cette option et ne modifie pas ce fichier hors
dépôt : ces deux appels mal formés seront désormais refusés explicitement.

### Note de migration

Les appelants concernés sont :

- `/home/moi/.codex/skills/agent-loop/scripts/agent_loop.py:1831` ;
- `/home/moi/.codex/skills/agent-loop/scripts/agent_loop.py:2364`.

Aujourd'hui, ils terminent avec succès tout en préfixant silencieusement le
corps par `--no-reply`. Après cette session, ils termineront avec le code 2 et
nommeront `--no-reply`. La réponse n'étant pas demandée par défaut, la réparation
consiste à retirer ce faux drapeau et à protéger le corps dynamique par `--` :

```python
cmd = [str(BRIDGE), "send", "--to", resolved_target, "--hops", "1", "--", msg]
cmd = [str(BRIDGE), "send", "--to", pane, "--hops", "1", "--", message]
```

Cette migration appartient au propriétaire de l'outillage personnel et n'est
pas appliquée dans le dépôt Bridget.

## Hors périmètre

- cohérence entre `--reply`, l'identité humaine et `--timeout` ;
- ajout de `--body` ou `--no-reply` ;
- modification des appelants externes à ce dépôt ;
- grammaires qui transmettent des arguments opaques par contrat ;
- exception fail-soft documentée pour les noms de hooks inconnus.

## Critères de réussite

- La base compte 9 tests dans le harnais d'arguments et reste à 9 réussites.
- Les nouveaux témoins sont rouges avant correction puis verts après.
- Un mutant supprimant uniquement la garde d'option inconnue tue les témoins
  `send` et `reply` qui traversent le vrai binaire.
- Un mutant neutralisant uniquement `--` tue les contrôles du corps littéral.
- Après restauration des mutants, les condensats des fichiers reviennent aux
  valeurs nominales et le harnais complet est vert.

## Hypothèses

- Un jeton commençant par un tiret avant `--` exprime une option et non du
  texte ; l'utilisateur exprime l'intention inverse par `--`.
- Le protocole sérialisé du faux daemon demeure l'oracle de l'effet réellement
  produit.
