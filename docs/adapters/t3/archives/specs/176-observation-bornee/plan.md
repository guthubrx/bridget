# Archives techniques de l’adaptateur T3

Document d’origine : [specs/176-observation-bornee/plan.md](../../../../../../specs/176-observation-bornee/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Relever les données réelles lsof/macOS/Linux avant de choisir le traitement.
2. Réutiliser busy_paths/opened_paths : observation partagée, captures bornées, erreurs de stdout et stderr contrôlées. Sur Linux, examiner une lecture non interactive via sudo -n de /usr/bin/lsof seulement quand l'accès existant le permet. Aucun changement des droits.
3. Objection de revue traitée : aucun périphérique étranger ne suffit à ignorer une erreur. Une seconde lecture lsof -e doit encore énumérer les fichiers concernés : tout UNKN, annotation (-e), format non reconnu ou chemin sous montage exempté bloque tout retrait. Lire mountinfo avant/après identique ; refuser les alias bind chevauchant la portée et les montages internes. Des couches superposées hors portée restent décrites ; toute ambiguïté sur la portée ou le montage exempté refuse. Le type REG avec cible absolue et stat ENOENT conserve cette cible occupée ; les objets non fichiers ont une liste fermée. Aucun -Ki : les threads peuvent avoir leurs propres fichiers.
4. Reprendre le principe T3 executeRaw : délai et maxOutputBytes explicites, sortie tronquée distincte. Bibliothèque standard Python, capture incrémentale, plafond stdout/stderr et budget monotone partagé. Git lecture seule, aucun shell ; arrêt TERM des seuls enfants possédés.
5. Résoudre la base une fois ; mémoriser les observations immuables par HEAD dans un passage ; statut Git compact conservant tous fichiers ignorés/non suivis ; index des ancêtres pour imbrications, pas de croisement W×W.
6. Tests discriminants, mesure avant/après32/128 et dépôt réel. Guides, revue adverse, Converge et audit ciblé ; Git et scripts livrés sans reconstruire Rust inchangé.
