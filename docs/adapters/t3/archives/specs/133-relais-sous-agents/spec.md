# Archives techniques de l’adaptateur T3

Document d’origine : [specs/133-relais-sous-agents/spec.md](../../../../../../specs/133-relais-sous-agents/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Branche** : session-133-relais-sous-agents
**Date** : 2026-10-04
**Statut** : Implemented
**Priorité** : P1
**Demande** : reprendre le pipeline `$my-specify-all` commencé pour permettre aux sous-agents de communiquer par Bridget.
**Dépendances** : 064 (liens parent-enfant), 090 (agents interactifs), 098 (pont T3), 099 (preuve auxiliaire), 101 (identité T3), 114 (joignabilité), 115 (identité CLI), hotfix 20261004-1401.

<a id="extrait-2"></a>

## Extrait 2

Un agent principal présent dans T3 possède une identité Bridget attestée. Un
sous-agent interne créé par son fournisseur peut aussi voir les outils Bridget,
mais il ne possède pas une identité Bridget indépendante. Son appel échoue alors
avec `identity_not_found`, ou risque d'emprunter silencieusement tous les droits
du parent si la filiation de processus suffit par hasard.

<a id="extrait-3"></a>

## Extrait 3

- **FR-13301** : reconnaître un sous-agent interne seulement si un processus
  fournisseur principal T3 possède une identité Bridget vivante et si le
  fournisseur interne se rattache sans ambiguïté à ce processus.
- **FR-13302** : conserver une preuve distincte pour le sous-agent. Cette preuve
  doit contenir la naissance du processus, l'instance du parent, le fournisseur
  et une référence enfant opaque. Elle ne doit contenir ni corps de message, ni
  secret fournisseur, ni raisonnement.
- **FR-13303** : accorder au sous-agent uniquement la consultation bornée de
  l'annuaire et l'envoi de message. Tout autre outil MCP doit être refusé avant
  son exécution avec un code stable.
- **FR-13304** : refuser l'usage de la ligne de commande Bridget depuis un
  processus reconnu comme sous-agent interne. La délégation vaut uniquement
  pour la façade MCP contrôlée.
- **FR-13305** : utiliser l'identité et l'instance du parent pour le routage,
  l'idempotence, les demandes suivies et les réponses. Ne jamais créer une
  identité Bridget durable pour le sous-agent.
- **FR-13306** : joindre à tout message émis par un sous-agent une provenance
  structurée et fermée. Le destinataire doit pouvoir distinguer ce message d'un
  message écrit directement par le parent.
- **FR-13307** : une réponse demandée par un sous-agent doit être remise au
  parent. Bridget ne doit pas promettre de réveiller ou de retrouver le
  sous-agent après sa fin.
- **FR-13308** : retirer les preuves enfant devenues périmées sans supprimer les
  preuves appartenant à un autre pont ou à un autre processus.
- **FR-13309** : refuser les rattachements ambigus, les marqueurs non privés,
  les liens symboliques, les fichiers trop grands, les PID recyclés et les
  références enfant invalides.
- **FR-13310** : préserver le comportement, les droits et le format historique
  des agents principaux, des agents lancés par Bridget et des anciens messages.
- **FR-13311** : ne pas ajouter de dépendance externe, de service résident ni
  de registre d'identités enfant durable.
- **FR-13312** : les diagnostics et journaux ne doivent exposer ni corps de
  message, ni secret, ni identifiant natif complet du sous-agent.
- **FR-13313** : l'inventaire des processus T3 doit vider la sortie de `lsof`
  pendant son exécution. Un inventaire volumineux ne doit pas bloquer sur le
  tube de sortie ni rendre toutes les identités T3 indisponibles.

<a id="extrait-4"></a>

## Extrait 4

- **SC-13301** : 100 % des essais nominaux de consultation et d'envoi depuis un
  sous-agent attesté aboutissent sans `identity_not_found`.
- **SC-13302** : 100 % des outils hors liste permise et 100 % des commandes CLI
  testées sont refusés sans effet.
- **SC-13303** : 100 % des messages enfant testés portent une provenance, gardent
  le parent comme expéditeur routable et livrent la réponse au parent.
- **SC-13304** : 100 % des cas ambigus, périmés ou non privés sont refusés.
- **SC-13305** : les suites d'identité, MCP, T3, protocole et workspace restent
  vertes. Le formatage et l'analyse statique passent.
- **SC-13306** : aucun nouveau secret, contenu libre ou identifiant natif enfant
  complet n'apparaît dans les journaux et diagnostics inspectés.
- **SC-13307** : un inventaire synthétique d'au moins 192 sessions ouvertes se
  termine en moins de trois secondes et conserve toutes les sessions.

<a id="extrait-5"></a>

## Extrait 5

Le pont T3 fournit la preuve de parenté. Un fournisseur ou un hôte sans preuve
équivalente reste refusé. La prise en charge n'est pas déduite du seul nom du
fournisseur.

<a id="extrait-6"></a>

## Extrait 6

- Créer une identité Bridget durable par sous-agent.
- Donner au sous-agent les outils de contrôle, de journal, d'artefact ou Maicie.
- Modifier T3 Code ou les fournisseurs.
- Ajouter un moteur générique de délégation ou un nouveau service.
- Déployer, fusionner ou committer automatiquement pendant ce pipeline.
