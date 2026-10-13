# Contrat de livraison du client161

## Entrées et autorité

Réutiliser scripts/deploy-remote.sh : label, host, user, port, identity, known-hosts, source, remote-prefix et remote-cargo explicites. --expect-commit <40hex> est optionnelle dans le script existant, obligatoire pour161 avec e6812e3e7b2d051d630192857cc3f440180d51fa ; contrôle HEAD exact, arbre propre, index sans assume-unchanged/skip-worktree avant SSH. Le mode historique sans option conserve sa compatibilité. Le runner ignore la configuration SSH implicite ; les paramètres réels viennent de la liaison cartae-core attestée. Aucun mot de passe ou clé dans le reçu. Source Git au commit160 propre ; runner161 extérieur à cette source.

## Invariants pré-build

1. Hôte épinglé et chemins validés par les gardes existantes.
2. Cargo/rustc1.92 déjà installés, aucune installation automatique, mode locked/offline. Préflight des configurations Cargo effectives sans secrets ; CARGO_TARGET_DIR fixé au préfixe, jobs2/nice10 vérifiés au processus.
3. Préfixe neuf, propriétaire attendu, modes privés ; un préfixe occupé n'est pas réutilisé.
4. Liste unique filtrée pour manifest et transfert ; format SHA standard existant, empreinte du manifest transmise par SSH. Noms relatifs ASCII avec espaces autorisés, newline/backslash refusés ; modes Git et exclusions rsync explicites, sans réduction aux seuls crates. Liste des octets transférables et empreintes dérivées de la référence source propre. Contrôle distant avant invocation du build : empreinte du manifest, comparaison exacte des ensembles de fichiers et absence de liens. Corruption, absence ou fichier supplémentaire provoque un refus.
5. Compilation unique à nice10, deux jobs ; propager le code d'échec de Cargo malgré la réduction du journal.
6. Installation bin/bridget complète, mode700, sans activer la commande globale.

Les détails de sérialisation du manifest doivent réemployer les outils installés et traiter correctement les chemins permis par le projet. Le script/tests définissent un refus explicite pour tout nom non représentable plutôt qu'un parser ambigu. Aucun contenu téléchargé n'est exécuté implicitement.

## Activation

Candidat vérifié via son chemin absolu avant bascule : architecture Linuxx86_64, Build ID160, empreinte et permissions, connexion au maître160. Sauvegarder la cible textuelle et l'empreinte de096. Comparer de nouveau le lien courant. Créer le lien voisin avec nom propre au lot et vérifier qu'il n'existe pas déjà ; le renommer atomiquement sur la référence courante. Aucun remplacement récursif ni changement global des permissions.

Le contrôle concurrent est conservateur : si une autre action change la référence détectablement, ne pas l'écraser. La fenêtre entre comparaison et rename n'est pas une transaction inter-processus générale ; le propriétaire de livraison assure une seule bascule autorisée sur cet alias pendant l'opération et consigne cette limite.

## Recette et retour arrière

Depuis le binaire candidat puis la commande courante : status/who. Séparément, depuis le Mac avec sa socket locale et SSH Cartae : recette synthétique Python aller-retour095. Elle écrit deux agents et deux messages identifiés via l’API normale et atteste le transport/maître, pas le binaire client. Les lectures du maître identifient le même maître160 ; aucune relance n'est demandée. Ne pas exécuter une commande spawn pour prouver la communication. Échec : rétablir096 atomiquement si le lien courant correspond encore à161, sinon conserver et signaler la mutation tierce. Après rollback, relire le lien et exécuter status par le client restauré ; conserver ces preuves. Un rollback réussi reste une livraison161 non accomplie.

## Compatibilité

Aucune évolution du protocole métier, des bases ou des permissions dans161. Le build utilise la source160 déjà livrée sur Mac. La compatibilité affirmée couvre les commandes réellement exercées, pas toutes les plateformes, fournisseurs ou missions distantes. Aucun nouveau runtime T3 chargé.

## Extrait versionné de bascule à vérifier en fixture

Le corps Python ci-dessous est la primitive de livraison, extraite et exécutée telle quelle par la fixture du harness089. Aucun helper permanent. Les quatre arguments sont lien courant, cible attendue, nouvelle cible et lien temporaire voisin propre au lot. ROOT relève propriétaire/groupe du parent avant exécution ; le parent partagé n'est pas chmodifié. Pour rollback, inverser cible attendue et nouvelle cible, choisir un autre nom temporaire et vérifier ensuite readlink/status.

```python
# BEGIN SPEC161_ATOMIC_LINK
import os
import sys
from pathlib import Path

link, expected, target, temporary = sys.argv[1:]
if not all(Path(p).is_absolute() for p in (link, expected, target, temporary)):
    raise SystemExit("chemin absolu requis")
if Path(link).parent != Path(temporary).parent or link == temporary:
    raise SystemExit("lien temporaire voisin distinct requis")
if not os.path.islink(link) or os.readlink(link) != expected:
    raise SystemExit("reference courante modifiee")
if not os.path.isfile(target) or not os.access(target, os.X_OK):
    raise SystemExit("cible inexecutable")
# symlink echoue si le nom existe deja ; ne jamais effacer un tiers.
os.symlink(target, temporary)
try:
    if not os.path.islink(link) or os.readlink(link) != expected:
        raise SystemExit("reference courante modifiee avant remplacement")
    os.replace(temporary, link)
finally:
    if os.path.islink(temporary) and os.readlink(temporary) == target:
        os.unlink(temporary)
if not os.path.islink(link) or os.readlink(link) != target:
    raise SystemExit("reference finale inattendue")
print(os.readlink(link))
# END SPEC161_ATOMIC_LINK
```

Limite explicite : la comparaison puis os.replace ne verrouille pas les processus tiers non coopérants. L'opération est réservée à un seul livreur, et toute divergence observée impose un refus. Les fixtures couvrent aller, retour, cible initiale étrangère et nom temporaire occupé. Ce bloc est un contrat à tester, pas une preuve qu'une bascule réelle a déjà réussi.
