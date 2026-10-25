# Identité MCP et chemin installé — 25 octobre 2026

## Cause vérifiée

Base commune Android : 4c198b1822833725c37fe942abb36d820e8665a7,
contenant la livraison r10e e631e47c. Mac et Cartae propres au départ.
Le refus Monuments portait sur bridget_status, avant toute admission.
La même lecture a échoué dans le fil de maintenance. L'attestation native
T3 réussissait. Les processus MCP lancés par Codex ne recevaient aucune
preuve privée de session.

L'entrée globale Codex pointait sur target/release/bridget. Son chemin réel
et son empreinte différaient du lien installé choisi par T3. Le mécanisme
T3 refreshBridgetMcpServer refuse alors le montage privé (user_server).
Cette garde empêche de donner un credential à un autre programme. Elle
reste inchangée. Ce n'est pas une erreur du dépôt Monuments.

## Correction

L'ancienne entrée officielle a été sauvegardée puis son seul champ command
a été remplacé par le lien installé stable. La configuration parsée avant
et après est identique hors ce champ. Aucun credential, droit, namespace,
serveur étranger ou opt-out modifié.

Le diagnostic identity_not_found indique désormais la lecture native T3,
la comparaison des exécutables et la reprise officielle du montage.
Le code de refus et le refus avant toute exécution restent inchangés.
Documentation EN/FR complétée. L'inventaire CLI inclut aussi mobile : son
absence héritée de la177 a été révélée par le test documentaire094.

## Référence T3 relue

Fork local/main-20261009, commit ac26460e13603c70b6b151b56c8ce4d981748b5b :
BridgetExecutable.ts, BridgetMcp.ts, CodexMcp.ts, CodexAdapterV2.ts et
ProviderSessionManager.ts. Réemploi du lien installé, de la comparaison
realpath et du montage privé par fil. Aucune nouvelle résolution d'identité
ni modification T3. Les 42 tests CodexMcp passent dans cette référence.

Le gestionnaire T3 peut conserver un fil natif chargé. Il prévoit un
déchargement après inactivité (30 minutes par défaut), avec garde des
travaux d'arrière-plan, puis un remontage à la reprise. Un tour actif
n'est pas interrompu pour accélérer cette opération. Corriger le fichier
ne réécrit pas l'environnement d'un MCP existant.

## Validation

RED : le nouveau test de projection refuse le message ancien, car il manque
la consigne t3_code.bridget_session. Aucun exécuteur appelé.
Premier lot MCP : 166 PASS, 2 FAIL, 2 ignorés. Les deux échecs sont l'entrée
mobile absente dans l'inventaire et une collision HOME entre tests parallèles.
Le test du wrapper passe seul (1 PASS). Après correction documentaire,
le lot MCP exécuté séquentiellement donne 168 PASS, 0 FAIL, 2 ignorés.
Le nouveau test couvre status et auto_delegate, avec zéro exécution.
Aucun succès global revendiqué.

Revue indépendante indisponible : délégation bloquée pour ce tour.
Pas de mission de remplacement, aucun modèle lancé. Pas de diaporama requis
pour ce correctif de diagnostic technique.

## Choix utilisateur pour la suite

Choix explicite du 25 octobre : installer automatiquement les mises à jour
Bridget dès que les missions permettent une activation sûre. Ce choix est
retenu ; aucun service de mise à jour n'est créé ni annoncé actif par ce lot.
La future réalisation doit distinguer version disponible, installée et
chargée ; vérifier la provenance et la compatibilité ; attendre la fin des
travaux concernés ; activer sans doublon ; vérifier la santé et garder une
version de retour. Elle doit contrôler le montage MCP après activation et
signaler les processus encore sur l'ancienne version. Le lien stable règle
la dérive de chemin ; il ne recharge pas les processus existants.
