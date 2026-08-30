# Plan d'implémentation - SPEC-073

## Décision de conception

Faire évoluer la fiche globale de SPEC-071 en panneau interactif ouvert
uniquement par un bouton à trois points. La ligne de sélection et le bouton
d'actions restent deux contrôles frères afin d'éviter tout bouton imbriqué.

Le décommissionnement traverse un nouvel adaptateur HTTP local très mince qui
valide la demande puis transmet l'ordre `StopOrder` existant au daemon. Le
cycle de vie, le superviseur de processus, les délais et les verdicts restent
ceux de `bridget stop`. Aucun signal système, stockage ou service parallèle
n'est créé.

## Contexte technique

- Langage backend : Rust, relais HTTP local déjà présent dans `ui.rs`.
- Interface : HTML, CSS et JavaScript natifs embarqués dans le binaire.
- Transport interne : socket Unix et trames `WrapperToDaemon` existantes.
- Source d'éligibilité : `AgentInfo.persistent: Option<bool>` ; `Some(true)` et
  `Some(false)` attestent tous deux un agent lancé et géré par Bridget, `None`
  signifie que cette gestion n'est pas attestée.
- Authentification : jeton de session local et présence humaine existants.
- Tests : tests Node intégrés à `app.js`, tests unitaires Rust dans `ui.rs` et
  tests d'intégration du relais existants.
- Dépendances nouvelles : aucune.
- Inconnues restantes : aucune.

## Constitution Check

| Gate | Verdict | Justification |
|---|---|---|
| Processus SpecKit | PASS | Spec, plan, audit d'existant, tâches, analyse et implémentation sont séparés. |
| Isolation worktree | PASS | Branche et worktree dédiés `session-073-actions-agent-ui`. |
| Réutiliser avant de créer | PASS | Réutilisation de la fiche SPEC-071, de `AgentInfo.persistent`, de `StopOrder` et du superviseur existant. |
| Minimalisme Article XIX | PASS | Un endpoint adaptateur et deux états UI, sans service ni dépendance. |
| Charge future Article XX | PASS | Le chemin UI et la CLI convergent vers le même contrat d'arrêt. |
| Complexité Article XVIII | PASS | Rendu O(n), une requête seulement après confirmation, aucun appel par ligne. |
| Sécurité et données | PASS | Pas de secret nouveau, pas de suppression d'historique, pas de signal direct. |
| Accessibilité | PASS | Déclencheur sémantique, panneau nommé, confirmation modale et restitution du focus. |

## Architecture cible

```text
UiAgentRowV1 existant
  identité SPEC-071 + persistent explicite
                     |
                     v
ligne enveloppe
  bouton sélection   bouton trois points
                            |
                            v
             panneau global interactif
             identité + état + action
                            |
                            v
                confirmation modale
                            |
                            v
POST /v1/agents/stop?token=...
                            |
                            v
WrapperToDaemon::StopOrder existant
                            |
                            v
superviseur géré existant + StopOutcome
```

## Lots réversibles

1. Étendre la projection UI avec le fait `persistent` existant et tester les
   trois états `Some(true)`, `Some(false)` et `None`.
2. Ajouter le contrat local de décommissionnement et son adaptation exacte vers
   `StopOrder`, avec tests de chaque `StopOutcome`.
3. Séparer le bouton de sélection du bouton à trois points, puis convertir la
   fiche SPEC-071 en panneau volontaire et accessible.
4. Ajouter la confirmation, l'état en cours et les messages de verdict, sans
   mise à jour optimiste de la présence.
5. Vérifier les régressions SPEC-071, les parcours clavier, les bords de fenêtre
   et l'arrêt réel d'un agent jetable géré.

## Projection et contrat backend

- `UiAgentRowV1` projette `persistent: Option<bool>` sans le transformer en
  déduction depuis le runtime ou le transport.
- Le navigateur envoie une requête versionnée contenant le nom exact et un
  identifiant de commande créé au moment de la confirmation.
- Le relais relit la liste d'agents avant l'ordre : absent, arrêté ou
  `persistent=None` sont refusés avant toute mutation.
- Le relais transmet `StopOrder` sur la socket daemon et attend `StopResult`.
- `Stopped` et `StoppedForced` sont des succès distincts. `NotManaged`,
  `NotFound` et `Timeout` conservent un résultat d'erreur distinct.
- Le navigateur interdit une seconde demande pour le même agent pendant la
  première. Aucun retry automatique n'est ajouté.

## Structure DOM et accessibilité

- Une enveloppe neutre porte deux boutons frères : sélection de conversation et
  actions. Aucun contrôle interactif n'est imbriqué dans un autre.
- Le bouton à trois points porte un nom accessible incluant l'agent,
  `aria-haspopup="dialog"`, `aria-expanded` et `aria-controls`.
- Le panneau d'identité porte `role="dialog"`, un titre visible et une action
  de fermeture. Il ne prétend pas être modal.
- L'ouverture déplace le focus dans le panneau. Échap ou fermeture restitue le
  focus au bouton à trois points encore présent.
- La confirmation destructrice porte `role="alertdialog"` et
  `aria-modal="true"`. Le focus initial va sur « Annuler ».
- Le panneau se ferme au clic extérieur ; la confirmation modale ne le fait
  pas tant qu'un choix n'est pas exprimé.

## Fichiers prévus

| Fichier | Modification |
|---|---|
| `crates/bridget-daemon/src/ui.rs` | projection `persistent`, route locale, adaptation `StopOrder`, réponses et tests |
| `crates/bridget-daemon/assets/ui/app.js` | structure de ligne, panneau interactif, confirmation, appel et tests |
| `crates/bridget-daemon/assets/ui/theme.css` | alignement trois points, panneau et confirmation |
| `crates/bridget-daemon/tests/ui_relay_test.rs` | contrat HTTP si la couverture unitaire ne suffit pas |

## Éléments explicitement évités

- Aucun nouveau service de cycle de vie.
- Aucun endpoint de suppression de données.
- Aucun `kill`, PID ou signal exposé au navigateur.
- Aucun composant ou framework UI ajouté.
- Aucun rôle utilisateur ou mécanisme d'authentification supplémentaire.
- Aucun traitement spécial fondé sur le nom `bridget` ; la sûreté repose sur
  une confirmation uniforme et un fait de gestion explicite.
- Aucun redémarrage, clonage ou arrêt en masse.

## Validation

- Node : ouverture seulement par les trois points, non-sélection, fermeture,
  focus, éligibilité, annulation, demande unique et messages de résultat.
- Rust : projection des trois valeurs de `persistent`, validation de requête,
  adaptation des cinq résultats du daemon et refus avant socket si inéligible.
- Intégration : token obligatoire, méthode HTTP, demande versionnée et réponse
  JSON stable.
- Non-régression : suites UI SPEC-071 et tests existants de `StopOrder`.
- Manuel : souris, clavier, zoom 200 %, première et dernière lignes, agent
  jetable géré, annulation puis arrêt confirmé.

## Déploiement

La feature sera embarquée dans le binaire `bridget`. Le pipeline ne committe,
ne fusionne et ne déploie pas automatiquement. Une mise en production demandera
une compilation, l'installation du binaire validé et le redémarrage explicite
du daemon et du relais UI.
