# Diagnostic de la remise V4 et du catalogue des rôles

Contrôles du 2 novembre 2026 après le chargement du nouveau serveur.
Signalement reçu : psych189-signalement-remise-maintenance-20261102-01.
Projet source : /Users/moi/Nextcloud/10.Scripts/69.opus2D.
Aucun rejeu métier, aucune mutation de droits et aucun changement de service.

## Remise historique

Clé psych189-admission-introspection-v4-20261102-01, issued_at 1793584534.
Lecture seule du maître : les reçus ecfb837e-97d0-412d-81da-ad0a351c33a1 et
73f60565-0f40-43cf-82c1-fcbfc6d35d5e sont tous deux indeterminate, acked_at null.
Corps identiques, SHA-256 cc695855ed14e78ffc35c60b82a7aa072117d6edfeb714d16fa5822215e8fcb2.
Le premier vient de psychologie (db1ed92d-f23d-4e3e-8a94-f20bbf9b52b3), le second
de Demander le redémarrage du monde (8bcb83ac-e589-48c4-8097-a5acdb88953b).
Ils ont donc deux issuer_scope distincts. Même clé, instant et corps depuis un
autre agent ne constituent pas un rejeu strict de l’expéditeur initial.
Ne pas créer un troisième envoi ni transformer ces reçus en accusés.

Journal du pont : début de remise à 01:55:39 UTC, réponse RPC interrompue à
01:55:51 UTC, fermeture des liens après expiration de snapshot à 01:56:01 UTC.
Ces faits précèdent les corrections 226 et le nouveau paquet de lecture.
La transaction exacte de cet appel historique n’a pas été chronométrée ; ne pas
présenter la lenteur SQL comme une cause individuelle prouvée pour cet appel.

Dans la base active T3, aucune ligne de reçu pour la clé exacte dans les deux
tables de reçus contrôlées ; aucun message utilisateur contenant la clé dans
le fil e4999bf4-8f05-401b-bdc2-a9bb32b0b9f7. Cette dernière lecture prend 0,118 s.
L’absence observée ne requalifie pas les deux issues indéterminées et n’autorise
aucune réinjection automatique. Le propriétaire garde la décision de reprise
métier après rapprochement de ses états. Aucun succès V4 annoncé.

## Catalogue et mandat

Le catalogue natif répond. Le mainteneur est présent et occupé. Son enregistrement
porte projects=[bridget], transversal=false, mandate vide, revision=1.
La liste depuis Opus2D masque donc ce spécialiste et get refuse role_scope_denied.
La présence globale ne donne aucun accès supplémentaire au catalogue.
Le code List/Get filtre ce périmètre : refus conforme, pas panne du catalogue.

La demande humaine native b3ec1c56-e2ea-4c56-b746-a2d2f6a4bd8c reste pending.
Elle porte exactement configure_mandate, clé mainteneur-bridget-transversal-20261102-01,
revision attendue 1, mandat Maintenance de Bridget pour tous les projets,
transversal=true, auto_start=false. L’accord dans le fil n’est pas un reçu de
cette garde native ; aucune validation humaine simulée ni écriture SQL.
Après son approbation réelle, relire le mandat et déclarer le rôle transversal
sous le grant approuvé. ConfigureMandate seul n’étend pas la portée du rôle :
specialist_candidates vérifie les deux portées. Aucun nouveau lancement requis.

Étape humaine disponible : ouvrir les questions de Bridget sur le client natif
et approuver cette proposition existante. Consultation terminal :
/Users/moi/.local/bin/bridget questions show b3ec1c56-e2ea-4c56-b746-a2d2f6a4bd8c.
La CLI answer exige un terminal humain ; l’agent ne doit pas lui emprunter cette
autorité. Les autres travaux autorisés suivent leur propre état.
