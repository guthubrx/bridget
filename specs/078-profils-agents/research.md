# Recherche de conception - SPEC-078 Profils d'agents et notifications

## Décision 1 - Séparer l'identité de profil du nom de routage

**Décision**: créer dans la SQLite existante du daemon un registre d'identités
opaque, des alias de routage et un profil par identité. Le nom de routage
actuel reste une adresse interne. Le profil porte un `agent_id` UUID opaque,
un `display_name` unique, les labels, l'apparence et les instructions.

**Rationale**: `DesiredFleet.equipiers` est indexé par le nom de routage et son
fichier `fleet.json` est relu par des versions antérieures. Il ne couvre que
les agents gérés, alors qu'un agent externe, arrêté ou historique doit garder
son identité visible. Le ledger SQLite est déjà l'autorité de l'historique et
supporte des migrations transactionnelles.

**Alternatives considérées**:

- Étendre `DesiredEquipier`: rejeté. Cela couple le profil au cycle de vie,
  force une migration de `fleet.json` et oublie les agents non gérés.
- Réutiliser `name` pour le nom affiché: rejeté. `name` route les messages,
  retrouve les reprises et indexe l'historique. Le renommage technique existant
  ne migre d'ailleurs pas le ledger.
- Conserver l'apparence uniquement dans `localStorage`: rejeté. Elle ne se
  partage ni entre le Web et Desktop, ni entre deux clients, et l'origine
  Desktop change avec le port de tunnel.

**Impact futur mainteneur**: un seul lien explicite entre alias technique et
identité évite les renommages destructifs. Le profil demeure lisible et
supprimable sans toucher aux messages ou à la flotte.

**Preuves locales**:

- `crates/bridget-daemon/src/desired_state.rs:121-145`
- `crates/bridget-daemon/src/store.rs:504-515`
- `crates/bridget-daemon/src/cli.rs:1232-1259`

## Décision 2 - Migration additive et idempotente

**Décision**: au démarrage, une migration SQLite crée les tables de profils et
importe les noms connus de la flotte, de l'annuaire vivant et du ledger. Chaque
nom reçoit un `display_name` initial égal au libellé aujourd'hui visible. Les
messages existants ne sont jamais réécrits. Un renommage technique ultérieur
met à jour le binding actif et conserve l'ancien nom comme alias.

**Rationale**: l'historique est indexé par `sender`, `target` et
`conversation_key`. Le rattachement se fait à la projection, sans mutation des
lignes du ledger. Pour deux anciens agents ayant employé exactement le même nom
de routage, aucune donnée historique ne permet une séparation fiable: ils
partagent donc le même profil historique plutôt que de créer une fiction.

**Alternatives considérées**:

- Réécrire le ledger en UUID: rejeté, trop risqué et incompatible avec les
  conversations et clés existantes.
- Exiger une migration manuelle: rejeté, elle rendrait inutilisables les
  agents historiques précisément concernés par la feature.

**Impact futur mainteneur**: la migration peut être relancée sans perte et les
cas ambigus sont documentés par le modèle plutôt que masqués dans l'UI.

## Décision 3 - Un contrat de profil projeté par le relais

**Décision**: étendre `/v1/snapshot` et `/v1/watch` avec une projection de
profil et ajouter des mutations typées, authentifiées par le jeton du relais.
La référence opaque reste dans les données JavaScript mais n'est jamais rendue
dans le DOM, les libellés, les erreurs ou les notifications. Les lecteurs
continuent à accepter les agents sans profil pendant une migration ou une
dégradation du store.

**Rationale**: Web et Desktop consomment déjà le même relais. Bridget Desktop
ouvre une WebView vers cette UI, il ne possède pas une seconde application de
conversation à maintenir. Centraliser la projection garantit que les deux
surfaces affichent exactement le même nom, les mêmes labels et la même bouille.

**Alternatives considérées**:

- Ajouter une UI de profil dans Tauri: rejeté, elle créerait un deuxième état
  de profil et deux implémentations des mêmes règles.
