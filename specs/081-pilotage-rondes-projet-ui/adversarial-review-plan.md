# Revue adverse du plan - SPEC-081

Date: 2026-08-31

## Contre-revue inter-fournisseur

Aucun agent d'un fournisseur différent n'était connecté au moment de la revue. `bridget who` ne présentait que l'UI humaine et un agent tmux de type Codex connectés; les agents Claude étaient arrêtés. Aucune session n'a été démarrée uniquement pour satisfaire ce gate.

## Attaque du plan

| # | Risque recherché | Sévérité | Décision |
|---:|---|---|---|
| 1 | Créer un second état de ronde dans le navigateur | Haute | Rejeté : `GET /v1/projects` porte la projection autoritaire et le navigateur relit après mutation. |
| 2 | Coupler l'UI au timer systemd | Haute | Rejeté : seul `interval_secs` est projeté; aucune heure exacte ni état systemd. |
| 3 | Réutiliser une politique après rebind | Critique | Génération obligatoire à la lecture et à la mutation; absence de ligne sur la nouvelle génération. |
| 4 | Analyser le texte des messages pour déduire le dernier passage | Haute | Rejeté : fait fermé persisté au moment du dispatch. |
| 5 | Ajouter un journal complet pour un seul dernier état | Moyenne | Rejeté : trois colonnes optionnelles sur la politique existante. |
| 6 | Rendre la liste projets N+1 ou O(p²) | Haute | Une lecture groupée et une jointure HashMap O(p). |
| 7 | Afficher un succès optimiste | Haute | Aucun changement de `projects` avant reçu et relecture. |
| 8 | Casser les anciens enregistrements | Haute | Champs optionnels, migration additive et tests de base historique. |
| 9 | Fuite de chemin ou fournisseur | Haute | Le contrat ajoute uniquement génération, booléens, révision, états fermés et instants. |
| 10 | Étendre le scope à un panneau global | Moyenne | Hors périmètre explicite; menu contextuel existant seulement. |

## Minimalisme et frugalité

- Nouveau service: 0.
- Nouvelle table: 0.
- Nouvelle dépendance: 0.
- Nouvelle vue ou overlay: 0.
- Potentiel minimalisme du plan: environ 0 ligne de conception suppressible à comportement constant après retrait de l'heure exacte et du journal complet.

## Vertus LLM et responsabilité future

Le changement augmente légèrement le contrat et le schéma mais réduit l'ambiguïté opérateur. Les abstractions servent des invariants réels : capacité locale, génération, migration et résultat fermé. La solution reste explicable sans contexte de conversation et chaque ajout possède une preuve ciblée.

## Conclusion

Plan accepté sans finding bloquant. Les deux contrôles à surveiller pendant l'implémentation sont l'atomicité de l'enregistrement du dernier passage et l'absence totale de mutation optimiste dans le navigateur.
