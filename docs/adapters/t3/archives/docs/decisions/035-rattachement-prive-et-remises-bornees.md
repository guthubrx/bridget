# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/035-rattachement-prive-et-remises-bornees.md](../../../../../decisions/035-rattachement-prive-et-remises-bornees.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

L'accès au socket privé donnait accès à trois chemins d'attribution insuffisante :
inscription MCP par identifiants déclarés, compatibilité CLI et SendIdempotent
dont issuer_scope est une clé de rejeu. Les preuves reproduites montrent aussi
qu'une sortie bloquée immobilise l'état global et qu'une réponse t3code peut être
oubliée sans confirmation.

<a id="extrait-2"></a>

## Extrait 2

Le pont t3code conserve les réponses prêtes dans son état de fil existant jusqu'à
confirmation. Il ne crée ni second broker, ni table, ni service. Sa boucle traite
les contrôles durant l'attente, sans nouvelle exécution d'une demande annulée.

<a id="extrait-3"></a>

## Extrait 3

Limites : aucune séparation garantie entre processus hostiles du même compte
système ; aucune interruption atomique d'un tour déjà accepté par t3code ; une
écriture partielle reste indéterminée. Aucun déploiement implicite de cette décision.