- Créer un endpoint par champ: rejeté, une mutation atomique du profil évite
  les états partiellement sauvegardés et simplifie la validation.

**Impact futur mainteneur**: le contrat versionné isole l'UI du schéma SQLite
et donne une surface de tests stable aux clients futurs.

**Preuves locales**:

- `crates/bridget-daemon/src/ui.rs:656-704`
- `crates/bridget-daemon/src/ui.rs:1049-1096`
- `apps/bridget-desktop/src-tauri/src/lib.rs:366-405`

## Décision 4 - Réutiliser le panneau droit et l'avatar existant

**Décision**: le clic sur la bouille de l'en-tête ouvre un mode
`agentProfile` du panneau droit existant. Le menu contextuel des trois points
reste le seul menu d'actions rapides SPEC-077. Les formes, couleurs et le
renderer d'avatar sont réemployés, mais leur source passe du stockage local au
profil serveur.

**Rationale**: l'interface possède déjà le panneau de détail des échanges et
un avatar cliquable. Les réemployer évite une nouvelle colonne, une nouvelle
famille de composants et une divergence Web/Desktop.

**Alternatives considérées**:

- Transformer le menu contextuel en formulaire: rejeté, il dégraderait les
  actions de cycle de vie et contredirait SPEC-077.
- Ajouter une modal globale: rejeté, elle casse la continuité de la
  conversation et le modèle déjà disponible du panneau latéral.

**Impact futur mainteneur**: un état de panneau nommé rend explicite le choix
entre détail d'échange et profil sans superposer deux comportements implicites.

## Décision 5 - Instructions individuelles appliquées à la prochaine naissance

**Décision**: les instructions sont rendues par Bridget dans un bloc borné,
versionné et inférieur aux invariants Bridget, aux permissions et au mandat.
Elles sont chargées en snapshot au démarrage ou à la reprise du fournisseur.
Une édition durant une session active est marquée `en attente de relance`; elle
n'est jamais annoncée comme appliquée rétroactivement.

**Rationale**: aucun transport actuel n'atteste une primitive sûre de mise à
jour d'instructions dans une session active. Détourner `steer`, `interrupt` ou
un message utilisateur modifierait le travail en cours et créerait un faux
succès. Le point commun de lancement et de relance est le wrapper du daemon.

**Stratégie par famille**:

| Famille | Application au prochain démarrage | État honnête |
|---|---|---|
| Claude stream, GLM, DeepSeek | carte interne Bridget contrôlée avant le premier travail, jamais le texte en argument de processus | appliquée après spawn confirmé |
| Codex app-server | carte interne Bridget avant le premier travail, sans utiliser `steer` | appliquée après création du thread |
| Cursor / ACP | carte interne Bridget avant le premier prompt de session | appliquée après `session/new` |

Le texte ne va ni dans `AgentDefinition.args`, ni dans `agents.json`, ni dans
les profils `CLAUDE_CONFIG_DIR`: ces emplacements sont fournisseur-globaux ou
visibles dans des diagnostics. La persistence ne garde dans la projection que
la révision, la date et le verdict d'application, jamais le texte complet.

**Alternatives considérées**:

- Personnas partagées: hors périmètre explicite de SPEC-078.
- Injection dans l'argv commun ou dans la définition fournisseur: rejetée,
  car visible, figée et susceptible de croiser les agents GLM/DeepSeek.
- Mise à jour active via une commande fournisseur non documentée: rejetée,
  car aucune capacité ne l'atteste dans les contrats présents.

**Impact futur mainteneur**: une interface de contexte initial, petite et
testée, permet d'ajouter une primitive native à un fournisseur plus tard sans
mentir sur les autres.

