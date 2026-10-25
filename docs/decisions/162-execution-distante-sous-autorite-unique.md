# ADR 162 — Exécution distante sous autorité unique

**Date :** 2026-10-13. **Statut :** proposition, à valider avec la revue SPEC-162.

## Besoin

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/162-execution-distante-sous-autorite-unique.md#extrait-1).

## Décision

Le maître garde Fleet, les tâches, les permissions, la reprise et la complétion. Un module worker distinct possède les processus et des reçus techniques privés sur l’hôte distant. Il réemploie managed_process et fsutil. Le canal existant fédéré par SSH transporte un handshake worker explicite et des opérations corrélées. La préparation distante se fait hors mutex du maître, selon le modèle delegate_worktree.

Prepare observe les droits locaux. Start persiste un reçu avant bootstrap et un marqueur avant ready. Release exige une autorisation récente du maître ; son délai court depuis l’événement local ready, pas depuis une réception réseau tardive. Après panne ambiguë, observer ou refuser, jamais recréer par supposition.

Les preuves parent sont relues sur leur hôte réel. Origine et cwd d’exécution restent distincts. La projection conserve le plafond parent et ajoute les restrictions de destination. Une conversion opaque est refusée.

Le résultat final distant est publié atomiquement par le wrapper dans un blob immutable préalloué. Le worker écrit seul les métadonnées du store technique distinct de ReceiptStore. Aucun fichier mutable partagé entre ces deux écrivains. Le worker peut relayer le blob après mort du wrapper via capture_remote_result, qui contrôle placement, époque et identités d’exécution. Une disposition spécifique atteste le canon et la transaction maître : accepted, discarded_terminal, rejected_conflict ou deferred. RetainedResultAuthority ne sert qu’à la notification sortante après capture. Réemployer ensuite la complétion, l’attente et les notifications existantes. Le journal entier reste sur le worker, lu à la demande via Attach ; aucune réplication permanente.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/162-execution-distante-sous-autorite-unique.md#extrait-2).

## Alternatives écartées

- SSH exec par mission : seconde voie de lancement sans les garanties natives.
- Daemon métier complet sur chaque worker : autorité et reprises concurrentes.
- Fusion des étapes Start et Release : suppression du contrôle tardif après marqueur.
- Miroir permanent du journal : stockage et synchronisation inutiles au besoin demandé.
- ACK legacy comme preuve de résultat : certains messages obsolètes sont absorbés sans comparaison du canon.
- Nouveau moteur Git distribué : l’isolation 154 et les missions d’agent suffisent ; un intégrateur unique gère main.

## Conséquences

Le contrat transporte explicitement les preuves parent, les pages et abonnements de journal, le résultat final et sa disposition, en plus du lancement. Les parents déjà distants restent pris en charge avec profondeur maximale existante de 8. Tous les échanges réseau sont hors mutex métier. Une reconnexion autorisée remplace atomiquement la connexion précédente et invalide ses trames, sans restaurer de permis.

Le worker Cartae est supervisé par une unité systemd utilisateur. KillMode=process est un choix exceptionnel et assumé : systemd le déconseille dans le cas général car les enfants survivent. Ici ces enfants sont les wrappers gérés à réconcilier ; un arrêt du worker ne vaut pas annulation de leurs missions. SIGTERM et SendSIGKILL=no préservent cette séparation. La recette vérifie la survie d’un enfant synthétique et sa réconciliation. Aucun nouvel IPC, daemon métier ou tunnel par mission.

Un nouveau module est justifié par la frontière d’hôte et de confiance, sans nouvelle dépendance. Les reçus techniques sont bornés ; le budget compte enveloppes sérialisées, réserves, métadonnées et temporaires. Une saturation bloque de nouvelles admissions. Collecter un blob exige une disposition finale durable, un producteur terminal et une commande ou un slot irrévocablement fermé ; un ACK seul ne suffit pas. Le reçu compact, l’empreinte et la disposition restent conservés 7 jours, sans imposer cette durée au blob lourd. Les permissions non traduisibles et les exécutions ambiguës restent refusées ou à réconcilier. Le journal peut être indisponible pendant une panne du worker ; le résultat déjà committé reste au maître.

L’ADR 044 sur le relais sous autorité parent reste applicable. SPEC-162 étend ce principe à une frontière de machine. Les paramètres de production, l’audit de réemploi et la contre-revue conditionnent l’activation ; cette proposition ne prétend aucun déploiement exécuté.

## Domaine de confiance du worker

Le worker, les wrappers et les fournisseurs sous le même UID partagent un domaine de confiance. Les modes 0600/0700 ne les isolent pas entre eux. Un fournisseur full-access compromis peut lire le credential ou modifier l’état ; l’authentification prouve la possession du credential, pas l’intégrité du processus. À Prepare et avant RELEASE, refuser tout chevauchement canonique dans les deux sens entre cwd, mappings, writableRoots ou additional_directories et le credential, son répertoire privé ou l’état worker. Une résolution impossible entraîne un refus. Cette garde prévient l’exposition explicite ou accidentelle, pas un accès global dangerFullAccess/bypassPermissions. Le wrapper d’infrastructure peut publier uniquement dans son slot préalloué ; ce slot n’est jamais ajouté aux droits du fournisseur. Un UID séparé ou une sandbox OS restent hors périmètre. Journaliser les remplacements de connexion avec machine, worker et anciennes/nouvelles époques, jamais le secret.

## Compléments après revue R2

Le placement est stocké dans Task et DesiredEquipier ; Fleet écrit le schéma 5. Les anciens types stricts ne garantissent pas le downgrade. Retour arrière par ensemble sauvegardé DB/Fleet/registre/worker seulement après quiescence et sans perte d’activité postérieure ; sinon garder 162 avec route désactivée.

Le résultat distant n’a qu’une voie réseau : final_result émis par le worker. Le wrapper publie son blob, sans Send/SendIdempotent. La capture locale refuse explicitement toute mission distante, sans passage au dispatch normal. Les rondes et reprises quota réservent leur slot par reserve_result_slot avant leur dispatch, hors verrou puis revalidation.

Le credential est chargé depuis le fichier privé dans la seule mémoire du worker. Il ne passe jamais par l’environnement du wrapper ou du fournisseur. build_environment/pass_env ne constitue pas un filtre générique de secrets ; ne pas créer une variable d’environnement contenant ce credential.
