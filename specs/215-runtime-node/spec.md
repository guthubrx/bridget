# Session215 — cohérence du runtime Node et diagnostic Git SYS

Demande du propriétaire monuments : aligner Node26.12.1 sur SYS et KS pour les missions Bridget ; examiner le refus compact_plan/git_timeout de la tâche728861ad annulée. Aucune nouvelle mission ni reprise de cette clé. Aucun service Opus2D/NE02/Roadie à modifier.

Installer seulement la distribution officielle Node26.12.1 Linuxx64, vérifier son SHA256 publié et conserver les versions précédentes. Les liens utilisateur déjà prioritaires dans le PATH worker sélectionnent la nouvelle version pour les futurs processus. Aucun remplacement du paquet système ni redémarrage des processus métier. Cette opération ne garantit pas à elle seule le déterminisme de toutes les productions.

La base Git existe sur SYS. Le diagnostic doit distinguer le fait historique et la mesure actuelle ; une cause non reproduite reste inconnue. Ne pas augmenter un délai ni modifier le code sans défaut établi. Livraison documentaire et configuration runtime, aucun nouveau binaire Bridget.
