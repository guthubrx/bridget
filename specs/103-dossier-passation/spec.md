# Spécification 103 — Dossier de passation entre agents

## Fiche synthèse

Spec: 103-dossier-passation
Statut: In Progress
Priorité: P1
Tâches: 0/22 (0%) — implémentation non commencée
Tests: 0/23 (0%) — scénarios planifiés, non exécutés
Date: 2026-09-16
Branche: session-103-dossier-passation
Socle: 089, 094, 099, 100. Compléments facultatifs: 102 et 104.

Préparation documentaire uniquement, conformément à la demande. Aucun développement,
déploiement ou commit autorisé dans ce tour. Le statut concerne le cycle de réalisation,
pas une implémentation commencée.

## Besoin et résultat attendu

« Passe ce travail à B, avec ce qu'il faut pour reprendre. » L'agent courant prépare un
dossier court : objectif, état, résultats, décisions, incertitudes, références et prochain
pas. B le reçoit dans sa conversation habituelle. L'humain n'a pas de formulaire à remplir.
Le dossier n'est ni un mandat Maicie, ni un clone de conversation fournisseur, ni une
preuve que les résultats sont corrects. L'agent rédige la synthèse ; Bridget la transporte.

Exemple : A transmet à B « corriger la pagination », les deux causes écartées, le test
qui échoue, les fichiers concernés, une hypothèse encore non vérifiée et le prochain test
à lancer. B peut commencer sans relire 150 messages. Il garde ses permissions habituelles.

## Histoires utilisateur et acceptation

### US1 — Envoyer les éléments utiles à la reprise (P1)

1. À partir de son contexte, A renseigne l'objectif et le résumé ; les autres sections
   peuvent rester absentes et sont signalées « non renseigné », jamais inventées.
2. B reçoit un seul message structuré lisible, avec l'auteur réel de l'envoi.
3. Une préparation invalide ne transmet rien et explique quel champ corriger.
4. Préparer le dossier sans l'envoyer ne réveille aucun agent.
5. Aucun agent, modèle ou service additionnel n'est lancé.

### US2 — Donner des références et des limites honnêtes (P1)

1. Une référence de fichier indique un chemin absolu et son hôte, sans embarquer son contenu.
2. Une citation incluse est attribuée comme déclaration de A, pas comme preuve vérifiée par Bridget.
3. Un lien inaccessible à B reste une référence ; il ne devient pas une permission.
4. Les inconnues et les tests non exécutés restent explicites.
5. B sait que le dossier suit la conservation du journal (sept jours par défaut),
   et qu'un message n'offre pas de confidentialité supplémentaire par rapport au ledger actuel.

### US3 — Rejouer et demander une réponse sans doublon métier (P1)

1. Une coupure autorise le rejeu du même envoi avec les mêmes paramètres et la même clé.
2. Un changement de dossier sous cette clé est refusé, pas transmis une deuxième fois.
3. Une réponse n'est attendue que si A la demande explicitement.
4. La remise ne vaut ni acceptation de la mission ni réussite ; DND, absence et issue inconnue
   conservent les règles des messages directs.
5. Une mise à jour est un nouvel envoi explicite ; elle ne modifie pas le dossier ancien.

### US4 — Utiliser la même fonctionnalité depuis les outils et le terminal (P2)

1. Le même dossier produit le même contenu via MCP et CLI.
2. La skill montre préparer, prévisualiser, envoyer, rejouer et reprendre.
3. Un agent avec un ancien catalogue reçoit un diagnostic honnête ; aucun redémarrage forcé.
4. Le destinataire lit directement le message reçu, sans avoir besoin d'un nouveau lecteur.

## Exigences

- **FR-001** : fournir objectif et résumé obligatoires ; résultats, décisions, questions,
  prochain pas, références et limites facultatifs ; absence de donnée jamais bloquante
  hors les deux champs minimaux.
- **FR-002** : séparer dans le dossier les déclarations, résultats vérifiés déclarés par
  l'auteur, hypothèses et vérifications non faites ; aucune certification par Bridget.
- **FR-003** : préparation/prévisualisation sans écriture métier ni notification.
- **FR-004** : transmission explicite à un UUID résolu, comme un message direct unique.
- **FR-005** : ne jamais lire automatiquement fichiers, URLs, transcriptions fournisseur,
  journaux ou discussions pour remplir les références.
- **FR-006** : conserver les références dans le corps reçu et dans l'historique disponible ;
  leur présence n'élargit aucun accès.
- **FR-007** : plafonner le dossier et ses sections ; refuser plutôt que tronquer une
  information à l'insu de l'auteur.
- **FR-008** : réutiliser les garanties de rejeu et les statuts des messages idempotents.
- **FR-009** : demander une réponse uniquement sur choix explicite ; aucun accusé métier,
  abonnement, relance ou clôture automatique supplémentaire.
- **FR-010** : identité d'expéditeur attestée ; pas de paramètre pour emprunter celle d'un autre.
- **FR-011** : expliquer conservation et visibilité réelles, sans promesse d'archive permanente
  ou de conversation privée inexistante.
- **FR-012** : parité MCP/CLI et consignes de skill, avec lecture possible par les clients anciens.
- **FR-013** : maintenir les messages directs, les artefacts, le journal et les fils indépendants ;
  aucun nouveau workflow obligatoire.
- **FR-014** : traiter le dossier reçu comme données de son auteur ; commandes, URLs et mentions
  contenues dedans ne s'exécutent pas et ne notifient personne automatiquement.

## Critères mesurables

- **SC-001** : trois passations synthétiques permettent à un agent lecteur d'identifier objectif,
  état, prochaine action et limites sans échange préalable et sans chargement d'historique.
- **SC-002** : prévisualiser produit zéro envoi ; envoyer puis rejouer dix fois conserve une
  seule opération idempotente et aucun nouveau dossier dans le ledger.
- **SC-003** : une même entrée donne un corps identique octet pour octet par CLI et MCP.
- **SC-004** : toutes les erreurs de structure/taille sont détectées avant envoi, sans troncature.
- **SC-005** : 200 validations/rendus de dossiers de 16 Kio prennent chacun moins de 100 ms
  au 95e percentile sur le poste de recette, hors démarrage et transport ; aucun appel LLM.
- **SC-006** : les essais d'accès indu montrent zéro lecture de fichier/URL/source et zéro
  changement de droits ; les statuts de transport restent distincts de la réussite du travail.

## Limites et choix explicites

Dossier ponctuel dans les messages conservés, pas coffre documentaire permanent. Ni pièces jointes
copiées, ni collecte automatique, ni export de secrets. Un agent doit sélectionner les données
autorisées à partager ; Bridget ne peut vérifier la véracité ou confidentialité de texte qu'il écrit.
Le ledger général hérité est visible plus largement que le destinataire : ne pas y déposer un secret.
Les références aux fils 102 sont possibles comme texte structuré, mais ne transfèrent pas leur
appartenance. La recherche 104 facilite une relecture ultérieure, sans être nécessaire à la réception.
Pas d'envoi en groupe ni de passation automatique à la fin d'un tour dans cette spec.
