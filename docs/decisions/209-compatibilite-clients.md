#209 — Contrôle local du candidat avant sa bascule

Date :2026-10-29. Statut : accepté dans la correction demandée.

Le candidat Bridget publie les versions minimales prises en charge de ses pilotes natifs. Claude2.1.280 découle du refus API400 conservé dans la preuve208 ; Codex0.162.1 est le schéma de référence ADR155. Ces minima relèvent de la politique de support de cette release ; ils ne prétendent pas dater une rupture de tous les anciens clients.

L’installateur et l’updater exécutent le contrôle du candidat avant tout arrêt/bascule. Seuls les programmes locaux présents sont concernés ; absence et version inconnue restent distinctes. Les comptes et modèles ne sont pas testés par une sonde de version. Aucune mise à jour client n’est automatique : consentement distinct, mécanisme officiel adapté à l’installation, puis qualification réelle.

Complexité O(C+P), Cconnexions, Pprogrammes distincts<=16 ; aucun appel répété par modèle. Temps/volume bornés. Le rapport comporte uniquement identités techniques autorisées, versions et conseils fermés ; jamais stdout/stderr brut.

Sources officielles consultées : https://support.claude.com/en/articles/14554922-claude-code-user-faq et https://learn.chatgpt.com/docs/cli ; les mises à jour doivent respecter le gestionnaire d’installation existant. Aucun lien/version disponible en ligne n’est traité comme preuve d’accès au modèle.
