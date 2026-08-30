# Contre-revue hostile du plan - SPEC-075

## Round 1

**Verdict initial: FAIL.**

### Majeur 1 - Collision d'identité après décommissionnement

Le plan initial supprimait complètement l'entrée de flotte tout en conservant
l'historique et autorisait la réutilisation du nom. Or le nom est actuellement
la clé logique visible. Un nouveau spawn du même nom aurait mélangé deux agents
et leurs historiques.

**Correction appliquée**: ajout de `lifecycle_state=decommissioned`, caché de
l'annuaire et de la reprise. La tombstone réserve le nom tant qu'une future
purge explicite ne l'a pas libéré.

### Majeur 2 - Réactivation implicite d'une délégation close

Le plan conservait le lien parent et le mandat lors d'une relance. Or l'arrêt
ferme ce lien, et le parent ou l'objectif peut être terminé.

**Correction appliquée**: une relance administrative conserve nom, runtime,
projet et politique de persistance, mais démarre sans ancien ownership actif.
La filiation passée reste seulement historique.

### Mineur 1 - Sens de « retirer »

Le mot « retirer » pouvait signifier supprimer le registre durable ou seulement
la flotte visible.

**Correction appliquée**: le contrat distingue tombstone durable cachée,
absence de l'annuaire et conservation de l'historique.

## Round 2 local

**Verdict: PASS.**

- Stop reste réversible et visible.
- Relaunch ne réactive aucune délégation.
- Decommission cache durablement l'identité et réserve son nom.
- Une purge future est explicitement hors périmètre.
- Les trois opérations partagent le superviseur sans partager leur sémantique.

## Revue hostile de l'implémentation

Trois fournisseurs externes ont été sollicités. Claude était hors quota,
Gemini n'était pas authentifié et GLM n'a produit aucun verdict. Une revue
Codex indépendante en lecture seule a ensuite inspecté le diff. Le CLI serveur
a expiré avant de rédiger son message final, mais sa trace a isolé deux constats
majeurs vérifiables.

### Majeur 3 - Course relance contre décommissionnement

Une relance pouvait réserver une nouvelle génération entre la fin de l'arrêt
et l'écriture de la tombstone. Le résultat pouvait être un processus actif sous
un nom décommissionné.

**Correction appliquée**: le daemon réserve désormais le nom pendant toute la
saga de décommissionnement. Toute relance concurrente est refusée jusqu'au
verdict et un test reproduit cette intercalation.

### Majeur 4 - Repli d'une action UI inconnue vers stop

Le constructeur d'URL remplaçait silencieusement toute action inconnue par la
route d'arrêt. Une faute de programmation pouvait donc déclencher une action
destructive différente de celle demandée.

**Correction appliquée**: une action inconnue lève désormais une erreur avant
toute requête réseau. Le test Node couvre ce refus.

## Verdict après corrections

**PASS sous réserves connues**: aucun constat critique ou majeur ouvert dans le
périmètre SPEC-075. Les limites de la suite globale et de l'outillage externe
sont détaillées dans `audit.md`.

Une seconde contre-revue Codex indépendante, limitée aux deux correctifs, a
rendu **PASS**. Elle ne relève aucune faille P0, P1 ou P2 restante. Son verdict
intégral est conservé dans `adversarial-implementation-review-2.md`.
