# Archives techniques de l’adaptateur T3

Document d’origine : [specs/153-rondes-contexte/spec.md](../../../../../../specs/153-rondes-contexte/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11. Accord utilisateur : feuille de route « Bridget cinq étoiles »
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`). Critère
visé : « Contexte et rondes », note 2 vers 5 (`specs/150-socle-delegation/research-t3-bridget.md`).

<a id="extrait-2"></a>

## Extrait 2

**Doctrine reprise de T3.** Une tâche par ronde, une clé distincte par ronde,
tout le contexte dans le brief. T3 relance un enfant sans mémoire à chaque ronde.

<a id="extrait-3"></a>

## Extrait 3

Un appel sans `keep_alive_secs` ni `brief` produit les mêmes octets canoniques
et le même comportement qu'en 150 : l'enfant est arrêté après son résultat. Les
tâches créées avant la mise à jour se relisent sans erreur. La vue Lineage T3
garde ses neuf états.

<a id="extrait-4"></a>

## Extrait 4

- SPEC-150 est livrée : `reply_timeout`, `delivery_acked`, purge par arbre,
  `AutonomousEffect::NativeDelegation`, comptages actifs indexés.
- SPEC-151 est livrée : résultat structuré, attente d'une tâche et boîte de
  fin. SPEC-153 s'y branche par la ronde courante. Les points de couplage sont
  vérifiés en T001.
- SPEC-152 est livrée : reprise d'une mission et chaîne d'exécutions lue par
  `latest_in_chain`. Le journal de pause et `pause_overlap` déterminent le gel
  des échéances métier sans modifier les dates brutes. Le statut `suspended`
  est dérivé ; aucun champ `suspended_at` n'existe. Une tâche inactive est
  terminale : son délai d'inactivité n'est pas gelé par une pause. Son résultat
  reste protégé quand l'enfant se termine pendant une pause.
- L'enfant garde sa mémoire parce que son processus fournisseur reste vivant.
  La taille de son contexte croît à chaque ronde. Le plafond de 8 rondes limite
  ce risque.
- Une revue indépendante (sans biais de la ronde précédente) reste un cas pour
  une nouvelle délégation sans garde.
- Les tests suivent la recette `umask 077` de la session 150.
- Le principal livre chaque session, puis nettoie son worktree. Il distingue
  les sources Git, les fichiers installés et le code actif. L'activation est
  coordonnée pour préserver les autres fils T3.

<a id="extrait-5"></a>

## Extrait 5

- Extrait de l'historique du parent (le fournisseur garde cet historique, pas
  Bridget). Le parent cite des tranches de journal ou de fil dans les
  références du brief.
- Délai de réponse propre à une ronde (le délai de la tâche s'applique).
- Fork de contexte entre fournisseurs et fusion de retour (`merge_back` de T3).
- Reprise d'un enfant inactif après redémarrage (SPEC-152).
- Nouvel essai automatique d'une ronde échouée (SPEC-152).
- Attente synchrone d'une ronde (SPEC-151 : la ronde réutilise son mécanisme).
- Isolation de l'espace de travail (SPEC-154).
- Questions de l'enfant au parent pendant une ronde (SPEC-155).
