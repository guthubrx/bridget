# SPEC-070 - Preuves d'activité et notifications de réponse

**Statut** : Implémentée - validation locale réussie, non livrée

## Contexte observé

Le 30 août 2026, un message humain adressé à Bridget a été réellement remis à
Codex à 09:12:10 CEST. Codex a demandé et obtenu huit autorisations d'outil,
puis le tour a échoué à 09:13:10 CEST avec « échéance Codex dépassée ». Aucune
réponse durable n'a ensuite été reliée au message humain.

L'interface donne aujourd'hui deux informations insuffisantes : elle peut
afficher un état de remise qui ne prouve pas que l'agent travaille et elle ne
montre pas, pendant le travail, l'activité déjà présente dans le journal.

## Objectif

Donner à l'humain une preuve lisible du travail réellement commencé par un
agent, sans accusé automatique, puis le prévenir lorsqu'une réponse finale ou
un échec réel arrive.

## Histoires utilisateur

### US1 - Voir une activité réellement émise par l'agent - P1

Comme humain, après avoir écrit à un agent, je veux voir une indication sobre
uniquement lorsque le fournisseur a produit une vraie trace de travail, afin
de ne jamais confondre acceptation par Bridget et activité de l'agent.

Critères d'acceptation :

- L'acceptation HTTP seule ne crée ni « pris en compte » ni état équivalent.
- Une sortie textuelle, un appel d'outil, une commande, une lecture, une
  autorisation ou un raisonnement réellement journalisé peut mettre l'agent
  dans l'état visuel « travaille ».
- L'état est compact : bouille connue de l'agent, une seule ligne atténuée et
  une formulation humaine de la dernière activité. Il ne révèle pas
  automatiquement un argument de commande ou du contenu sensible.
- Sans activité fournisseur, aucun faux signal de travail n'est affiché.
- Entre la remise par Bridget et la première trace fournisseur, l'interface
  montre seulement trois points animés, sans bouille ni libellé de travail. Ils
  disparaissent dès une activité réelle ou un terminal corrélé.
- L'état disparaît ou devient terminal à la réponse finale, à l'interruption
  attestée ou à l'erreur terminale attestée.

### US2 - Attendre une réponse sans tuer son exécution - P1

Comme humain, je veux que « Attendre une réponse » engage l'agent à répondre,
sans transformer une question normale en tour limité à soixante secondes.

Critères d'acceptation :

- La case reste un choix de suivi d'une réponse, pas un minuteur d'exécution
  du fournisseur.
- Un tour humain qui a dépassé 60 secondes mais reste sous l'échéance du type
  d'agent peut continuer et produire sa réponse.
- L'interruption explicite d'un tour reste bornée et distincte d'un envoi
  humain ordinaire.
- À une vraie échéance fournisseur, le transport tente l'arrêt borné prévu,
  journalise un terminal unique et ne laisse pas de tour orphelin.

### US3 - Comprendre un échec réel - P1

Comme humain, si l'agent n'a pas pu finir, je veux voir une erreur reliée à ma
question, compréhensible et dépliable, afin de savoir si je dois attendre,
réessayer ou interrompre.

Critères d'acceptation :

- L'erreur terminale met à jour le message humain concerné, sans conserver un
  statut de remise optimiste.
- Le résumé exprime la nature de l'échec et le détail dépliable expose la
  raison durable et une référence de journal, sans secret fournisseur.
- Une réponse finale ultérieure n'est jamais masquée par une erreur associée à
  un autre tour.

### US4 - Revenir à la vraie réponse depuis une notification - P2

Comme humain, je veux être averti lorsqu'une réponse finale ou un échec réel
arrive pendant que je lis autre chose, puis atteindre exactement cet élément en
cliquant la notification.

Critères d'acceptation :

- Les notifications navigateur sont activées uniquement après un geste et une
  permission explicites de l'utilisateur.
- Elles ne sont créées que pour une réponse finale ou une erreur terminale
  attestée, jamais pour une simple acceptation ou activité intermédiaire.
- La notification porte l'agent, un résumé sûr et une cible de fil stable.
- Son clic sélectionne l'agent et positionne le fil sur la réponse ou l'erreur
  concernée. Si la plateforme ne délivre pas le clic, la puce interne de
  nouveaux messages conserve la même cible.

## Hors périmètre

- Notification quand l'application et son navigateur sont entièrement fermés :
  cela demanderait Push API, service worker et un backend de souscription.
- Déduction de l'intention humaine à partir des mots « arrête » ou « urgent ».
- Refonte esthétique générale de l'interface ou du transport Bridget.

## Dépendances

- Le journal EventSource existant demeure la source de vérité de l'activité.
- Les notifications dépendent de la permission du navigateur et de la page ouverte.

## Validation réalisée

- Les 11 tâches sont cochées dans `tasks.md`.
- Les tests Node et Rust ciblés passent, consignés dans `evidence.md`.
- La convergence est `CONVERGED`, détaillée dans `convergence.md`.
- La contre-revue adverse est indisponible avec l'émetteur UI actuel, documentée dans `adversarial-review-cartae0-flux.md`.
- La vérification visuelle automatisée reste bloquée par l'absence de Chromium Playwright local.

## Correctif de flux vivant - 30 août 2026

Le premier incrément compactait l'activité à son dernier événement et retenait
les événements survenus après un envoi humain pendant le rattrapage initial du
journal. Cette correction reste dans la même SPEC : elle ne change ni le
transport, ni l'autorité de Bridget, ni les commandes émises aux fournisseurs.

Critères supplémentaires :

- Un événement journalisé après l'envoi humain traverse le rattrapage et est
  rendu dans la prochaine fenêtre de rendu de l'interface, sans attendre
  `SnapshotCaughtUp`.
- La réponse texte déjà journalisée est rendue progressivement ; l'interface
  ne la retient pas jusqu'à la fin du tour.
- L'activité vivante conserve chaque acte réellement journalisé du tour actif,
  au lieu de n'afficher que le dernier.
- Une demande d'autorisation affiche son état réel : demandée, accordée ou
  refusée. Aucun état « attend » ne persiste après une décision journalisée.
- Chaque acte affiche le détail effectivement journalisé par le fournisseur : commande, outil ou chemin. Il ne fabrique ni sortie ni succès si le fournisseur ne les a pas journalisés.


### Correctif de compaction du flux - 30 août 2026

- La zone d’activité est repliée par défaut : elle montre la dernière opération réelle et, si elle existe, sa décision d’autorisation.
- « Voir les N actes » ouvre le flux complet dans l’ordre ; l’état ouvert persiste pendant les rendus suivants du même tour.
- Le détail de commande reste disponible mais visuellement atténué pour que l’état et l’action soient lus avant la ligne technique.
