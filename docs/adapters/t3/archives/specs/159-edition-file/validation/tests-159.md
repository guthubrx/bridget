# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/validation/tests-159.md](../../../../../../../specs/159-edition-file/validation/tests-159.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T023 et sa recette privée SC010 restent à prouver. Cette recette ne sera pas
une preuve T3 réelle. T024/T025 attendent le checkpoint final, FULL et scoring.
Aucun cochage, livraison B, Converge ou succès global n’est annoncé ici.

<a id="extrait-2"></a>

## Extrait 2

Le FULL courant réussit les cinq E2E B : trois cancel et deux recettes,
plus un worker ignoré. SC010 garde ses 24 assertions privées ; aucune interop
T3 réelle ni consommation par un modèle réel. La pause explicite des E2E
cancel et la portée SC007 après Applied restent inchangées.
Proxy SC004 B : p95 fermé 41,385 ms, ouvert 1026,412 ms, delta 985,027 ms ;
daemon_idle_measured=false. Ce proxy ne prouve pas une latence de repos attesté.
held_queue reste fermé, aucune activation du réglage déduite.
