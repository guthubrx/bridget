# 270 : stockage configuré des compilations

Statut : livré et qualifié sur le volume externe de SYS, session 270.

Le nettoyage n’enregistrait que les caches directs du compte sous `.cache`.
Une compilation volumineuse pouvait donc remplir le système de fichiers système.
Un simple target-dir externe sortait du budget natif. Un lien vers un autre disque
était refusé par les gardes existantes. Le besoin est de déplacer les builds sans
perdre ces gardes et sans ajouter de chemin propre à une installation dans le code.

Le compte peut fournir un fichier privé `~/.config/bridget/build.json`, version 1,
avec `root` et `root_id`. La racine doit être réelle, absolue, privée, appartenir au
compte et résider sur un autre système de fichiers que `/`. Elle porte une ancre
privée `.bridget-build-root.json` avec cette même identité. Le lecteur ne crée
jamais une racine ou son ancre manquante. Un volume absent, une ancre résiduelle
sur le disque système ou une configuration indirecte refuse cette compilation
avant Cargo. Les autres travaux ne sont pas arrêtés.

Sans configuration, le comportement existant est conservé. Avec configuration,
la cible automatique est un cache `bridget-build-*` stable par dépôt, sous cette
racine. Un ancien override hors volume reçoit un refus précis. Les caches locaux
et configurés gardent ensemble le budget de 10 Gio et 7 jours, les marqueurs de
propriété, les verrous et le lecteur natif des processus. Une racine externe
indisponible n’interdit pas le ménage des caches locaux encore vérifiables.

L’identité du répertoire est revérifiée après l’entretien préalable et après
Cargo. Si le volume disparaît pendant Cargo, le résultat n’est pas qualifié comme
un succès et les fichiers produits restent à inspecter. Le script ne peut pas
garantir les opérations internes de Cargo pendant un démontage concurrent. Il ne
relance pas la compilation et ne retire pas de cible non qualifiée.

La lecture des petits JSON est bornée, sans suivi du dernier lien et sans attente
sur un FIFO. Les cibles contenant `..` sont refusées avant normalisation, pour
éviter qu’un lien suivi d’un parent sorte de la racine. Le parsing court de la
cible reste dans ses deux appelants ; aucun adaptateur supplémentaire n’est créé.

Le cache commun sérialise les compilations qui le partagent, comme les caches de
staging déjà partagés par ce projet. Un besoin réel d’isolation peut choisir un
autre enfant direct `bridget-build-*` de la racine, toujours dans le même budget.
L’énumération lit la configuration une fois par passage, puis parcourt les caches
en O(n). Aucune dépendance ajoutée et aucun changement des exécutables Bridget.

Le rangement de l’installation conserve les données utiles : les anciennes
sauvegardes sont copiées hors du disque système, vérifiées puis redirigées par
leur ancien chemin. Une observation incomplète, un lecteur actif, une copie
différente, un dépôt Git ou un fichier spécial conserve la source. Les versions
active et de retour, les états actifs et les programmes d’entretien référencés
restent en place. Les chemins de cette installation restent dans ses reçus privés.
