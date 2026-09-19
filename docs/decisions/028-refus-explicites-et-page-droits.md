# ADR-028 : Refus explicites et page Droits

Date : 2026-09-03. Statut : Accepté (mis en service le 2026-09-04).

## Contexte

Six couches d'autorisation coexistent dans Bridget : sécurité du contenu locale (SPEC-083), sandbox d'artefacts (SPEC-083), posture d'agent du registre (sandbox et approbations du fournisseur), runtime des agents (SPEC-085), projet système (SPEC-086), contrôle référent (SPEC-087). Le 2026-09-03, un refus d'interface a été lu comme un blocage de l'agent, et un blocage réel de sandbox n'a été visible que dans un journal brut. Chaque couche parle sa langue ; le référent ne sait pas laquelle a refusé.

## Décision

1. **Aucune couche nouvelle.** Les autorisations restent où elles sont ; SPEC-088 les rend lisibles.
2. **Forme unique du refus** : couche (ensemble fermé), chose empêchée, geste, ligne brute, attribution. Une seule fonction de rendu dans l'interface. Un refus dont la couche est inconnue le dit et cite la ligne brute ; Bridget n'attribue jamais une cause par déduction.
3. **Signalement de sandbox, jamais attestation** : une ligne de la liste fermée dans la sortie brute n'est un signalement que si la même commande s'est terminée en échec ; il est affiché comme non attesté, avec le geste « Tester ». Une ligne imprimée par un agent n'établit rien (contre-revue Jim, 2026-09-03).
4. **Une page « Droits » par question** (« Ce que je vois », « Ce que les agents peuvent faire », « Combien ils décident seuls ») avec trois profils fermés et un mode expert qui montre le mécanisme et le lieu. Les lignes qui vivent dans le navigateur ne sont modifiables que depuis ce navigateur : un serveur relié ne peut pas desserrer la sécurité du référent, et la page le dit.
5. **Un droit se vérifie** : « Tester » est une tentative corrélée (identifiant du message envoyé, commande canonique exacte, fin de commande) ; sans terminaison, l'issue est inconnue ou expirée, jamais un refus ni une réussite par défaut.
6. **Propriété** : posture d'agent et réassignation automatique sont des colonnes de `control_state`, écrites par `ControlStateSet` sous la génération de l'ADR-027 ; le profil est dérivé ; l'absence sur le fil vaut inconnu et diffère ; le service compagnon lit, n'écrit pas.

## Conséquences

Positives : un refus est actionnable en un clic ; les six couches sont visibles en un lieu ; un droit affiché peut être contredit par une mesure. Négatives : les trois lignes d'agent sont liées à une seule posture tant que Bridget ne sait lancer que deux postures ; la reconnaissance de sandbox ne couvre que Codex dans ce lot ; un champ additif sur `ControlStateFrame` touche tous ses initialiseurs.
