# Convergence - SPEC-081

## Résultat

Convergence atteinte en deux boucles. Les 17 exigences fonctionnelles et les 6 critères de succès ont une réalisation et une preuve. Aucun écart critique, élevé ou moyen ne reste ouvert.

## Boucle 1 - Couverture du diff

| Exigences | Réalisation | Preuve |
|---|---|---|
| FR-08101 à FR-08103 | Action de ronde dans le menu existant, état occupé, rafraîchissement après reçu seulement | test Node `spec_081_ronde_projet_reste_confirmee_accessible_et_generique` et code du menu |
| FR-08104 à FR-08107 | Projection absente sûre, génération obligatoire, refus fermé et rebind isolé | tests store SPEC-079/081 et test de jointure du relais |
| FR-08108 | Aucun appel aux contrôles d'agent depuis la mutation de politique | revue du diff et scénario Gherkin |
| FR-08109 à FR-08112 | Ligne discrète, dernier résultat fermé et délai maximal fondé sur la constante 420 | tests Node, projection UI et styles |
| FR-08113 | Erreurs fermées et aucune mutation locale avant confirmation | test de perte de réponse et chemin `catch` du navigateur |
| FR-08114 | Store, contrat, scheduler et menu existants réutilisés | audit de réutilisation et diff |
| FR-08115 à FR-08117 | Aucun fournisseur, secret, cadence libre, déclenchement manuel ou commande globale | recherche sur les lignes ajoutées et absence de dépendance |

## Boucle 2 - Écarts corrigés

| Écart | Correction | Verdict |
|---|---|---|
| La perte de réponse utilisait le code générique `daemon_unavailable` | toutes les erreurs de connexion, lecture et écriture de la frontière ronde utilisent `round_service_unavailable` | corrigé et testé |
| Deux commandes du quickstart exécutaient zéro test à cause d'un filtre inexact | filtres remplacés par les noms de preuves exacts | corrigé et rejoué |
| La jointure refusait les incohérences mais n'avait pas de preuve dédiée | ajout d'un test sur politique absente, génération divergente et triplet de dernier passage incomplet | corrigé et testé |

## Critères de succès

| Critère | Verdict | Justification |
|---|---|---|
| SC-08101 | PASS | l'état et l'action sont dans le menu directement accessible depuis la ligne projet |
| SC-08102 | PASS | génération obsolète, projet inactif et perte de réponse sont couverts par les preuves Rust; le navigateur ne remplace `projects` qu'après un rafraîchissement réussi |
| SC-08103 | PASS | le suffixe et le délai maximal sont dérivés de la projection confirmée |
| SC-08104 | PASS | activation, désactivation, rejeu, enveloppe divergente, rebind, inactivité, capacité absente et perte de réponse sont couverts par les suites ciblées ou héritées de SPEC-079 |
| SC-08105 | PASS | aucune fonction de cycle de vie agent ou d'exécution n'est appelée par la route de ronde |
| SC-08106 | PASS | les trois états fermés ou l'absence explicite sont validés avant projection UI |

## Conclusion

Le diff converge avec la spécification sans extension de périmètre. La validation visuelle opérateur reste une étape de livraison future, après compilation et déploiement du même binaire.
