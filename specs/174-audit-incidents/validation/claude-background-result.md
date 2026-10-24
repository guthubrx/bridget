# Résultat Claude après les tâches d’arrière-plan

Correctif du 24 octobre 2026, dans le périmètre des incidents 174.

## Défaut établi

Sur une mission distante réelle, Bridget a publié la réponse provisoire à
07:26:47 UTC. Le calcul Bash s’est terminé à 07:27:37, le commit applicatif
à 07:28:06 et le vrai rapport Claude à 07:28:36. Les reçus privés, le journal
Bridget et la conversation fournisseur concordent. Aucune mission n’a été
relancée pour le diagnostic. La ronde suivante engagée par le parent continue.

Le transport clôturait toute mission dès un `result` Claude réussi. Il ne
suivait pas les tâches du fournisseur. Le résultat et le rapport Git sont
ensuite figés par ronde : le dernier rapport ne pouvait plus les remplacer.

## Référence T3 et correction

Lecture de `ClaudeAdapter.ts` (événements `task_started`, `task_updated`,
`task_notification`), `ThreadBackgroundLiveness.ts` et `ThreadSettlementPolicy.ts`
dans T3. T3 sépare la fin du tour de la fin de ses tâches et suspend la clôture
quand une tâche reste active. Le SDK installé documente aussi les tâches Bash.
Le binaire Claude installé émet ces événements pour les tâches enregistrées.

Bridget reprend ce suivi structuré dans son transport, sans nouveau service,
schéma durable ou droit. Une réussite provisoire conserve la même mission,
occupée, avec son échéance initiale. Une notification ne publie rien : le
rapport du tour fournisseur suivant doit arriver. Le journal conserve les
textes provisoires, mais la réponse finale utilise le dernier tour. Une erreur
ou une annulation conserve son comportement terminal.

Les identifiants sont bornés à 256 octets et le registre à 1024 tâches actives.
Une observation invalide ou tronquée reste inconnue jusqu’à l’échéance,
jamais considérée comme terminée. Les tâches internes marquées `ambient` ou
`skip_transcript` ne bloquent pas la mission, selon le contrat du SDK.

## Vérifications

- RED : fournisseur simulé, premier `result` pendant un calcul actif ; le test
  échoue sur une publication prématurée à la première étape.
- GREEN : même parcours avec réponse provisoire, notification étrangère,
  achèvement corrélé, puis rapport final. Une seule remise, même mission,
  avec le texte final. Le test passe par un vrai sous-processus et le transport.
- Gardes : identifiants, doublons, borne mémoire, tâche suspendue, annulation,
  erreur fournisseur et échéance initiale conservées.
- 38 tests du module Claude réussis, aucun échec ni saut. Pas de suite globale.

## Limites

Le suivi porte sur les tâches déclarées par Claude. Un processus externe
lancé sans événement fournisseur n’est pas détecté par ce registre. Les
résultats historiques ne sont pas réécrits. Les wrappers déjà lancés gardent
leur code jusqu’à leur arrêt normal ; aucune mission applicative n’est
recréée ni interrompue par ce correctif.
