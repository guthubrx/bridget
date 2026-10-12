# Contre-revue du code 159B — Claude R1/R2

Date : 12 octobre 2026. Fournisseur : Claude ; modèle des reçus :
`claude-haiku-5-5`. Les deux missions sont terminales `completed`, sans enfant
encore en attente. Les champs `latestTerminalSummary` ont été lus entièrement.
Cette trace ne lance aucun test et ne modifie ni code ni worktree.

## Question et verdicts

Question couverte, reformulation du mandat : les opérations US4/US5/US6
respectent-elles les droits frais, la transaction commune, le rejeu et les
garanties réduites après promotion ? La seconde passe vérifie la résolution
du MEDIUM de R1, sans étendre la revue au projet entier.

| Passe | Mission | Verdict | Portée |
|---|---|---|---|
| R1 | `159B-code-review-20261012-r1` | APPROVE_WITH_CHANGES | US4/US5/US6 partiellement relues ; 0 C/H confirmé, un MEDIUM plausible et une LOW. |
| R2 | `159B-code-review-20261012-r2` | APPROVE | US4/US5/US6 ciblées ; MEDIUM documentaire clos, LOW conservée explicitement. |

## Objections et réponses ROOT

| Point | Objection | Réponse et état final |
|---|---|---|
| MEDIUM R1, rejeu après purge | L’envoi d’origine peut disparaître alors que son reçu reste sept jours. L’auteur n’est plus vérifiable ; le refus frais peut remplacer l’accès au résultat historique. La promesse écrite de rejeu était trop absolue. | ROOT conserve identité et droits avant reçu. La proposition d’utiliser `queue_edits.actor` comme preuve durable d’autorité est écartée : ce champ reste une trace, pas un grant. Le contrat précise les trois cas de rejeu sous « droits frais admis ». Si l’envoi est purgé ou `message_bytes` absent, un auteur non vérifiable peut recevoir `intent_not_permitted` avant le reçu. Aucun droit neuf, code ni test supplémentaire. R2 confirme la cohérence de cette clause avec le code et clôt le MEDIUM. |
| LOW R1/R2, JSON corrompu | `unique_held_public_id` utilise `json_extract` sans `json_valid` ; une ligne Held corrompue peut faire échouer l’édition de ce destinataire. | ROOT conserve cette limite visible de stockage : `storage_unavailable`, aucun effet ni gain de droit. Les octets viennent du daemon ; aucune voie d’abus n’est démontrée. L’inaccessibilité absolue n’est pas affirmée. Pas de correction de confort ; LOW active acceptée. |
| Écart 54/60 entrées | R1 annonçait 54 fichiers modifiés ; R2 voyait 60 entrées sans en vérifier tout le contenu. | La capture T027 distingue 54 fichiers suivis et six nouveaux, soit 60 fichiers et 197 hunks. Cette preuve de lecture est distincte des deux revues Claude ; elle ne leur est pas attribuée. |

La décision protège aussi les autres acteurs : parent ou humain doivent toujours
avoir leur propre preuve fraîche. Un reçu ne contourne jamais l’ordre
identité → droits → reçu. Un rejeu autorisé ne réémet ni effet ni notice fresh.

## Lecture effective et limites

R1 lit les trois mutations du Store, `queue_edit_once`, des fragments de
libération/clôture, les gardes et le handler daemon, la garde156 et le routage
Codex. Elle ne lit pas les tests/E2E, CLI/MCP, Claude/ACP, contrats/modèle,
les 26 fichiers Register mécaniques ni le reste du diff. Elle ne calcule pas
d’empreintes fraîches et ne prouve aucune course runtime.

R2 lit la clause du contrat, l’ordre des gardes du handler et la garde d’auteur.
Elle calcule quatre SHA, dont daemon650ca, Store2ee9, contratb6e1 et
référencec281. Elle vérifie seulement l’empreinte de la référence, pas son texte.
Elle ne relit pas le Store/`queue_edit_once` en entier. Elle ne vérifie pas la
durée de rétention des envois, les interfaces, les E2E, T023 ou le FULL.
Ces SHA décrivent son checkpoint ; les métadonnées documentaires ultérieures
ne deviennent pas implicitement relues par Claude.

La self-review T027 attribuée à `audit_existant151` lit indépendamment
197/197 hunks de 60 fichiers, dont six nouveaux fichiers entiers dans le diff.
Elle inclut les interfaces, tests et transports laissés hors revue Claude.
Elle ne prétend pas lire tous les corps hérités des 44 fichiers Rust. Ni cette
lecture ni APPROVE ne constituent un PASS runtime ou un GREEN global.

## Reçus immuables et provenance

R1 :
/Users/moi/.cache/bridget-analysis159B-20261012/ROOT-review159B-code-r1-terminal.json
SHA256 ed4cfff581bf32c70929a516e302c97e80feba6ac194e8622b9c6e5c39c0ba90

R2 :
/Users/moi/.cache/bridget-analysis159B-20261012/ROOT-review159B-code-r2-terminal.json
SHA256 04de006f542898c5960bdd670a776b02c3ffb6d63b516df04855f097d6a73d2e

Self-review distincte :
/Users/moi/.cache/bridget-selfreview159B-final-20261012/read-manifest.json
SHA256 5765b490d87077db8022a55d9792e9bd5052232ebe28e095b6ff7be179415126
/Users/moi/.cache/bridget-selfreview159B-final-20261012/hunk-coverage.json
SHA256 e631748d062d3afcdb55bb1963a198cafa26d809e3115f52af71a02e793a14c8
