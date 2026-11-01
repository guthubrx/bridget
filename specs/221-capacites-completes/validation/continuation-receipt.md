# Reçu durable de continuation — session221

Statut : correction vérifiée, activation en préparation. Programme inchangé :
52/182, In Progress.

## Incident et diagnostic

Signalement `opus201-incident-continue-jsonl-20261101-01`. Tâche
`be04d2d9-fa1a-41a4-82aa-ef2f0db18a04`, clé de continuation
`opus201-patrimoine-modernes8-photos-r2-20261101-01`. Le propriétaire a reçu
« réponse daemon indisponible : budget JSONL dépassé ». Le registre confirme
la ronde2 `working`, erreur nulle, mission
`897ba2e4-53b5-4585-9c52-685d40ee2440`. Aucun rejeu de cette consigne.
Ce libellé JSONL correspond à un délai expiré, pas à un volume trop grand.
La trace historique ne permet pas de répartir son temps entre les étapes.

Deux écarts de code sont établis : Continue utilisait le budget des lectures
(10 s, plus33 s avec attestation) malgré une lecture des sources d’origine
plafonnée à30 s puis une observation worker plafonnée à30 s ; après le commit
de `begin_round`, le handler attendait aussi le tick général et ses éventuels
rapports/nettoyages d’autres tâches avant de retourner le reçu. Ces écarts
expliquent un risque vérifiable, pas une attribution historique certaine.

## Correction bornée

- Retour du reçu après `begin_round`, sans tick général dans cet appel Continue.
  La boucle native existante reprend les tâches persistées `mission_pending`.
  Aucun thread, autorité, store, protocole ou rejeu automatique ajouté.
- Budget spécifique de continuation :70 s sans attestation,103 s avec le budget
  d’attestation actuel. Sources30 s + observation worker30 s restent inchangés ;
  aucun délai métier, droit, compte ou modèle modifié.
- Lors d’une réponse perdue, le client récent nomme les deux identifiants,
  indique que la ronde peut déjà être remise et demande la lecture du statut.
- `mission_pending` est un reçu durable, distinct de la preuve `working`.
  Cadence cible1 s ; les autres opérations et la réservation distante peuvent
  retarder la remise. Aucune garantie de départ en1 s.

## Vérifications ciblées et contre-revue

Trois nouveaux tests bibliothèque Linux PASS : vrai socket avec réponse après
11 s et aucun rejeu ; déconnexion avec identifiants conservés et aucun rejeu ;
vrai handler Continue, même enveloppe deux fois, reçu avant tick, une seule
ronde/mission/enfant puis remise par tick. Régression existante native153
(plafond8rondes et rejeux) PASS.

Le premier appel Cargo sans `--lib` a rencontré cinq fixtures d’intégration
antérieures, dans `tests/fixtures/remote_worker_162.rs`, sans le champ requis
`service_tier`. Ce chemin n’est pas déclaré PASS. Les cibles bibliothèque et
les assemblages exécutables sont distincts ; aucune campagne générale lancée.

Contre-revue6622035b, moteurClaude choisi par Bridget : patch complet lu,
aucun défaut bloquant confirmé, sans compilation de sa part. Résultat lu,
ACK et clôture demandés une fois. Limites retenues : test de reçu local,
preuve distante toujours synchrone avant commit, cadence partagée et sans
garantie de latence. La remarque sur les30 s de sources est réfutée par
`handle_attested`→`reattest`, puis `remote_dispatch_proof`→`remote_parent_proof`
→`origin_proof::sources` (budget30 s) : le paramètre NativePermissionProof
est un accès au mécanisme d’attestation, pas une preuve déjà réutilisable.

Le nouveau budget et le message client exigent une image MCP récente.
Les clients déjà ouverts profitent du découplage serveur mais ne sont pas
déclarés rechargés. Aucune boucle de reprise de l’application hôte.
Aucune mission métier ni qualification fournisseur créée pour ce lot.
