# Amont 015 épinglé — G-1600

**Gate** : G-1600 ouvert le 2026-08-24 après le merge
`0b2b83e807c6055c0014d5fa21b0efbfe8fd6311` dans `main`.

La session 015 est la seule autorité de transport pour le point de départ de
la session 016. Son contrat public est gelé et son parcours réel G1504 est
livré. La présente épingle interdit à la session 016 de réinterpréter ou de
remplacer ses trames avant une extension versionnée explicitement revue.

## Sources épinglées

| Élément | Source 015 au merge | Copie 016 | Empreinte SHA-256 |
|---|---|---|---|
| Contrat public v1 | `specs/015-guichet-maicie/contracts/protocole-guichet.md` | référence seulement | contrat gelé au merge `0b2b83e` |
| Négociation service v1 | `specs/015-guichet-maicie/contracts/fixtures/service-negotiation-v1.jsonl` | `specs/016-coordination-active/contracts/fixtures/service-negotiation-v1.jsonl` | `494bc21a9eacb47296ab9ef0b22d6b830060c77b6d5c8cbc5ef0bb7534244303` |
| Corpus filaire normatif v1 (7 trames) | §5.1 de `protocole-guichet.md` | `specs/016-coordination-active/contracts/fixtures/service-frames-v1.jsonl` | `42e0ea32690e267221804a8af9aee538162f7c141a9cffacf6d6a44a9a8cda1f` |

La copie est volontairement octet pour octet : les lignes JSONL, leur ordre et
leur saut de ligne final font partie de la fixture. Le second corpus
matérialise les sept lignes normatives de la section 5.1 du contrat dans
l'objet Git épinglé : les trois opérations de service,
`guichet_claim_next` et les trois événements de cycle. Le vérificateur les
extrait de cet objet, jamais du Markdown courant. `Gap` et `Unavailable` ne
sont pas des trames gelées de l'amont 015 : les oracles 016 pourront les
préparer, sans les présenter comme des octets relus de 015.

## Invariants consommables par 016

- La connexion de service négocie `RoleHandshake(service)`, puis
  `ServiceHello` / `ServiceWelcome` v1 avec la capacité
  `maicie_guichet` ; cette capacité est une borne coopérative, pas une identité
  opposable.
- Les événements de cycle déjà attestés par Bridget sont uniquement
  `answered`, `cancelled` et `timed_out`, avec un `event_id` durable et des
  octets rejouables.
- `reminder_sent` est absent du contrat et du code 015. Il ne peut donc être
  ni inféré d'un texte, ni dérivé du ledger : T1603 devra en faire une
  extension 016 versionnée, ou n'en faire aucune.
- Maicie ne lit jamais la base Bridget directement. Toute relève future doit
  rester bornée et redonner les mêmes bytes et le même `event_id` après crash.

## Vérification mécanique

Exécuter depuis la racine du dépôt :

```bash
specs/016-coordination-active/contracts/verify-amont-015.sh
```

Le script échoue si le merge 015 n'est plus ancêtre, si le contrat gelé n'est
plus celui annoncé ou si un octet d'une fixture diverge. Son auto-test de
mutation est exécutable avec
`specs/016-coordination-active/contracts/verify-amont-015.sh --self-test`.
