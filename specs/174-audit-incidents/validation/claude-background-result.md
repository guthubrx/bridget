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

## Choix de conception Bridget

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/claude-background-result.md#extrait-1).

Bridget reprend ce suivi structuré dans son transport, sans nouveau service,
schéma durable ou droit. Une réussite provisoire conserve la même mission,
occupée, avec son échéance initiale. Une notification ne publie rien : le
rapport du tour fournisseur suivant doit arriver. Le journal conserve les
textes provisoires, mais la réponse finale utilise le dernier tour. Une erreur
ou une annulation conserve son comportement terminal.

Les identifiants sont bornés à 256 octets et le registre à 1024 tâches actives.
Un identifiant invalide ou un registre tronqué reste inconnu jusqu’à l’échéance,
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

## Revue indépendante et retouche

La revue confirme le cas corrigé. Elle identifie deux risques de blocage :
une progression tardive pouvait recréer une tâche finie ; une notification
masquée pouvait empêcher son retrait. La version finale crée une entrée
uniquement sur `task_started`. Les mises à jour ne créent aucune entrée et
une fin est traitée avant le filtre d’affichage. Les deux scénarios sont testés.
Aucun second registre ni mécanisme de reprise n’a été ajouté.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/claude-background-result.md#extrait-2).

## Livraison

Source finale `be00ace5c2dca3126e420214ec248c54083440d1` intégrée et poussée
sur main. Builds release optimisés sur Linux et macOS, sans modification de
profil pour accélérer la compilation. Signature macOS vérifiée.

- SHA-256 Linux : `d6256639fb9a6d4e807fde52e7b5469834a932d6c3a15e6ecb827e4b41292f23`.
- SHA-256 macOS : `a9226281b33535638c4d0b7708cf6d98361859a71bacf29c23c726f207ed69c3`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/claude-background-result.md#extrait-3).

Les wrappers déjà actifs gardent leur code. La correction est disponible pour
les nouveaux enfants ; aucune prétention de correction à chaud des missions
en cours ni de réécriture des résultats historiques.

Entretien obligatoire exécuté après installation : 0,40 Gio d’intermédiaires
Linux retirés ; caches utiles protégés. Ancien client Linux sans utilisateur
retiré ; version active et version de retour conservées. Aucun worktree ni
travail applicatif supprimé. Reçus et traces détaillés conservés hors Git.
