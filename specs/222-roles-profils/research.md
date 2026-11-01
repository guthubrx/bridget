# Recherche222 — décisions et limites
Date : 2026-11-01. Source inspectée : e88d1db46242de93142708fa3396293141ee8e92.

## Décisions
- Étendre le domaine AgentProfileStore, conserver FleetSupervisor et ExecutionStore.
  Le budget facultatif existe déjà : pas de second évaluateur ni état de quota.
- Une désignation officielle et un mandat de réveil sont des actes humains
  versionnés. Les références du contrôle humain existant sont la preuve ;
  un agent n'a pas de champ de confiance qu'il pourrait s'accorder.
- Une demande de rôle ne transmet pas de droits au demandeur. Le titulaire
  exerce son propre mandat approuvé, via la même frontière de lancement.
- Séparer personne logique/session/provider : la définition et l'expérience
  restent utiles après arrêt d'un processus ou changement de moteur.
- Mémoire sourcée, rappel explicable sans service externe. Absence de borne
  globale de conservation ; fenêtre de réponse bornée et lacunes visibles.
- Clients existants Android/MCP/CLI ; aucun nouvel orchestrateur ni client web
  remplaçant les travaux221. Les interfaces futures consommeront le même contrat.

## Alternatives écartées
Rôles encodés uniquement dans domain : une seule chaîne et pas de portée/autorité.
Étiquettes seules : absence de mandat, de révision et de désignation.
Nouvelle file de tâches : doublon de work_submissions/executions/send_deliveries.
Prompt = droit : non recevable ; la politique native reste vérifiée.
Résumé écrasant l'historique : contredit la conservation décidée.
Lecture intégrale de tout historique à chaque mission : coût et pertinence inutiles.
Import activant automatiquement un service : contredit l'autorisation humaine.

## Vérifications à mener par contrôle ciblé
Les points ci-dessous sont des obligations d'implémentation, pas des capacités
déclarées déjà acquises : propagation du mandat jusque la frontière native,
arrêt/reprise entre versions, capture d'expérience sans texte privé non autorisé,
conversion paginée, rappel/oubli interprojets, conflit de version et import neutre.
Un échec de preuve ne doit pas être contourné par un nouveau fournisseur.

## Sources et nature des preuves
- Code local : agent_profile.rs, fleet.rs, lifecycle.rs, execution_store.rs,
  daemon/native_delegation.rs, daemon/mobile_admin.rs, mobile/http.rs.
- Étude de mécanismes, pas dépendance : AutoSwarm commit
  03494ac03d51287298240b5962c88c07cfbeb260, modules talent_market/recruiter
  de l'archive et contrat actuel de communication. La version archivée ne
  démontre pas une intégration de flotte utilisable ici.
- Anthropic, Building effective agents :
  https://www.anthropic.com/research/building-effective-agents
  Simplicité et composition ; pas de promesse de gain universel.
- OWASP Authorization Cheat Sheet :
  https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html
- NIST least privilege :
  https://csrc.nist.gov/glossary/term/least_privilege
- Recherche sur provenance de mémoire :
  https://arxiv.org/abs/2608.29606
  Source de problématique, pas validation expérimentale de notre implémentation.

## Outils et limites de préflight
Sync utilisateur exécuté. Scripts/templates SpecKit projet absents, compétences
utilisateur lues ; exécution directe de leur protocole. Sequential Thinking
non exposé au catalogue, décisions écrites avant code.
Premier annuaire refusé primary_not_live ; après une réattestation, annuaire
fonctionnel. Incident transmis à bridge1016 sous clé
bridget222-demarrage-et-incident-lien-20261101-02 ; aucune route de remplacement.

## Forme réelle des journaux de la conversation
Lecture privée ciblée du journal de ce fil, sans sortie des textes :638événements. Forme v/seq/ts/session_id/event/payload ; turn_start, turn_end, update.kind=text ou tool. La collecte conserve les textes de conversation, pas les sorties de commandes ni les métadonnées de transport. Les sources sont citées par séquence et empreinte ; les journaux existants restent leur autorité. Le curseur avance aussi sur les événements ignorés ; toute la page/son reçu/son observation est atomique.
