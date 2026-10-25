# Plan140 — Bulles compactes et titre compatible

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/plan.md#extrait-1).

## Réutilisation et conception

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/plan.md#extrait-2).

## Sources concernées

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/plan.md#extrait-3).

## Tests et ordre

Tests RED de projection avant son implémentation. Tests des cinq familles, faux positifs, borne d'en-tête, nom absent, reply, lots et métadonnée JSON. Tests Rust de compatibilité du champ absent, du destinataire non membre, des titres hostiles et du canon/journal inchangés après remise/rejeu. Tests ou recette DOM du vrai composant : clic, Entrée, Espace, copie originale, détails intégraux, messages longs, thèmes, largeur320 et état entre fils/messages. Format, lint, TypeScript et build ciblés sur le frontend ; tests Rust ciblés, jamais un succès global présumé.

## Gates

Audit de réutilisation PASS avant tasks. Analyse puis mise en œuvre. Convergence par exigences et preuves code/test ; aucune case cochée sans preuve. Contre-revue autre fournisseur demandée si joignable, borne courte. Essais isolés, pas de base active, service fournisseur payant ou redémarrage. Les scripts/templates projet absents justifient le protocole manuel documenté, pas la suppression d'une phase.

## Charge future et complexité

Article XIX/XX : projection pure dans la logique existante ; pas de framework de cartes, nouvelle API ou store global. Lecture du titre réutilise la requête et l'autorité existantes à la remise. Reconnaissance de l'en-tête O(1) grâce à la borne1024 ; rendu du brut O(longueur du texte) quand demandé, comme aujourd'hui. Métadonnée titre bornée, aucun lookup navigateur. La reconnaissance visuelle ne devient jamais un contrôle de sécurité.
