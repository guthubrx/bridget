# Contre-revue adverse du plan 133

Date : 2026-10-04.
Relecteur : `29aaed9b-9f6f-4849-87a5-1a23bbe01948`, fournisseur distinct.
Message : `mcp-21744-6ac249cc-d`.
Verdict : **APPROVE_WITH_CHANGES**.

La revue a été faite en lecture seule. Aucun fichier, test, fournisseur payant
ou service n'a été modifié ou lancé par le relecteur.

## Défauts retenus

1. **Escalade sur preuve enfant périmée.** La boucle existante continue après un
   marqueur non correspondant. Sans règle nouvelle, elle pourrait trouver le
   marqueur principal du parent. Action : refus immédiat et test causal.
2. **Couverture CLI implicite.** Le plan supposait que toutes les commandes
   sensibles utilisent la résolution centrale. Action : ajouter `cli.rs` au
   périmètre de vérification et un garde-fou d'inventaire.
3. **Rejeu après fin de l'enfant.** Une provenance incluse dans l'empreinte de
   rejeu empêcherait le parent de relire le sort. Action : exclure la provenance
   de cette empreinte. Le message déjà stocké reste inchangé.
4. **Provenance forgeable par un client direct modifié.** Ce cas ne donne aucun
   droit supplémentaire et exige déjà le contrôle du compte système. Action :
   documenter cette limite. Une nouvelle autorité de protocole n'est pas ajoutée.

## Convergence du plan

- `plan.md` fixe le refus immédiat et ajoute le garde-fou CLI.
- `data-model.md` fixe la sémantique de rejeu.
- `contracts/delegated-mcp.md` fixe les refus, le rejeu et la limite de menace.
- Aucun changement d'architecture, dépendance ou service n'est requis.
