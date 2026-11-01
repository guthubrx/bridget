# Données222 — invariants
- DefinitionRevision : id, kind(profile/team), revision, contenu fermé, digest,
  créateur, provenance, date. Versions immuables ; pas de secret ni de droit transporté.
- Specialist : id logique, definition_id, active_revision, owner, scope_projects,
  durable, auto_start, mandate, budget facultatif, revision, timestamps.
- RoleClaim : role_id, titulaire, portée, déclaré/par, statut officiel, révision ;
  présence dérivée du runtime seulement. Désignation humaine distincte.
- MemoryEntry : id, spécialiste, projet/source, portée, type(fact/decision/hypothesis/
  reported_result), texte, source_ref, révision, forgotten, date. Correction
  traçable, source refusée invisible, réintroduction d'une source oubliée interdite.
- SpecialistRequest : propriétaire+clé unique, canon/digest, responsable figé,
  version du profil et référence native(submission/execution/task/delivery).
  Le statut s'obtient des magasins natifs, jamais d'un second état métier.
- Liaisons de session : projection des provider_bindings et du roster, aucune table nouvelle.

L'admission humaine conserve la preuve d'activation et sa révision. Les mutations
exigent expected_revision. Les listes sont paginées ; les bornes de réponse sont
distinctes d'une durée de conservation ou d'un budget.
Migration additive, aucun réétiquetage automatique de données anciennes.

## Précisions après revue
Specialist.id = agent_profiles.agent_id ; profile_id est la définition réutilisable.
MandateGrant et réservation atomique : contrats détaillés dans contracts/closure-after-review.md.
Oubli conservé comme tombstone par source, pas de réinsertion automatique.
