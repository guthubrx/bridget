# ADR 045 — Contrôle de suivi distinct des notifications

- **Date** : 2026-10-05
- **Statut** : Accepté
- **Session** : 135

## Contexte

Le heartbeat agent-loop traite une tranche d'âge comme un nouvel événement. Il
répète donc les mêmes messages. Il ne prouve pas non plus qu'une mission a été
prise en charge ou qu'elle progresse.

## Décision

Le contrôleur suit des obligations durables. Il enregistre la prise en charge,
le progrès vérifiable, la décision de suite et la clôture. Une machine d'escalade
adresse d'abord le responsable, puis le coordinateur, puis ROOT. Les messages
sont groupés par destinataire. Un état inchangé reste silencieux.

Le verdict métier reste séparé de la décision de suite. Le contrôleur ne change
jamais `review` en `pass`.

## Conséquences

- Positives : moins de bruit, inaction visible, reprise sûre après redémarrage,
  clôture non ambiguë.
- Négatives : les workers doivent utiliser `ack` et `progress`. Les anciens runs
  doivent enregistrer une date d'activation avant d'utiliser le nouveau contrat.
