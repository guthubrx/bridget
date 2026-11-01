# Noms humains dans les fiches — T035

Constat utilisateur : bridget specialists list présente le rôle et l'UUID, sans bridge1016. Lecture CLI réelle avant correction : trois spécialistes dont bc833746-3f24-41cd-bc64-46d60f4c0769, rôle mainteneur-bridget ; aucun champ display_name.

Correction : champ additif display_name dans chaque entrée list/search et dans get. Source : AgentProfileStore.profile_for_agent_id, le profil humain persistant ; aucun nom figé ni déduit du rôle. Un renommage est relu à la prochaine consultation, même hors ligne. L'identifiant et le rôle restent séparés. Les filtres de visibilité sont appliqués avant enrichissement ; aucun nouveau droit, changement de format persistant ou déclaration de rôle.

Vérifications et livraison à compléter. L'application Android ne change pas de parcours pour cette correction de réponse CLI/API.

Test ciblé unique PASS (0,02s) : list/search/get renvoient bridge1016 puis Gentil après renommage, même UUID et rôle patate ; fiche spécialiste strictement inchangée par les lectures, agent hors périmètre masqué. Première compilation de test corrigée pour comparer les sérialisations (SpecialistRecord n'implémente pas PartialEq), aucun code produit supplémentaire. Pas de campagne générale.
