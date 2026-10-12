# Verdict157 — T007 à T011
Deux causes métier RED réelles : permission_source_unavailable de l'admission, discoverytrue du catalogue.
Huit scénarios RED exécutés, huit échecs attendus. Les deux causes échouent sur l'oracle de cohérence, jamais sur compilation ni sur l'assertion du refus.
Première passe GREEN :167PASS/2FAIL. Les huit scénarios157 et la matrice60 cellules passent.
Deux anciens oracles148 :
- GLM : args de registre glm.old contre modèle demandé glm/custom-exact. Préflight vérifiait l'ancien argument avant pin. Réemploi du pin_delegation_model existant avant préflight, visibilitépub(crate), sans droits avant préflight.
- SameProject : contrat157 exige cwdnull sans racine attestée. Ancien oracle cataloguetrue actualiséfalse ; les admissions du dépôt, du worktreeexterne autorisé et le refus du dépôtétranger sont conservés.
Reprise ciblée aprèscorrectif :15PASS/0FAIL, tous scénarios cohérence/préflight157 et les deux148 concernés.
Pas de répétition des filtres longs. T022 et suite complète finale restent prévus.
T012 reste ouverte : mesure183.236ms/p95206.447ms contre40.201/42.698ms ; seuilx2 dépassé, optimisationR8 requise.
