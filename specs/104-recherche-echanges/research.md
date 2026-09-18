# Recherche — Recherche reprenable dans les sources existantes

## Décisions tranchées

1. Store::search_messages existe mais n'a aucun appel trouvé : l'exposer après correction,
   ne pas créer un moteur parallèle. Son SQL répète la normalisation et masque les erreurs.
2. Conserver la recherche littérale AND par sous-chaînes ; remplacer le double repli SQL/Rust
   par le seul fold_for_search. Pas de FTS5 : tokenisation et sous-chaînes ne sont pas équivalentes.
3. Lire les candidats par index et traiter un budget fini ; pagination possible sans résultat.
   Un LIMIT sur les occurrences seules n'aurait pas borné le travail.
4. Aucun rowid stable supposé, aucun offset qui se décale après purge : tuples existants.
5. Ledger modifiable/purgé : pagination vivante explicite, pas snapshot imaginaire.
   Fil102 append-only : borne de séquence immuable.
6. Connexion serveur read-only et limite globale de concurrence, aucun scan sous Mutex daemon.
   Pas de nouveau pool, queue ou service. Les index d'accès sont les seuls nouveaux objetsSQL.
7. Scope messages=participant et fil=membership. Cela ne corrige pas la projection globale
   historique ; cette différence doit être documentée pour ne pas promettre un cloisonnement global.
8. Une source par requête, fil identifié explicitement ; pas d'union inter-fils, filtre projet
   inféré ou historique provider. Étendue suffisante pour retrouver et citer une décision.
9. La relecture exacte est nécessaire : un extrait512octets seul ne permet pas de vérifier
   une occurrence située plus loin. Corps long fragmenté avec empreinte, sans voisins implicites.

## Points à mesurer pendant l'implémentation

Les seuils du test-plan sont des gates futurs, pas des résultats observés. Contrôler
EXPLAIN sur les nouvelles plages et mémoire/latence du cas16Mio. Si échec, ajuster le chemin
de lecture/normalisation et les index dans ce périmètre, sans réduire silencieusement
le corpus ni ajouter un moteur tiers. Les tables102 sont une dépendance non encore livrée.

## Vérifications et sources (2026-09-16)

Baselines lues : /Users/moi/.speckit/research/05-knowledge-management.md,
 /Users/moi/.speckit/research/06-security-compliance.md,
 /Users/moi/.speckit/research/10-data-privacy.md. Pas de métrique marketing reprise.
Recherche préalable DevKMS : command -v mem n'a retourné aucun binaire ; aucune mémoire
Bridget dans /Users/moi/.Codex/projects/*/memory/MEMORY.md. Repli : décisions capturées ici
et en ADR, pas de capture DevKMS prétendue.

Sources primaires consultées :
- [APQC, transfert de connaissances critiques](https://www.apqc.org/training/transferring-critical-knowledge-learning-lab) :
  sélectionner les éléments nécessaires à la reprise ; transposition à nos dossiers = décision de conception.
- [Survey RAG](https://arxiv.org/abs/2506.00054) : la récupération et la génération apportent
  leurs propres coûts et limites ; aucune justification locale pour ajouter un pipeline RAG.
- [OWASP Authorization](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html) :
  vérifier l'accès à chaque requête, y compris une suite paginée.
- [PortSwigger IDOR](https://portswigger.net/web-security/access-control/idor) :
  connaître un identifiant d'objet ne doit pas permettre d'en lire le contenu.
- [CNIL minimisation](https://www.cnil.fr/fr/minimiser-les-donnees-collectees) et
  [ICO minimisation](https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/data-protection-principles/a-guide-to-the-data-protection-principles/data-minimisation/) :
  ne transporter que le contexte nécessaire, sans collecte automatique de sources.
- [SQLite rowid](https://www.sqlite.org/rowidtable.html) : un rowid non aliasé à INTEGER
  PRIMARY KEY peut changer ; ne pas en faire un repère de pagination durable.
- [SQLite query planner](https://www.sqlite.org/queryplanner.html) : les index multi-colonnes
  organisent recherche/tri ; le plan réel doit être contrôlé avec les données représentatives.
- [MCP Tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) :
  schémas et erreurs d'outils explicites, sans transformer une erreur en contenu vide.

Requêtes exécutées notamment : « site:owasp.org authorization cheat sheet validate permissions
every request », « site:arxiv.org retrieval augmented generation limitations 2025 »,
« site:apqc.org knowledge transfer capture critical knowledge », « site:ico.org.uk data minimisation GDPR ».
Aucun envoi de journal privé, secret ou donnée client dans ces recherches ; exemples synthétiques.
Ces sources éclairent les choix ; elles ne prouvent pas la conformité ou la performance de Bridget.

