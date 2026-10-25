# Audit de réutilisation de l’existant — 156-guidage-file

## Decision

Statut: PASS
Date: 2026-10-12, audit frais après155 livrée et arbitrages root
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/specs/156-guidage-file

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/reuse-audit.md#extrait-1).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/reuse-audit.md#extrait-2).


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
