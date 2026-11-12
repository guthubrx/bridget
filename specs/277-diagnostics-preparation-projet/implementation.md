# Lot 277 : diagnostic sans sous-cause inventée

D1 : a-f884cf86cbd3094df2540dd0a321df2e est corrélé sur KS et maître.
Sonde 4c8fe26a-8d59-4d97-88ea-f2f9841b47ea terminée le 11 novembre
à 23:39:39 UTC : quatre candidats, 39 971 ms, chacun environ 10 s.
Le maître conserve quatre resource_limit. La même admission a ensuite réussi.
Le budget worker est de 45 s depuis la réception, attente comprise. Ce résultat
ne prouve pas une saturation CPU/RAM ni une panne de compte. La sous-étape
responsable des 10 s par candidat n’a pas été journalisée.

D3 : primary_absent est survenu pendant le passage276. Le connecteur a réexposé
le parent à 23:57:24 UTC ; l’admission est durable à 23:58:21 UTC. Aucun effet
métier n’a été rejoué pour ce diagnostic. La revue276R2 est déjà traitée dans
le commit a6fad403 et la version0.8.41.

Modèle D3/D4 : seq339 à 00:03:37 UTC atteste pinned=sonnet et
served=claude-sonnet-5-5. Le contrat compare deux étiquettes opaques et distinctes,
sans table d’alias. L’événement ne prouve ni substitution ni bascule de moteur.
Aucun modèle n’est réécrit et aucun alias n’est normalisé par supposition.

SYS : la remise 85164a8a-6b4b-4e77-b616-40330ae02d7f est maintenant acked.
Sa clé est mcp-3967244-6af5025b-4. Le projet inconnu a bien été signalé pendant
la mission5471b887 ; sa résolution initiale n’a pas de cause conservée.
Le contrôle dans le PATH du worker utilise Git2.56 et résout correctement
le common-dir du worktree. L’hypothèse de Git ancien est infirmée. Une présence
arrêtée sans projet ne prouve pas la cause du défaut initial.

SSH : corrélation abb5dd71-8b28-41e7-bec1-881d73e308b4, inspect, code255
sur le worker source Mac. Le helper jetait stdout/stderr et ne conservait que
l’opération et le code. Le journal SSH privilégié KS a été consulté : des clés
ont été acceptées à la même seconde, sans erreur corrélable de manière certaine.
Aucune relation causale n’est déduite du push Git précédent. Il est impossible
de reconstruire un stderr déjà détruit ; la correction porte sur cette perte.

Le helper classe désormais conservativement des signatures SSH connues,
uniquement pour255. Inconnu, ambigu ou signature hors des 64 Kio observés
restent unobserved. Seules des signatures textuelles fermées remontent au lecteur Rust. Même un code255 et stderr peuvent venir du distant : la cause et la provenance restent inconnues.
Ce lecteur valide les anciennes formes à deux clés et la nouvelle signature,
rejette les champs et valeurs libres, et garde le même code wire. Il ne renvoie
aucune sortie privée au parent. La résolution de projet garde ses règles ;
si elle ne produit pas de projet, un log corrélé à l’agent et à l’instance
sépare autorité non établie, worker indisponible, contrôle indisponible,
conflit et changement de connexion/commande. Aucun chemin ni nom libre.
Ces diagnostics ne prouvent pas une cause historique qui n’a pas été observée.

Sept tests Python passent. Le test SSH de bout en bout échoue sur l’ancien
helper (cause absente), puis passe avec la signature fermée. Le premier build
Rust a identifié deux noms non qualifiés dans le nouveau helper ; ils sont
corrigés en serde_json::Value/json, sans changement de logique. La recette
cumulative est produite contre276 ; elle conserve le travail260 réservé.
Les différences de mise en forme de la base sont traitées dans l’export,
sans copier le daemon principal entier ni contourner un échec de patch.


La revue R1 fdd544dc confirme confidentialité, compatibilité et invariants.
Son blocage Value/json est déjà corrigé et compilé. Le nouveau log est placé
après libération du verrou global. Les tests natifs253/254 et l’annonce138
exercent effectivement le chemin vide avec les étiquettes fermées observées.

Le contrôle élargi138 trouve un ancien échec afterclose138 : il est reproduit
avec les fonctions de résolution276 restaurées, sans le changement277.
Le test n’appelle pas le chemin de résolution modifié. Aucune réussite complète
de cette campagne élargie n’est revendiquée ; ce défaut reste conservé hors lot.
Les tests ciblés de résolution et d’annonce font partie de la qualification.

Trois tests138 avaient aussi une dépendance au dépôt entourant leur source :
CARGO_MANIFEST_DIR pouvait découvrir un dépôt sur Mac, pas dans l’export Linux.
Les fixtures créent désormais leurs dépôts temporaires. Le worktree orphelin
utilise le minimumGit2.44 déjà requis, sans créer de commit ni modifier l’identité.
Cela teste les mêmes racines Git et liens symboliques dans les deux environnements.
La recette ne reprend pas les deux lints Cargo.toml du travail260 réservé :
ils étaient absents du runtime276 et ne font pas partie de cette correction.


La revue R2 eb9f9a6d ne trouve pas de défaut bloquant. Sa remarque sur la
provenance SSH est traitée : le champ stderr_signature décrit le texte observé,
le champ cause reste unobserved. Cela ne prétend pas distinguer le client du
programme distant. Les tests vérifient cette séparation. L’environnement Git
est neutralisé pour la création des fixtures, sans changer celui que le test
doit précisément soumettre au résolveur. Le volume du nouveau log sera relevé
après activation ; aucune mesure de production n’est inventée avant ce passage.


Qualification ciblée : treize tests Rust distincts par plateforme couvrent les
nouveaux diagnostics, l’ancien helper, les projets natifs253/254, les résolveurs138,
l’annonce vide/conflit/auxiliaire et l’annonce à chaque reconnexion du wrapper.
Les sept tests Python du helper et les cinq tests de réparation passent.
La recette finale doit reproduire les neuf fichiers de l’export compilé sans fuzz.
Le seul échec élargi antérieur afterclose138 reste explicitement distinct de ce
résultat ciblé ; il n’a pas été supprimé, modifié ni déclaré réussi.

Clippy strict workspace/all-targets et les builds release passent sur Mac et
Debian12. Le binaire Linux exige glibc2.34, sous le plafond2.36. Le paquet Android
0.8.42/8042 est compilé et sa signature compatible est vérifiée contre0.8.41.
