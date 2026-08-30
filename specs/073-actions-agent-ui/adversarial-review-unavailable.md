# Contre-revue du plan - indisponibilite du fournisseur distinct

Date: 2026-08-30
Etape: apres `plan.md`
Statut: non executee par un fournisseur distinct

## Preuve de disponibilite

La commande `/home/moi/.local/bin/bridget who` a ete executee immediatement avant la generation des taches.

- Tous les agents Claude en mode FLUX annoncent `7d epuisee rst 06:00`.
- Aucun agent Gemini ou Cursor n'est connecte.
- Les sessions Claude en TMUX sont des sessions synthetiques ou deja engagees et ne constituent pas un fournisseur distinct disponible pour une nouvelle revue fiable.
- Les agents Codex disponibles utilisent le meme fournisseur que l'agent qui produit la spec.

Aucune mission n'a donc ete envoyee et aucun agent en cours de travail n'a ete detourne.

## Revue adversariale interne provisoire

1. L'exigence FR-7313 promet une idempotence daemon que le plan ne prouve pas encore. L'analyse devra soit prouver la deduplication par `command_id`, soit limiter le contrat au blocage navigateur d'une requete en vol.
2. La confirmation doit rester correcte si l'agent change d'etat entre l'ouverture et l'envoi. La relecture de l'annuaire par le relais est donc obligatoire.
3. Le rerendu periodique de la liste peut supprimer le bouton declencheur. La restitution du focus doit verifier que le declencheur est encore connecte au DOM.
4. La fiche informative ne doit pas adopter le role `menu`, car elle contient des faits et une action. Le role `dialog` non modal reste le contrat le plus exact.
5. La confirmation destructrice doit placer le focus initial sur `Annuler`, borner le focus et interdire le clic exterieur comme validation implicite.
6. L'interface ne doit pas deduire la gestion depuis le nom, le type ou le transport. Seul `persistent` projete depuis `AgentInfo` autorise l'action.

Ces points devront etre resolus par `/speckit-analyze` avant implementation. Une seconde tentative de contre-revue par fournisseur distinct sera faite apres implementation si une capacite devient disponible.
