# ADR-163 — Décision initiale sous autorité du maître

**Date :** 2026-10-13. **Statut :** Accepté par ROOT après relecture des corrections O1–O6 ; verdict externe R1 APPROVE_WITH_CHANGES conservé. Aucun code validé par cette ADR.

## Contexte

Le placement explicite 162 est livré. L’utilisateur veut automatiser au choix le moteur et la machine, avec des options indépendantes, désactivées par défaut et un serveur de décision générique déclaré par machine. Le choix ne doit ni augmenter les droits ni être recalculé après admission.

## Décision

Le maître filtre au plus huit profils configurés. Il réemploie les gardes locales et une lecture groupée des workers. Ce probe ne crée aucun worktree, fournisseur, bootstrap, reçu de lancement ou slot ; les commandes Git de lecture bornées restent nécessaires. Une négociation additive par époque autorise CandidateProbe, sans modifier Hello/Accepted v1 ni envoyer de variante inconnue à un pair ancien.

Un reçu technique dans DelegationStore conserve le canon automatique avant Task. Une transaction Immediate vérifie les collisions native/auto et écrit Task + lien au reçu de façon atomique ; publication après commit seulement. Le total UNION des clés retenues compte une seule fois reçu et Task associée sous le plafond de 4 096. Les deux index uniques historiques sont conservés. La clé appartient au demandeur attesté, pas au propriétaire racine. L’autorité interne chosen exige clé, canon et revision concordants.

La politique est relue pour chaque nouvelle admission puis figée. coordinator_epoch et revision CAS permettent de reprendre pending après redémarrage sans deuxième traitement concurrent. decision_started précède HTTP ; un état orphelin ne rappelle jamais le décideur. chosen précède l’admission native et fixe son canon. Un refus ultérieur ne déclenche aucun autre classement.

La frontière réseau reste un module étroit et le client minreq existant. Une tâche réseau unique possède le permis jusqu’à son vrai retour ; le demandeur dispose d’un budget distinct de deux secondes. with_timeout est exclu car il détache un thread interne. send_lazy impose les bornes réelles de corps, en-têtes et ligne de statut ; redirects 0. HTTPS utilise https-rustls et la validation normale du certificat. Aucun nouveau client HTTP, runtime async, tokenizer ou gestionnaire de secrets.

Bridget expose le résumé par le parcours natif Bridget. Les projections Lineage v1/v2 restent inchangées ; v3 négocie le résumé. Le lecteur passe de v3 à v2 uniquement sur unsupported_version structuré. Le passage historique de v2 à v1 reste permis sur unsupported_version ou invalid_request structurés. invalid_request en v3, timeout et refus d’autorisation ne provoquent aucun repli ; pages homogènes, journal/cancel sans repli.

## Alternatives écartées

Appeler Prepare pour chaque candidat crée des effets avant le choix. Conserver le choix en mémoire seule permet une décision répétée après crash. Une nouvelle base du décideur dupliquerait l’autorité métier. Libérer le permis au timeout peut accumuler des appels encore actifs. Étendre Hello v1 casserait les pairs stricts. Ces contraintes justifient les seules nouvelles frontières prévues.

## Conséquences et limites

Le mainteneur retrouve l’admission, sa politique figée et son issue dans le store maître existant. Les gardes de lancement et de reprise 162 restent le seul puits d’exécution. Le coût supplémentaire est une table technique, des variantes négociées et une projection de résumé. La politique et le probe restent bornés ; aucune boucle de polling ou file HTTP n’est ajoutée.

Un blocage DNS, de connexion, TLS, d’écriture ou de lecture serveur peut retenir le seul travail réseau indéfiniment ; les admissions suivantes utilisent leur repli autorisé ou refusent immédiatement. Un timeout n’arrête pas une inférence distante. La santé ne prouve pas le checkpoint ni la qualité du choix. Endpoint, protocole et recette réelle restent des gates d’activation. Aucun résultat produit ou service n’est testé par cette ADR.

## Validation prévue

Tests des axes indépendants, absence de réseau quand désactivé, probes sans effets métier, collisions/crashs, permis HTTP encore occupé, corps excessif, versions strictes et refus sans reroutage. Réemployer les harnesses existants ; aucune campagne facultative. Audit de réemploi et contre-revue avant Tasks, puis livraison et nettoyage complets.

## Précisions retenues après la contre-revue R1

La présence de MINREQ_TIMEOUT, même vide ou invalide, interdit l’activation et les appels du seul service de décision ; vérifier au démarrage et avant appel, sans modifier l’environnement global. Le daemon et la délégation explicite continuent. Le budget demandeur de deux secondes n’est pas un délai d’arrêt réseau. La raison de configuration non sûre ou de capacité occupée est exposée.

L’entretien existant traite les reçus d’époque précédente dès son premier passage après réconciliation, en lots bornés et CAS, E/S hors mutex, sans connexion ou rejeu du demandeur requis. pending mène à probes figés puis choix unique/repli autorisé ou decision_interrupted sans HTTP ; decision_started mène au repli/refus ; chosen rejoue l’admission idempotente puis lie la Task atomiquement. Les erreurs techniques ou de stockage restent non résolues, visibles et entretenues ; aucune clôture SQL non committée ni purge d’un état ambigu.

Tout refus après chosen est terminal pour le reçu auto, dont task_limit, pause ou retention_saturated. Parent ou droit indisponible interdit tout lancement. L’entrée native externe vérifie ce reçu avant by_agent_request sous Immediate ; le chemin interne conserve son refus uniquement dans le reçu auto, sans native_refusal sous la même clé. Les unicités et les cinq codes historiques restent inchangés.

Le plafond UNION inclut les refus auto. Sous pression, seuls les refus auto terminaux sans Task sont purgeables du plus ancien au plus récent, avec tombstone de trente jours dans la même transaction. Rejeu exact avant purge, request_expired après. Les admissions native et auto essaient cette purge sûre avant refus au plafond ; pending/chosen/ambigu/vivant ne sont jamais sacrifiés.

## Lieux de réalisation et provenance — orientation utilisateur retenue

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/163-decision-admission-sous-autorite-maitre.md#extrait-1).

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/163-decision-admission-sous-autorite-maitre.md#extrait-2).

Une seule compilation active à la fois dans toute la flotte, jobs=2, nice=10 et tests=2. Regrouper les tests prévus ; une suite FULL unique seulement si un gate la prévoit. Ne pas relancer une suite longue à chaque tâche ni après succès sans changement affectant ses résultats. Rejouer les seuls contrôles affectés après correction. Les preuves distinguent Linux, macOS, recette réelle et provenance installée.
