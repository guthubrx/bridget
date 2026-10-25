# ADR035 — Rattachement privé et remises bornées

2026-09-16 · Accepté pour implémentation · Session099

## Contexte

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/035-rattachement-prive-et-remises-bornees.md#extrait-1).

## Décision

Un propriétaire reçoit une preuve aléatoire liée à sa connexion et son instance.
Une unique attestation auxiliaire réutilise les maps et contrôles existants pour
MCP, CLI et clients idempotents. La preuve est renouvelée à chaque incarnation ;
les rattachements déjà ouverts sont révoqués avec le propriétaire. Elle est
conservée par le wrapper dans son espace privé, pas dans l'annuaire ni les logs.

Les écritures classiques utilisent la primitive bornée existante, hors verrou
global. Le suivi est préparé avant exposition au destinataire ; la clôture d'une
réponse reste après la remise réussie. Un échec n'est pas un succès fictif.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/035-rattachement-prive-et-remises-bornees.md#extrait-2).

## Alternatives écartées

- UUID ou issuer_scope secret : identifiant et autorisation ne sont pas équivalents.
- PID pair seul : avec SSH il désigne le tunnel, pas l'agent distant.
- Serveur d'authentification externe : disproportionné pour un socket privé.
- Réécriture asynchrone du daemon : inutile pour corriger le verrou identifié.
- Nouveau stockage outbox : l'état durable par fil existe déjà.

## Conséquences

Positives : mêmes droits contrôlés sur chaque voie, indépendance des agents lents,
réponses récupérables, état et tests inspectables sans nouveau framework.

Coûts : mise à jour conjointe des clients auxiliaires et wrappers, champs d'état
supplémentaires compatibles avec les anciens fichiers du pont, tests de révocation.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/035-rattachement-prive-et-remises-bornees.md#extrait-3).

## Amendement du 2026-09-16 — humain et annuaire

La recette complète de la session 099 a montré deux effets non voulus de la
règle « aucune attribution sans preuve », corrigés sans l'affaiblir :

- **L'étiquette humaine n'est pas une identité d'agent.** Un client négocié qui
  ne détient aucune identité (le clavier, le harnais 097) parle sous l'étiquette
  humaine sur le chemin idempotent, comme il le faisait déjà sur le chemin de
  contrôle. Un client porteur d'une identité d'agent ne peut pas se dire humain
  pour contourner la preuve. Côté CLI, un humain sans identité n'a rien à prouver
  et ne passe plus par le rattachement auxiliaire.
- **L'annuaire est public.** `ListAgents`, donc `bridget who` et l'outil MCP
  `bridget_who`, se lit sans preuve ni enregistrement : c'est la projection
  publique du daemon. Les outils qui écrivent ou lisent une portée privée
  (envoi, annulation, ledger, renommage) restent attestés.

Conséquences : les fixtures des tests 089 (sécurité, isolation) et de parité sont
alignées sur l'ordre réel des trames (attestation avant négociation) et sur la
remise asynchrone par destinataire ; un tour métier en trop a été observé une
fois sous charge dans la matrice de parité, sans reproduction à vide, et reste
une piste à surveiller (diagnostic inscrit dans le test).
