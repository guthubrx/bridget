# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/research.md](../../../../../../specs/138-priorite-projet/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Décision: porter un fait vivant sur la connexion. Utiliser la racine commune Git
et l'hôte attesté, ou la racine T3 attestée hors Git. Hériter du fait de l'instance
pour une connexion auxiliaire déjà prouvée.

<a id="extrait-2"></a>

## Extrait 2

Pourquoi: live_connection_identity et register_auxiliary existent. Les données
T3 contiennent workspace_root. derive_domain_at peut changer l'étiquette et ne
prouve pas une appartenance. ProjectReference/runtime est retiré.

<a id="extrait-3"></a>

## Extrait 3

Le choix du projet n'utilise aucun registre retiré. Le rattachement T3 et la
racine Git sont les seules preuves de cette session. Un chemin distant sans
preuve exploitable reste inconnu. Une vue globale n'autorise rien. Un ancien
client inconnu reçoit un warning ; un client dont le projet est prouvé applique
la même garde que les clients nouveaux.
