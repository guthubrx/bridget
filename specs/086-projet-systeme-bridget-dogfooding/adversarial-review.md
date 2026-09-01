# Revue adversariale interne - SPEC-086

## Verdict après corrections

PASS WITH EXPLICIT TRUST - Aucun blocker restant, mais le mode expert est un domaine de confiance partagé et doit être présenté ainsi.

## Findings

### HIGH-086-01 - Checkout principal writable

Le premier plan rendait sources et worktrees writable en mode enabled. Cela permettrait de modifier directement le checkout de référence. Correction: checkout principal toujours read-only; seul un worktree attribué, non principal et hors `main`, devient writable. FR-08611 et FR-08629 ont été corrigées/ajoutées.

### HIGH-086-02 - Fausse isolation entre agents

Un conteneur partagé ne protège pas les agents du même projet les uns des autres. Les commits nécessitent en plus un git common dir writable. Correction: FR-08627 et FR-08628 imposent un avertissement honnête et définissent un domaine de confiance coopératif.

### HIGH-086-03 - Livraison confondue avec édition

Couvert par l'exclusion permanente de merge main, push, install, restart et deploy automatiques. Le workflow hôte reste séparé.

### MEDIUM-086-04 - Course sur un worktree

Couvert par une lease canonique de worktree et le refus du même worktree pour deux travaux actifs.

### MEDIUM-086-05 - Projet système arbitraire

Couvert par chemin configuré, attestation structurelle sans exécution, rôle protégé et unicité serveur.

## Points solides

- Un seul projet système et un seul booléen expert.
- Aucun RBAC par agent ou temporisation.
- Docker obligatoire pour la frontière de montage.
- Coexistence avec les agents externes via worktrees distincts.

