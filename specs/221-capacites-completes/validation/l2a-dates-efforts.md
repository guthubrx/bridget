# Complément 221 — dates, efforts et garde du catalogue

État : sources contrôlées, livraison en préparation. Programme global In Progress. T181 reste ouverte jusqu'à activation et lecture native. C01–C07, T055, C19 et T177 ne sont pas soldés par ce complément.

## Résultat
Le catalogue expose séparément date de lecture et date de réception, âges et expiration locale estimée. Les écarts entre efforts configurés et annoncés restent des observations ; ils ne donnent aucun droit et ne changent aucune admission. Le contexte machine/connexion/compte est déjà porté par l'objet parent ; aucune répétition de ces identifiants par modèle.
Le signalement d'arrêt incertain survit aux erreurs du registre, à une politique désactivée et au retrait/rétablissement d'une connexion. Seule la preuve d'arrêt du groupe ou de récupération du marqueur libère la garde. Le cache reste borné à32 clés. Le conseil de définition divergente nomme la vérification requise. Une vue globale indisponible demande de vérifier les marqueurs avant toute bascule.

## Contrôles source
Commande via scripts/build.py : cargo test --workspace --lib spec22 -- --nocapture.
R9 :34 tests daemon et2 transport réussis,3 recettes privées ignorées. Les petits sous-processus internes ne sont pas additionnés comme tests indépendants. Log privé /home/moi/.cache/bridget221/l2a-review-fixes-r9.log.
Cas propres : horloges source/réception, recul de l'horloge locale, date future, observation ancienne reçue récemment ; efforts annoncés/configurés ; modèle identique sur deux serveurs ; aucune mutation des tâches/catalogue déclaré ;128connexions x256modèles, trame complète13047160octets sous16Mio ; registre invalide/off, retrait/réintégration, définition changée,33connexions dont32inactives, saturation du cache des gardes.
Le test220 existant utilise un vrai processus qui ignore TERM : garde conservée lors de refresh/reconnexion, arrêt constaté après sortie autorisée, puis libération explicite. Aucun SIGKILL ni nouveau privilège.
Guides :32pages/522liens/13pairesFR-ENPASS ; git diff --checkPASS. Deux tests catalogue212 réussis en complément, soit38testsdistincts sur la source finale.
Échecs conservés :R3 de mesure utilisait une borne inventée de demi-trame, remplacée par le vrai plafond ;R5 perdait les mots « marqueurs privés » exigés par un test existant, conseil corrigé sans affaiblir l'assertion. Aucun de ces échecs n'est caché ou compté comme réussi.

## Contre-revues
Réemploi :2ceacc8c. R1 :2824b7b8 ; R2 :f9611342 ; R3 :2272ec55. Lectures seules d'extraits, pas qualification runtime indépendante.
Les défauts confirmés ont été corrigés : perte de garde sur registre invalide ou retrait, conseil définition insuffisant, guard pouvant être remplacé par un résultat sans preuve d'arrêt, absence de conseil global quand la politique est fermée. La suggestion de libérer sur Observed n'a pas été suivie : seul le groupe récolté ou les marqueurs récupérés font preuve.
Hypothèses non retenues après lecture : limite1Mio par cycle (elle est par instance de Rpc/MetadataSocket d'une sonde) ;256modèles déclarés par connexion au maximum global (la politique borne le total à128). Le modèle est déjà corrélé au compte/serveur par l'objet connexion. Aucun rejet de dossiers Codex volontairement partagés.
Résultats lus, ACK et clôture demandés une seule fois. Worktrees conservés tant que les usages ne sont pas tous vérifiés.

## Analyze et Converge
Analyse manuelle comparant spec/plan/tasks/code/tests puis relecture ; primitive projet absente déjà constatée. Aucun CRITICAL retenu dans ce diff. Réemploi220, sans nouveau store ni autorité. Converge source du complément : aucune catégorie supprimée ; écarts de livraison encore ouverts dans T181. Aucun succès runtime annoncé ici.

## Limites
Les adaptateurs de découverte autres que Codex et le parcours complet multi-comptes ne sont pas qualifiés. La liste d'un programme ne prouve ni la fraîcheur de son cache fournisseur ni l'accès au modèle. Un login externe sans signal non secret attend un contrôle périodique/explicite. La cible120s n'est pas garantie.
Correction technique dans les données natives, pas modification d'une interface graphique. Pas de diaporama de clics fabriqué. Les preuves d'activation et la restitution native seront ajoutées après la bascule. Aucun modèle ou mission métier relancé.


## Audit source final
V15 /home/moi/.cache/bridget221/audit-l2a-r3 : finalize pass/release_blockedfalse, validate true,12/12contrôles applicables vérifiés. Semgrep pack figé et Gitleaks fichiers exécutés, zéro candidat. Historique et CVE globaux exclus car aucun commit historique ni dépendance modifié. R1/R2 conservées sans les présenter comme l'audit de la source finale.
Grille42critères :35notés/35applicables ;7hors ce périmètre source. Sécurité75(3/3), sécuritéIA75(3/3), fiabilité75(1/1), tests69(4/4), ressources69(4/4), complexité75(3/3), qualité67(3/3), architecture75(3/3), responsabilité75(3/3), minimalisme75(3/3), métier75(3/3), traçabilité75(2/2). Les points démontrés égalent ces notes sur le périmètre examiné. Détail des sous-contrôles, preuves et limites : /home/moi/.cache/bridget221/audit-l2a-r3/axis-scoring.md.
Les notes ne sont pas une mesure de tout Bridget : pas de benchmark de latence, de qualification graphique ou de runtime encore chargé. La décision source permet de poursuivre la livraison ; elle ne prouve pas son activation.
