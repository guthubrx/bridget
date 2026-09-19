# ADR035 — Rattachement privé et remises bornées

2026-09-16 · Accepté pour implémentation · Session099

## Contexte

L'accès au socket privé donnait accès à trois chemins d'attribution insuffisante :
inscription MCP par identifiants déclarés, compatibilité CLI et SendIdempotent
dont issuer_scope est une clé de rejeu. Les preuves reproduites montrent aussi
qu'une sortie bloquée immobilise l'état global et qu'une réponse t3code peut être
oubliée sans confirmation.

## Décision

Un propriétaire reçoit une preuve aléatoire liée à sa connexion et son instance.
Une unique attestation auxiliaire réutilise les maps et contrôles existants pour
MCP, CLI et clients idempotents. La preuve est renouvelée à chaque incarnation ;
les rattachements déjà ouverts sont révoqués avec le propriétaire. Elle est
conservée par le wrapper dans son espace privé, pas dans l'annuaire ni les logs.

Les écritures classiques utilisent la primitive bornée existante, hors verrou
global. Le suivi est préparé avant exposition au destinataire ; la clôture d'une
réponse reste après la remise réussie. Un échec n'est pas un succès fictif.

Le pont t3code conserve les réponses prêtes dans son état de fil existant jusqu'à
confirmation. Il ne crée ni second broker, ni table, ni service. Sa boucle traite
les contrôles durant l'attente, sans nouvelle exécution d'une demande annulée.

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

Limites : aucune séparation garantie entre processus hostiles du même compte
système ; aucune interruption atomique d'un tour déjà accepté par t3code ; une
écriture partielle reste indéterminée. Aucun déploiement implicite de cette décision.

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
