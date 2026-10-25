# Recherche — SPEC-165 Cycle de vie des liens adaptateur de bureau

## Décision 1 — Traiter les sous-agents comme des tâches, pas comme des pairs permanents

**Décision** : un sous-agent garde un lien Bridget pendant une activité non terminale ou le drain d'une obligation persistée. Sa conversation reste une preuve historique après la mission.

**Raison** : le runtime réel contient 1 100 sous-agents non rangés, dont 834 terminaux, pour seulement 2 activités. Les maintenir comme pairs connectés crée quatre ressources environ par fil et fait croître le pont avec l’historique.

**Sources** :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/research.md#extrait-1).

**Alternatives évaluées** :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/research.md#extrait-2).

**Impact mainteneur** : la règle suit le cycle de vie métier. Elle supprime la nécessité de surveiller et relever périodiquement une limite système.

## Décision 2 — Séparer existence du fil et besoin de connexion

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/research.md#extrait-3).

**Raison** : une conversation principale terminée reste reprenable. Un sous-agent terminé ne doit plus consommer de ressources. Un seul booléen ne peut pas exprimer ces deux responsabilités sans devenir trompeur.

**Alternatives évaluées** :

- Modifier `is_live()` : rejeté, car cette méthode sert aussi à vérifier que le fil existe avant une remise.
- Filtrer seulement dans la boucle principale : rejeté, car le statut et le chemin de remise pourraient encore annoncer ou utiliser un enfant inactif.

**Impact mainteneur** : le nom de la nouvelle décision rend l’invariant visible. Les trois usages restent cohérents.

## Décision 3 — Réutiliser la fermeture RAII existante

**Décision** : traiter la dernière clé terminale, retirer le lien de la collection puis finaliser le `Link`. Le propriétaire garde un clone du flux et les deux handles. Il ferme la socket avant d'attendre le lecteur réseau. Cette garantie s'applique aussi au retrait d'un worker mort.

**Raison** : cette méthode envoie déjà la désinscription, joint le worker, arrête le journal et ferme le flux. Le lecteur était toutefois détaché. Conserver son handle et un clone de fermeture rend la fin déterministe, même après une panique du worker. La destruction des objets possédés libère les descripteurs. La documentation Rust confirme qu’un `UnixStream` possède son descripteur et que son cycle de vie contrôle cette ressource : [documentation officielle `UnixStream`](https://doc.rust-lang.org/std/os/unix/net/struct.UnixStream.html).

**Alternatives évaluées** :

- Fermer manuellement chaque descripteur : rejeté, car cela dupliquerait la propriété RAII et augmenterait le risque de double fermeture.
- Supprimer les fichiers `.lock` : rejeté, car le verrou doit être libéré par la destruction du propriétaire, sans effacer une preuve sur disque.

**Impact mainteneur** : aucun nouveau nettoyeur. La séquence de fermeture reste unique.

## Décision 4 — Drainer la clé terminale et reprendre les obligations

**Décision** : un lien existant traite le résumé terminal jusqu'à ce que sa clé soit drainée. Le premier snapshot réussi rouvre seulement les enfants terminaux qui portent une attente persistée ou un dernier tour connu non projeté. Une mort imprévue du lien relit cet état. L'horloge de drain ne repart pas sur un nouveau résumé terminal.

**Raison** : `project_journal`, `settle_pending` et l'ACK se produisent après le Tick terminal. Fermer avant ce Tick perdrait le dernier résultat. Une obligation durable doit aussi survivre au redémarrage du pont.

**Alternatives évaluées** :

- Délai fixe avant fermeture : rejeté, car il ne prouve ni la fin du journal ni l'ACK.
- Garder tous les enfants terminaux : rejeté, car cela recrée la fuite.
- Ignorer les réponses non acquittées : rejeté, car cela casse la durabilité existante.

**Impact mainteneur** : un signal de quiescence lié à `change_key()`, une horloge appartenant au lien et deux ensembles bornés remplacent une temporisation fragile. `turn_wait` borne les défauts externes.

## Décision 5 — Compatibilité prudente avec les anciens snapshots

**Décision** : une filiation absente conserve le comportement existant.

**Raison** : le protocole V1 ne fournit pas nécessairement la relation parent/enfant. Retirer des liens sur une inférence de titre, d’identifiant ou de fournisseur serait dangereux.

**Alternatives évaluées** :

- Déduire un sous-agent depuis son titre ou son identifiant : rejeté, car ces champs ne sont pas des autorités de filiation.

**Impact mainteneur** : aucune heuristique cachée. La règle repose sur un champ contractuel explicite.

## Observation runtime reproductible

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/research.md#extrait-4).
