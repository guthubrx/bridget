# Spécification 134 — Noms humains dans les messages Bridget

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/spec.md#extrait-1).

## Pourquoi

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/spec.md#extrait-2).

Le résultat attendu est un libellé lisible et sûr. Le nom aide la lecture. L’UUID
reste visible et reste la seule identité utilisée pour le routage.

## Scénarios utilisateur et tests

### US1 — Reconnaître immédiatement l’expéditeur (P1)

Étant donné un agent qui possède un nom de présentation, quand Bridget remet un
message simple, l’en-tête affiche le nom puis l’UUID complet.

Test indépendant : remettre un message avec un nom connu et vérifier que le nom
et l’UUID apparaissent une seule fois dans l’en-tête.

### US2 — Lire un lot sans perdre l’auteur de chaque message (P1)

Étant donné plusieurs messages regroupés, quand Bridget construit le tour, chaque
élément affiche le même libellé humain que le rendu d’un message simple.

Test indépendant : regrouper deux messages issus de deux agents nommés et vérifier
que chaque nom reste associé au bon UUID.

### US3 — Conserver un repli fiable (P1)

Étant donné un message dont le nom est absent, vide, blanc après nettoyage ou
identique à l’UUID, quand Bridget le remet, l’en-tête affiche seulement l’UUID.
La remise ne doit pas échouer.

Test indépendant : exercer les quatre formes sans nom exploitable et comparer le
rendu au format historique.

### US4 — Réparer un profil manquant au prochain enregistrement (P1)

Étant donné une identité existante dont le profil de présentation manque, quand
le même agent s’enregistre à nouveau, Bridget recrée le profil puis accepte le nom
humain publié par le pont.

Test indépendant : supprimer uniquement le profil d’une identité de test, refaire
l’enregistrement, publier un nom et vérifier sa présence dans le message livré.

## Exigences fonctionnelles

- **FR-13401** : afficher le nom de présentation suivi de l’UUID complet quand un
  nom exploitable est disponible au moment de la remise.
- **FR-13402** : conserver l’UUID comme seule clé de routage, de réponse,
  d’idempotence et d’autorisation.
- **FR-13403** : appliquer le même libellé aux messages simples et à chaque
  message d’un lot.
- **FR-13404** : afficher seulement l’UUID si le nom est absent, vide, blanc
  après nettoyage ou égal à l’UUID.
- **FR-13405** : préserver la provenance visible des sous-agents après le
  libellé du parent.
- **FR-13406** : lors de l’enregistrement, garantir qu’une identité existante
  possède aussi son profil de présentation. La réparation doit être idempotente.
- **FR-13407** : ne pas modifier l’historique des messages ni exiger une
  migration globale. Les agents vivants se réparent à leur prochain
  enregistrement.
- **FR-13408** : ne pas ajouter de dépendance, de service résident, de table ni
  de nouveau format de message.
- **FR-13409** : ne jamais inclure le corps du message, un secret ou un
  raisonnement dans le nom ou les diagnostics de réparation.

## Entités

- **Identité routable** : UUID stable utilisé par Bridget pour toutes les
  décisions de routage et d’autorisation.
- **Profil de présentation** : données lisibles associées à une identité,
  notamment le nom humain.
- **Libellé d’expéditeur** : projection visible composée du nom, de l’UUID et,
  si nécessaire, de la provenance d’un sous-agent.

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/spec.md#extrait-3).

## Hypothèses et limites

Le nom de présentation est informatif. Il peut changer. L’UUID est stable et
reste visible pour permettre le diagnostic et une réponse sans ambiguïté.

La réparation est progressive. Un agent éteint dont le profil manque n’est pas
modifié tant qu’il ne s’enregistre pas à nouveau.

## Hors périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/spec.md#extrait-4).
