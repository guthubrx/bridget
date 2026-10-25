# Spécification 133 — Relais Bridget pour les sous-agents internes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/spec.md#extrait-1).

## Pourquoi

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/spec.md#extrait-2).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/spec.md#extrait-3).

## Entités

- **Parent Bridget** : agent principal et instance déjà attestés.
- **Sous-agent interne** : fournisseur imbriqué créé par le fournisseur du
  parent, sans identité Bridget indépendante.
- **Preuve de délégation locale** : rattachement privé et temporaire entre le
  processus interne et l'instance du parent.
- **Provenance enfant** : métadonnée fermée qui indique qu'un message vient
  d'un sous-agent, avec fournisseur et référence opaque.

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/spec.md#extrait-4).

## Hypothèses et limites

Le modèle de confiance reste local, coopératif et mono-utilisateur. Cette
fonction ne protège pas contre un processus hostile qui contrôle déjà le compte
système.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/spec.md#extrait-5).

Le parent reste responsable de la conversation. Le sous-agent ne possède pas
de boîte de réception Bridget, ne figure pas dans l'annuaire et ne reçoit pas
directement les réponses.

## Hors périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/spec.md#extrait-6).
