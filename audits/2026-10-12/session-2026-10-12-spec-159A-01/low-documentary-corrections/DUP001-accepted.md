# DUP-001 — limite de fixture assumée

ROOT conserve les trois corps identiques de57 lignes du compilateur de CLI, fingerprint6e7f97dc8103. Ils appartiennent aux intégrations149/152,156 et159 ; chemin froid, aucun défaut runtime démontré. La copie est identifiée et traçable, pas présentée comme absence de duplication.

Aucun refactor demandé pour cette livraison. Éviter une nouvelle couche partagée ou l’import d’un fichier de tests qui enregistrerait des cas étrangers. Le support commun existant reste réemployé pour les primitives déjà disponibles. Les scripts fournisseur et leurs oracles restent distincts. Une mutualisation future de la seule construction pourra être examinée lors d’un besoin de maintenance réel ; elle n’est pas une condition de livraison159A.

Les rapports cycle1 restent historiques. Cette note porte le choix ROOT ; elle ne modifie ni la baseline de suppressions ni le finding initial.
