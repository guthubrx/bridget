Préparation T002160 — lecture seule

Verdict réel : **non vu en réel**. Dans /Users/moi/.cache/bridget-core/daemon-stderr.log, 19922 lignes ne contiennent aucun fait de quota retenu. Ledger /Users/moi/.cache/bridget-core/bridget.db : 3765 lignes, six mentions lexicales de quota, zéro rate_limit_event/usage_limit/rate_limit et zéro raison exécution quota/limit. 1079 fichiers de sessions, 657151 lignes : aucun objet structuré rate_limit_event/rateLimitReachedType ; les 16 mentions se trouvent dans update/turn_start. Aucun corps ni secret publié. Chemin A/B, raison DeliveryRejected et ordre quota/échec réel restent inconnus.

Parent failed : accepté en principe par /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/managed_supervisor.rs:64–109 puis /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/execution_store.rs:953–957, sous gardes budget, génération/révision/preuve et absence d’exécution concurrente. Réserver ne signifie pas reconstruire.

T036 : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:4607–4655 lie l’exécution idempotente AVANT l’injection. Si contexte présent, le refus synchrone conserve cette liaison et ne publie pas failed. DeliverExecution4558–4591 lie aussi avant, mais publie failed sur refus. Les transitions terminales5988–5991 retirent la liaison. Refus asynchrone6747–6763 et réponse vide6680–6695 : capturer les métadonnées avant la transition, et ne pas supposer bindings.get disponible après. Aucune nouvelle API ni correction produite.

Les chemins A/B sont vérifiés par lecture, pas observés sur un quota réel : wrapper6747→daemon22376→native_delegation2839 pour A ; native_delegation3790–3828 pour B. Les anciennes références WT150/base1f2c661d restent historiques et ne prouvent pas la base159B livrée.

Limites : fichiers runtime vivants sans gel global ; DB snapshot RO seulement ; aucune tâche cochée, aucun test/fournisseur/processus lancé. Le manifeste JSON donne les plages et SHA des sources réellement relues.
