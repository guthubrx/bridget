# Spécification 047 — Signaler un verdict attaché à une tête réécrite

**Statut** : Implémentée — prête à relire

**Base gelée** : `bc745335530985ce305e82fea4007071c752d5b0`

**Objectif Maicie** : `7a376ec5-2957-4bd4-b477-1c27941ec062`

## Problème mesuré

Un verdict peut être rendu sur un commit puis la branche distante peut être
réécrite sur un commit frère. Le verdict reste alors visible dans une phrase,
mais aucun fait durable ne relie mécaniquement son SHA à la tête distante
courante.

Le cas du lot 040 est attesté : le verdict complet sur
`47bc76dcdfc84bc3f74d78d14cb68680ef28429b` n'existe que dans le corps libre
du ledger Bridget. La base Maicie v20 mesurée porte 535 objectifs, 535
délégations et 50 réceptions guichet, mais zéro verdict de revue structuré sur
le chemin de délégation MCP courant.

Le contrat 021 fournit déjà `ReviewTarget` et `ReviewVerdictEvidence`. Le
défaut vient du raccord : `maicie_delegate` ne transporte pas la cible et le
service Maicie construit aujourd'hui `review_target=None`.

## Propriété

Quand un verdict typé existe, le système mesure si le SHA jugé est encore
ancêtre de la tête distante actuelle :

- un commit ajouté au-dessus du SHA jugé conserve le verdict ;
- une branche réécrite sur un frère invalide sa continuité et déclenche une
  alerte ;
- une cible, un verdict, un dépôt ou une observation Git absents produisent un
  état explicitement inobservable ou en attente, jamais un succès implicite.

La mesure est projetée dans les surfaces existantes `maicie status` et carte
de reprise. Aucun outil de référent séparé n'est créé.

## Décisions

### D1 — Réutiliser le contrat 021

Le chemin MCP expose atomiquement `review_ref` et `expected_head`, les porte
dans `ServiceRequestPayload::Delegate`, puis les remet à `DelegateRequest`.
Fournir un seul des deux champs est refusé avant le dépôt.

Le verdict reste celui du dépôt `delivery_report` existant. Sa
`ReviewVerdictEvidence` est relue depuis les octets de réponse déjà persistés
dans `guichet_receptions`; aucun nouveau schéma SQLite n'est ajouté.

### D2 — États fermés

La projection distingue au minimum :

- `target_absent` : aucune cible typée ne permet de qualifier la continuité ;
- `verdict_absent` : une cible existe, mais aucun verdict typé n'est déposé ;
- `still_ancestor` : le SHA jugé est encore ancêtre de la tête distante ;
- `rewritten` : le SHA jugé n'est plus ancêtre de la tête distante ;
- `unobservable` : Git, le dépôt configuré ou les faits persistés ne permettent
  pas de conclure, avec un motif fermé.

`target_absent`, `verdict_absent` et `unobservable` ne sont jamais rendus comme
verts.

### D3 — Observation Git sans altérer le dépôt

La référence distante est lue par `git ls-remote`. Si la tête observée n'est
pas déjà disponible localement, ses objets sont téléchargés dans un object
store temporaire isolé, sans modifier les références, l'index ni le worktree.
Le prédicat final est `git merge-base --is-ancestor <sha-jugé> <tête-distante>`.

Une erreur de commande, une sortie ambiguë ou un objet indisponible produit
`unobservable`; elle n'est jamais rabattue sur `still_ancestor`.

### D4 — Compatibilité

Une délégation historique sans cible reste en version 1 et produit
`target_absent`. Une délégation portant une cible utilise exclusivement la
version 2 ; un nouveau daemon refuse toute combinaison v1/v2 ambiguë.

Le vrai daemon de la base contractuelle refuse la trame v2 ciblée avant toute
écriture SQLite. Il ne peut donc pas créer silencieusement une délégation
ordinaire en perdant la cible.

## Scénarios

### US1 — Délégation de revue par MCP

Un appel `maicie_delegate` portant les deux champs crée une délégation dont
`review_target` contient la référence et le SHA complets. Un appel n'en portant
qu'un est refusé.

### US2 — Empilement normal

Le verdict vise A et la branche distante pointe sur B, descendant de A. Le
statut et la carte rendent `still_ancestor`; aucune alerte de réécriture n'est
émise.

### US3 — Réécriture

Le verdict vise A et la branche distante pointe sur C, frère de A. Le statut
et la carte rendent `rewritten` avec la référence, A et C.

### US4 — Provenance absente

Une délégation historique sans cible rend `target_absent`. Une cible sans
verdict rend `verdict_absent`. Une panne Git rend `unobservable`. Ces trois
issues sont différentes de `still_ancestor`.

## Exigences fonctionnelles

- **FR-4701** : `review_ref` et `expected_head` sont atomiques et validés par
  `ReviewTarget::is_valid`.
- **FR-4702** : le chemin MCP, le payload de service et la greffe transportent
  la même cible typée, sans reconstruire depuis le texte du but.
- **FR-4703** : les verdicts sont relus depuis les octets du dépôt existant ;
  aucun nouveau tableau ou colonne SQL n'est créé.
- **FR-4704** : le prédicat de continuité est l'ancêtralité, jamais l'égalité
  des SHA.
- **FR-4705** : la mesure Git ne modifie ni référence, ni index, ni worktree.
- **FR-4706** : `status` et la carte de reprise rendent l'alerte de réécriture
  et l'inobservabilité avec leurs faits exacts.
- **FR-4707** : un verdict accepté mais incohérent avec la délégation est
  traité comme corruption/inobservabilité, jamais comme un fait fiable.
- **FR-4708** : l'ancienne trame sans cible reste décodable.
- **FR-4709** : le cas « lot ouvert sans branche distante » reste hors lot et
  est déclaré comme dette ; aucune analyse de prose n'est admise.
- **FR-4710** : une campagne relit les verdicts persistés en une seule requête
  SQLite, puis les indexe en mémoire ; elle ne lance jamais une lecture par
  objectif ou par délégation.

## Critères de succès

- **SC-4701** : le chemin MCP réel crée une délégation portant le
  `ReviewTarget` exact.
- **SC-4702** : une réception de verdict acceptée ressort sous forme
  `ReviewVerdictEvidence` avec le SHA complet.
- **SC-4703** : A puis B descendant de A produit `still_ancestor` dans le
  statut et dans la carte de reprise.
- **SC-4704** : A puis C frère de A produit `rewritten` dans les deux surfaces.
- **SC-4705** : remplacer le test d'ancêtralité par une égalité tue l'oracle du
  cas empilé.
- **SC-4706** : cible absente, verdict absent et Git indisponible restent trois
  issues non vertes distinctes.
- **SC-4707** : l'ancien daemon ne transforme pas silencieusement une mission
  de revue nouvelle en délégation ordinaire.
- **SC-4708** : sur l'état durable mesuré de 540 délégations, la lecture groupée
  effectue une requête au lieu des 540 appels qu'exigerait le chemin scalaire.

## Hors périmètre

- Détecter un numéro de session attribué sans branche distante.
- Analyser les buts, instructions ou messages Bridget pour reconstruire un
  SHA ou une référence.
- Refaire le greffe ou ajouter un schéma SQLite.
- Fusionner, pousser, réécrire ou supprimer une branche.
