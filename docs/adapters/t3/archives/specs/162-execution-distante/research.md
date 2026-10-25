# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/research.md](../../../../../../specs/162-execution-distante/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

lineage_protocol.rs contient déjà la requête et sa version. Le nouveau CLI accepte succès version=2 et erreurs historiques version=1, avec authentification humaine v1 distincte. Le reader T3 utilise un schéma fermé et une pagination de 4 096 tâches. Une version enrichie explicite évite de casser l’ancien runtime. ROOT a vérifié que le CLI 160 refuse un flag inconnu avec invalid_request JSON RC2. Repli unique de lecture sur ce refus ou unsupported_version seulement. La projection thread:bridget-task existante n’est pas un nouveau fil fournisseur.

<a id="extrait-2"></a>

## Extrait 2

Réemployer les tests T3 149 des contrats, reader, orchestrateur et composants. Pas de suite globale ni de relance T3. Pour les tâches de code, réemployer l’isolation 154 sur l’hôte choisi, avec commit de base puis travail local autorisé. Aucun moteur Git réseau ni push concurrent vers main.

<a id="extrait-3"></a>

## Extrait 3

Mappings de production, authentification fournisseur et route réelle restent à attester par ROOT après choix utilisateur. La présence d’un binaire ne vaut pas login. L’activation Bridget 162 est autorisée par le pipeline, avec sauvegardes et contrôle des missions vivantes ; seule la relance T3 reste interdite.
