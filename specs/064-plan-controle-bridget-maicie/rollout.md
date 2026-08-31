# Activation et retour arrière - SPEC-064

Chaque lot est livré désactivé par défaut. Une activation ne dépend jamais du
worktree : elle exige une release matérialisée, une preuve de version et un
verdict de test du lot concerné.

## Lots et bascules prévues

| Lot | Bascules désactivées par défaut | Gate avant activation | Retour arrière |
|---|---|---|---|
| Fondations | `execution_store_write`, `execution_projection_v2` | migrations additives et tests de reprise | lecture héritée, écritures v2 conservées |
| US1 | `execution_control` | admission, FIFO, non-duplication, erreur bornée | QueueOnly et projection héritée |
| US2 | `agent_graph_v2` | cycle, quota, propriétaire et rollback prouvés | aucun nouveau spawn, liens durables conservés |
| US3 | `native_resume` | capacité/version du fournisseur attestées | fallback `reconstructed` explicite |
| US4 | `maicie_execution_projection` | aucune transition métier déduite du runtime | Maicie masque la projection sans modifier ses missions |
| US5 | `provider_capability_enforcement` | version, empreinte et contrat supportés | refus typé ou fallback annoncé |
| US6 | `execution_budget_enforcement` | réservations atomiques et absence de clôture Maicie | pause durable, jamais clôture implicite |
| Observabilité | `execution_alerts_v2` | métriques bornées et contenu redacted | seuils désactivés, données déjà émises conservées |

## Règles des bascules

- Une seule bascule de comportement est activée par release.
- Aucune bascule ne transforme silencieusement une intention en une autre.
- Les anciennes lectures restent disponibles pendant la double écriture.
- Une version, empreinte ou capacité fournisseur inconnue provoque un refus
  typé, jamais une supposition.
- Les valeurs sont déclarées dans le registre et validées au démarrage.

## Gates d activation

1. Toutes les suites ciblées du lot sont vertes et leur commande est consignée.
2. La baseline fournisseur correspond au binaire qui sera exécuté.
3. La migration est additive, idempotente et sa reprise après crash est prouvée.
4. Les journaux, métriques et projections n exposent aucun corps de message.

## Retour arrière

- Les tables et événements nouveaux ne sont jamais supprimés par un rollback.
- Le rollback rétablit la lecture de compatibilité, jamais un ancien état en
  écrasant une exécution nouvelle.
- Toute remise déjà admise conserve son identifiant d idempotence.
- Une opération fournisseur non supportée devient une issue structurée ou un
  fallback annoncé.

## Suppressions de compatibilité

- Une bascule ne peut être retirée que si ses consommateurs mesurés sont nuls.
- Une projection héritée est retirée après au moins une release sans lecteur
  recensé et après une migration de lecture documentée.
- La date et la preuve de chaque retrait sont ajoutées ici par T085; aucune
  suppression n est autorisée avant cette preuve.

## Etat de cloture T085 - 2026-08-29

Aucune bascule de cette specification nest activee dans un service. Aucun retrait
physique nest donc admis dans ce worktree : il supprimerait une compatibilite
sans mesure de lecteur de release.

| Compatibilite | Etat dans cette livraison | Date ou condition de retrait |
|---|---|---|
| `execution_store_write` | desactivee par defaut | Revue le 2026-09-12 apres une release avec inventaire des lecteurs |
| `execution_projection_v2` | desactivee par defaut | Revue le 2026-09-12 apres une release avec inventaire des lecteurs |
| Projections heritees | conservees, aucun consommateur mesure ici | Retrait seulement apres une release complete sans lecteur recense |

La prochaine revue verifie la version effectivement publiee, les consommateurs
mesures et les preuves de reprise. Sans ces trois elements, aucune bascule ni
suppression ne sera autorisee.
