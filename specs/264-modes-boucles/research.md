# Recherche et réemploi264

La base utilisateur sur les architectures agentiques a été consultée le 11 novembre 2026. Le contrat Bridget reste l'autorité produit.

Sources récentes consultées : [autorisation MCP](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/specification/2025-06-18/basic/authorization.mdx) et [interruptions et reprise persistante](https://www.langchain.com/blog/making-it-easier-to-build-human-in-the-loop-agents-with-interrupt). Ces sources renforcent deux contrôles retenus : la décision humaine doit être distincte d'un simple texte reçu ; une reprise doit conserver la décision avant de rejouer un effet. Aucun framework ni fournisseur nouveau n'est introduit.

Réemploi réel : contrôleur Loop existant, outbox et enveloppe de dispatch ; SendIdempotent ; connexion de projet ; garde guidance156 ; quotas d'interruption ; transports Codex, Claude et ACP existants.

Signaux d'alerte : le contrôleur périodique n'a pas d'identité parente ; un `from=human` ne donne pas de droit ; la FIFO reste applicable après interruption ; une admission ne prouve pas l'effet fournisseur ; les anciens connecteurs peuvent refuser le contrôle. Aucun repli silencieux n'est ajouté.

Les anciennes recettes156 de délégation native ont un contexte projet absent et un résultat natif qui reste `working` malgré le journal `turn_end`. Ces constats sont conservés, sans correction métier hors périmètre. La recette264 emprunte le vrai CLI, la socket, le daemon, l'acquittement wrapper et le rejeu. Les effets fournisseur sont vérifiés séparément avec les tests de transport existants.
