# language: fr
Fonctionnalité: Fiabilité des communications Bridget 099
  Les agents restent indépendants et les demandes conservent une issue honnête.

  Scénario: Un destinataire lent ne bloque pas l'annuaire
    Étant donné un daemon isolé et un agent qui ne lit plus sa socket
    Quand un message remplit sa file de sortie
    Alors dix consultations d'annuaire et un échange témoin aboutissent en moins d'une seconde chacun
    Et la remise en échec obtient une issue explicite en moins de deux secondes

  Scénario: Une réponse immédiate rejoint sa demande
    Étant donné deux agents enregistrés dans une instance isolée
    Quand le destinataire répond immédiatement à une demande suivie
    Alors la demande devient répondue sans repasser à ouverte

  Plan du scénario: Les identifiants publics n'autorisent pas une identité
    Étant donné un agent propriétaire connecté et un autre client
    Quand l'autre client tente <voie> sans preuve privée valable
    Alors aucun message ne porte l'identité du propriétaire
    Exemples:
      | voie |
      | inscription MCP auxiliaire |
      | client CLI avec from_declared faux |
      | ClientHello puis SendIdempotent |

  Scénario: La fin de l'incarnation révoque un auxiliaire
    Étant donné un auxiliaire muni d'une preuve valide
    Quand son propriétaire se déconnecte et se réinscrit
    Alors l'ancien auxiliaire et son ancienne preuve sont refusés
    Et un auxiliaire utilisant la nouvelle preuve peut communiquer

  Plan du scénario: Aucun démarrage différé après perte d'autorisation
    Étant donné un fil t3code occupé et une demande en attente
    Quand survient <événement> avant le démarrage
    Alors le faux fournisseur ne reçoit aucun démarrage pour cette demande
    Et aucun rappel ni contrôle ne devient un tour fournisseur supplémentaire
    Exemples:
      | événement |
      | une annulation attestée |
      | l'expiration du délai |
      | la perte du daemon |

  Scénario: Une réponse survit à la déconnexion et à la reprise
    Étant donné une réponse fournisseur prête et son destinataire déconnecté
    Quand le pont redémarre puis le destinataire revient avant échéance
    Alors la réponse liée est remise sans relancer le tour fournisseur
    Et les autres attentes du fil sont toujours présentes

  Scénario: Le journal ne coupe pas silencieusement un texte long
    Étant donné une sortie de dix mille caractères Unicode
    Quand elle est projetée dans le journal Bridget
    Alors le texte est intégral ou une lacune est explicitement visible
    Et un refus d'écriture ne fait pas avancer silencieusement le repère

  Scénario: Une publication refusée reste à reprendre même sans nouveau message
    Étant donné un journal ayant refusé une publication
    Quand le journal redevient disponible et le fil reste inchangé
    Alors la publication est retentée dans son ordre initial

  Scénario: Une réponse HTTP illisible ne prouve pas un refus fournisseur
    Étant donné une commande acceptée par le faux fournisseur
    Quand son reçu HTTP 200 est malformé
    Alors la corrélation durable reste présente pour retrouver la réponse
