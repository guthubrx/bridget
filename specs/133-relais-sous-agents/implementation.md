# Journal d'implémentation 133

Statut : Completed — 12/12.
Date : 2026-10-04.
Worktree : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/133-relais-sous-agents`.

## T001 — Baseline ciblée

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/implementation.md#extrait-1).

Résultats avant modification du code :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/implementation.md#extrait-2).

Limite : le filtre `prompt_for` de `bridget-transport` n'a sélectionné aucun
test existant. Les nouveaux tests SPEC-133 devront donc nommer explicitement ce
comportement.

### Self-review Article XIX/XX

- Pourquoi cette solution est nécessaire : figer les garanties avant changement.
- Solution la plus simple : tests existants ciblés, sans nouveau harnais.
- Hypothèses : les suites filtrées représentent les frontières modifiées.
- Vérifications : compilations et 46 tests ciblés verts.
- Non vérifié : suite workspace complète, réservée à T012.
- Code supprimé ou évité : aucun script de baseline ajouté.
- Complexité ajoutée : documentation seulement.

## Analyse SpecKit

Analyse lecture seule : 18 exigences et critères, 12 tâches, couverture 100 %.
Aucune ambiguïté, duplication, tâche orpheline ou violation constitutionnelle.

## T002–T003 — Provenance et rejeu

Les tests ont d'abord échoué à la compilation : `DelegatedOrigin` et le champ
`delegated_origin` n'existaient pas. Après ajout, les deux tests SPEC-133 passent.
Le sous-objet refuse les champs inconnus. Un message historique se réencode sans
provenance. La provenance ne modifie pas les bytes de rejeu du parent.

### Self-review Article XIX/XX

- Pourquoi cette solution est nécessaire : rendre l'origine enfant visible sans créer d'identité.
- Solution la plus simple : un sous-objet optionnel dans l'enveloppe existante.
- Hypothèses : fournisseur et référence sont validés avant construction.
- Vérifications : test RED observé, puis 2 tests ciblés verts.
- Non vérifié : rendu fournisseur et validation de la preuve, tâches suivantes.
- Code supprimé ou évité : aucun nouveau canon, identifiant ou service.
- Complexité ajoutée : deux chaînes optionnelles, schéma fermé.

## T004–T007 — Relais MCP et provenance visible

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/implementation.md#extrait-3).

Les tests couvrent le rattachement nominal, la réponse vers le parent, le rendu
des trois transports et la compatibilité des anciens messages.

## T008–T009 — Autorité minimale

Le répartiteur MCP refuse tout outil autre que `bridget_who` et `bridget_send`
avant l'appel du handler. La ligne de commande applique une garde centrale avant
les commandes sensibles. Un contexte enfant ne peut donc pas obtenir les droits
du parent par une autre façade.

La matrice teste l'inventaire MCP complet. Le garde-fou CLI inventorie les
commandes sensibles afin qu'une future commande ne puisse pas éviter le refus.

## T010–T011 — Preuves fermées et incident réel adaptateur de bureau

Les preuves enfant sont privées, bornées, non symboliques, liées au PID et à sa
naissance, puis retirées seulement par leur propriétaire. Une preuve enfant
présente mais invalide arrête la résolution. Elle ne déclenche jamais un repli
vers un marqueur principal plus haut.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/implementation.md#extrait-4).

Preuves ciblées :

- 11 tests `spec133_` du daemon : PASS ;
- 2 tests `spec133_` du transport : PASS ;
- 1 test `spec133_` du cœur : PASS ;
- inventaire `lsof` avec 192 fichiers de session : PASS en moins de trois secondes ;
- marqueur invalide, lien symbolique, dépassement, PID recyclé et propriété : PASS ;
- matrice MCP et garde-fou CLI : PASS.

### Self-review Article XIX/XX

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/implementation.md#extrait-5).

## T012 — Validation finale et convergence

- `cargo fmt --all -- --check` : PASS ;
- `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` : PASS ;
- `cargo build --offline --locked --release` : PASS ;
- daemon en série : 1 000 PASS, 10 ignorés ;
- transport final : 293 PASS, 1 ignoré ;
- cœur : 41 PASS ;
- tests SPEC-133 : 14 PASS ;
- `git diff --check` : PASS.

La première passe workspace a détecté deux régressions dans l'en-tête historique
Codex. Elles ont été corrigées. Le transport complet est vert après correction.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/implementation.md#extrait-6).

La convergence finale couvre FR-13301 à FR-13313 et SC-13301 à SC-13307. La
contre-revue locale finale est `adversarial-review-final.md`. Aucun défaut ouvert
de sévérité haute ou critique n'est retenu.
