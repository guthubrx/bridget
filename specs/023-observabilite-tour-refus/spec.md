# Session 023 — Observabilité des tours et des refus

**Branche** : `session-023-observabilite-tour-refus`
**Base** : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`
**Statut** : Reprise après STOP validée, nouvelle tête à remettre au jury
**Priorité** : P1

## Contexte

Une présence vivante et une délégation ouverte ne prouvent pas qu'un agent
travaille encore. Plusieurs agents sont restés classés occupés après la fin de
leur dernier tour, jusqu'à lecture manuelle de leur journal. Inversement, un
tour réellement ouvert peut rester silencieux longtemps pendant une revue ou
une compilation et ne doit pas devenir suspect à cause de son âge.

Un second incident a produit le refus générique `cible indisponible` alors que
la condition réellement exécutée était `state=busy`. L'absence de motif a fait
attribuer le refus au domaine, lequel n'entre pas dans la décision sur la base
mesurée. Le défaut porte donc sur l'explication du refus, pas sur la politique
d'éligibilité.

## Propriétés

### P1 — Dernière borne de tour observable

Pour tout participant d'une délégation ouverte, la ronde croise l'annuaire avec
les événements durables de son tour. Un `turn_end` corrélé atteste la borne
fermée `turn_completed`. Un `error` ne ferme le tour que si son producteur porte
le code fermé `terminal_kind=turn_failed` ; son texte `reason` reste un détail
libre et ne décide jamais. Une ancienne `error` sans code est indéterminée. Une
`update` corrélée ultérieure atteste que le tour a continué, tandis qu'un
nouveau `turn_start` atteste sa reprise. Hors attestation instantanée
`state=busy`, une source absente, illisible ou incohérente rend la conclusion
indéterminée ; elle ne devient jamais une preuve d'inactivité.

Une ouverture n'est une preuve positive que si son transport écrit aussi les
terminaux de succès et d'échec. Les wrappers interactifs `unix` et `ssh-unix`
n'écrivent actuellement que `turn_start` : hors état instantané `busy`, leur
projection reste donc indéterminée au lieu de prétendre que le tour est ouvert.

Cette projection ne crée aucun nouvel état durable et n'attribue aucune cause
à une fin de tour. Elle lit le journal append-only déjà détenu par Bridget.

Les détails libres du fournisseur restent présents dans le JSON structuré,
mais ne peuvent piloter la sortie opérateur : le rendu texte échappe les
caractères de contrôle et le JSON sérialisé ne porte aucun contrôle Unicode
brut. Le code terminal et le détail sont deux champs distincts.

### P2 — Condition de refus nommée

Tout refus de cible explicite rendu par la commande de délégation nomme la
condition qui l'a produit : cible pilote, agent absent de l'annuaire, profil
absent, `state=dnd`, `state=busy` ou autre état non éligible. Si toutes les
gardes observées paraissent satisfaites malgré le refus, le message nomme une
divergence d'invariant au lieu d'inventer une cause.

La session ne modifie pas la décision qui rend `busy` temporairement non
éligible. Choisir entre refus et mise en file est une décision métier séparée.

## Scénarios et critères mesurables

### US-2301 — Distinguer fin sans reprise et tour ouvert

1. Un participant dont le journal finit par `turn_end` ou par une `error`
   portant `terminal_kind=turn_failed`, sans `turn_start` ultérieur, n'apparaît
   plus dans `OCCUPES`.
2. Un participant dont le dernier tour reste ouvert demeure `OCCUPE`, même si
   sa remise a plus d'une heure.
3. Un participant qui a terminé puis ouvert un nouveau tour demeure `OCCUPE`.
4. Un journal absent ou corrompu sans attestation `state=busy`, ainsi qu'un
   terminal contredisant `state=busy`, produit `INDETERMINE` avec la condition,
   jamais `BLOQUE` par déduction.
5. Une ouverture issue d'un transport sans borne terminale reste
   `INDETERMINE`, sauf si l'annuaire atteste simultanément `state=busy`.
6. Le rendu texte et le JSON nomment la borne terminale et gardent l'âge de la
   remise comme contexte seulement, jamais comme condition de classement.
7. Une `error` non marquée suivie d'une `update` conserve le tour ouvert ; sans
   continuation elle reste `INDETERMINE`. Les séquences ESC, retour chariot et
   contrôles bidirectionnels présentes dans `stop_reason` ou `error.reason`
   restent diagnostiquables sans aucun contrôle brut dans la sortie.

### US-2302 — Comprendre un refus sans enquête

1. Le refus d'une cible `busy` contient littéralement le nom de la cible,
   `state=busy` et une indication de réessai après le tour.
2. Les refus pilote, DND, agent absent, profil absent et état inconnu ont chacun
   un motif distinct et exhaustif.
3. Un agent `connected` d'un autre domaine reste soumis à la politique
   existante : le domaine n'est ni ajouté comme garde ni cité comme cause.
4. Le JSON d'erreur porte le même diagnostic que le rendu texte.

## Hors périmètre

- Changer la politique qui refuse une cible pendant un tour `busy`.
- Mettre en file, réessayer automatiquement ou choisir une autre cible.
- Créer un nouveau type d'événement, un état daemon ou une migration de schéma.
  `terminal_kind` est un enrichissement additif du payload v1 existant.
- Modifier le schéma `provider_request` de 019 ou la boucle `attach.rs` de 022.
- Activer un outil depuis une branche non admise sur `origin/main`.

## Dépendances et composition

- La session 019 ajoute `provider_request` et enrichit les erreurs Codex. La
  passerelle Codex de 023 ajoute le code terminal après cet enrichissement ;
  aucune décision de 023 ne lit le texte ou les empreintes de 019.
- La session 022 modifie la boucle d'entrée de la vue ; 023 ne touche pas
  `attach.rs`.
- La session 021 touche aussi Maicie. La composition doit être mesurée sur sa
  tête gelée, puis 023 doit être rebasée après son intégration si nécessaire.
- La session 018 gouverne l'activation des outils de pilotage. Le script de
  ronde ne doit être activé qu'après admission sur `origin/main` par ce chemin.
- Aucune migration n'est nécessaire ; le numéro v19 reste inutilisé.