**Références**: les garde-fous doivent rester distincts des instructions
personnalisables et les actions sensibles conserver une supervision humaine,
conformément au [guide OpenAI sur les agents](https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/) et à
[l'analyse Anthropic des agents effectifs](https://www.anthropic.com/engineering/building-effective-agents).

## Décision 6 - Journal sémantique d'attention et préférences par client

**Décision**: le daemon publie et conserve des événements sémantiques,
idempotents et typés: `human_input_needed`, `task_completed`,
`terminal_failure`. Les outils, commandes, chunks et statuts routiniers ne
produisent aucun événement d'attention. Les préférences de livraison sont
scopées par `client_id` stable et `agent_id`, sans modifier les autres clients.
Le centre d'activité est alimenté par ce journal et non par les toasts.

**Rationale**: un fait serveur permet à chaque client de rattraper une absence,
dédoublonner après reconnexion et afficher un badge fiable. La préférence reste
locale au sens fonctionnel: elle appartient à un appareil/client précis, jamais
au profil de l'agent. Pour Desktop, l'origine loopback varie à chaque tunnel;
un `client_id` local persistant évite de perdre les choix à chaque port aléatoire
et permet au backend Tauri d'émettre une notification macOS.

**Alternatives considérées**:

- Déduire les alertes des chunks UI: rejeté, bruit, absence de déduplication et
  perte à la reconnexion.
- Préférences uniquement dans `localStorage`: conservable pour le Web direct,
  mais insuffisant pour Desktop à origine variable et impossible à lire par la
  coque Tauri.
- Notifier chaque client depuis le serveur: rejeté, un choix d'un appareil ne
  doit pas réveiller les autres.

**Règles UX**:

- maximum une alerte système locale par occurrence;
- badge et centre sont consultables sans déplacer le focus;
- libellé bref, nom affiché seulement, action ouvrant l'agent concerné;
- une permission refusée désactive seulement l'alerte système, pas le journal;
- une application entièrement quittée ne promet pas de notification, mais le
  journal est retrouvé au prochain démarrage.

**Impact futur mainteneur**: le journal sémantique est réutilisable pour une
vue de supervision sans faire dépendre le produit de Notifications navigateur.

**Référence**: les notifications doivent être rares, non intrusives, accessibles
et ne pas déplacer le focus. La guidance [Home Office sur les notifications](https://design.homeoffice.gov.uk/accessibility/notifications)
recommande précisément de limiter les alertes, garder une place stable et
utiliser une région ARIA polie.

## Décision 7 - Notifications Web et native Desktop sans seconde UI

**Décision**: Web reçoit le journal et affiche son centre d'activité avec les
préférences du navigateur. Bridget Desktop conserve son `client_id` et ses
préférences stables dans son `app_data_dir`, suit le même flux relayé via le
tunnel et utilise le plugin officiel de notification Tauri pour macOS. La
WebView enfant reste sans privilège Tauri.

**Rationale**: la sécurité de SPEC-074 interdit de donner les capacités Tauri
au contenu relayé. La coque Desktop, qui possède déjà le tunnel, est le bon
propriétaire de la permission système et de l'émission native. L'UI partagée
reste l'unique lieu de réglage et de consultation.

**Alternatives considérées**:

- Laisser la WebView relayée invoquer Tauri: rejeté, violation de la frontière
  de capacités SPEC-074.
- Ajouter un second centre d'activité Desktop: rejeté, duplication de l'UI
  et divergence de lecture.

**Impact futur mainteneur**: la couche Rust Desktop est limitée à la livraison
native et à l'état local, tandis que toute présentation reste dans le relais.

**Référence**: le [plugin officiel Tauri Notification](https://v2.tauri.app/plugin/notification/)
fournit la permission et l'envoi natif macOS sous le modèle de capacités Tauri.

## Décision 8 - Validation et confidentialité

**Décision**: valider côté serveur et côté client le nom affiché, labels,
apparence et longueur des instructions. Les mutations sont transactionnelles,
protégées par le jeton existant et suivies d'un verdict serveur. Les logs,
traces, messages d'erreur et notifications n'incluent ni UUID interne, ni texte
d'instructions, ni secret.

**Rationale**: le profil est une surface d'écriture exposée par le relais. Une
validation fermée et le rejet atomique d'un doublon empêchent les profils
partiels, les collisions et les fuites de contexte.

**Impact futur mainteneur**: des limites documentées, des tests de refus et une
projection sans texte secret permettent de diagnostiquer sans réexposer le
contexte d'un agent.
