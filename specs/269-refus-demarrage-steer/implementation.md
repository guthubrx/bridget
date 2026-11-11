# 269 : refus de démarrage, steer et cache de compilation

## Incidents vérifiés

- SYS : aucune compilation encore active au contrôle. Cache partagé à22Gio physiques, disque libre1,1Gio. Le cache existant est traversable et sudo lsof demande une authentification ; le repli limité au compte ne s’applique donc pas. Après saisie sécurisée du mot de passe et lecture administrateur, le nettoyeur natif inchangé, sous verrous Cargo, a retiré27658693770octets logiques. Espace libre :1217060864 ->23992938496octets. Aucun agent Parix arrêté. Le secret n’est ni dans Git, ni dans les arguments, ni dans les journaux.
- Deux refus de démarrage : c06bd9f9 et ca2dcf0b. Le reçu Fleet conserve quota_exceeded avec limite24 ; le parcours natif a remplacé ce fait par remote_spawn_not_new et laissé queued sans échéance. Aucun Start ni mission remis. Les propriétaires ont depuis annulé les tâches ; aucune reprise ni nouvelle clé par la maintenance. Leurs worktrees restent conservés et sans rapport natif, car aucune instance/lien de commande n’a été créée.
- Deux steer Claude :8aa5f1bd et d2907ee3 restent dispatching, aucune injection journalisée. Le pilote refuse le steer strict, puis le wrapper ignore un DeliveryRejected non lié à une exécution ni à un message ordinaire actif. Une consigne queue attend sa fin de tour ; aucun contrôle intermédiaire sans interruption n’est pris en charge par ce pilote.
- Les quatre steer Codex comparés sont déjà acked. Les journaux portent prompt_dispatched aux quatre heures transmises au responsable. Le premier reçu in_flight est asynchrone ; la remise ne prouve pas que le modèle suit la consigne.

## Correction du cache

Un cache neuf était créé par Cargo avant son enregistrement, avec des droits publics par défaut. Le script crée maintenant le seul staging autorisé en0700 et l’enregistre avant Cargo. Les droits des caches préexistants ne changent pas. Le test neuf échoue avant correction ; les71tests du nettoyeur passent ensuite, dont3ignorés pour les conditions de plateforme déjà prévues. Aucune garde de suppression n’est retirée. Les prochaines qualifications utilisent explicitement CARGO_INCREMENTAL=0 pour éviter l’accumulation mesurée de20Gio logiques d’incrémentaux sur SYS.

Preuves privées : /Users/moi/.cache/bridget269-incidents. Retours conservés sans réveil : mcp-86189-6af48954-1b, mcp-86189-6af48ac6-1c, mcp-86189-6af48ac8-1d, mcp-86189-6af48bda-1e et mcp-86189-6af48c46-1f. Une conservation ne vaut pas lecture.

## Refus et conservation

Les deux régressions initiales échouent sur le runtime 268 : la demande de
lancement reste queued et le rejet corrélé ne produit aucun reçu de livraison.
Les quatre nouveaux tests passent après correction. Le contrôle de conservation
est ensuite étendu à la vue de lecture : le rapport reste indisponible et la
clôture exige toujours son accusé worker. L’état des tâches annulées et leur erreur
historique ne sont pas réécrits. Leur lecture affiche le refus Fleet corrélé.

La revue initiale 8db01550 confirme les deux causes. Les objections sont traitées :
Failed seul, transport de la définition figée, aucune reprise de commande terminale,
rejet seulement pour la tête d’une livraison idempotente en attente, aucun faux ACK
ou événement de fin de tour. Résultat lu, ACK, garde close, vue conservée.
La contre-revue 6f134d15 reproduit le quota de flotte24 avant démarrage. Commande
native failed/quota_exceeded, sans Start ni mission ; demande annulée par son
propriétaire après lecture du reçu, aucune alternative ni modification du plafond.
Elle n’a donc produit aucun avis. Sa vue de lecture reste à conserver par le
worker, ce qui a révélé le second cas couvert par le correctif.

La qualification regroupe les filtres dans une seule commande Cargo et désactive
les fichiers incrémentaux. Les fixtures de CLI appellent directement la bibliothèque
ou créent leur socket, sans le point d’entrée qui pose normalement umask077.
Un contrôle élargi lancé avec umask022 a constaté des sockets0755 et a échoué à
raison sur les gardes du namespace. La qualification utilise désormais umask077,
comme le point d’entrée natif, et construit d’abord l’exécutable debug requis.
La tentative de canonicaliser le chemin temporaire n’était pas la cause : elle
est retirée. Aucune garde ni permission produit n’est changée pour les tests.
