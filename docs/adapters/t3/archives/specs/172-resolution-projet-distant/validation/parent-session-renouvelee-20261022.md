# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/parent-session-renouvelee-20261022.md](../../../../../../../specs/172-resolution-projet-distant/validation/parent-session-renouvelee-20261022.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le worker Cartae PID 2219043 reste sur 172-76b682d6. Aucun redémarrage du worker
ni de T3. Pont 898, application 50681 et serveur 50799 préservés.
La mission becec88c-197d-4653-8452-0b936d04945e a gardé son wrapper
2479540, sa naissance 219447002 et les processus fournisseur mesurés pendant
l'activation. Son état après activation est working, sans erreur.

<a id="extrait-2"></a>

## Extrait 2

La continuation d'un même propriétaire T3 était refusée quand Codex renouvelait
son identifiant de session. Le fil, l'instance fournisseur et la politique
restaient identiques. Le comparateur traitait pourtant cet identifiant technique
comme un changement de droits.

<a id="extrait-3"></a>

## Extrait 3

Neuf tests ciblés PASS, sans suite complète. RED puis GREEN sur la comparaison.
Les tests couvrent le renouvellement, les changements de politique refusés, les
deux gardes, la même ronde rejouée et la liaison T3 opaque.
La revue 9e5b4b1c-cfbb-449a-9f8b-5809f2b69f7f rend APPROVE. Résultat lu,
acquitté et garde clôturée.

<a id="extrait-4"></a>

## Extrait 4

L'admission monuments-notre-dame-qualification-intrants-reels-20261022-01
était restée chosen, sans Task, depuis 13:52 UTC. Le propriétaire a confirmé son
abandon dans t3-8f8ffab4-341b-4441-9851-571267a62568. Sa récupération native au
redémarrage l'a refusée avec permission_not_inherited ; TaskID toujours nul.
Aucun rejeu manuel, changement de clé ni mutation SQL opérateur.
La qualification sur une nouvelle base reste au propriétaire.

<a id="extrait-5"></a>

## Extrait 5

identity_not_found signifie qu'aucune identité Bridget vérifiable n'est liée
à cet appel. La seule présence du connecteur MCP ne fait pas cette inscription.
Le parcours interactif actuel est bridget codex, ou bridget codex resume pour
reprendre une conversation existante. Il ne dépend pas de T3.
