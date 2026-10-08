# Contrat — ajout de membres US147-08

Statut : plan et gate5/5 acceptés, deux passes Analyze principales sans finding CRITICAL. NON installé/NON activé. Tests T042–T048 ouverts.

## Surface

Étendre ThreadAction par AddMembers, action MCP add_members et commande CLI add-members. Utiliser références thread/members et UUID d'opération existants, UUID normalisés avant hash/SQL. Capacité ClientCapability::ThreadMembersV1, wire thread_members_v1 via serde snake_case, figée par le principal avant code ; test de sérialisation obligatoire. Négociation unique avec CommunicationProjectsV1 dans ClientHello auxiliaire. Ancien peer non capable refuse sans mutation ; daemon vérifie capacité attestée.

Syntaxe CLI réutilisant l'option existante : `bridget thread add-members FIL --member UUID --member UUID --id UUID [--cross-project-reason MOTIF]`. Aucun nouveau --members. La façade MCP utilise action add_members et membres selon le schéma existant étendu.

## Autorisation et transaction

Créateur initial seul, fil ouvert. Union complète anciens+candidats validée par garde projet avant DB ; transaction relit et compare cette union autorisée, créateur et état, puis refuse divergence concurrente. Maximum16 après normalisation/dédoublonnage. Entrées storage bornées entries+16. Pas retrait/transfert.

NoChange : tous candidats déjà présents, aucun nouveau reçu ni engagement de clé. Vraie addition : résultat idempotent durable, replay sans nouvelle appartenance. Migration ciblée schéma filv2→v3 change CHECK de thread_operations, conserve anciennes opérations, résultats/reçus/index et relations. Tester aussi v1→v3 via fixtures136 existantes, seconde ouverture sans effet et rollback atomique, reçus exacts conservés. Aucun corps/kind/activité réécrit.

## Lecture et silence

Chaque nouveau membre reçoit curseur0 et accès à tout l'historique autorisé, même antérieur. read et history gardent leurs sémantiques ; history conserve les corps exacts/supersessions. Aucune ancienne consigne rejouée.

L'adhésion ne crée aucune entry/alerte/mission. AddMembers n'entre pas dans mutates/dispatch_thread_wakes, même avec un wakepending préexistant : ce pending reste inchangé. Futur notify=all inclut le nouveau membre ; anciens targets effectifs inchangés. Invalidation humaine autorisée seulement après commit de vraie addition ; aucun signal sur replay/refus/NoChange, aucun body dans le watch.

## Preuves obligatoires

CLI/MCP vrais avec hello auxiliaire combiné ; garde capacité ancienne/nouvelle ; creator/closed/max16/invalides/unionprojet/concurrence ; migration préexistante et replay/NoChange ; historique multipage complet/bodyexact ; cursor0/wakepending nondispatch ; all futur/targets passés ; watcher postcommit silencieux hors changement. Aucun modèle, identité fabriquée, processus actif ou publication globale.
