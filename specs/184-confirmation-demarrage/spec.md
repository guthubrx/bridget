# Session 184 — Confirmation du démarrage distant

Statut : source corrigée, testée, intégrée et poussée ; candidat installé ; activation refusée par le garde natif de maintenance. Correction autorisée d'un incident récurrent KS-STOR.

## Besoin
Conserver et confirmer la même commande quand le lancement distant réussit mais que son accusé dépasse le délai réseau. Ne pas recréer de mission métier.

## Faits
Deux admissions réelles ont renvoyé remote_release:WorkerUnavailable alors que le wrapper et Claude tournaient dans leur worktree exact. Pour f0b0a469, observation des PID 2865011 et 2866251 ; aucune mutation de la tâche par le responsable du correctif. Le parent conserve le pilotage.
Le maître attend cinq secondes pour Release, malgré une revalidation distante et une file partagée. Après expiration de l'attente, un accusé Released d'une commande connue est refusé ; le lecteur worker ferme son canal sur ce refus.

## Exigences
- Attendre une confirmation lente dans une limite bornée, sans prolonger les droits du permis.
- Accepter sans mutation un accusé tardif pour la même identité worker, époque et commande.
- Refuser une époque périmée ou une autre instance.
- Conserver la tâche, le choix de routage et les protections contre les doublons.

## Complément — refus de reprise explicites
Instruction utilisateur du 27 octobre : les refus motivés doivent être explicites pour permettre une résolution par l'agent, ou une demande humaine précise.
La tâche SYS-LE-2 58459353-0d42-4fc2-acc4-692949616648 est encore continuable mais son exécution pointe vers un ancien dossier supprimé. Le chemin courant se résout désormais sur un autre disque. Le refus est justifié ; sa cause et ses actions étaient insuffisantes.
Distinguer exécution, rapport de dossier et preuve de droits. Ajouter le contexte de l'exécution uniquement après contrôle du parent. Ne pas transformer une preuve absente en certitude de suppression ou révocation.
Une ancienne incertitude de lancement doit rester consultable comme historique dès que la tâche a progressé ou dispose d'un résultat, sans rester une erreur courante. Aucun effacement du fait durable.
