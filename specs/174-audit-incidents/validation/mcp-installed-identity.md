# Identité MCP et chemin installé — 25 octobre 2026

## Cause vérifiée

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/mcp-installed-identity.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/mcp-installed-identity.md#extrait-2).

## Correction

L'ancienne entrée officielle a été sauvegardée puis son seul champ command
a été remplacé par le lien installé stable. La configuration parsée avant
et après est identique hors ce champ. Aucun credential, droit, namespace,
serveur étranger ou opt-out modifié.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/mcp-installed-identity.md#extrait-3).

## Choix de conception Bridget

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/mcp-installed-identity.md#extrait-4).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/mcp-installed-identity.md#extrait-5).

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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/mcp-installed-identity.md#extrait-6).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/mcp-installed-identity.md#extrait-7).

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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/mcp-installed-identity.md#extrait-8).
