# Revue indépendante R1 et décisions

Source relue : da8706a71e6d7c0099e3dd6a0a3b7df36c4b53d3.
Tâche fa727643-e0e1-4493-bf9b-ea549d78c691, lecture seule, aucun test exécuté
par le relecteur. Résultat conservé dans le cache privé de livraison.

1. Réveil par message T3 : objection réfutée par la source du client.
   `apps/server/src/orchestration-v2/Orchestrator.ts` remet `settledOverride`
   à `null` lors de dispatchMessage. La restauration explicite emploie `active`.
   L'adaptateur traduit null par aucune action, déjà couvert par le test natif.
   Le relecteur indique ne pas avoir lu cette source externe à la vue.
2. Désactivation pendant une commande incertaine : conservation intentionnelle.
   Effacer cette commande perdrait son issue. Le rapprochement garde exactement
   son identifiant ; aucun nouvel ordre ne part après désactivation. Nouveau
   test ciblé et précision dans la documentation de l'adaptateur.
3. Rattrapage programmé après sommeil : défaut confirmé et corrigé. Une
   occurrence due pendant le sommeil est figée puis close sans départ. La
   prochaine échéance reste programmée. Test sur le vrai tick et une socket.
4. Lecture groupée du sommeil : repli limité à une lecture de chaque fiche.
   Seules les fiches illisibles diffèrent leurs rappels. Aucun repli ouvert
   n'autorise la sollicitation d'un dormant dont l'état n'est pas lisible.
5. Filtre Android Tous : ajouté. Réglage mobile de synchronisation : déjà
   présent dans le menu de fiche (`changeTracking(dormant,!synchronize)`),
   contrairement à l'objection de lecture. Circuit visible à qualifier.

Contrôle complémentaire auteur : conservation atomique d'une réponse classique
à une demande déjà admise lorsque le destinataire dort. Corps dans le journal,
reçu answered, aucun nouveau tour. Identité, paire de correspondants et état de
la demande contrôlés ; une réponse étrangère ou contradictoire ne la remplace
pas. Les résultats natifs déjà canoniques restent capturés sans notification.

Acquis après corrections : 17 tests ciblés verts, puis 2 tests anciens de
liaison attestée verts. Le test de liaison valide accepte la seule nouvelle
métadonnée agent_tracking ; les refus de liaison gardent leur comparaison
stricte de toutes les tables. Assemblage Android 0.8.0 vert après ajout Tous.
Activation et preuves UI restent ouvertes. Ancien retour 0.7.8 incompatible
avec le marqueur 8 : un retour compatible 0.8 est requis avant toute activation.

Clôture de la revue : appel natif cancel a perdu sa réponse (budget JSONL).
Lecture privée de maintenance confirme cleanup_done=true et l'enfant arrêté
par le parcours natif. Vue et résultat conservés. Accusé informatif non confirmé,
aucun rejeu de mission. Incident de transport conservé séparément.
