# Journal d'implémentation: SPEC-067

**Statut**: implementation terminee, validation de regression preexistante documentee
**Branche**: `session-067-profils-extensions-secrets-projet`
**Base**: `dda4ec2198b941cc38915a00f847df435d80934d`
**Derniere mise a jour**: 2026-08-31

Toutes les ressources de test sont synthétiques. Aucun credential réel, appel
fournisseur payant ou déploiement n'est admis par cette session.

## Journal des tâches

### T001 - Scénarios métier Gherkin

- Statut: complétée.
- Preuve: `git diff --check` passe et les six scénarios couvrent les quatre user stories.
- Décision: un seul fichier déclaratif, sans runner décoratif ni dépendance.

### T002 - Fixtures extensions et secrets

- Statut: complétée.
- Preuve: `git diff --check` passe; trois sentinelles synthétiques ont le mode `0600`.
- Décision: données de test seulement, aucun credential ni mécanisme runtime.

### T003 - Rejeu de l'audit de réutilisation

- Statut: complétée.
- Preuve: 18 items audités sur `dda4ec2198b941cc38915a00f847df435d80934d`, zéro doublon évident.
- Décision: étendre profils, registre, environnement et runtime existants; aucun broker ni second registre.

### T004 - Profil projet et invalidation runtime

- Statut: complétée.
- Preuve: `$HOME/.cargo/bin/cargo test -p maicie spec_067_profil_devient_stale_si_runtime_diverge --no-fail-fast` passe, 1 test.
- Self-review Article XIX/XX: un petit type métier réutilise `DomainError` et les profils agents existants; aucune donnée secrète, dépendance ou registre parallèle.

### T005 - Contrat proposition/résolution

- Statut: complétée.
- Preuve: `$HOME/.cargo/bin/cargo test -p bridget-transport spec_067_proposition_ne_porte_pas_de_revision_source --no-fail-fast` passe, 1 test.
- Self-review Article XIX/XX: les références publiques restent opaques; la révision et l'attestation n'existent que dans la forme résolue, sans chemin ni valeur.

## T006 a T038 - Livraison SPEC-067

- Statut: terminees.
- Fonctions: profil projet durable, resolution hote des definitions agents et ressources, approbation humaine locale, montage Docker read-only, process-env apres admission, redaction streaming et invalidation sur rotation ou divergence.
- Preuves: voir evidence/us1-approval.md, evidence/us2-extensions.md, evidence/us3-secrets.md, evidence/us4-providers.md et evidence/security-review.md.
- Validation: fmt et Clippy passent. La commande cargo test --workspace --quiet atteint quatre echecs managed_parity_test, reproduits sans modification SPEC-067 sur dda4ec2198b941cc38915a00f847df435d80934d. Le detail est dans evidence/final-validation.md.
- Analyse: aucun SpecKit global n a ete cree ou modifie.
