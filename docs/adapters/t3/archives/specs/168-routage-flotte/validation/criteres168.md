# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/validation/criteres168.md](../../../../../../../specs/168-routage-flotte/validation/criteres168.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Santé finale reçue : B en ligne, catalogue 27, pont 32 fils vers 97328 ; DB readonly quick_check ok/FK0, versions 7/12 ; worker B 3767260 actif, configuration inchangée. Application T3 65544 et tunnel 60206 préservés ; le serveur 65608 a été remplacé extérieurement par 97328 né à 16:51:02 CEST, sans signal T3 de notre part. La restriction temporaire de 16:56 CEST est levée.

<a id="extrait-2"></a>

## Extrait 2

Limites : FULL unique 2820 PASS/32 FAIL/71 ignorés ; 29 échecs initiaux PASS après reprises, trois annulations 159 non validables sur Linux, cause initiale T3 inconnue malgré replay PASS. Deux LOW rustls-webpki et une INFO unsafe-libyaml restent documentés ; aucun GREEN global. Aucun nouvel audit/FULL/P4/build/service/test/modèle par cette rédaction. Aucun retour direct au binaire 0.3.0 après reçu admission v2. Les commits documentaires descendants conservent la source binaire B et ne nécessitent aucune reconstruction.

<a id="extrait-3"></a>

## Extrait 3

