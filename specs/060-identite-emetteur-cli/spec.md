# Session 060 — Identité stable et adressable pour l’émetteur en ligne de commande

## Métadonnées

- **Statut** : livré, non intégré
- **Branche** : `session-060-identite-emetteur-cli`
- **Base gelée** : `70ef619` (`origin/main` relevé après `fetch`, 2026-08-28 15:35:54 +0000)
- **Objectif** : `50060f62-5097-4890-9899-80bcd962024a`
- **Délégation** : `07f57a95-b616-439a-9f18-844e4e89f8f8`
- **Message** : `a1d49848-1126-42c6-8eb8-c2723509f359`
- **Priorité humaine** : `un-agent-peut-emettre-sans-exister-a-l-annuaire`, inscrite le 28/08 à 08h30, quatrième manifestation.

## Problème mesuré

Mesure du référent à 15h38 : 701 messages portent un expéditeur dérivé du PID,
sous 701 identités distinctes, dont 8 dans la dernière heure. Le destinataire ne
peut répondre à aucune.

Deux sondes réelles ont été tirées avant toute modification, depuis
`/home/moi/revue/rc7/bridget`, et lues dans `bridget ledger` :

| sonde | commande | expéditeur inscrit | verdict |
|-------|----------|--------------------|---------|
| T1 | `send --from priorites-humain --to rc7-flux` | `cli-send-3675366` | nom déclaré **écrasé**, sortie `OK`, code 0 |
| T2 | `send --from humain --to rc7-flux` | `humain` | nom déclaré **conservé**, adressable |

Le diagnostic initial — « `bridget send` ne sait pas se nommer » — est donc
inexact. `--from` existe déjà (`cli.rs`) et fonctionne quand le nom est celui
d’un agent connecté (T2). Deux défauts réels le masquaient :

1. **L’écrasement est silencieux.** `daemon.rs` remplaçait un `from` inconnu par
   le nom de connexion `cli-send-<pid>` en rendant `OK` et le code 0.
   L’émetteur croit s’être nommé ; le destinataire reçoit un inconnu jetable.
   C’est la fabrique des 701 identités.
2. **`--from` n’était pas documenté** dans l’aide de `send`. L’option
   existait sans être découvrable, d’où l’essai de `--as`, refusé — le refus
   mesuré portait sur le nom de l’option, non sur la capacité.

## Propriété

Tout émetteur, y compris en ligne de commande, doit pouvoir déclarer une
identité stable et **adressable en retour**.

Une identité n’est adressable que si un agent connecté la porte : le routeur
(`router.rs::resolve`) rejette tout destinataire absent de sa table. Déclarer
un nom que personne ne porte ne peut donc pas produire une adresse de retour ;
la seule réponse honnête est le refus, et non une réattribution muette.

- `send --from <nom>` où `<nom>` est adressable : le nom est conservé, et le
  destinataire peut répondre à cette adresse.
- `send --from <nom>` où `<nom>` n’est adressable par personne : l’envoi est
  **refusé**, avec un motif qui nomme la cause et la sortie.
- `send` sans `--from` : comportement inchangé. Le mandat ne demandait pas de
  changer le défaut, et il ne l’est pas.
- Un wrapper conserve son nom enregistré même s’il passe `--from` : la garde
  anti-usurpation existante est préservée.

## Portée

Hors périmètre, conformément au mandat : les 701 messages existants ne sont pas
renommés, et le comportement par défaut n’est pas modifié.
