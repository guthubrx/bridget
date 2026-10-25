# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/implementation-b.md](../../../../../../specs/159-edition-file/implementation-b.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les tests lus exercent les handlers, SQLite, sockets et transactions réels du périmètre : panne entre écritures, deux écrivains, conservation des octets, refus par identité et pause, réouverturev7, réinscription et reprise. Le scénario froidcancel utilise la pause explicite pendant la reprise : il ne prouve pas une annulation gagnée contre une libération automatique sans pause. Le scénario ouvert vérifie les24 appels CLI/MCP SC010 avec propriétaires/auxiliaires réels de la fixture privée. Ce n'est pas une preuve T3 réelle.
