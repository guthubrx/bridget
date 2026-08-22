# ADR 008 — Serveur MCP comme façade du daemon Bridget

**Statut : Accepté — 2026-08-22**

## Contexte

Les agents Bridget utilisent déjà deux voies distinctes : le transport livre
les messages entrants, tandis que le daemon applique l'annuaire, les gardes et
le ledger. La voie montante restait une commande shell, donc dépendante du
prompt, fragile pour les corps riches et contrainte de parser stderr.

Le Model Context Protocol (MCP) permet d'exposer des outils typés aux harness
sans modifier la livraison existante. Le serveur doit néanmoins parler à deux
mondes qui n'ont pas les mêmes garanties : stdin/stdout JSON-RPC avec le
harness, socket Unix avec le daemon. Toute écriture parasite sur stdout
corromprait le protocole MCP.

## Décision

Bridget fournit la sous-commande `bridget mcp`, un serveur MCP local en stdio
qui agit comme une façade mince du protocole daemon.

- Le serveur expose exactement trois outils : `bridget_send`, `bridget_who` et
  `bridget_ledger`.
- Il ne possède aucun état métier : chaque appel d'outil contacte le daemon
  par son protocole existant. Les gardes, issues et écritures restent donc sous
  l'autorité du daemon (D-401).
- Le protocole MCP est traité par un lecteur stdin unique, une corrélation
  JSON-RPC qui accepte des ids chaîne ou nombre, et un writer stdout sérialisé.
  Les travaux d'outil peuvent se terminer hors ordre sans entrelacer les
  frames.
- `initialize` annonce uniquement la capacité `tools` à la version MCP
  pinnée. `tools/list` est stable et rappelable ; les erreurs de protocole
  restent JSON-RPC, alors que les refus Bridget sont des résultats métier
  structurés.
- stdout est réservé exclusivement aux frames JSON-RPC. Les diagnostics vont
  sur stderr ; cette pureté est un invariant testé.
- Aucune dépendance MCP n'est ajoutée : le serveur réutilise les primitives
  publiques du workspace sans coupler les machines d'état ACP et MCP (D-405).

## Conséquences

### Positives

- Les harness disposent d'outils structurés, sans échappement shell ni parsing
  de texte humain.
- Les outils et la voie CLI appliquent les mêmes règles daemon et produisent le
  même ledger.
- Le canal MCP reste lisible et débogable : une ligne stdout est toujours une
  frame JSON-RPC complète.

### Négatives

- Le serveur doit maintenir deux protocoles et borner explicitement ses
  lectures, écritures et appels concurrents.
- Une indisponibilité du daemon devient une erreur d'outil explicite : aucune
  file locale ni relance implicite ne masque l'état réel.
- Les capacités MCP hors outils ne sont pas annoncées ; leur ajout exigera une
  décision et une matrice de conformité dédiées.
