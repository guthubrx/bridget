# Revues 177 — traitement des retours antérieurs

État au 24 octobre 2026, main 7669d8bd. Relecture ciblée du code courant ;
aucun nouvel audit V15 global et aucune nouvelle activation déduits de ce bilan.
Les résultats reçus tardivement citent un ancien worktree désormais retiré.
Ils ne remplacent pas les faits du code actuel ni les recettes physiques ouvertes.

## Préparation d’origine — ff277e40

Les quatre corrections demandées sont présentes :

- origin_proof.rs refuse le lien absent avec project_preparation_link_missing.
- daemon/origin_proof.rs borne le budget à 595 s sous le plafond de 600 s.
- native_delegation.rs partage une échéance unique sur toutes les machines.
- La relecture finale compare encore host_for_owner à l’origine retenue.

Le traitement et le lot de tests antérieur figurent déjà dans migration.md.
La question de parité d’instance est conservée comme limite à examiner, sans
assouplir les droits à partir d’une seule observation statique.

## Liaison SSH — d8b64194

Le runner attend un premier octet par MSG_PEEK avant de lancer SSH. Une sonde
vide de la socket ne lance donc pas une session SSH vers le maître.
La garde locale du runner précède le préflight distant dans la direction
to-master. Le test réel du relais vérifie deux échanges espacés et la sonde vide.

Pour la réutilisation d’une liaison, le choix produit est explicite : sans
--direction, conserver la configuration enregistrée et afficher son sens.
Une direction explicite divergente est refusée. Le défaut supposé de sens par
défaut ne justifie donc pas de remplacer une liaison existante par from-master.
Cette règle figure dans l’aide CLI et les deux guides mobiles.

Restent des limites de qualification : chemins du runner avec un HOME contenant
des espaces, fin de tous les descendants sous chaque gestionnaire de service,
et justification mesurée de la borne de 1024 canaux du relais. Ces points ne
sont pas annoncés comme couverts par une recette physique complète.

## Frontière mobile — 675e8aee

Le quota global de bootstrap n’existe plus dans le code courant :

- Association : quota par adresse fournie par le proxy loopback, qui remplace
  l’en-tête client. Aucun en-tête arbitraire ne devient une identité d’appareil.
- Renouvellement : sujet authentifié par renewal_subject avant quota par appareil.
  Une clé invalide ne consomme pas le quota d’un autre appareil.
- Follow refuse la portée d’historique insuffisante avant la lecture native.

Les tests ciblés HTTP couvrent l’isolement des quotas. Ils ont été exécutés dans
le lot mobile177 de livraison ; aucun nouveau lot complet n’est nécessaire pour
ce retour ancien. Le risque de saturation des 32 connexions est une limite
connue, pas une preuve de fuite d’autorité.

La proposition de passer uniquement un Device validé par HTTP au daemon n’est
pas retenue : les relectures natives protègent contre la révocation et les
changements de portée à leur propre frontière. Aucune mesure ne justifie de
retirer cette protection pour économiser une ouverture SQLite.

## Projet distant et garde humaine — ec6d4c8a

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/177-bridget-mobile/validation/audit.md#extrait-1).

Le déplacement du coordinateur doit préserver le droit de déléguer des parents
existants. Franchir la garde de projet ne vaut pas autorisation de lancement :
les attestations, droits hérités, contrôles de source et permis de remise restent
obligatoires. Aucun retour à une garde strictement locale n’est appliqué, car
cela casserait le parcours de migration autorisé sans corriger une exploitation
démontrée. Le modèle de confiance du même compte SSH reste une limite documentée.

## Statut des gates

Les revues ont produit des corrections et des limites, pas un certificat de
sécurité global. T039 reste ouverte pour l’audit applicable complet. T030, T033
et T036 restent ouvertes pour les recettes physiques et réseau. La dernière
mise à jour Android r10b attend un téléphone de nouveau visible en USB.

## Complément r10c

La revue ciblée 7903c2f3-aa56-4b6d-a18e-31cf4548219c relève deux défauts de
bornage des détails d’outil. Leur correction et la preuve par le vrai journal
figurent dans r10c.md. Les 67 tests ciblés passent ; aucune nouvelle suite
complète ni approbation globale n’en est déduite.
