# Worker Mac — enregistrement de la matrice
1 novembre2026. Enregistrement corrigé en configuration ; admission réelle non requalifiée.

Signal opus201-signal-revue-capitalisation-probe-20261101-01 :
demande opus201-revue-capitalisation187-20261101-01,18capability_unavailable,
not_recorded/tasknull. Les fichiers temporaires cités ne sont plus présents ici.
La cause historique exacte reste donc non attestée.

## Constat courant et correction
LaunchAgent /Users/moi/Library/LaunchAgents/com.bridget.worker.monordinateur.plist :
worker machine-id monordinateur, mais registre
/Users/moi/.cache/bridget-core/origin-agents177.json dont les4connexions Mac
et la politique locale sont déclarées sous local.
candidate_probe_v2 recherche exactement identity.machine_id dans machine_policies
et dans la clé de matrice. Ce décalage suffit à refuser18entrées avant préparation.
Le même décalage était déjà décrit dans probe-targets.md pour Roadie148.

Le registre d'origine sert aussi aux lancements locaux : il n'a pas été modifié.
Premier emplacement du registre dédié :
/Users/moi/.config/bridget/worker-monordinateur-agents.json,
copie privée0600 avec les seules normalisations local→monordinateur pour la
politique machine et les4connexions existantes. Aucun programme, modèle, effort,
limite, mapping, compte ou droit ajouté. Transformation inverse égale exactement
au JSON d'origine ; fichier d'origine inchangé, SHA256
1bedf775f64d1c27e3a85cfbadad87fb3ef898f68a5966377bb5836a07ccd659.
Registre dédié SHA256
b2219f3a1871e5a82c843ec980fe8050c8b4da8ea76a233979b1a9dc0d3b9c83.

## Contrôles et activation
Chargement natif AgentRegistry via clients check sur le nouveau registre :
compatible, host et machine_id monordinateur. Aucun appel modèle.
Matrice courante : politique monordinateur activée,4connexions/18entrées de
modèle déclarées, correspondances explicites ; aucune connexion créée.
Store commands présent et vide avant le passage : aucune mission active.
Sauvegarde du plist et du store ; bootout/bootstrap natifs du seul worker Mac,
sans signal forcé. PID27189→89501, même binaire0.7.3/SHA5bd1e2dff81e9021ee04873d5e0a0c1cc3e1c9eecfef417bd21ca8d7b61ff6a0.
Argument registry-file relu sur le processus chargé : nouveau registre dédié.
Maître, pont, services métiers, tâches et résultats non relancés.

Lecture native capabilities revenue :18options Mac. CodexPro annonce8modèles,
dont un hors politique toujours non admissible. Cela ne prouve aucun entitlement.
codex-mac-normal reste definition_changed ; la définition à qualifier n'est pas
changée dans cette correction. Claude/GLM restent unsupported_provider pour la
découverte automatique, sans déduction d'absence de connexion.
Aucune admission ni revue métier de qualification lancée. Les gardes suivantes
Git/droits/programmes peuvent toujours refuser ; ne pas annoncer une admission
garantie. Aucun rejeu de capitalisation187 ou autre mission d'issue inconnue.

## Passation et sources
Monuments informé par bridget221-mac-registration-capitalisation187-20261101-01,
remise9bdd621e-f6b3-4ec8-a42a-36234b18431d in_flight, lecture non attestée.
Responsable225 informé de préserver le nouvel argument du LaunchAgent,
remise578ed369-5ff4-4ae5-9b70-cca96dd97eeb in_flight.
Reçus et sauvegardes privés : /Users/moi/.cache/bridget221-mac-registration.
Ne pas rétablir origin-agents177.json comme registre du worker lors d'une mise à
jour. Son rôle local reste distinct. Correction technique de configuration :
aucun build, changement de version ou nouveau diaporama nécessaire.
Programme221 inchangé dans son avancement ; ce reçu ne clôt pas sa parité globale.

## Raccord au namespace fédéré — correction du premier emplacement

Le signal bridget225-project-preparation-link-evidence-20261101-01 a révélé
une régression de cette configuration. origin_proof.rs utilise le parent du
registre comme namespace pour has_link. prepare-project.py utilise le même
parent pour installed_links. Le placement initial sous .config/bridget était
donc incorrect pour ce parcours. Il a été choisi par ce responsable ; aucune
panne réseau ni modification de droits n'a été constatée.

Registre désormais chargé :
/Users/moi/.cache/bridget-core/worker-monordinateur-agents.json.
Ses octets sont strictement identiques à la première copie ; même SHA256
b2219f3a1871e5a82c843ec980fe8050c8b4da8ea76a233979b1a9dc0d3b9c83.
Le registre d'origine et les configurations/reçus de fédération sont inchangés.
Le helper réel installed_links, en lecture seule sans SSH, trouve zéro lien
pour l'ancien parent et trois liens validés pour le namespace existant.

Store sans commande active avant le passage. Sauvegardes puis rechargement
natif du seul worker Mac : PID89501 vers PID27140, même image0.7.3 et SHA.
L'argument registry-file du processus chargé a été relu au nouveau chemin.
Aucun changement de compte, de droit, de lien fédéré ou de tâche.
L'ancienne copie privée reste conservée comme sauvegarde inactive.
Reçu : /Users/moi/.cache/bridget221-mac-registration/namespace-r2/activated.json.

Préserver le parent attesté du registre dans les prochaines installations,
tant que le parcours utilise registry.parent comme namespace. Ce correctif de
configuration ne découple pas ces deux notions dans le code.
La demande bridget225-cli-table-code-review-20261101-01 n'a pas été rejouée.
Le contrôle du lien ne garantit pas l'admission ni le résultat d'une revue.

Le guidage project_preparation_link_missing est aussi corrigé : il nomme le
contrôle manquant et demande de comparer registre effectif et root des liens.
Il conserve code, clé et admission ; il ne transforme pas le refus local en
panne réseau ou arrêt global. Ce delta exécutable doit être intégré au paquet
0.7.4 du responsable225 ; il n'est pas annoncé chargé avec la réparation0.7.3.

Contrôles du delta : un test Rust ciblé PASS
spec224_project_preparation_link_refusal_names_only_the_failed_check, via
scripts/build.py ; compilation du module de tests terminée en 1min26s.
git diff --check sans erreur. Les avertissements hérités ne sont pas modifiés.
L'entretien automatique a conservé le cache : observation des processus
incomplète. Aucun nettoyage forcé ni nouvelle compilation de qualification.

Contre-revue indépendante demandée une fois sous
bridget224-namespace-link-guidance-review-20261101-01 : refus brut
auxiliary_identity_unproven : primary_absent, sans identifiant de tâche fourni.
Aucun nouveau lancement ni route de remplacement. Auto-relecture déclarée :
la branche ne change ni préparation, ni admission, ni preuve, ni état ; elle
projette un code fermé. failure(permission_attestation_unavailable) ne passe pas
par cette branche, donc absence de récursion confirmée par le test exécuté.

Après correction, catalogue natif lisible : 18options Mac et gel désactivé.
updates status : rapports frais des quatre hôtes en0.7.3. La réparation du
raccord est active ; le nouveau guidage reste à charger par le lot225.
