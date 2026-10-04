# Spécification 133 — Relais Bridget pour les sous-agents internes

**Branche** : session-133-relais-sous-agents
**Date** : 2026-10-04
**Statut** : Implemented
**Priorité** : P1
**Demande** : reprendre le pipeline `$my-specify-all` commencé pour permettre aux sous-agents de communiquer par Bridget.
**Dépendances** : 064 (liens parent-enfant), 090 (agents interactifs), 098 (pont T3), 099 (preuve auxiliaire), 101 (identité T3), 114 (joignabilité), 115 (identité CLI), hotfix 20261004-1401.

## Pourquoi

Un agent principal présent dans T3 possède une identité Bridget attestée. Un
sous-agent interne créé par son fournisseur peut aussi voir les outils Bridget,
mais il ne possède pas une identité Bridget indépendante. Son appel échoue alors
avec `identity_not_found`, ou risque d'emprunter silencieusement tous les droits
du parent si la filiation de processus suffit par hasard.

Le résultat attendu est un relais borné. Le sous-agent parle au nom du travail
du parent, avec une provenance explicite. Il ne devient pas un agent durable. Il
ne reçoit pas les droits d'administration du parent. Les réponses restent
routées vers le parent, qui conserve la responsabilité de la conversation.

## Scénarios utilisateur et tests

### US1 — Un sous-agent contacte un autre agent (P1)

Étant donné un sous-agent interne rattaché sans ambiguïté à un fil Bridget,
quand il consulte l'annuaire puis envoie un message, l'appel aboutit sans
`identity_not_found`. Le destinataire voit l'identité du parent et la provenance
du sous-agent. Une réponse attendue revient au parent.

Test indépendant : créer un fournisseur principal et un fournisseur interne
attestés par le même arbre de processus, puis appeler l'annuaire et l'envoi depuis
le fournisseur interne. Vérifier le routage, la provenance et le retour au parent.

### US2 — Une délégation ne donne pas tous les droits du parent (P1)

Étant donné le même sous-agent, quand il demande un outil autre que l'annuaire
ou l'envoi, Bridget refuse l'appel avec un code stable et la liste des actions
permises. Le même processus ne peut pas contourner la règle avec la ligne de
commande Bridget.

Test indépendant : tenter renommage, annulation, lecture de journal, publication,
contrôle, Maicie et envoi CLI depuis le processus interne. Aucun effet ne doit
être produit.

### US3 — Aucun rattachement ambigu ou périmé (P1)

Étant donné un processus interne qui correspond à zéro ou plusieurs parents,
ou dont la naissance a changé, quand il appelle Bridget, l'identité est refusée.
La disparition du parent retire la délégation au cycle d'inventaire suivant.

Test indépendant : exercer absence de parent, deux correspondances, recyclage
de PID, marqueur périmé et redémarrage du pont. Aucun rattachement n'est deviné.

### US4 — Les agents existants ne changent pas de comportement (P1)

Étant donné un agent principal, un agent lancé par Bridget et un appel hors de
tout agent, quand ils utilisent Bridget, les deux agents conservent leurs outils
actuels et l'appel non attesté reste refusé. Les messages historiques restent
lisibles sans nouvelle métadonnée.

Test indépendant : rejouer les contrats d'identité, de protocole et de remise
existants, puis relire une ancienne enveloppe de message.

## Exigences fonctionnelles

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

## Entités

- **Parent Bridget** : agent principal et instance déjà attestés.
- **Sous-agent interne** : fournisseur imbriqué créé par le fournisseur du
  parent, sans identité Bridget indépendante.
- **Preuve de délégation locale** : rattachement privé et temporaire entre le
  processus interne et l'instance du parent.
- **Provenance enfant** : métadonnée fermée qui indique qu'un message vient
  d'un sous-agent, avec fournisseur et référence opaque.

## Critères de succès

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

## Hypothèses et limites

Le modèle de confiance reste local, coopératif et mono-utilisateur. Cette
fonction ne protège pas contre un processus hostile qui contrôle déjà le compte
système.

Le pont T3 fournit la preuve de parenté. Un fournisseur ou un hôte sans preuve
équivalente reste refusé. La prise en charge n'est pas déduite du seul nom du
fournisseur.

Le parent reste responsable de la conversation. Le sous-agent ne possède pas
de boîte de réception Bridget, ne figure pas dans l'annuaire et ne reçoit pas
directement les réponses.

## Hors périmètre

- Créer une identité Bridget durable par sous-agent.
- Donner au sous-agent les outils de contrôle, de journal, d'artefact ou Maicie.
- Modifier T3 Code ou les fournisseurs.
- Ajouter un moteur générique de délégation ou un nouveau service.
- Déployer, fusionner ou committer automatiquement pendant ce pipeline.
