# 221 — Choisir le prochain placement sans modifier les tâches vivantes

Date : 30 octobre 2026. Statut : accepté pour le lot C16, sous réserve de qualification et de chargement.

## Contexte
Une Task conserve project_id et mapping_revision. Remplacer le mapping A par B retire une autorité encore nécessaire à ses rechecks et slots. Le reçu figé seul ne peut pas remplacer une autorité courante : cela masquerait les révocations. Deux analyses indépendantes et la fixture privée ont vérifié ces points.

## Décision
Conserver A exactement. Ajouter B sous une nouvelle clé dans les registres existants. Une section racine additive project_preferences choisit B seulement pour une admission implicite future, entre mappings de même source et même autorité additionnelle. Le registre est déjà tolérant à sa racine ; pas de sidecar ni de magasin d'historique. Valider la section seulement au choix futur, pour ne pas casser les lectures exactes des tâches A. Conserver le canon de la demande ; transmettre l'identifiant résolu à la préparation via son champ existant.

## Conséquences
Les tâches A et les révocations restent régies par le mapping A exact. Les anciens lecteurs ignorent la préférence et refusent l'ambiguïté implicite ; ils comprennent A/B explicites. Une préférence invalide est un refus guidé de cette sélection, pas un arrêt global. Configuration source, maître et worker à publier de façon contrôlée après qualification de B ; aucune migration de données ni nouvelle autorité opérateur n'est créée. Une publication partielle refuse jusqu'à cohérence, sans changer une Task. Le retour retire la préférence, pas les mappings encore utilisés.

Charge future limitée : réemploi de deux sélecteurs existants Rust/Python, sans dépendance, opération publique ou schéma de reçu neuf. La sélection Rust est O(M log P), avec M et P bornés à64 ; les tests rapprochent les deux implémentations. Les garanties de tous les projets, origines et plateformes ne sont pas déduites de cette seule fixture.
