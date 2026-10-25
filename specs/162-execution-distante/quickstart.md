# Quickstart 162 — recettes après implémentation

Ce document conserve le plan de validation et ses checkpoints historiques. Le complément ci-dessous consigne les prérequis Linux vérifiés pendant G8 ; le statut de livraison reste dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/162-execution-distante/validation/livraison162.md.

## Prérequis Linux vérifiés pendant G8 — 13 octobre 2026

Avant une mission distante isolée, vérifier le Git effectivement résolu dans le PATH du worker. Le Git système 2.43 échouait à la garde existante de l’isolation 154, qui exige Git ≥ 2.44. Le Git privé 2.56.0 installé sous /home/moi/.local/lib/bridget-tools/git-2.56.0/bin est placé en tête du PATH du worker. Le Git système reste intact. La validation des versions, identités Git et HEAD des trois projets figure dans /Users/moi/.cache/bridget-delivery162-20261013/git162-validation.json. Ce reçu précède le changement du PATH ; il ne prouve pas, seul, l’environnement du service après relance.

Pour la sandbox Codex sous Ubuntu 24.04, bubblewrap doit aussi pouvoir créer son espace utilisateur. Le profil Ubuntu ciblé /etc/apparmor.d/bwrap-userns-restrict a été installé et chargé ; les profils bwrap et unpriv_bwrap sont en mode enforce. Le réglage kernel.apparmor_restrict_unprivileged_userns reste à 1. Ne pas désactiver cette restriction globale pour faire passer la recette. Ce choix suit les [prérequis OpenAI pour la sandbox](https://learn.chatgpt.com/docs/sandboxing#prerequisites).

Les preuves sont /Users/moi/.cache/bridget-delivery162-20261013/sandbox162-profile-installed.json et /Users/moi/.cache/bridget-delivery162-20261013/sandbox162-probes-r3-result.json. La sonde R3 a réussi : lecture autorisée, écriture refusée avec EROFS et réseau refusé avec EPERM, profil bwrap//&unpriv_bwrap actif. Elle valide ces restrictions dans ce contexte ; elle ne vaut ni appel de modèle ni validation globale des missions.

Retour arrière ciblé AppArmor : fermer les nouvelles admissions et attendre l’arrêt attesté des sandboxes concernées. Vérifier que /etc/apparmor.d/bwrap-userns-restrict porte encore le SHA256 11d39094f044f0cda0febb3ad517b830301da6b2ce929664af09ee9e4dd264f9. Le fichier était absent avant 162. Décharger uniquement ses profils avec /sbin/apparmor_parser -R /etc/apparmor.d/bwrap-userns-restrict, puis retirer uniquement ce fichier si son empreinte est inchangée. Si le fichier a changé, conserver l’état et réexaminer le retour arrière. Ne pas décharger tous les profils AppArmor, modifier les sysctl ou supprimer les reçus worker. Le retrait rend de nouveau indisponible le prérequis sandbox ; garder la route fermée tant qu’il manque. Ces opérations de retour arrière ne sont pas exécutées par cette documentation.

## Préflight

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/quickstart.md#extrait-1).

Le choix utilisateur autorise Bridget, Roadie et Opus2D ; Cartae est exclu. Vérifier racines, identité Git, commit de base et modifications à préserver. Le dépôt Cartae distant observé comporte une modification ; ne pas la supprimer. Contrôler les chevauchements canoniques avec credential, répertoire privé et état worker à Prepare puis RELEASE. Tester les deux sens du chevauchement et une résolution impossible. Les modes privés ne constituent pas une isolation entre fournisseurs du même UID ; ne pas prétendre confiner full-access. Vérifier le fournisseur réel et son authentification, pas seulement la présence de son binaire. Provisionner les credentials privés sans les afficher. Routes désactivées par défaut ; paramètres manquants : tests isolés seulement.

## Validations prévues

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/quickstart.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/quickstart.md#extrait-3).

## Livraison

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/quickstart.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/quickstart.md#extrait-5).

## Nettoyage et rollback

Archiver les preuves avant retrait. Fleet passe au schéma 5 et les requêtes stockées sont strictes. Aucun downgrade de l’ancien binaire sur l’état 162 inchangé. Avant activation : sauvegarde cohérente DB/Fleet/registre/état worker après fermeture des admissions et quiescence attestée. Une restauration jointe n’est permise que sans perdre une activité postérieure ; sinon garder 162, désactiver la route et conserver l’état. Révoquer une route empêche les nouveaux lancements ; elle n’efface pas les reçus des exécutions actives ou ambiguës. Aucun nettoyage d’un worker perdu, d’un résultat non confirmé ou d’un commit non récupéré. Conserver les branches de tâche selon les instructions de leur mission jusqu’à intégration attestée.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/quickstart.md#extrait-6).


## Procédure opérationnelle préparée, NON EXÉCUTÉE

Le CLI worker est compilé et ses recettes isolées R3 passent. Son installation Linux et l’activation réelle restent à faire. La procédure canonique est /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/livraison162.md. Elle produit ExecStart depuis le préfixe absolu /home/moi/.local/lib/bridget-communication/162-SHA12/bin/bridget, avec SHA complet et identités contrôlés ; aucune variable non développée dans l’unité systemd.

Avant toute bascule : lire /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget control status, fermer les admissions par control halt-delegations --reason livraison162 sans cancel-active, puis prouver la quiescence complète. La liste active limitée à 100 n’est pas une preuve exhaustive. Conserver l’état humain initial ; control resume-delegations ne doit pas rouvrir un arrêt humain préexistant ou postérieur.

Les stores maîtres observés sont /Users/moi/.cache/bridget-core/bridget.db, /Users/moi/.cache/bridget-core/bridget.fleet.json et /Users/moi/.cache/bridget-core/agents.json. Après arrêt attesté de leurs writers, sauvegarder SQLite et copier Fleet/registre dans une racine privée neuve, avec l’état worker cohérent ou sa preuve d’absence. Vérifier les copies et le manifeste joint avant remplacement. Les chemins et la quiescence doivent être réattestés au moment réel, pas déduits du préflight.

Les candidats privés restent désactivés dans /Users/moi/.cache/bridget-delivery162-20261013/worker-config-candidate. Le registre worker prévu est /home/moi/.config/bridget/agents.json ; sa socket fédérée /home/moi/.cache/bridget-core/bridget.sock reste à réattester. Ne pas confondre ce registre Linux avec le registre maître sous bridget-core. Les secrets restent privés. L’authentification Codex observée ne vaut pas preuve de quota ni appel réel de modèle.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/quickstart.md#extrait-7).


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/quickstart.md#extrait-8).

Checkpoint courant : Converge1 est clos non convergé sur une trace de remplacement worker manquante (T008). ROOT a ajouté T043, correctif et validation encore attendus. Le développement repasse In Progress, 42/43 ; le checkpoint 42/42 précédent est historique. Aucun changement aux gates G8/G9 ni activation. Attendre la preuve et le GO ROOT avant clôture 43/43.

État courant après GO ROOT T043 : trace worker corrigée et oracle ciblé réussi, développement/préparation Implemented, 43/43. Converge1 reste historiquement non convergé ; Converge2 ciblé doit encore statuer. Les gates G8/G9 restent NON EXÉCUTÉS ; aucune activation, installation ou livraison finale déduite de 43/43.

État courant : contre-revue code R1 BLOCKED, développement rouvert à 43/44 par T044 (réconciliation starting/refus durable et E2E sans état terminal injecté). Converge2 reste historique ; aucun passage 3 encore, audit pré-R1 suspendu. Attendre correction et validations avant reprise des gates. Aucun changement de l’autorisation des projets, aucune installation ni activation.
