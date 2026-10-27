# Livraison189 — Guidage des refus

Sources candidates : 001a97aaf814ac7fe0c7fbd0f9604665604a0e7b.
23 contrôles ciblés PASS et guides FR/EN vérifiés. Aucune FULL. Contre-revue indépendante b45d760a favorable ; résultat lu/ACK et garde fermée.

## Incidents inspectés
- Tâche d605c969-e6aa-4f26-822f-225134e38e8c, clé opus166-preuves-lot-autonomes-cartae-20261027-01 : le propriétaire a ensuite lu son résultat et fait ACK par le parcours natif au tour201. La lecture transitoirement refusée au tour200 n’est pas attribuée à une cause non observée. Aucune nouvelle mission, reprise, annulation ou lecture sous son identité.
- Admission opus166-correction-saut-planificateur-cartae-20261027-01 : refus durable resource_saturated, sans tâche. Le reçu183 ne conserve pas la preuve de la mesure du contrôle final. Son snapshot initial CPU35/mémoire11/charge28, seuils90/10/100, n’est pas saturé. Il ne prouve ni erreur d’admission ni saturation finale réelle. Ce reçu historique reste inchangé.

## Changement
Les prochains refus de capacité qualifiés conservent les mesures et seuils effectivement évalués, dans la transaction de refus du même reçu. Aucun seuil ou critère d’admission changé. Le code durable reste resource_saturated pour les anciens lecteurs. La preuve ferme et bornée ne remplace ni le canon ni l’histoire initiale ; les rejeux ne la recalculent pas. Une preuve absente reste inconnue.
Le précontrôle MCP d’attestation rend son étape caller_scope, une vérification fermée ou unknown, et une action précise. Aucun contexte de tâche n’est exposé avant attestation. L’opération métier n’est pas envoyée après ce refus.

## Runtime
Maître183 conservé ; aucune activation, aucun service redémarré, aucun store réel modifié. Le chargement189 dépend du même parcours de maintenance encore bloqué par une annulation historique sans preuve d’arrêt. Le candidat189 ne sera pas présenté comme actif. Aucune nouvelle préparation de maintenance identique répétée.

## Candidat Linux de la qualification initiale — remplacé
Chemin : /home/moi/.local/lib/bridget-communication/189-guidance-001a97aaf814/bin/bridget
SHA256 : 330d5e713febe617b94a23d309488698568340d657d4b97a799e3a3ec558f4a9.
Version0.6.3, build189-guidance-001a97aaf814 présent dans l’exécutable ; copie hors staging identique au build et commande installed passée. Manifeste eligible_for_activation=false. Cache partagé7,11Gio. Le correctif du précontrôle doit aussi être chargé dans le client MCP concerné ; ce candidat Linux ne prouve pas une mise à jour du client Mac.

## État de livraison
Sources et reçus intégrés/poussés sur main8d8d9cc5 ; checkouts Mac et Cartae propres et alignés avant cette clôture documentaire. Travail de la mission Opus2D, reçus historiques, binaire183 actif et versions de retour conservés. Worktree et branche source189 retirés après contrôle d’intégration, absence d’ignorés et de fichiers ouverts ; référence distante supprimée. Worktree de revue b45d760a retiré après clôture native et mêmes contrôles ; seule sa référence légère reste conservée car non intégrée au HEAD ancien du miroir SYS. Aucun forçage de Git ; aucun travail d’autrui retiré.

## Retour au demandeur
Message bridget-politique-qualification189-20261027-01, issued_at1793128518, delivery312e0abe-e5ae-44cd-b93f-a3cda99ceaf0 : accepté in_flight. Résolution native déjà constatée pour la lecture d605c969, limites du reçu historique de capacité et correctif source/candidat non actif transmis en un envoi. Aucune confirmation de lecture déduite du reçu de transport.
T004 reste ouverte : chargement maître et client MCP puis vérification réelle. Cette session n’est pas déclarée livrée dans le runtime actif.

## Complément — file des sondes KS
Signalement opus166-ksstore-resource-limit-vue-20261027-01 ; demande opus166-publication-vue-sans-reprise-monde-ksstore-20261027-01, base4481f4b6a38a0b23ebe10d92b1f6890ee24c0b1f, machineKS, sans tâche ni admission enregistrée. Le binaire KS réellement chargé est183-b6c045cd, source résolue. Le motif candidate/resource_limit est émis uniquement par le refus d’entrée dans la file worker pleine (8 demandes techniques en attente). Ce fait est distinct du refus de télémétrie resource_saturated. L’état actuel de la file et la cause amont du remplissage ne sont pas observés ; aucune demande métier rejouée pour le diagnostic.
Sources complémentaires550b096f9b3242c00ba602d37ac6749b8da99ca6 : projection commune pour refus de sonde initial et relecture, code et motifs conservés, contexte historique de file pleine et conditions explicites d’un rejeu strict. 14 contrôles ciblés PASS ; guides vérifiés. La première revue a trouvé le chemin initial manquant et un test worker non valide ; les deux points ont été corrigés et couverts. Contre-revue finale dd494bef favorable, résultat lu/ACK et garde fermée. Le candidat849 rejeté n’a jamais été installé.

## Candidat courant du complément
Build189-queue-550b096f9b32, source550b096f9b3242c00ba602d37ac6749b8da99ca6.
Chemin : /home/moi/.local/lib/bridget-communication/189-queue-550b096f9b32/bin/bridget
SHA256 : 2c12fbdc830f97c25da00c5ebb59607a46fd7a4e4687bd38bffc2deeb7afc438.
Build release PASS (nice10, deux jobs,1min48s), copie hors staging et identité vérifiées, commande installed passée. Manifeste non activable ; maître183-admission-7478e9e7 confirmé en fin de qualification. Aucun chargement du client MCP Mac. T004 reste ouverte.
L’ancien candidat001a97aa a été retiré après vérification SHA, exclusion d’usage par lsof et vérification du lien actif distinct. Son manifeste de preuve reste archivé : /home/moi/.cache/bridget189/manifest-candidat-remplace-001a97aa.json. Binaire actif et retour conservés ; aucun service touché.
