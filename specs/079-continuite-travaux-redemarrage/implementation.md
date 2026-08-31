# Implémentation - SPEC-079

Date: 2026-08-31
Commit de base: 4ad487e
Branche: session-079-continuite-travaux-redemarrage
Worktree: /home/moi/bridget-referent/.worktrees/session-079-continuite-travaux-redemarrage
État: implémentée et auditée localement, non commitée et non livrée

## Résultat

La continuité du travail et la ronde sont séparées.

- Un message humain UI est admis comme exécution durable.
- Remise, soumission, exécution et lien causal sont gravés avant l'injection.
- Après redémarrage, Bridget rejoue la remise existante ou crée une seule
  continuation `reconstructed` depuis le message exact.
- Un tour encore vivant interdit la reconstruction.
- Une seule ronde globale dessert des politiques indépendantes par
  `project_id + binding_generation`.
- Désactiver une ronde ne stoppe, n'annule et ne reprend aucun travail.
- Aucun timer, daemon, Bridget ou Maicie supplémentaire n'est créé par projet.

## Réalisation

### Contrats et admission

`DeliverIdempotent` accepte un contexte d'exécution optionnel tout en décodant
les anciennes trames. Le relais UI pose `origin=human` et
`intent=trigger_turn`. Le wrapper crée le binding d'exécution avant son
filtre de doublon.

### Reprise

ExecutionStore sélectionne au plus une exécution active par agent, relit
`message_json` et conserve la référence projet. La reprise:

1. rejoue toute remise `dispatching` déjà liée;
2. refuse un candidat ambigu ou un tour vivant;
3. termine le parent;
4. crée une continuation `reconstructed`;
5. grave une remise interne idempotente;
6. rejoue cette même remise après un second redémarrage avant acquittement.

Une enveloppe absente ou corrompue produit un état visible, jamais un prompt
inventé.

### Politique de ronde par projet

L'absence de politique signifie `disabled`. Les commandes
`list/status/enable/disable` sont idempotentes par `command_id` et
exigent une liaison active de génération exacte. Un rebind ne transporte pas
l'autorisation de l'ancienne génération.

### Scheduler unique

Le dispatcher calcule une occurrence globale de sept minutes. Pour chaque
projet actif et autorisé, il émet au plus un réveil portant
`origin=routine`, `intent=trigger_turn`, une `ProjectReference`
exacte et une clé stable. Il ne contacte aucun provider directement.

## Validation

Les tests ciblés SPEC-079, le formatage, Clippy, le contrôle du diff et le
harness du dispatcher passent. Les anomalies de la suite workspace sont
consignées dans `evidence/final-validation.md`; plusieurs sont reproduites
sur `origin/main`.

## Production

Aucun commit, merge, push, déploiement, redémarrage productif ou changement du
timer productif n'a été effectué.
