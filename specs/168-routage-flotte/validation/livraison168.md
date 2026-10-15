# Livraison SPEC168 — reçu initial avant commit A

Date : 15 octobre 2026. **Statut : In Progress.** Ce reçu décrit un développement vérifié dont la livraison reste à effectuer. Il ne déclare pas la session Implemented ou Livré.

## État observé

| Étape | État de ce checkpoint |
|---|---|
| Code et contrat | Réalisés dans le worktree de session ; source initiale 1f69e5d8e651dd27907fd244aee6f90b6b734565. |
| Tâches | 45/56 cochées après T036 ; cette observation ne modifie pas Tasks. |
| Converge et contre-revue | P3 CONVERGED ; T037 close après corrections et contrôles affectés. Verdict Sonnet conservé : APPROVE_WITH_CHANGES, sans prétendre à une seconde revue intégrale. |
| Audit T038 | Décision pass, release_blocked=false, A et compteur de pénalités 98 ; 14 contrôles, 0 critique/élevé/moyen actif, 2 LOW et 1 INFO. Axes qualité non notés. |
| FULL T036 | Exécutée une fois ; résultat et limites détaillés ci-dessous. |
| Documentation T039 | En publication. Son cochage reste à ROOT. |
| Version visée | 0.4.0, évolution compatible après 0.3.0 ; métadonnées et commit exact encore à fixer par T040. |
| Git du lot 168 | Pas encore committé, intégré ou poussé à ce checkpoint ; aucun SHA source futur n’est inventé. |
| Paquets 0.4.0 | Non construits ; aucune signature ni empreinte de paquet n’est anticipée. |
| Production | 0.3.0 inchangée par ce lot ; aucun service ni configuration de production modifié par ce checkpoint documentaire. |
| Qualification, activation et recettes réelles | À effectuer après les prérequis T041–T044. Les brouillons privés ne valent pas exécution. |
| Tag, release et nettoyage | Futurs ; aucune suppression de branche, worktree, TMP ou sauvegarde déclarée faite. |

Source de travail actuelle : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte

## Tests et limites conservées

La FULL unique termine RC101 en 775,777964 s : **2 820 PASS / 32 FAIL / 71 ignorés**. Les huit sommaires de harnais enfants sont exclus du total ; aucun double comptage de leurs PASS. Les sorties sont complètes et les sources restent stables pendant cette exécution.

Chronologie des reprises : neuf migrations PASS au même binaire FULL original, avec l’entrée de sauvegarde réelle auparavant absente ; lot r1 à 12 PASS / 3 FAIL / 8 différés ; lot r2 à exactement 8 PASS, feature existante test-support et umask 0077, sources stables. **29 des 32 échecs initiaux passent après reprise**. Les trois annulations 159 restantes dépendent de watch_marker/kqueue Darwin/BSD et ne sont pas validables sur Cartae Linux. Aucun ignored n’est ajouté. Les échecs et logs intermédiaires restent conservés ; aucun GREEN global n’est déduit.

Le replay instrumenté d’identité T3 passe sans atteindre le diagnostic conditionnel. La cause initiale de process_arguments=None reste inconnue. Les neuf fichiers modifiés après FULL ne changent que les tests et leur diagnostic ; le code produit reste inchangé. Deux contre-lectures indépendantes sont reliées au reçu T036, avec leurs limites.

Preuve de tests, noms exacts, logs et SHA :

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md

Audit et archive privée canonique :

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/audit168.md

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/converge168.md

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/adversarial-review-code168.md

L’audit conserve deux risques LOW rustls-webpki (RUSTSEC-2026-0098 et RUSTSEC-2026-0099) et une INFO de maintenance unsafe-libyaml (RUSTSEC-2024-0449). Aucun succès global de production ne vient du grade A.

## Source binaire et documents : éviter une référence circulaire

Le **commit A** doit fixer le code et les métadonnées de version autorisées par T040. Son SHA ne peut pas être écrit dans son propre contenu avant création. Ce reçu conserve donc ce champ non établi à ce checkpoint.

Après création et contrôle du commit A, ROOT consignera son SHA exact comme source des binaires. Les builds et paquets devront provenir de cet arbre propre. Les commits documentaires ultérieurs pourront référencer A et les hashes des paquets, puis les faits d’activation, de recette, de release et de nettoyage. Ils ne changeront pas rétroactivement la provenance des binaires. Un dernier commit documentaire ne peut pas s’autociter : son SHA sera vérifié séparément dans le reçu Git privé ou un checkpoint ultérieur.

Aucun commit, push ou changement de version n’est exécuté par cette mise à jour documentaire.

## Étapes restant à prouver

T040 : version et commit source exact ; T041 : builds/paquets/signatures/sommes ; T042 : intégration, push et rapprochement des checkouts propres ; T043 : qualification du registre et sauvegardes cohérentes ; T044 : installation/activation contrôlée et vérification des services, DB et configuration ; T045 : recettes réelles avec résultats corrélés et rejeu ; T046 : tag et release v0.4.0 vérifiés ; T047–T049 : reçus durables, nettoyage justifié et contrôle final.

Les données metadata_version=2 exigent une procédure de rollback compatible ; un ancien binaire ne doit pas ouvrir ces lignes. Les sauvegardes et le contrôle des schémas appartiennent au préflight d’activation à venir ; aucune restauration automatique de DB n’est annoncée.

Le statut final ne passera à Implemented/Livré qu’après preuve des étapes applicables. Le développement, les tests et l’audit seuls ne clôturent pas la livraison demandée.
