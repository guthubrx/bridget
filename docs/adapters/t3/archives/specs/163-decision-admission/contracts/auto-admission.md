# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/contracts/auto-admission.md](../../../../../../../specs/163-decision-admission/contracts/auto-admission.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Au point commun finalise_auto_choice, une nouvelle admission AutoDelegate avec auto_engine=false et auto_placement=false, dont le tuple de profil fixe réel est connu et validé, conserve le mode Explicit. Un choix de repli reste Fallback. Hors repli, si au moins un axe est automatique, le mode reste Automatic, même avec un seul candidat. Le parcours Delegate162 historique conserve decision:null lorsque son profil n’est pas connu. Le parcours configuré R3 met en œuvre cette règle selon le journal et les contrôles ciblés indépendamment attestés. La lecture du consommateur T3 confirme les modes ; aucune recette du runtime T3 ni validation de livraison déduite.

<a id="extrait-2"></a>

## Extrait 2

Afficher uniquement mode, choix et raison courte dans les surfaces existantes. Aucune raison interprétée comme HTML ou consigne. Ancien reader sans résumé : information de décision indisponible, pas mode explicite inventé. Pas de nouvelle conversation ni vue globale T3. Les surfaces de compatibilité v3 sont identifiées dans l’audit ; leur plan est validé par ROOT après corrections R1 ; leur implémentation reste à vérifier.
