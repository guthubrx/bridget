# Validation200

Cause vérifiée dans le maître183 : agent_type.is_some() refuse AutoDelegateV2 avant toute réservation. Lecture opérateur de la clé opus2d-sys-03-production-67-apres-refus-terminal-20261028 : aucun reçu, aucune tâche. Le propriétaire a confirmé la fin de l’appel et l’absence de rejeu. Conseil transmis : omettre agent_type, utiliser engine=codex en conservant machine_id=sysle2, le SHA exact et le workspace ; connection_id uniquement si une connexion précise est imposée. Même clé corrigée possible seulement avant tout reçu et sans appel concurrent.

Correction commune : champ retiré du schéma automatique, refus MCP avant connexion sans reprise de valeur fournie, garde native inchangée avec diagnostic de champ et code public auto_admission_invalid. Le parcours explicite conserve agent_type. Aucun changement de droits, de routage ou de clé métier effectué par le mainteneur.

Deux tests200PASS, un test174PASS ; un contrôle de compatibilité168PASS. Guides32pages/522liens/13pairesPASS ; gitdiffcheck propre. Aucun binaire actif modifié. Les exigences et le blocage de maintenance198/199 restent distincts. Aucun diaporama requis pour cette correction technique.
