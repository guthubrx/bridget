# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/156-guidage-et-lecture-de-file.md](../../../../../decisions/156-guidage-et-lecture-de-file.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le protocole porte déjà SteerCurrent, InterruptAndStart et Interrupt. Les
façades ne les exposent pas et plusieurs chemins échappent aux droits entre
agents. Une origine Human cliente peut aussi provoquer une interruption.
La CLI sans preuve peut signer human. Le pont T3 peut guider implicitement un
pair et convertir un steer Cursor en interrupt_restart. CancelDelivery ACP
peut interrompre un tour actif alors que l’outil décrit une annulation de demande.

<a id="extrait-2"></a>

## Extrait 2

Ajouter seulement la lecture bridget_queue/CLI queue --target, sous QueueListV1.
Projection send_deliveries idempotent dispatching :200 entrées maximum,
preview120 caractères, état pending_or_injecting, borne64 T3/null sinon.
Destinataire/parent/Attach/UI Client attestée lisent toute la projection ; autres : leurs messages.
Attach négocie ClientHello uniquement avec la liste exacte [QueueListV1].
Son rôle reste Attach, sans identité agent/conn_names ni autres capacités.
L’UI Client attestée peut lire toute la projection ; elle reste Client et
n’acquiert aucun droit de contrôle explicite par cette lecture.
ChildProof est refusée. Ni Send historique ni files internes n’entrent dans la
vue. La lecture n’écrit rien. LIMIT200 ne borne que la sortie ; le scan coûte
O(log n+s) plus jointures indexées, s candidats examinés incluant les expirés.

<a id="extrait-3"></a>

## Extrait 3

Les trois choix de la spec sont retenus : pairs sans contrôle, édition reportée,
pairs T3 occupés en attente et CLI nue sans contrôle. Root accepte également la
fermeture Cursor et REST/V1. Aucune nouvelle commande de supervision, droit humain
ou configuration fournisseur n’est ajouté. Les tests prévus156 devront prouver
les gardes et la compatibilité ; leur succès n’est pas acquis par cette ADR.
