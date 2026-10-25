# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/research.md](../../../../../../specs/148-identite-delegation/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Sources primaires : code local T3 commit1535fc0844, Bridget6e167cb0 et données
runtime relues le10octobre. Baseline utilisateur agentic AI consultée : architecture
orientée responsabilité, coût de contexte et réutilisation avant framework.

<a id="extrait-2"></a>

## Extrait 2

Décision : utiliser le credential MCP T3 par session. Justification : sa portée
contient environnement/fil/fournisseur et sa révocation est déjà centralisée.
Alternative rejetée : choisir le premier rollout d'un PID partagé, qui usurperait
une identité. Alternative rejetée : nouveau serveur de jetons Bridget concurrent.

<a id="extrait-3"></a>

## Extrait 3

Décision corrigée par instruction utilisateur : moteur Bridget entièrement
autonome. Réutiliser sa flotte gérée, son registre, ses remises et exécutions.
Adapter les principes T3 (appel unique, tâche parente, résultat, annulation),
jamais son moteur. Le connecteur T3 peut évoluer séparément pour l'identité.
Un simple alias de delegate_task est explicitement exclu.
