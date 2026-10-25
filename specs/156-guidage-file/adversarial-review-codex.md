# Contre-revue adverse - Session156

## Ronde 1 (plan)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/adversarial-review-codex.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/adversarial-review-codex.md#extrait-2).

## Décision de périmètre (Art. XIX)

Les points 5 à 8 et 10 viennent d'une même cause : éditer une file exige une
saga durable (réservation avant émission, accusé durable, réconciliation à la
reprise, phase d'annulation, saga de promotion). Ce coût dépasse le gain :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/adversarial-review-codex.md#extrait-3).

## Statut final du plan

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/adversarial-review-codex.md#extrait-4).

## Ronde 2 (plan réduit)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : objections 3, 4, 8, 11 fermées ; 5, 6, 7, 10 hors périmètre justifié ; 1, 2, 9 partiellement ouvertes ; 4 points restants
- **Cadre de menace** : relu `docs/decisions/011-approbation-exposee-puis-second-facteur.md` (garantie « par impossibilité » face à un agent qui ne peut pas appuyer sur un clavier réel) et `027-etat-de-controle-et-voie-humaine-attestee.md` (borne : « arrête un agent qui agit par ses outils déclarés, pas un processus hostile du même compte »). La garde 156 couvre un agent qui utilise ses outils Bridget. Un processus du même compte qui forge des trames brutes est hors modèle, ce que la spec et l'ADR disent explicitement.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/adversarial-review-codex.md#extrait-5).

Effets : 3 tâches ajoutées (T016, T021, T022), 31 tâches au total. Questions
utilisateur : la question sur la pause devient une hypothèse (ADR 027) ; la
question 3 porte sur les deux changements visibles (guidage implicite des
pairs, CLI nue). Les tests de trames brutes sont maintenus comme défense en
profondeur, sans promesse.

## Statut final du plan

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/adversarial-review-codex.md#extrait-6).
