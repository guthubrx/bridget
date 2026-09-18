# Recherche — Passation structurée dans Send

## Décisions tranchées

1. Dossier = corps d'un Send existant, pas nouvelle base. La valeur ajoutée est le contrat,
   le rendu commun et les recettes de reprise, pas un second transport.
2. Artefact durable écarté : scope projet/conversation et partage incomplet ; un reçu transmis
   ne donne pas accès. Élargir la sécurité des artefacts serait un autre besoin.
3. Le dossier suit la rétention existante (sept jours par défaut), documentée avant envoi.
   Aucun archivage permanent promis. Pas d'invocation Maicie.
4. Validation locale et aperçu facultatif ; aucun formulaire humain ni validation métier imposée.
5. Pas de génération de résumé interne : l'agent rédige déjà à partir de son contexte autorisé.
6. Un nouvel outil pour cette règle métier, pas un outil par section ; CLI et MCP partagent
   uniquement la validation/rendu et le transport existant.
7. Pas de chaîne de versions autonome. Nouvelle passation = nouveau message explicite.
   Une référence à l'ancienne permet de préciser le contexte sans créer un workflow.

## Inconnues résolues par lecture

Références perdues si uniquement dans BridgetMessage.references : les garder dans body.
Limite16Kio choisie pour l'usage, inférieure au Send canonique1Mio, pas une limite découverte.
L'historique est global au daemon ; ne pas annoncer du destinataire-only.
Catalogue fermé évolutif : ajouter le nom à la liste réellement présente, pas coder
un nombre absolu dépendant de l'ordre de fusion103/102.

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

