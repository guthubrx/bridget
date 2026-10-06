# Recherche 138 — Choix et limites

Date de consultation: 2026-10-06.

## Baselines et sources primaires

Baselines utilisateur lues:
/Users/moi/.speckit/research/01-ai-agents-agentic-ai.md,
/Users/moi/.speckit/research/03-cognitive-load-productivity.md,
/Users/moi/.speckit/research/04-architectures-patterns.md.
Elles servent d'index de recherche. Aucune métrique de marché, de productivité
ou de conformité n'est reprise comme preuve de fonctionnement de Bridget.

| Source consultée | Enseignement utilisé | Limite |
|---|---|---|
| [Anthropic, contexte des agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) | Sélectionner le contexte utile et limiter les outils ambigus | Conseils d'un fournisseur ; pas une mesure sur Bridget |
| [Jiang et Lu, communication attentive](https://arxiv.org/abs/1805.07733) | Une diffusion générale peut gêner la coopération ; une restriction rigide peut aussi la limiter | Travail de 2018 en apprentissage multi-agent ; pas une validation des agents LLM |
| [NN/G, divulgation progressive](https://www.nngroup.com/articles/progressive-disclosure/) | Garder une voie simple et proposer les options avancées sur demande | Principe UX ; application à une CLI décidée ici |
| [OWASP, autorisation](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html) | Vérifier les demandes à l'autorité commune et tester les contournements | La portée projet 138 n'est pas une politique d'autorisation de sécurité |
| [NIST, SSDF 1.1](https://www.nist.gov/publications/secure-software-development-framework-ssdf-version-11-recommendations-mitigating-risk) | Concevoir, vérifier et conserver les preuves du développement | Consultation documentaire ; aucune conformité certifiée |
| [Fowler, pyramide de tests](https://martinfowler.com/articles/practical-test-pyramid.html) | Combiner tests rapides et quelques essais d'intégration utiles | Conseils généraux ; proportions décidées selon le risque local |
| [Google, serveurs hermétiques](https://testing.googleblog.com/2012/10/hermetic-servers.html) | Tester un vrai serveur isolé sans dépendre du réseau extérieur | Ouverture directe en erreur ; contenu retrouvé par recherche sur le même domaine |

Recherche live effectuée sur les échecs de coordination multi-agent et la charge
cognitive. Le [retour d'expérience Anthropic](https://www.anthropic.com/engineering/multi-agent-research-system)
décrit des tâches dupliquées et des périmètres mal spécifiés. Il motive la clarté
du mandat. Il ne prouve pas un gain chiffré de cette session.

## Décisions

### D1 — Projet de communication attesté, distinct du domaine

Décision: porter un fait vivant sur la connexion. Utiliser la racine commune Git
et l'hôte attesté, ou la racine T3 attestée hors Git. Hériter du fait de l'instance
pour une connexion auxiliaire déjà prouvée.

Pourquoi: live_connection_identity et register_auxiliary existent. Les données
T3 contiennent workspace_root. derive_domain_at peut changer l'étiquette et ne
prouve pas une appartenance. ProjectReference/runtime est retiré.

Alternatives: comparer domain produit des faux locaux ; recréer un registre
ajoute une autorité ; vérifier Git à chaque envoi ajoute des entrées/sorties
sous verrou. Ces options sont écartées.

Impact futur: un mainteneur inspecte un fait annoncé et ses preuves. Un chemin
non vérifiable devient inconnu. Pas de correction automatique inventée.

### D2 — Annuaire scoped sans modifier le diagnostic global

Décision: ajouter une variante scoped et conserver ListAgents. who/agents et les
suggestions de communication utilisent la portée locale, avec --global explicite.

Pourquoi: les tests et diagnostics utilisent ListAgents sans identité active.
Changer son sens casserait ces clients. Le modèle de divulgation progressive
inspire la voie locale par défaut ; cette application est une déduction de conception.

Impact futur: deux intentions nommées remplacent un contrat global ambigu.
Il n'existe pas de repli automatique extérieur.

### D3 — Motif structuré, garde commune et avertissement résultat

Décision: un seul champ cross_project_reason optionnel exprime le choix volontaire.
La garde commune vérifie le projet avant dépôt. Le résultat caller contient
le warning ; aucun ajout au corps. Les inconnus restent compatibles et avertis.

Pourquoi: canonical_send et prepare_dispatch possèdent déjà l'enveloppe et les
contrôles d'envoi. Un prompt seul ne peut pas garantir leur application.
La demande explicite est l'autorisation ; aucun second dialogue humain requis.

Alternatives: interdire tout échange extérieur contredit le besoin ; demander
une confirmation à chaque tick casse les missions ; analyser le texte libre
ne constitue pas une preuve. Options écartées.

Impact futur: règle testable sur une matrice de faits, sans classifieur ni service.
La borne de 512 octets limite le coût ; elle sera testée aux frontières UTF-8.

### D4 — Mandat borné aux membres ou à la demande suivie

Décision: porter le motif dans chaque opération nouvelle du fil. Le caller peut
réutiliser le motif d'un mandat explicite pour ses membres immuables.
Une réponse directe hérite seulement d'une demande suivie OPEN
validée avec les participants inversés. Agent Loop garde des mandats explicites
par agent/rôle attribué et racine du run.

Pourquoi: notify=[] reste lisible par tous les membres. Les fils et demandes
suivies ont déjà l'audience et la corrélation nécessaires. Un rôle ROOT configuré
ne suffit pas à autoriser un échange extérieur.

Impact futur: aucune nouvelle colonne ou table de consentement de fil, aucune
migration de mission et aucun nouveau système de permissions. Un motif historique
n'accorde pas de consentement implicite pour une autre opération.
Le complément T019, décidé ensuite, ajoute une seule colonne JSON project_warnings
à execution_control_commands existante. Cette exception conserve les warnings
du résultat de contrôle durable sans changer canonical_bytes ni refusal_reason.

### D5 — Rejeu historique avant nouvelle garde

Décision: rendre le résultat déjà durable avant d'appliquer une garde à une
opération nouvelle. Garder les canons legacy sans suffixe lorsque le motif manque.
Ne pas reprendre les anciennes files par une tâche de balayage.

Pourquoi: changer rétroactivement une opération acceptée casserait l'idempotence.
102 et 136 possèdent déjà les reçus, snapshots et remises silencieuses.

Impact futur: la recette vérifie l'ancien reçu, le nouveau conflit et l'opération
acceptée encore en remise. Les anciens messages restent exacts et lisibles.

## Incertitudes résolues et limites vérifiables

Le choix du projet n'utilise aucun registre retiré. Le rattachement T3 et la
racine Git sont les seules preuves de cette session. Un chemin distant sans
preuve exploitable reste inconnu. Une vue globale n'autorise rien. Un ancien
client inconnu reçoit un warning ; un client dont le projet est prouvé applique
la même garde que les clients nouveaux.

Tous les choix ci-dessus sont des décisions locales de conception. Aucun gain
en tokens ni réduction d'erreurs n'est annoncé avant les essais. Les sources
ne remplacent ni la lecture du code ni la recette du daemon isolé.
