# Contre-revue finale locale — SPEC-133

Date : 2026-10-04.
Portée : implémentation, contrats, tests et incident de production T3.
Verdict : **APPROVE** après correction d'un défaut de compatibilité détecté.

## Vérifications hostiles

- Autorité : une preuve enfant donne seulement `bridget_who` et `bridget_send`.
  Les autres outils sont refusés avant handler. La CLI sensible refuse aussi la
  preuve enfant.
- Identité : le parent reste l'expéditeur et la portée d'idempotence. Aucun
  agent durable n'est créé pour l'enfant.
- Preuve : PID, naissance, propriétaire, permissions, taille et absence de lien
  symbolique sont contrôlés. Une preuve enfant invalide interdit tout repli vers
  l'identité principale.
- Confidentialité : la provenance contient seulement un fournisseur fermé et
  une empreinte opaque de 16 caractères. Aucun corps, secret ou identifiant de
  session natif n'est journalisé.
- Disponibilité : la sortie `lsof` est vidée pendant l'exécution et reste bornée.
  Le test de 192 fichiers reproduit le volume qui bloquait la production.
- Compatibilité : la première passe complète a détecté une modification de
  l'en-tête Codex historique. Le rendu a été corrigé. Un message principal garde
  exactement l'ancien en-tête et n'ajoute aucun champ nul. La suite complète du
  transport passe après correction.

## Limite de la revue

Cette revue finale est locale. Bridget ne permettait pas de joindre un relecteur
externe avant son propre redémarrage. La contre-revue distincte du plan reste
consignée dans `adversarial-review-claude.md`. La livraison exige donc aussi une
vérification post-déploiement des marqueurs T3 réels.
