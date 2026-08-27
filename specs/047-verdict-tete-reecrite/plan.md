# Plan 047 — Signaler un verdict attaché à une tête réécrite

1. Brancher `ReviewTarget` sur `maicie_delegate`, le payload de service et
   `greffe_service`.
2. Relire les verdicts typés depuis les réponses guichet persistées, sans
   migration SQLite.
3. Construire un observateur Git pur quant au dépôt : tête distante, objets
   temporaires isolés, ancêtralité.
4. Projeter les états fermés dans `maicie status` et la carte de reprise.
5. Éprouver les cas empilé, réécrit, cible absente, verdict absent et Git
   indisponible.
6. Rejouer le mutant égalité à la place de l'ancêtralité, puis mesurer la
   compatibilité avec un daemon antérieur.

## Risques bornés

- Le réseau Git peut être indisponible : résultat `unobservable`, jamais vert.
- Les constructeurs Rust exhaustifs du payload seront suivis jusqu'au dernier
  E0063/E0004 ; les listes littérales seront cherchées séparément.
- La carte de reprise ne doit pas transformer une absence Maicie ou Git en
  permission de travailler.
