# C06 — comptes isolés par connexion
Statut : candidat du lotL2a, pas d'activation attestée.

## Réemploi et données
Un compte est une connexion du registre existant, pas un coffre de secrets.
AgentProfileStore reste réservé aux instructions et à l'identité visuelle.
codex_home fixe le dossier Codex. claude_config_dir fixe le dossier Claude.
Le compte Claude géré utilise aussi son sous-dossier anthropic comme
ANTHROPIC_CONFIG_DIR : Claude.ai, clés locales et profils Console ont ainsi
leur stockage privé. HOME global ne change pas. Aucun secret du compte
principal n'est lu, copié, haché, journalisé ou sérialisé.
Le worker, le fournisseur et le collecteur partagent SourceEnvironment figé.
Les signaux de connexion ne lisent que les métadonnées des chemins choisis.

## Fermeture et compatibilité
AdapterCapabilities porte account_profiles=true, dans sa matrice fermée.
Un ancien lecteur refuse ce champ au lieu d'ignorer codex_home et de lancer
le compte principal. codex_home exige cette garde. Absence : canon et digests
historiques identiques. Un profil absent ou non privé refuse sans repli.
ProviderAccountsV1 est exigée à l'inspection et au nouveau départ, jamais
pour bloquer la lecture, l'annulation ou le contrôle d'une tâche vivante.
launch.provider_account.v1 protège les lecteurs et le retour installés.

## Parcours humain
accounts list affiche les connexions locales avec auth=not_verified.
prepare --from CONNEXION --id NOM produit un aperçu sans effet. L'application
exige un terminal humain, --expected SHA --apply et le même registre privé.
Le verrou project-preparation.lock, le CAS relu avant rename, la sauvegarde0600
et les dossiers0700 réutilisent les règles du préparateur existant.
La connexion reste disabled. Le résultat donne la commande officielle de login,
les variables à retirer puis les deux dossiers Claude ou le dossier Codex à
fixer avant le login. Aucun programme de connexion n'est lancé par Bridget.
Les arguments libres et intermédiaires pouvant copier un secret refusent.
Aucun --settings inline n'est ajouté : il gênerait la composition native.
enable exige une nouvelle décision/CAS après le login choisi par l'utilisateur.
Une déclaration activée ne prouve ni l'authentification ni l'accès à un modèle.

disable et remove concernent les futures admissions. remove retire connexion,
liste et alias courant, tout en conservant fichiers, résultats et définitions
figées des tâches. Il n'arrête aucun processus vivant. Un nouveau départ sous
l'alias retiré, même pour une ancienne tâche, exige une autorisation de nouveau.
Finir les travaux qui doivent encore repartir avant le retrait.
Un alias encore partagé refuse ; aucune suppression forcée n'est proposée.
applied_local n'atteste pas la publication de flotte, distincte dans C08.

## Interruption et bornes
32 comptes gérés et32 sauvegardes au maximum. Une sauvegarde au même SHA est
réutilisée seulement après lecture privée et égalité exacte du contenu.
Un dossier préparé sans publication est conservé sous une cause nommée,
sans adoption silencieuse. Une publication dont la dernière synchronisation
échoue demande une lecture avant toute suite. Aucun historique n'est purgé.
Un éditeur étranger doit respecter le verrou ; le CAS ne prétend pas le bloquer.

## Sources et preuves
Documentation officielle lue : https://code.claude.com/docs/en/authentication,
https://code.claude.com/docs/en/env-vars et
https://platform.claude.com/docs/en/manage-claude/wif-reference.
La règle ANTHROPIC_CONFIG_DIR est aussi celle de Claude Code, pas une hypothèse
fondée sur le seul SDK. La fixture exerce le préparateur, lifecycle,
capture_context_with_env et apply_child_arguments sans login ou appel réseau.
La preuve CODEX_HOME/config/read antérieure est réutilisée. Aucun compte réel
n'est connecté, migré ou modifié pour vérifier ce lot.