| Exigence | Preuve disponible | Portée ou limite |
|---|---|---|
| FR-001 | P01, P06 ; trois recettes natives et rejeux dans un terminal. | Aucun signal T3 envoyé ; son serveur a été remplacé extérieurement. |
| FR-002 | P02 ; catalogue actif de 27 options. | Modèle, moteur, machine, connexion et effort séparés ; droit fournisseur inconnu. |
| FR-003 | P02 et préflight : sept IDs réels par connexion Codex ; déclarations Claude par machine. | Une déclaration ou model/list ne prouve pas l’accès à chaque modèle. |
| FR-004 | P01/P02 : 12 et 128 options ; registre 129 refusé ; score et Haiku réels sur 21 candidats. | Catalogue 27 distinct des ensembles admissibles ; aucune troncature à huit. |
| FR-005 | P03 ; arguments explicites des recettes conservés. | Chaque axe est filtré indépendamment ; aucune bascule silencieuse. |
| FR-006 | P03/P05/P14 : droits, contraintes et recontrôles avant effets. | Le score et les décideurs ne créent aucun droit. |
| FR-007 | Préflight : environnements déjà authentifiés ; aucun login ni copie fournisseur automatique. | La sauvegarde privée du credential worker sert au retour arrière. |
| FR-008 | P04 et preuves de crédits/fraîcheur : connexion, instance, définition et époque séparées. | Les faits d’un compte ne comblent pas les inconnues d’un autre. |
| FR-009 | P04 : quota inclus à 100 % et crédits utilisables ; faits conservés dans le POST et Lineage. | Aucun solde opaque ni autorisation de dépense déduit. |
| FR-010 | P02 et préflight : connexions Claude représentées ; OAuth Cartae attestée. | Pas de gestion de facturation ; le profil Mac déconnecté reste un fait distinct. |
| FR-011 | P05 et collecteur : âge, provenance, absence, péremption et horloge future vérifiés. | Les faits périmés deviennent inconnus ; aucune mesure de débit revendiquée. |
| FR-012 | P03/P05 : workload general/compilation fermé ; capacité déclarée et poids bornés. | Aucun mot-clé de mission ni framework de benchmark. |
| FR-013 | P06 : saturation avant décideur ; refus réel Mac resource_saturated avant reçu et Task. | Les anciens namespaces Mac à télémétrie OFF sont explicitement isolés. |
| FR-014 | P13 : deux Worktrees simultanés et rapports ; recettes réelles Darwin et Linux/Cartae. | Les recettes réelles sont Shared ; la séparation Worktree est prouvée par contrôle. |
| FR-015 | P12 : reprise depuis le choix figé. | Aucune migration à chaud ; pas de promesse de toutes les pannes possibles. |
| FR-016 | P06/P07/P08 ; réel Laya 2, Haiku 21 et score 21, avec résultats et rejeux. | Les anciennes tentatives en repli restent conservées. |
| FR-017 | P07 : candidat unique et axes fixes sans consultation. | Le rejeu consulte le reçu avant une politique nouvelle. |
| FR-018 | P02/P08 : politique et niveaux par machine ; activation B ON vérifiée. | Les scénarios OFF sont prouvés par les contrôles instrumentés. |
| FR-019 | P08 : niveau désactivé ou inaccessible sans HTTP ni enfant. | Le compteur réseau réel n’a pas été instrumenté dans les recettes. |
| FR-020 | P06 et adaptateur de service ; Laya réel sur deux candidats entiers. | Contrat indépendant du nom Laya ; aucune promesse Laya 128. |
| FR-021 | P06 et charges contrôlées : modèle, effort, ressources, quotas et crédits ; Haiku réel sur 21. | ID opaque seul autoritaire ; ensemble complet ou refus de capacité explicite. |
| FR-022 | P09 et tests du service : ID, effort, capacité et sortie tardive invalides rejetés. | Aucun effet métier depuis ces réponses invalides. |
| FR-023 | P09/P10 : outils métier interdits, isolation et bornes ; Haiku réel sur Cartae. | Seul StructuredOutput formate le JSON ; brief utilisateur borné, sans détecteur parfait de secrets. |
| FR-024 | P10/P11 ; correctif de délai T045B : un test exact PASS et trois sous-cas. | Le service respecte sa configuration dans le budget global ; défaut 2 s conservé. |
| FR-025 | P11/P15 et store : réservation durable, CAS de niveau et choix avant admission. | Les écritures Task et metadata sont atomiques dans les contrôles SQLite. |
| FR-026 | P11 ; trois rejeux réels après redémarrage avec mêmes canons, décision et résultat. | Payload Task brut différent ; aucun compteur HTTP/processus réel inventé. |
| FR-027 | P12 et rejeux réels cohérents. | Zéro redécision prouvé par les oracles instrumentés ; pas mesuré par HTTP réel. |
| FR-028 | P14/P15 et régressions : plafonds, permissions, rétention, annulation et legacy conservés. | Trois anciens cas annulation 159 non validables sur Linux restent signalés. |
| FR-029 | P13/P14 ; résultats réels au SHA B exact. | Shared hors Git général reste distinct ; aucune base distante implicite. |
| FR-030 | P13 et Lineage : base, branche, tête, commits et dirty contrôlables. | Un résultat disponible ne signifie pas intégration ; reçus finaux Git à venir. |
| FR-031 | Audit de réemploi et procédure Git : aucune synchronisation par le daemon. | Le coordinateur autorisé récupère et intègre ; cette synthèse n’exécute aucun Git. |
| FR-032 | PROUVÉ : récupération C1 intégrée/poussée, retraits Mac archivés et conservations Cartae motivées ; reçus de nettoyage et gardes reliés. | WT168/branche et 18 candidats conservés ; garde handles incomplète, aucune absence d’usage ni suppression présumée ; DB et preuves protégées. |
| FR-033 | P16 : mode, connexion et faits figés dans Lineage v4 ; v3 conserve null pour 168. | Faits historiques ; pas de nouvelle mesure au moment de la lecture. |
| FR-034 | P15, tests CLI/MCP et worker legacy : V1 préservé ; V2 refusé sans capacité, sans downgrade. | La nouvelle façade V2 reste stable lors d’un changement ON/OFF. |
| FR-035 | Préflight, activation B active_verified, health Mac/Cartae et trois voies réelles. | Catalogue et possibilités déclarées distincts des accès réellement observés. |

<a id="extrait-4"></a>

## Extrait 4

