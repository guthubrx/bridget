# Contre-revue du plan222 — ronde1
Date : 2026-11-01. Agent ff23c251-ed60-4d69-814b-67420a2c93a6,
fournisseur Claude, tâche21b30002-6661-494c-bd15-f1df70ef3acf.
Verdict reçu : BLOCKED pour tasks. Lecture seule, pas de test/build.
Demande : réveil durable, attribution, identité, mémoire et réutilisation ;
borne15min, aucune dépense externe ni modification.
Résultat lu et ACK, clôture demandée une fois ; dossier conservé sous garde.

| Objection | Vérifiée comment | Retenue | Raison / traitement |
|---|---|---|---|
| Grant non relié au lancement | fleet.rs:891/901 et lifecycle.rs:175 | oui | MandateGrant explicite ; même mutex que réservation et recontrôle avant remise ; aucun request_recovery |
| Réservation à plusieurs endroits | daemon.rs:4958-4959, execution_store.rs:1768 | oui | Même fichier SQLite ; attribution+prepared atomiques sur connexion soumission, état natif seul |
| Identité logique indéfinie | agent_profile.rs:185-196, roster | oui | Specialist.id = agent_id existant ; profile_id distinct |
| Oubli / capture / classement vagues | données et points d'entrée réels | oui | Tombstone par source, hooks Task/ObservedActivity, rappel lexical explicite sans FTS |
| RuntimeBinding redondant | provider_bindings/roster déjà présents | oui | Retiré des données persistées ; dérivé à la lecture |
| ControlStateV1/MobileAdministrationV1 inexistants | protocol.rs:200 et224 | non | Variantes réelles de ClientCapability ; vocabulaire clarifié |
| Réutiliser applications comme historique | schéma agent_profile.rs:204 | non | Révision appliquée sans ancien texte ; conserver états d'application et historique de définitions distincts |
| Réduire à MVP et reporter import/conversion/équipes | cadrage utilisateur validé | non | Le pipeline demandé couvre tout le périmètre ; découpage technique sans réduction |

Correction ciblée : contracts/closure-after-review.md précise les invariants,
les formes et les contre-épreuves. Aucun code produit avant fermeture du gate.
