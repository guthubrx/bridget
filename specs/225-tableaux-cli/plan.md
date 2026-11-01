# Plan225 — rendu CLI commun
Base08e51c8d. Contrat validé par l'utilisateur : --table global avant/après commande, toutes les sorties éligibles, colonnes décidées par le responsable.
1. Grammaire globale avant ouverture du namespace ; protéger wrappers, --, arguments libres et valeurs d'options. Incompatibilité explicite --json/--table, sans effet.
2. Module de présentation commun, sans subprocess de CLI : recevoir une sortie native structurée déjà obtenue, renderer à colonnes métier, Unicode/largeur/couleurs/NO_COLOR. Aucun stockage ni appel fournisseur ajouté.
3. Raccords dans les façades existantes. JSON actuel inchangé hors --table. Raccord spécifique au helper de fédération existant ; pas de double opération. Flux/protocoles binaires ou interactifs exclus avec diagnostic clair.
4. Couverture par inventaire des commandes et fixtures de formes natives, test du renderer et des arguments regroupé, puis lectures CLI réelles représentatives. Pas de campagne métier. Revue de conception bornée déjà admise.
5. Intégration sur main0.7.3 ; version suivante cohérente si nécessaire, build.py pour Cargo, publication APK via garde223, maintenance native/rollback et ressources conservées si observation incomplète. Pas d'activation concurrente avec224.
