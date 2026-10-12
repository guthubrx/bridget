# Audit de réutilisation de l’existant — 156-guidage-file

## Decision

Statut: PASS
Date: 2026-10-12, audit frais après155 livrée et arbitrages root
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/specs/156-guidage-file

Conclusion courte : 24/24 items confrontés à l’existant. Aucun doublon non arbitré.
Le plan réutilise les magasins, transports, preuve UI, liens, canon et ACK.
Les ajouts prévus portent une règle ou une lecture absente. Claude seul a le
guidage T3 natif ; Cursor explicite/implicit busy et REST/V1 sont fermés selon
les décisions root. Ce PASS concerne la conception, pas l’implémentation156.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits/audités |24/24|
| Duplications évidentes non arbitrées |0|
| Sources de la lecture fraîche |17|
| Gates avant tasks |5/5|
| Tests156 exécutés par l’auditeur |0|
| Baseline T001 par root |774 PASS/0 FAIL/7 ignored|

## Reutilisations correctement identifiees

Décisions item par item du rapport frais. Les chemins courts de la colonne
preuve ont pour racine absolue /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/ ;
les chemins complets, plages et limites sont dans research.md et son reçu T002.
| Item | Décision | Preuve/raison |
|---|---|---|
| MessageIntent/canon | REUTILISER | message.rs:54 ; daemon.rs:9035/23888 |
| Refus/rejeu | REUTILISER | daemon.rs:8132 ; idempotency_records |
| Remise/reprise/ACK | REUTILISER | daemon.rs:4496/4593/16915 |
| Contrôle actif | REUTILISER | daemon.rs:9429 ; wrapper.rs:5833 |
| Identité/provenance UI/Attach | REUTILISER | daemon.rs:8607/8768/16289/14722 |
| Parent lien et natif | REUTILISER | agent_links.rs:79 ; delegation.rs: 174/193 |
| Pause/AgentGuidance | ETENDRE | referent_control.rs:129 |
| Transports deliver | REUTILISER | codex_app_server.rs:902 ; claude_stream_json.rs:310 ; acp.rs:614 |
| dispatch pont T3 | ETENDRE | t3code_contract_v2.rs:253 ; t3code.rs:1960/3126 |
| GuidanceFacts/authorize/refus/module | CREER JUSTIFIE | Pas garde inter-agent équivalente ; règle pure >=3 puits |
| guidance_events + deux index | CREER JUSTIFIE | control_events fermé pause/droits referent_control.rs:44 ; pas reçu de rejeu |
| record/count/purge audit | CREER dans magasin EXISTANT | run_maintenance daemon.rs:7630 ; aucune couche service |
| send_deliveries source | REUTILISER | idempotency.rs:527 ; phases inchangées |
| index recipient/phase | CREER JUSTIFIE | index opération/clé :541/546 non équivalents |
| queued_deliveries_for_recipient | CREER JUSTIFIE | dispatching_deliveries_for_instance send_delivery.rs:394 mute ; lecture pure distincte |
| QueueListV1/QueueList/Result | CREER JUSTIFIE | patron capability protocol.rs:3992 ; pas lecture file publique |
| NotPermitted contrôle | ETENDRE | résultat contrôle existant daemon.rs:9429 |
| mid_turn_ok remises | ETENDRE compatible | protocol.rs:4228/4309 ; option de projection hors canon |
| CancelDelivery.interrupt_active | ETENDRE | protocol.rs:4322 ; daemon.rs:18764 |
| cancel_queued | CREER JUSTIFIE | ManagedSession existant ; ACP cancel_delivery :511 touche actif |
| MCP intent/queue | ETENDRE/CREER demandé | mcp.rs ; patterns schema/auth/whitelist existants |
| CLI intent/queue | ETENDRE | cli.rs:3496/4358 |
| tests/E2E | CREER/ETENDRE prévus | native_delegation_e2e existant ; aucun scénario de confort |
| docs/ADR156 | ETENDRE/CREER demandé | documentation existante ; ADR du choix nécessaire |


## Existant potentiellement pertinent non mentionne

Aucun existant proche non arbitré. queued_deliveries_for_recipient est une
lecture pure distincte de dispatching_deliveries_for_instance qui mute.
control_events est fermé aux événements de pause/droits ; guidance_events
reste audit/rythme et jamais reçu de rejeu. cancel_queued retire atomiquement
sans le tour actif qu’atteint déjà cancel_delivery. REST/V1 ne sait pas porter
l’intention ; refus local au lieu d’un nouvel adaptateur ou protocole.

## Duplications evidentes

Aucune duplication évidente non arbitré. La garde pure a au moins trois puits
réels à raccorder : SendIdempotent, Send historique, ControlExecution, puis
CancelRequest, remise implicite et QueueList. Elle centralise identité, parenté,
pause et rythme. Les dates LAST/THIRD_LATEST sont dérivées des lectures T006,
pas des états supplémentaires. Aucun service, framework ou dépendance ajouté.

## Memoires et regles applicables

