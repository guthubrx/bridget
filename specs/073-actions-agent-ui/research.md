# Recherche - SPEC-073

## Sources internes consultées

- `~/.speckit/research/03-cognitive-load-productivity.md` : réduire les
  déclenchements involontaires et garder l'action destructive progressive.
- `~/.speckit/research/04-architectures-patterns.md` : maintenir une frontière
  déterministe entre l'interface, le transport et le superviseur.
- SPEC-071 : conserver le catalogue runtime, les logos locaux et la fiche
  globale plutôt que créer une seconde vue.
- Code existant : `AgentInfo.persistent`, `WrapperToDaemon::StopOrder`,
  `DaemonToWrapper::StopResult` et les tests du superviseur.

## Validation externe

- W3C WAI-ARIA APG, Menu Button Pattern :
  https://www.w3.org/WAI/ARIA/apg/patterns/menu-button/
- W3C WAI-ARIA APG, Dialog Modal Pattern :
  https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/
- W3C WAI-ARIA APG, Alert and Message Dialogs Pattern :
  https://www.w3.org/WAI/ARIA/apg/patterns/alertdialog/

La première source impose un bouton sémantique, un état ouvert explicite et un
transfert de focus cohérent. Les deux autres recommandent de contenir le focus
dans une confirmation réellement modale, de fermer avec Échap, de restituer le
focus au déclencheur et de privilégier l'action la moins destructive au focus
initial.

## Décision 1 - Panneau d'identité interactif, pas menu ARIA

**Décision**: le bouton à trois points ouvre un panneau portant le rôle de
dialogue non modal. La confirmation seule est modale.

**Rationale**: la vue mélange des informations structurées, des logos, des
faits et une action. Un rôle `menu` conviendrait à une simple liste de commandes
mais appauvrirait la sémantique des informations d'identité.

**Alternatives considérées**:

- menu ARIA contenant toute la fiche : rejeté, car le contenu n'est pas une
  liste homogène de commandes ;
- tooltip enrichi : rejeté, car un tooltip ne doit pas porter d'action ;
- panneau permanent : rejeté, car il surcharge chaque ligne.

**Impact mainteneur futur**: une seule fiche globale existante change de rôle
et de déclencheur ; aucune nouvelle famille de composants.

## Décision 2 - Deux boutons frères dans chaque ligne

**Décision**: envelopper le bouton de sélection et le bouton à trois points dans
un conteneur neutre.

**Rationale**: un bouton ne peut pas contenir légalement un autre bouton. Cette
structure conserve les deux intentions et empêche le clic sur les actions de
sélectionner la conversation.

**Alternatives considérées**:

- bouton imbriqué : rejeté pour validité HTML et accessibilité ;
- transformer la ligne en pseudo-bouton avec `role` et `tabindex` : rejeté car
  il faudrait recréer le comportement natif ;
- action seulement dans l'en-tête principal : rejeté car l'utilisateur demande
  l'accès depuis chaque ligne.

**Impact mainteneur futur**: changement DOM local, sans gestion clavier
artificielle pour la sélection.

## Décision 3 - Réutiliser `persistent` comme attestation de gestion

**Décision**: projeter l'option existante sans ajouter un nouveau registre.
`Some(true)` et `Some(false)` signifient que Bridget connaît le cycle de vie ;
`None` interdit l'action.

**Rationale**: le commentaire du type existant définit déjà exactement cette
frontière. Le booléen indique la reprise après redémarrage, tandis que la
présence de la valeur atteste le lancement par `bridget spawn`.

**Alternatives considérées**:

- déduire depuis TMUX ou le transport : rejeté car faux et contraire à
  SPEC-071 ;
- ajouter un champ stocké `managed` : rejeté comme état redondant ;
- tenter l'arrêt et traiter `NotManaged` uniquement après coup : rejeté car
  l'interface présenterait une action qu'elle sait inéligible.

**Impact mainteneur futur**: aucun modèle durable ni migration.

## Décision 4 - Adaptateur HTTP vers `StopOrder`

**Décision**: ajouter une route locale versionnée qui valide puis relaie
`StopOrder` et traduit `StopOutcome`.

**Rationale**: le navigateur ne peut pas parler directement à la socket Unix.
L'adaptateur réutilise le même superviseur que la CLI et ne porte aucune logique
de terminaison propre.

**Alternatives considérées**:

- invoquer la CLI comme sous-processus : rejeté, car cela ajoute parsing,
  processus et erreurs indirectes ;
- dupliquer l'arrêt dans le relais : rejeté, car dangereux ;
- exposer la socket au navigateur : rejeté, car elle élargit l'autorité.

**Impact mainteneur futur**: un chemin HTTP mince testable et supprimable ; la
logique opérationnelle reste unique dans le daemon.

## Décision 5 - Confirmation modale uniforme

**Décision**: tous les agents éligibles passent par la même confirmation, avec
nom exact, conservation d'historique et avertissement supplémentaire si un tour
est actif. Le focus initial est placé sur « Annuler ».

**Rationale**: la prévention d'erreur ne dépend pas d'un nom spécial ni d'une
connaissance implicite du rôle de l'agent.

**Alternatives considérées**:

- traitement spécial de l'agent nommé `bridget` : rejeté, car le nom n'est pas
  une attestation de rôle ;
- saisie obligatoire du nom pour tous : rejetée comme friction excessive pour
  une opération réversible par relance et sans suppression de données ;
- `window.confirm` : rejeté pour intégration visuelle et contrôle du focus.

**Impact mainteneur futur**: une confirmation unique, sans liste protégée à
maintenir.
