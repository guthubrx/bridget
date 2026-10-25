# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/mcp-installed-identity.md](../../../../../../../specs/174-audit-incidents/validation/mcp-installed-identity.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Base commune Android : 4c198b1822833725c37fe942abb36d820e8665a7,
contenant la livraison r10e e631e47c. Mac et Cartae propres au départ.
Le refus Monuments portait sur bridget_status, avant toute admission.
La même lecture a échoué dans le fil de maintenance. L'attestation native
T3 réussissait. Les processus MCP lancés par Codex ne recevaient aucune
preuve privée de session.

<a id="extrait-2"></a>

## Extrait 2

L'entrée globale Codex pointait sur target/release/bridget. Son chemin réel
et son empreinte différaient du lien installé choisi par T3. Le mécanisme
T3 refreshBridgetMcpServer refuse alors le montage privé (user_server).
Cette garde empêche de donner un credential à un autre programme. Elle
reste inchangée. Ce n'est pas une erreur du dépôt Monuments.

<a id="extrait-3"></a>

## Extrait 3

Le diagnostic identity_not_found indique désormais la lecture native T3,
la comparaison des exécutables et la reprise officielle du montage.
Le code de refus et le refus avant toute exécution restent inchangés.
Documentation EN/FR complétée. L'inventaire CLI inclut aussi mobile : son
absence héritée de la177 a été révélée par le test documentaire094.

<a id="extrait-4"></a>

## Extrait 4

Fork local/main-20261009, commit ac26460e13603c70b6b151b56c8ce4d981748b5b :
BridgetExecutable.ts, BridgetMcp.ts, CodexMcp.ts, CodexAdapterV2.ts et
ProviderSessionManager.ts. Réemploi du lien installé, de la comparaison
realpath et du montage privé par fil. Aucune nouvelle résolution d'identité
ni modification T3. Les 42 tests CodexMcp passent dans cette référence.

<a id="extrait-5"></a>

## Extrait 5

Le gestionnaire T3 peut conserver un fil natif chargé. Il prévoit un
déchargement après inactivité (30 minutes par défaut), avec garde des
travaux d'arrière-plan, puis un remontage à la reprise. Un tour actif
n'est pas interrompu pour accélérer cette opération. Corriger le fichier
ne réécrit pas l'environnement d'un MCP existant.

<a id="extrait-6"></a>

## Extrait 6

Clients installés hors staging, empreintes comparées aux builds, liens
.local/bin/bridget remplacés atomiquement. Chaque exécutable installé a
traité une séquence JSON-RPC stdio réelle initialize/initialized/status,
dans un namespace temporaire sans identité. Le refus conserve son code,
isError=true et expose le nouveau conseil. Ce test négatif ne prouve pas
une identité T3 fonctionnelle.

<a id="extrait-7"></a>

## Extrait 7

Core1520358 et mobile1520533 restent en r10e. Connecteur Mac35661, T3 650/975
et workers existants préservés. Ce changement porte sur le client MCP,
pas sur l'autorité, le protocole mobile ou les permissions. Les processus
MCP déjà ouverts restent anciens : le status de ce fil renvoie encore
identity_not_found. Aucun rétablissement Monuments n'est revendiqué avant
une lecture positive depuis son prochain rattachement natif. La seule
entrée Codex officielle corrigée suit maintenant le lien installé.

<a id="extrait-8"></a>

## Extrait 8

La boîte à idées globale est en lecture seule pour ce processus ; le choix
automatique reste durablement consigné dans la section précédente. Il ne
s'agit pas d'une option déjà implémentée. Une différence de build entre
composants ne suffit pas à prouver qu'une version publiée est plus récente :
la future détection doit utiliser les métadonnées de livraison. T3 sépare
aussi disponibilité et téléchargement (DesktopUpdates.ts et updateMachine.ts),
puis l'application de la version. Aucune mise à jour automatique de T3 ou
Android n'a été ajoutée par ce correctif.