AGENTS, ponts constitution/standards et constitution globale ont été chargés.
XVIII : COUNT O(log n+k) ; QueueList O(log n+s) et jointures indexées, LIMIT200
borne sortie seulement. XIX : garde commune et réemploi avant création.
XX : preuves/limites attribuées et vérification T031 prévue. Les compétences
speckit-analyze/audit-existing sont appliquées dans la portée explicitement
autorisée ; aucune permission ou réinstallation de primitive nécessaire.

## Specs livrees applicables

087/150 pause et entretien ;149 identité/liens ;151 ACK/attente ;152/153 reprise
sous pause/panne et interruption utilisateur terminale ;155 Task et droits
hérités, sans nouveau registre. Les zones pertinentes ont été confrontées,
sans nouvelle lecture globale ou reprise de leurs anciens scores d’audit.

## Journal de recherche

- Lecture source attribuée à audit_existant151 : 17 sources ; égalité des blobs
  base155 59da3968 et HEAD be7957ee attestée dans manifest.json.
- Recherche des producteurs, refus asynchrones, chemins Deliver/Cancel et des
  patrons de stockage ; six axes détaillés dans research-T002.md.
- Lecture du bundle effectivement installé149 : Claude activeSteering=true,
  Cursor=false/interrupt_restart ; aucun modèle lancé. Ancien CommandPolicy absent.
- Contrôle manuel des artefacts et checklist 16/16 ; script et répertoire absents,
  RC127 déjà observé par root, sans second essai ni modification des ponts.

## Arbitrages

Claude seul et fermeture Cursor/V1 ; JSON Deliver aplati + options omises,
aucun champ dans le canon BridgetMessage ; QueueList même garde sans écriture ;
compteurs+dates T006 et garde temporelle exacte ; coût de parcours explicité ;
mission_active reste pour agent malgré152, humain autorisé exempt. Tous sont
validés par root dans les tâches prévues, sans nouveau besoin ou scénario.

### Complément T032 — traçage FR016 des refus avant la garde

Recherche attribuée à ROOT et git154 : record_guidance_refusal est absent.
record_guidance_outcome existe mais demande des faits déjà disponibles après
la garde. Il ne couvre pas seul les refus de préflight ni les faits indisponibles.
Créer un helper privé record_guidance_refusal est donc justifié : quatre puits
réels (préflight SendIdempotent, Send historique, projet Control, projet Cancel),
plus un cinquième appel pour l’échec de lecture des faits. Il réemploie
la source d’identité attestée, guidance_event_labels et record_guidance_event.
Aucune nouvelle table, API publique, type de faits, couche service ou framework.

La fonction ne relit ni guidance_facts, ni taux, ni projet. Elle trace seulement
Steer/Interrupt, avec acteur réel, cible normalisée, ID original et catégorie
fermée refused:<code>. Sans action de contrôle ou pour la pause interne, aucun
événement. Avec faits disponibles, record_guidance_outcome reste le point réutilisé.
Un refus déjà tracé par authorize n’est pas doublé. Aucun nouveau refused après
un admitted réussi ; l’échec fournisseur garde l’admission et son issue existante.
JSON invalide, invalid_intent, envelope_mismatch et reçu rejoué restent exclus.
Une panne du journal conserve le warning et n’autorise aucun effet.

L’oracle existant guidance156_rate_refus_non_compte est étendu dans son fichier
prévu : premiers refus pair/projet/persistance, une trace réelle sans admission,
quota nul, aucune émission ; rejeu projet inchangé sans nouvelle trace. Les
assertions existantes d’admission ultérieure restent. Aucun helper de test ni
nouveau scénario nommé. Le diagnostic et les patches sont préparés en cache ;
aucun résultat RED/GREEN de T032 n’est annoncé par cet arbitrage.

Preuve : /Users/moi/.cache/bridget-delivery156.iuy6stuo/convergence-fix/daemon-fr016-diagnostic.md
SHA-256 : ef92ca242cc9ef57cff126ae3b82eee78bb2c858faf3a7e2d9ac742ae4c13bc1.

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d’audit (24/24)
- [x] Regles projet applicables lues, XVIII/XIX/XX appliqués
- [x] Specs proches livrées vérifiées dans les zones pertinentes
- [x] Plan et divergences explicités (Claude/V1, Deliver, dates/coûts, mission)

## Provenance et limites

Le PASS 14 items de 2026-10-11 était historique. Sa copie est conservée dans
/Users/moi/.cache/bridget-plan156-20261012-r1/candidate-docs/reuse-audit.md.
Le présent gate frais est basé sur :
- /Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/research-T002.md : SHA256 b5f74b863238a31c36b8bd5351b7ad8e1584686c411a6315fcbe4521d77d2324.
- /Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/reuse-audit.md : SHA256 5a184496ba498c5b2e8344dcd48da0cc4046da9ee9a807de7fa5415fbe74af52.
- /Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/manifest.json : SHA256 dac18d38653c204b6631e4eb9f51345b61efa91f830123942b516365342b99d7.

Lecture source ciblée, pas runtime ni audit global. Les grands fichiers n’ont
pas été relus entièrement ; sorties tronquées non comptées comme lecture intégrale.
La baseline et les corrections de conception ne prouvent aucun test d’implémentation.
T002 reste non cochée par cet owner ; root fait sa relecture et son cochage seul.
