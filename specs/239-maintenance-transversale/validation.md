# Validation 239 — maintenance interprojets et administration Android

## Résultat activé le 7 novembre 2026

Bridget 0.8.6 / 239-mobile-admin-r2 est chargé sur les sept composants des
quatre hôtes. Source exécutable : 1d7e1e1416c762c933503ffc450cc6741d3889c1.
La décision humaine Android d'origine est appliquée par le port natif.
Aucune réponse humaine fabriquée, nouvelle question ou mutation SQL.

La fiche initiale avait une portée bridget seule, un mandat vide et aucun
grant. Les gardes expliquaient donc le catalogue vide depuis Revue.
configure_mandate a produit la révision 2, puis declare_roles la révision 3.
Le rôle mainteneur-bridget et le mandat ont maintenant transversal=true.
auto_start=false, durable=false et les budgets facultatifs sont conservés.
Les permissions sur les données métier des projets ne sont pas étendues.

Question conservée : a80e8ab5-c21e-4247-acc9-68b700f6a85c.
Opération conservée : a8efcad4-f4e0-4210-96b7-819d059ab5f9.
Grant : 7a6df19e-890b-4f19-aa15-69ddd115388b, révision 2, rattaché à la
réponse Android authentifiée et au digest de l'opération exacte.
La recherche native de ce fil rend la fiche révision 3, le rôle et la présence
busy/monordinateur. Depuis le contexte Codex Revue existant, search puis get
réussissent avec caller_projects=[auto-5c29175617db400dab394fa74bc97948].
La fiche révision 3, le mandat complet, le rôle transversal et la présence
observée sont visibles. Preuves natives du journal aux séquences 891 et 895.
Le premier appel de ce vérificateur omettait version=1 ; son refus -32602
était une erreur de paramètres, corrigée une fois. Aucun refus de droits.
La demande distincte au contexte Claude Revue reste remise dans son fil occupé.

## Défaut Android et correction livrée

La réponse approve était bien enregistrée. Son appareil administrateur ancien
portait exactement les sept opérations de l'ancien --all. L'ajout ultérieur
de specialists_manage empêchait l'application de cette décision.

La compatibilité reconnaît uniquement la portée globale complète historique,
sur les appareils créés avant l'introduction de specialists_manage
(created_at entre 1 et 1793532130 inclus). Deux associations Android actives
correspondaient à cette définition avant activation. Les nouvelles associations,
portées partielles, listes incomplètes ou dupliquées ne sont pas promues.
Les appareils explicitement autorisés gardent leur parcours existant.
La preuve humaine, la portée, l'expiration et la révocation restent vérifiées.
La liste persistante des droits et les clés du téléphone ne sont pas réécrites.

Il s'agit d'une extension explicite et bornée de compatibilité administrative.
La décision, la limite d'horloge et la sortie lors d'une réassociation,
révocation ou expiration sont documentées dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/docs/decisions/239-administration-mobile-historique.md.
Le refus d'un appareil réellement non administrateur explique désormais
pourquoi recliquer ne lui attribue pas le droit manquant.

## Vérification utile et revue

Quatre cas distincts acquis sur Linux : trois tests spec239 et une régression
spec222 sur la réponse humaine native et l'opération exacte. Ils couvrent la
frontière historique, les refus des associations nouvelles ou partielles,
l'application native/idempotente, la révocation, l'expiration et la mutation
mobile directe. Les répétitions de compilation ne sont pas recomptées.
Assemblages finaux macOS/Linux réussis. Aucun changement de code depuis.

Revues e78c14cd-5d3c-49f4-98a0-6a86d5889b7b puis
7548c7dd-c8b8-410e-8a89-06d14c024f04 terminées, résultats lus et ACK acquis.
La seconde accepte le correctif sans objection bloquante. Les réserves de la
première sont traitées par le bornage, l'inventaire et les tests des mutations.
Le relecteur final n'a pas exécuté les tests ni lu leur dernier ajout : les
quatre résultats auteur constituent leur preuve. La limite d'affichage des
sept droits stockés, distincts du droit effectif compatible, reste documentée.
Correction technique : aucun changement d'interface ni recette visuelle inventée.

## Activation, publication et lien natif

Empreintes des exécutables réellement chargés :

- macOS : 7d10517901b12e227a952e1f57cce02b2ef932bcb9d8834b2c94870fed3839a0.
- Linux : aaabd11c31ddea2c9605be0537913424373c4f6f971882aa7301eeabe7b44346.

PIDs au contrôle : core Cartae 975637, mobile 975691, worker Cartae 977340,
SYS 2504378, KS 3787728, worker Mac 68901, pont Mac 69016.
Coupure du cœur : 1,97 s. Superviseur métier Cartae 2500061 et naissance
270585460 conservés. Registre worker Mac dédié conservé, empreinte
cacf9ba06704faf0699e13a6827f556d799282f96fda02b6e4dfb643e8667374.
Retour 237-local-project-r2 compatible ; pointeurs rollback écrits atomiquement
sous les verrous natifs sur les quatre hôtes, aucune activation concurrente.

Release publique v0.8.6, six actifs téléchargés : tailles et SHA tous vérifiés,
exécutables des deux archives identiques aux images chargées. APK 8006 signé,
certificat de production et lignée comparés à 8005. Aucun test UI/FCM annoncé.
Les quatre rapports natifs sont 0.8.6, up_to_date et fresh au contrôle final.

T3 n'a pas été relancé : ses sept processus et dates de démarrage sont identiques.
Pendant le retour du pont, primary_absent puis binding_transport_unavailable
et des délais JSONL ont été observés. Aucun mandat n'a été annoncé appliqué
sur ces seuls refus. La sonde Mac a ensuite répondu en 0,495 s ; les ports
natifs d'attestation, mandat, rôle, annuaire, journal et envoi ont réussi.
La cause précise des délais transitoires n'est pas établie. Aucun changement
de compte, permission ou mission n'a servi de repli.

Les façades MCP encore ouvertes en 237 sont conservées avec leur image ; de
nouvelles façades 239 sont présentes. Le rechargement complet du client reste
manuel, selon le choix explicite de François. Cette configuration et le
correctif serveur sont déjà appliqués sans ce redémarrage.

## Conservation, avis et nettoyage

Payloads 7024f34c, 77ba483e et 702d272e identiques à l'état avant maintenance.
2fb et les revues métier T3 restent conservées ; aucun rejeu métier.
Dix préavis admis, dix refus de transport conservés sans rejeu aveugle.
Dix-neuf avis de reprise après activation et les demandes ciblées de lecture
Revue ont un reçu. Ces reçus n'attestent pas la lecture par leurs destinataires.

Nettoyage de cette livraison : 19 fichiers de transfert ou doublons publics
retirés, 260843356 octets logiques. Les preuves, la copie originale de release,
l'actif et le retour sont conservés. Entretien installed exécuté sur Mac et SYS.
Mac : 0,52 Gio de cache sous budget. SYS : nettoyage différé par le contrôle
natif car l'observation privilégiée des processus est indisponible ; aucun
contournement. Son worktree de compilation propre et les anciens espaces
non qualifiés restent conservés. Le worktree Cartae 1cc44f4d reste modifié et
utilisé par le superviseur 2500061. Les vues de revue sous rétention restent
conservées. Ne pas supprimer ces travaux pour atteindre artificiellement zéro.

Preuves privées et reçus :
/Users/moi/.cache/bridget239-maintenance-transversale.
