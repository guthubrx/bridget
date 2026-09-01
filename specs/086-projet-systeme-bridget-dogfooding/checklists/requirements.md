# Checklist qualité - SPEC-086

## Complétude fonctionnelle

- [x] Unicité et attestation du projet système sont définies.
- [x] Activation, désactivation et transition Docker sont définies.
- [x] Les droits communs à tous les agents du projet sont explicites.
- [x] La coexistence avec des worktrees externes est couverte.
- [x] Édition et livraison système sont strictement séparées.
- [x] Le checkout principal reste read-only dans tous les modes.
- [x] Le worktree writable est non principal et hors branche main.

## Sécurité et responsabilité

- [x] Le mode est désactivé par défaut.
- [x] Aucun projet standard ne reçoit le checkout système.
- [x] Aucun merge, push, install, restart ou deploy automatique n'est permis.
- [x] Aucun rôle ou service de secrets supplémentaire n'est introduit.
- [x] L'absence d'isolation entre agents du projet système est explicitement documentée.

## Bornage

- [x] Un seul projet système par installation.
- [x] Pas de permission par agent, mission ou durée.
- [x] Maicie reste incluse dans le checkout Bridget.
- [x] Git et les worktrees restent le mécanisme de concurrence.

## Verdict

PASS - La spécification peut entrer en planification sans clarification bloquante.
