# Migration : explicite, sur copie, sans bascule implicite

La version extraite ne lit jamais le namespace historique par défaut. Aucune
base utilisateur n'a été importée pendant la session 089. Les tests utilisent
uniquement leurs propres données synthétiques dans le schéma historique épinglé.

## Ce qui est prouvé

- Copie d'une base v1 : mêmes clés, canons opaques et issues terminales ; une
  remise avec payload conserve ces octets. Trois réouvertures ne changent pas
  les lignes migrées ; le fichier source complet reste identique.
- Une remise historique SANS payload reste indéterminée : jamais reconstruite
  depuis le message voisin, jamais déclarée livrée. C'est la migration v2
  conservée, pas une nouvelle tentative automatique.
- Les migrations historiques v3/v4/v6 restent testées. La v4 peut écarter une
  remise sans parent idempotent (corruption hors contrat) avec diagnostic ; une
  telle copie n'est donc pas une reprise zéro-perte certifiée.
- Une entrée de version hors 1..6 dans `idempotency_schema_migrations` est
  refusée avant mutation par Store, IdempotencyStore ET le daemon. Cette table
  versionne le socle idempotent ; elle ne prétend pas reconnaître tous les
  schémas possibles de plugins externes.

## Procédure avant un import demandé par l'utilisateur

1. Obtenir les chemins source et destination **absolus et explicites**. La
   destination doit être un nouveau namespace privé, jamais celui de la flotte.
   Aucun import depuis une variable héritée ou une découverte automatique.
2. Vérifier type régulier, UID propriétaire et absence de symlink de la source.
   Produire une sauvegarde SQLite cohérente par l'API backup, avec source ouverte
   en lecture seule, plutôt que copier un fichier `.db` actif en oubliant son
   WAL. Ne pas arrêter le daemon de production sans autorisation distincte.
3. Créer la copie en 0600 dans une racine 0700 neuve. Inventorier schémas,
   références et versions avant migration ; une base d'un autre produit ou une
   version inconnue n'est pas une base vierge à « réparer ».
4. Migrer uniquement cette copie, contrôler intégrité SQLite, clés étrangères,
   mêmes IDs/canons/terminaux, messages, demandes et événements. Conserver
   l'inventaire AVANT/APRÈS et tout diagnostic de quarantaine ou suppression.
5. Ne pas démarrer automatiquement les anciens wrappers ni recopier leurs
   PID, sockets, marqueurs de processus ou credentials fournisseur. Un état
   de communication importé n'autorise pas à reprendre une flotte ; les
   présences, profils et sessions fournisseur exigent une recette distincte.
6. Faire valider le bilan et le plan de bascule. Pour revenir en arrière,
   arrêter seulement l'instance extraite explicitement identifiée et reprendre
   l'ancien namespace intact. Ne jamais réinjecter une copie déjà migrée dans
   une version ancienne ni fusionner deux bases SQLite.

La sauvegarde/export réel et la bascule restent des opérations d'adoption
volontaires. Ce guide n'est pas l'annonce qu'elles auraient déjà été exécutées.
