# Archives techniques de l’adaptateur T3

Document d’origine : [specs/158-reveils-controle/spec.md](../../../../../../specs/158-reveils-controle/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Feuille de route : `docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`.
Comparaison : `specs/150-socle-delegation/research-t3-bridget.md`.
Contre-revue : `adversarial-review-codex.md` (10 objections vérifiées).
Base requise : livraison157 terminée et vérifiée par ROOT avant le GO158A.
Le reçu canonique de livraison157 et ses empreintes seront renseignés à ce GO ;
aucun SHA157 livré n'est déduit de cette préparation. Les preuves anciennes
(base `1f2c661d`) restent historiques. Revalider les fonctions sur cette base,
puis renseigner la branche et la racine du worktree158A.

<a id="extrait-2"></a>

## Extrait 2

**Défaut 1 - Vue des délégations liée à T3.** `bridget lineage` exige
`--t3-thread` et `--project-root` (`cli_lineage.rs:19-20`). Le daemon rattache
ensuite la requête à un fil T3 (`daemon/native_lineage.rs`, `authorize`). Un
humain hors de T3 ne voit rien et ne peut pas lister toutes les tâches.

<a id="extrait-3"></a>

## Extrait 3

**Défaut 2 - Pas d'arrêt global.** La pause (ADR 027) suspend l'autonomie. Elle
ne refuse pas durablement les délégations et ne se confond pas avec un arrêt
d'urgence. T3 n'a pas d'arrêt global non plus.

<a id="extrait-4"></a>

## Extrait 4

**Défaut 3 - Aucune portée déclarée par outil MCP.** Le catalogue est une liste
de valeurs `json!` (`mcp.rs:1891-2235`), l'exécution un `match` sur le nom
(`mcp.rs:470-598`). Un enfant en posture `discovery` appelle les mêmes outils
qu'un parent. T3 déclare des portées par outil (`McpToolAccess.ts`).

<a id="extrait-5"></a>

## Extrait 5

L'humain tape `bridget control delegations`. Il voit les tâches de tous les
parents : propriétaire, enfant, état, âge, échéance restante. Il filtre sur les
actives. Il annule une tâche par son identifiant. Aucun fil T3 n'est demandé.
Un client agent qui négocie la capacité de contrôle est refusé.

<a id="extrait-6"></a>

## Extrait 6

- **SC-001** : Avec 3 propriétaires et 10 tâches, `control delegations` liste les 10 sans paramètre T3. Un client agent négocié est refusé.
- **SC-002** : 100 % des délégations avec nouvelle clé pendant l'arrêt sont refusées `delegations_halted`, 0 tâche créée.
- **SC-003** : Avec `--cancel-active`, 0 tâche active ne reste après l'annulation en cascade, dans le scénario de test. Cette mesure du scénario exclut un crash, un refus de stockage ou un échec d’arrêt physique ; le reçu cancelled ne confirme pas ces arrêts.
- **SC-004** : Aucun outil MCP ne peut poser ou lever l'arrêt.
- **SC-005** : La séquence A, B, rejeu de A laisse l'état « repris », avant et après redémarrage.
- **SC-006** : Le test exhaustif de la table échoue si un outil est ajouté sans portée (outil factice dans le test).
- **SC-007** : Un enfant `discovery` voit 100 % des appels d'écriture refusés, 0 effet de bord, et lit un fil complet par la séquence lecture, `ack`.
- **SC-008** : Hors arrêt posé, un appel sans ajout158 garde le comportement de la base157 livrée. Les filtres de non-régression existants148–150 sont comparés à la baseline fraîche158 ; aucun succès global n'est déduit de cette préparation.

<a id="extrait-7"></a>

## Extrait 7

- Budget de tours et de tokens (question 3).
- Réveils, `bridget wake`, rejeu des faits, secrets : 158b.
- Suspension des missions pendant une pause (152).
- Authentification forte, OAuth, jetons par session (T3 D) : borne ADR 011 inchangée.
- Interface graphique, service web.
- Historique d'audit de l'arrêt au-delà de l'état, des reçus bornés et du journal du daemon.

<a id="extrait-8"></a>

## Extrait 8

Un agent programme un réveil récurrent (`interval`, minimum 60 s) ou à heure
fixe (`HH:MM`, jours de semaine), lié à lui-même, à un autre agent ou à un fil.
À l'échéance, un agent reçoit une notificationSystem produite par le daemon. Un fil reçoit une contribution attribuée au propriétaire attesté membre, kindNone, avec le libellé entier2048 octets et un préfixe court. Son UUID d'opération102 est gelé et persisté avec l'occurrence. Une remise vers fil signifie contribution+intentions102 committées ; ses alertes partagées ne sont pas expirées après 10minutes. Une heure
fixe manquée de plus de 10 minutes est ignorée et reprogrammée. Un intervalle en
retard rattrape une seule fois (règles de T3, `scheduledTasks/Schedule.ts`).

<a id="extrait-9"></a>

## Extrait 9

L'humain saisit le secret hors de l'agent (terminal interactif, sans écho).
L'agent reçoit une référence à usage unique, limitée au projet, 24 h (modèle T3,
`secrets/SecretRequests.ts`). **Condition préalable** : un consommateur côté
Bridget qui prend la référence sans la montrer à l'agent. Il n'existe pas. Cette
histoire reste spécifiée et non planifiée.
