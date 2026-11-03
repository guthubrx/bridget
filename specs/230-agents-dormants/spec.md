# Session 230 — Agents dormants

État : en cours. Autorisation : François, 3 novembre 2026.

## Besoin

Garder un annuaire utile sans perdre l'identité ni les travaux des anciens
agents. « Mettre en sommeil » déplace une fiche dans l'historique et empêche
les nouvelles sollicitations. « Réveiller » la rend disponible sans démarrer
un tour. Le nom anglais est « Sleeper agent » ; actions « Put to sleep » et
« Wake up ». Le sommeil est indépendant de la présence réseau et de la fin
d'une mission.

## Contrat accepté

- L'état natif persiste par identité d'agent et se partage entre appareils.
- Les nouvelles sollicitations directes, collectives, programmées et les
  nouvelles missions adressées à un spécialiste dormant sont refusées ou
  exclues de la sélection. Le motif explique comment réveiller la fiche.
- Une émission de groupe reste lisible dans son fil. Aucun réveil différé
  n'est accumulé pour un membre dormant. Les destinataires écartés sont visibles.
- Une mission déjà admise, ses reçus, ses résultats et les questions humaines
  restent conservés. Dormir ne signifie ni arrêter, ni terminer, ni approuver.
  Les retours de missions existantes restent collectés. Un résultat ne lance
  pas un nouveau tour chez un destinataire dormant.
- Un message, une pulsation, une reconnexion ou un résultat ne réveille pas
  automatiquement l'agent. Le réveil est une action explicite.
- Le droit de modifier cet état utilise les contrôles humains natifs et
  l'appairage mobile existants. Aucun champ déclaratif « humain » ne suffit.
- Les listes proposent Actifs / Dormants / Tous. L'historique est paginé au
  serveur. Les caches et favoris Android respectent cet état.
- Les rôles et les données restent conservés ; aucun effacement automatique.

## Synchronisation des clients liés

La synchronisation est **bidirectionnelle par défaut**, décision explicite de
François. Un réglage permet de conserver l'organisation propre à Bridget.
L'adaptateur utilise seulement les fils déjà liés et conserve ces liens après
fermeture de leur présence. Pas d'import de tout l'historique du client.

Un rangement explicite dans le client met l'agent en sommeil. Une restauration
explicite le réveille. Une nouvelle sollicitation automatique dans le client
ne doit pas être interprétée comme un réveil humain. La mise en sommeil native
est propagée au fil exactement lié ; ses effets locaux sont ceux du contrat
du client. Un client indisponible laisse la synchronisation en attente, sans
annuler le choix natif. Les reprises gardent le même identifiant d'opération.
Les échos, les instantanés répétés et les événements anciens ne créent pas de
boucle et n'écrasent pas un choix plus récent.

L'adaptateur ne peut pas empêcher un envoi qui contourne Bridget. La portée
de cette limite et les éventuelles modifications nécessaires dans le client
doivent être établies par lecture du chemin réel, sans garantie inventée.

## Acceptation

1. Dormir et réveiller depuis la CLI et Android, avec état persistant.
2. Refus direct explicite, groupe sans notification du dormant, envoi aux
   autres membres conservé, aucun rattrapage automatique au réveil.
3. Rôle dormant non sélectionné ; mission antérieure et résultat inchangés.
4. Synchronisation dans les deux sens, répétition sans effet et client absent.
5. Ancien état absent des caches actifs Android, historique accessible.
6. Revue indépendante, preuves UI réelles, sources intégrées et poussées,
   publication et activation vérifiées sur les cibles, nettoyage qualifié.

## Exclusions

Aucun arrêt de mission ou de fournisseur ; aucun rejeu métier ; aucune
suppression de données ; aucune création d'agent pour tester la présence.
La tâche R56 7024 et la revue métier T3 distincte sont strictement préservées.

## Faits de conception

Le client T3 remet un fil settled en activité lors de message.dispatch
(Orchestrator.ts, dispatchMessage). Settle seul n'interdit donc pas les
sollicitations. Cette différence impose de distinguer le sommeil natif des
simples changements de visibilité et de vérifier l'origine du réveil.
