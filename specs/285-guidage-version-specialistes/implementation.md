# Guidage version du catalogue

Le parseur de spécialistes attend un entier `u16`. Deux appels réels KS
utilisaient `version:"1"` et `action:"search"`. L’ancien texte indiquait
systématiquement que `catalogue` n’existe pas, sans identifier le type invalide.
Le correctif nomme le nombre JSON attendu. La branche catalogue ne s’applique
que si cette action est effectivement fournie et si la version est entière.
Aucune coercition, modification de schéma, de droits ou d’état.

Revue indépendante : ac39f5df-dd6e-4265-ade2-cddd17e73129, résultat lu et accusé.
Aucun défaut bloquant relevé. Deux suggestions non retenues dans ce lot :

- Ne pas exposer le texte libre de serde : il peut reproduire une valeur ou un
  nom de champ libre. Les diagnostics restent fermés et l’allowlist existante
  traite déjà les clés inconnues avant ce parseur.
- Lors de deux erreurs simultanées, conserver la première correction utile,
  sans construire un second validateur du schéma dans le chemin d’erreur.

Le test du nouveau refus et le groupe des tests MCP passent sur Mac et Linux,
ainsi que Clippy strict workspace/all-targets. Les tests se recouvrent :
21 tests distincts sont qualifiés par plateforme, pas 23. Le chemin numérique
valide atteint le transport ; la socket de test volontairement absente prouve
qu’il n’est pas refusé comme paramètre invalide.

La recette reconstruit le runtime 284 puis applique quatre fichiers modifiés.
Le résultat est identique aux fichiers qualifiés ; main n’est pas compilé.
Le programme réservé 260 reste exclu. Les preuves sont conservées dans
/Users/moi/.cache/bridget285-guidage.

L’identité Git manque dans le dépôt et le worktree Parix KS examinés.
Le worktree est propre après le commit du parent. Ne pas inventer d’auteur,
ne pas recopier une configuration et ne pas modifier user.name/user.email.
Ce prérequis reste à traiter par le propriétaire hors de ce correctif de message.

Les façades MCP déjà chargées peuvent conserver l’ancien texte jusqu’à leur
rechargement. T3 n’est ni arrêté ni remplacé par cette livraison.

Les releases Mac/Linux et l’APK 8049 réussissent. Le Linux exige glibc 2.34,
compatible avec la cible Debian 12 (maximum 2.36). La signature Android reste
celle de la version 0.8.48.

Le harnais MCP supplémentaire a d’abord omis initialized, puis supposé à tort
la forme du schéma projeté. L’outil et sa description corrigée ont été lus,
mais l’enregistrement de preuve complet a échoué. Le harnais est arrêté ;
aucun résultat de validation complète ne lui est attribué. Les tests ciblés
exécutent le parseur et le chemin vers le transport.

## Publication et activation observées

Code poussé : `46a8bbe13befbcca334f1d99c21d4ac90963e27b`. Version publique :
https://github.com/guthubrx/bridget/releases/tag/v0.8.49.
Les six fichiers publics sont téléchargés anonymement et leurs empreintes
vérifiées. Les archives portent bien les exécutables qualifiés.

Services chargés : coordinateur, mobile et worker Cartae ; workers SYSLE2,
KS, Mac et M1 ; connecteur et outil de mise à jour Mac. Le lien natif répond
avec `285-guidage-version-r1`. Le mobile répond. Les images de retour 284 sont
vérifiées avec leurs formats et les pointeurs privés de mise à jour.

- Mac : `aed5340a2e0efcdd8b8a63d4fcd591937fb1012d07d7faa751eaa8864ccc9218`.
- Linux : `29f753c40bd6c2128ca487a952ace0d248966fbbe56fc27b0033382de4a93429`.
- APK : `5077a18a59a34b0c678ce4a17bf7bc4aef228950564ca6991fb2c2295ffc5115`.

La première bascule core/SYS a été refusée par la garde de démarrage : tâche
`a3bef922-235c-4bb9-91f0-e2ffdb1b79a5` encore `starting`. Une lecture suivante
atteste `released`, `native_ready=true`, puis la bascule réussit en conservant
son processus. La coupure du coordinateur dure 3,66 secondes. Les lectures de
garde ne sont pas un verrou atomique contre toute admission concurrente.

**Activation du guidage incomplète dans les anciennes façades MCP.** Un appel
volontairement invalide `version:"1"`, refusé avant tout envoi, prouve que la
façade de cette session renvoie encore l’ancien texte. T3 conserve PID 77762,
né le 11 novembre à 14:10:41. Il n’est pas redémarré, conformément au choix
utilisateur. Les prochaines façades chargent le correctif ; les sessions
existantes peuvent utiliser immédiatement le nombre `version:1`. Aucun
rechargement global ni remplacement de mission n’est imposé.

Six avis avant/après sont conservés en `information`, sans réveil. Le parent
Vie des habitants reçoit aussi la cause exacte et la limite Git confirmée.
Le reçu `stored` ne prouve pas la lecture.

## Ménage gardé

1,15 Gio de caches Mac et 0,70 Gio de debug SYS retirés (tailles logiques
arrondies). Exports et anciennes images inutilisées retirés :
864923762 octets supplémentaires. Le profil release SYS reste
conservé : le descripteur 7 du worker PID 4189965 n’a pas pu être qualifié
par le lecteur natif. Copies Cartae et images réellement utilisées restent
protégées. Aucun processus métier arrêté pour nettoyer.

Le ménage M1 n’a pas commencé : la copie préalable du helper a échoué avec
le statut SSH 255 après la vérification d’activation réussie. La cause de cet
échec de transport reste inconnue ; aucun état actif n’en est déduit, aucun
repli d’accès. Les fichiers restent conservés. Le rapport du dépôt ne trouve
aucun worktree/branche éligible ; les vues de revue restent des preuves.

Preuves et limites : /Users/moi/.cache/bridget285-guidage.