| Critère | Preuve disponible | Portée ou limite |
|---|---|---|
| SC-001 | P01/P06 ; missions et rejeux natifs privés par le parcours natif Bridget, résultats corrélés. | Aucun signal T3 ; remplacement externe du serveur consigné. |
| SC-002 | Catalogue 27 ; score et Haiku réels sur 21 ; P01/P02 pour bornes 12/128/129. | Le catalogue ne prouve pas le droit fournisseur à chaque entrée. |
| SC-003 | P03 et contraintes des requêtes réelles. | Aucune sélection des cas vérifiés ne viole l’axe imposé. |
| SC-004 | P04 et observations séparées normal/pro, crédits et quotas isolés. | Aucun regroupement financier ni crédit transféré. |
| SC-005 | P05/P06 : cible saturée exclue devant alternative fraîche ; refus réel Mac conservé. | Les recettes privées à télémétrie OFF ne servent pas de preuve de saturation en production. |
| SC-006 | Laya : decision_service sur 2 ; Haiku : lightweight_model sur 21 ; score : deterministic sur 21. | Niveaux OFF sans consultation prouvés par contrôle ; pas de compteur réseau réel universel. |
| SC-007 | P10/P11 et les trois sous-cas de délai configuré T045B. | created_at 1792075807 et updated_at 1792075812 sont des timestamps durables, pas un chronomètre monotone. |
| SC-008 | P11/P12 ; trois rejeux réels après redémarrage, mêmes canons et résultat. | Haiku : global 2/2/2 inclut l’essai à sept ; par clé matricielle un reçu et une Task. HTTP/processus non comptés. |
| SC-009 | P09/P10 : isolation, sortie fermée et StructuredOutput seul ; rôle Haiku réel distinct de l’exécutant. | Pas de détecteur universel des secrets du brief ni d’inventaire fournisseur non exécuté. |
| SC-010 | PROUVÉ : P13/P14, SHA B réel, rejeux et récupération C1 ; examen T048 terminé avec retrait ou conservation motivée. | Aucun gain physique inventé ; garde incomplète ferme le retrait. Contrôle final ROOT T049 validé. |
| SC-011 | Préflight par machine/connexion, catalogue public et health ; modèles exécutants et décideur identifiés. | Les déclarations et l’activation observée restent distinguées ; pas d’accès universel annoncé. |

<a id="extrait-5"></a>

## Extrait 5

- /Users/moi/.cache/bridget-delivery168/activation/B-runtime-followup168.json — SHA-256 6d50c3181012c13b2334722f3e3ae13468800c0f8c75d988cfeca27be2c8f86e : B active_verified, routage ON, catalogue 27, quick_check=ok et FK0.
- /Users/moi/.cache/bridget-delivery168/final-health/mac-health.json — SHA-256 be3ec722c86f8fcdd6de90353e1748e318b0de3beb086690cc66fdd276d114ba : daemon B en ligne, catalogue 27 ; pont running avec 32 fils vers serveur T3 97328.
- /home/moi/.local/state/bridget-delivery168/final-health/cartae-health.json — SHA-256 a409e7698a2f7ff7bdede77a2a2126eeda0d835052765473f0ef60850818f32a : worker 3767260 actif sur B, binaire et registre inchangés ; PID privés inventoriés absents.

<a id="extrait-6"></a>

## Extrait 6

Le serveur T3 65608 a été remplacé extérieurement par 97328, né à 16:51:02 CEST. Aucun signal T3 n’a été envoyé par nos opérations. L’application 65544 et le tunnel de production 60206 restent ceux des observations protégées. Le checkpoint initial ne prouve pas que 65608 est encore vivant.

<a id="extrait-7"></a>

## Extrait 7

- FR-032 et SC-010 sont prouvés : récupération/intégration C1, retraits Mac et conservations Cartae avec motifs et gardes documentés. Les gardes incomplètes ne sont pas contournées. Aucune exigence technique ne reste future ; contrôle final T049 validé et SHA documentaire contrôlé dans les reçus Git privés.
- T046, T047 et T048 sont closes : release vérifiée, C1 intégré/poussé, examen nettoyage terminé. T049 est validée ; aucune tâche ouverte.
- SC-007 est soutenu par les tests contrôlés de deadline. L’écart de cinq secondes entre created_at et updated_at du reçu Haiku ne mesure pas toute la chaîne avec une horloge monotone.
- FULL unique : 2 820 PASS / 32 FAIL / 71 ignorés. Après reprises, 29 échecs initiaux passent ; trois anciens cas annulation 159 restent non validables sur Linux. La cause initiale T3 reste inconnue malgré son replay PASS. Aucun GREEN global ni seconde FULL.
- Converge P3 CONVERGED ; audit V15 pass A sur son manifeste, deux LOW TLS et un INFO ouverts. Les axes qualité restent N/A. Aucun nouveau scanner, score ou audit n’est déduit de cette synthèse.
- Aucun modèle, test, build, service ni commande Git exécuté pour publier ce document. Les champs inconnus de quotas, crédits et accès fournisseur restent inconnus.
