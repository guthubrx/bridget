# Plan 053 — Méta-gate des témoins de présence

## Point d'autorité

Le binaire de tests compilé est l'inventaire autoritaire : une liste statique
de noms divergerait dès qu'un témoin serait ajouté ou renommé. Le gate lance
donc deux listings du même binaire : univers complet, puis sélection munie des
arguments libtest proposés.

## Comparaison

Les lignes terminant par `: test` sont normalisées en noms exacts. La famille
attendue est le sous-ensemble dont le nom commence par
`daemon::presence_tests::`. Une table associative `awk` compare la sélection à
l'attendu en O(N), où N est le nombre de tests listés.

Trois états sont fermés : inventaire attendu vide, sélection vide et famille
incomplète. Seule une sélection non vide contenant chaque nom attendu est
acceptée.

## Cycle rouge puis vert

Le banc est écrit contre une première version volontairement permissive du
gate. Il doit d'abord échouer parce que la sélection amputée est acceptée.
Après implémentation, le même banc exige :

1. refus d'une sélection à laquelle un seul témoin réel est retiré ;
2. acceptation de la sélection complète ;
3. refus d'une sélection vide comme inobservable.

Le témoin retiré est découvert depuis le listing réel, pas recopié dans le
banc. Le diagnostic doit néanmoins le nommer exactement.

## Portée de validation

Le delta fonctionnel est Bash et Make. La bibliothèque daemon est compilée
avant tout listing. Les gates sont le banc 053, `bash -n`, le listing réel,
`git diff --check` et ShellCheck s'il est disponible. Aucun test de présence
n'est exécuté par le méta-gate : son objet est la complétude de la sélection,
pas le verdict de la famille.
