# Rouge transport155

T012/T014 : quatre tests compilés et exécutés, quatre échecs, RC101, 11,86 s.
Trois échecs établissent les comportements manquants : commandes et questions non remontées ; ancien tour encore accepté par le chemin historique.

L’oracle MCP initial attendait à tort une erreur JSON-RPC. Le contrat T002 établit une réponse native action:decline sans content. L’assertion a été corrigée, puis le seul test MCP a été rejoué : RC101, un échec, 2,51 s. La réponse native est correcte ; le motif approval_method_unsupported est absent. Ce résultat corrigé remplace la preuve MCP initiale.

Aucun scénario supplémentaire, aucune suite complète. Production T013/T015 autorisée après ce constat corrigé.
