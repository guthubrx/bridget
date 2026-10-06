# Décisions137
Style côté interface, pas dans le pont qui ne transmet qu'une chaîne.
Réutiliser className et les paragraphes de ChatMarkdown.
Rejeté: HTML injecté (inerté volontairement), texte Markdown modifié (modifierait
le prompt), opacity sur la bulle (griserait le corps), nouveau composant inutile.
Version installée issue de local/v0.0.45, pas de la branche ancienne du clone.
Lecture des composants parents avant patch: CSS et copie restent sous ChatMarkdown.
