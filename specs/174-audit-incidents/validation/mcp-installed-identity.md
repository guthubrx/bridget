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

## Livraison et limites effectives

Source d496598224ff424e4522b9bfafd677661fc4ab82 intégrée et poussée sur
main, puis récupérée sur Mac. Releases construites par scripts/build.py :
Linux 1 min 47 s ; Mac 2 min 10 s. Identifiant commun :
174-mcp-installed-d4965982. Binaire Mac signé ad hoc et signature vérifiée.
SHA-256 Mac : d8c2c9ef7be5b36277a62e36f44709fe36b6347a8e4c4f447f3bd15de18dbca5.
SHA-256 Linux : 64276359c008ddff1b1954c99b8af0fd3fd78261ce18651734bbc762e4b55849.

Clients installés hors staging, empreintes comparées aux builds, liens
.local/bin/bridget remplacés atomiquement. Chaque exécutable installé a
traité une séquence JSON-RPC stdio réelle initialize/initialized/status,
dans un namespace temporaire sans identité. Le refus conserve son code,
isError=true et expose le nouveau conseil. Ce test négatif ne prouve pas
une identité T3 fonctionnelle.

Core1520358 et mobile1520533 restent en r10e. Connecteur Mac35661, T3 650/975
et workers existants préservés. Ce changement porte sur le client MCP,
pas sur l'autorité, le protocole mobile ou les permissions. Les processus
MCP déjà ouverts restent anciens : le status de ce fil renvoie encore
identity_not_found. Aucun rétablissement Monuments n'est revendiqué avant
une lecture positive depuis son prochain rattachement natif. La seule
entrée Codex officielle corrigée suit maintenant le lien installé.

L'envoi de coordination Android par Bridget a lui aussi été refusé avant
remise pour identity_not_found. Aucune autre route utilisée, aucun rejeu
de mission. Aucun enfant créé. L'activation des services n'a pas été forcée.
Un ancien état cancelling avec annulation non vérifiable est conservé ;
aucune tâche étrangère n'a été modifiée pour obtenir un état sans activité.

Entretien installed appelé sur les deux machines après installation.
Linux : 3,50 Gio logiques d'intermédiaires debug retirés, puis observation
incomplète des processus : reste conservé, code retour1. Mac : 1,12 Gio
logiques de cache réutilisable conservés, code retour0. Ancienne copie Mac
r10c conservée car le contrôle des usages est incomplet. r10e reste actif
et sert de retour pour les clients ; r8 reste utilisé par les workers.
Aucune preuve ni base retirée. Branche de correction supprimée après
intégration. Aucun worktree créé par ce lot ; les deux worktrees177 du
serveur restent conservés pour leurs gardes de clôture/usage non acquises.

La boîte à idées globale est en lecture seule pour ce processus ; le choix
automatique reste durablement consigné dans la section précédente. Il ne
s'agit pas d'une option déjà implémentée. Une différence de build entre
composants ne suffit pas à prouver qu'une version publiée est plus récente :
la future détection doit utiliser les métadonnées de livraison. T3 sépare
aussi disponibilité et téléchargement (DesktopUpdates.ts et updateMachine.ts),
puis l'application de la version. Aucune mise à jour automatique de T3 ou
Android n'a été ajoutée par ce correctif.
