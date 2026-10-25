# Archives techniques de l’adaptateur T3

Document d’origine : [specs/081-conversation-renderer/spec.md](../../../../../../specs/081-conversation-renderer/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

L'objectif est de reprendre les qualités de lisibilité observées dans T3 Code,
dont le code est disponible sous licence MIT, sans recopier aveuglément son
architecture ni élargir les pouvoirs d'un contenu non fiable.

<a id="extrait-2"></a>

## Extrait 2

- Exécuter une commande depuis un bloc de code ou un lien de conversation.
- Ajouter un terminal intégré, un éditeur de code complet ou un navigateur
  distant.
- Modifier le protocole agent, le mécanisme de remise, les autorisations des
  agents ou le modèle de sécurité SSH.
- Ajouter des pièces jointes binaires, une gestion de fichiers générale ou un
  système de partage d'images.
- Afficher un lien, une image ou un fichier sans que la préférence locale
  correspondante l'autorise.
- Reproduire intégralement l'interface ou l'architecture de T3 Code.

<a id="extrait-3"></a>

## Extrait 3

- **FR-001** : Bridget DOIT présenter une demande humaine, les traces de son
  tour et la réponse associée comme une interaction identifiable.
- **FR-002** : Bridget DOIT afficher la demande humaine à droite et la réponse
  d'agent à gauche avec une hiérarchie visuelle différente.
- **FR-003** : Bridget DOIT conserver les états de remise, d'activité,
  d'interruption, d'échec et de durée lorsqu'ils sont attestés.
- **FR-004** : Bridget DOIT permettre de développer et replier les détails
  d'activité d'un tour sans masquer son résumé ni modifier silencieusement la
  position de lecture.
- **FR-005** : Bridget DOIT rendre les structures Markdown prises en charge,
  les tableaux et les blocs techniques de façon lisible dans les deux thèmes.
- **FR-006** : Bridget DOIT permettre de copier un bloc de code et un tableau
  avec un retour visuel de réussite ou d'échec.
- **FR-007** : Bridget DOIT proposer une présentation sûre de contenus riches,
  dont liens, fichiers et images, contrôlée par des préférences locales
  distinctes.
- **FR-008** : Les préférences de contenu enrichi DOIVENT être désactivées par
  défaut pour une nouvelle installation, un profil inconnu et une valeur
  invalide.
- **FR-009** : L'opérateur actuel DOIT pouvoir activer localement les trois
  préférences de consultation demandées, sans requête de mutation vers un
  serveur Bridget.
- **FR-010** : Aucun contenu conversationnel ne DOIT déclencher seul une
  exécution, une écriture, une navigation automatique ou une élévation de
  privilège.
- **FR-011** : Bridget DOIT préserver la politique de défilement honnête de
  SPEC-069 et éviter toute duplication d'une demande humaine durable.
- **FR-012** : Toute réutilisation directe d'éléments T3 Code DOIT conserver
  les avis de licence MIT applicables et être traçable dans les artefacts de la
  feature.

<a id="extrait-4"></a>

## Extrait 4

- **Sécurité** : les préférences sont locales, versionnées et validées ; aucun
  secret, chemin sensible ou contenu de conversation n'est ajouté à un journal
  de préférence.
- **Accessibilité** : les détails, boutons de copie, actions de navigation et
  contenus bloqués sont utilisables au clavier, avec un nom accessible et un
  retour de statut non exclusivement visuel.
- **Performance** : l'ouverture d'un détail, la copie et l'arrivée d'une
  nouvelle entrée ne doivent pas figer le fil perceptiblement ; l'historique
  long ne doit pas imposer le rendu visible de toutes ses entrées en même temps.
- **Compatibilité** : le fil reste utilisable lorsque la coloration, le presse-
  papier, les préférences persistées ou un type de contenu enrichi ne sont pas
  disponibles.
- **Conformité de licence** : la provenance et les avis requis sont conservés
  pour tout code directement adapté depuis T3 Code.

<a id="extrait-5"></a>

## Extrait 5

Les choix de ce parcours relèvent du contrat Bridget : identité attestée,
état durable, budgets bornés et vérification avant annonce de réussite.
Les anciennes comparaisons de produits ont été retirées le25octobre2026.
