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
