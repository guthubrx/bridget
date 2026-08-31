# Revue hostile de l'implémentation - SPEC-081

## Contre-revue externe

Une contre-revue par un fournisseur différent a été recherchée via Bridget. Aucun agent Claude, Gemini ou autre fournisseur n'était connecté; seuls l'interface et un client Codex étaient disponibles. Aucune contre-revue externe n'est donc revendiquée. La revue ci-dessous est manuelle et fondée sur le diff et les tests.

## Findings

| Niveau | Finding | Preuve | Décision |
|---|---|---|---|
| Moyen | Une perte de réponse produisait le mauvais code d'erreur public | test `spec_081_perte_de_reponse_ronde_reste_une_indisponibilite_fermee` initialement rouge | corrigé |
| Faible | Deux commandes documentées filtraient zéro test | exécution réelle des commandes du quickstart | corrigé |
| Faible | La jointure cohérente n'avait pas de test direct | absence initiale de test de `ui_project_list_entry` | corrigé |
| Faible | L'extension additive de `ProjectRoundProjection` suppose une mise à jour cohérente du daemon et du CLI embarqué | le type historique refuse les champs inconnus | accepté comme contrainte de livraison atomique du même binaire; aucun ancien client durable ne consomme cette projection |

## Axes hostiles

### Minimalisme

PASS. Aucune dépendance, table, page, timer ou abstraction générique n'est créée. Les ajouts productifs restent dans les six fichiers déjà propriétaires des contrats, du store, du daemon, du relais et du menu.

### Complexité

PASS. La lecture fait deux requêtes locales puis une jointure par table de hachage O(p). La mutation et l'écriture du dernier passage utilisent la clé primaire et restent O(1).

### Frontière UI

PASS. Le navigateur n'accède pas au store, au scheduler ni à une commande libre. Il envoie seulement version, identifiant de commande, projet, génération et booléen.

### Génération et idempotence

PASS. La génération affichée est renvoyée dans la mutation. La politique de l'ancienne génération reste historique et n'est pas jointe après rebind. L'idempotence existante reste l'unique autorité.

### Honnêteté de l'état

PASS. Aucun changement optimiste. Le prochain passage n'est pas daté localement. Une politique absente, une capacité absente, une réponse perdue ou un triplet incomplet échoue fermé.

### Confidentialité

PASS. Aucun secret, fournisseur, profil, corps de message ou nouvelle racine n'est ajouté au contrat. Le chemin canonique existait déjà dans la projection administrative locale.

### Responsabilité LLM

PASS. La fonctionnalité ne demande aucune interprétation à un modèle. Le scheduler, la décision de politique, la génération, la classification du dispatch et la projection sont déterministes.

## Verdict

PASS. Tous les findings confirmés corrigibles dans le périmètre sont fermés. Il reste une contrainte de livraison connue: compiler et installer le daemon et le CLI issus du même commit avant redémarrage.
