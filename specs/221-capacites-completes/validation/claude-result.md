# Résultat final Claude sans raison facultative

Statut : code corrigé ; vérification et chargement en cours.

## Incident réel et frontière de preuve
Tâche f7ab1d73-8496-456f-83e5-2d43ac1ac4a4, ronde3,
mission8779e0bc-2061-4c39-9511-995564e18277, SYS.
Journal privé corrélé à instancea84ca3bb-9482-498b-8c55-ec84141de7c9,
génération1742 : permission_denied puis error / terminal Claude inconnu à
2026-10-31T22:26:50Z. Les rondes1/2 ont leurs reçus acceptés, pas la3.
Le reçu worker est terminal et le slot3 closed=cancelled/producer_terminal=true.
Le résultat fournisseur brut n’est conservé ni dans ce journal ni dans le
transcript Claude. Le commit c331a8caa0139d669bf5766c077a9b3613756106
et les produits existent selon le propriétaire ; ils ne prouvent pas un
résultat fournisseur réussi. Aucun état de cette tâche n’est réécrit.

## Défaut confirmé et correction
Le contrat officiel SDKResultMessage rend terminal_reason facultatif :
https://code.claude.com/docs/en/agent-sdk/typescript#sdkresultmessage
L’ancien adaptateur exigeait terminal_reason=completed pour reconnaître la
réussite. Une enveloppe type=result/subtype=success/is_error=false sans cette
raison produisait donc un faux échec. C’est un défaut démontré du code, sans
preuve que l’enveloppe historique de cette tâche avait exactement cette forme.

Le repli success ne s’applique qu’à une raison absente ou null. Une raison
présente, inconnue ou mal typée reste refusée. Les sous-types d’erreur connus
conservent une cause fermée. Les interruptions Bridget, annulations et
limites gardent leurs gardes. Un refus d’outil conserve son avertissement
sans changer un résultat réussi en échec.
La réussite signifie ici résultat du programme, pas validation métier :
un hook UserPromptSubmit peut aussi rendre success/num_turns=0 sans raison.
Aucun assistant end_turn, texte ou commit ne sert de preuve terminale.

Le journal existant reçoit provider_result corrélé à message_id :
sous-type et raison issus de listes fermées, booléen is_error, nombre de tours
et refus bornés, interruption Bridget et issue retenue. Aucun result/errors
ou texte inconnu n’est copié. L’événement est additif, sans nouveau format
persistant obligatoire ni état d’admission.
Le statut des erreurs anciennes ou nouvelles donne directement la vérification
manquante, le geste propriétaire et les prérequis d’une suite distincte.
Le détail complet autorisé ajoute ronde/mission à la référence fournisseur.
Pas de relance, de substitution ni de reclassification automatique.

## Vérification ciblée
Classification sans raison, erreurs connues, contradictions, raisons inconnues,
annulation, limite et interruption d’autorisation. Fixture de session gérée
réelle avec faux programme : remise du rapport sans raison, refus d’outil,
refus de contrôle maintenu, journal corrélé et borné sans texte privé.
Le test de statut vérifie la portée, le recours humain et l’absence de mutation.
Pas de modèle réel ni de nouvelle mission métier.

## Revue et arbitrage
Revue native7e52f8b3-08d0-47ef-869e-ef17a8741191, lecture complète du patch,
sans build ni réseau : pas de défaut bloquant. P2 étroit subtype=null avec
raison completed confirmé et corrigé par is_none_or(Value::is_null), couvert
dans le test existant. La contradiction is_error=true/raison completed reste
un échec, avec cause inconnue plutôt que le libellé trompeur completed.
Aucune nouvelle campagne de revue : ces changements suivent l’objection et
le contrat déjà relu. Limites explicites : hooks à raison non completed
restent fermés, absence de métadonnées sans tour actif, pas de preuve terminale
historique récupérable. Résultat lu/ACK et clôture demandée une fois.

Les41 tests transport distincts passent ; une première erreur d’assertion de
la nouvelle fixture a été corrigée, puis un ETXTBSY isolé du fixture P4 sous
concurrence a repassé seul. Un test daemon de statut PASS.
La dernière correction null/contradiction est recontrôlée seule.
Audit source ciblé : priorité des droits conservée, replis fermés, journal borné,
aucune mutation de tâche/profil/admission, aucun nouvel appel fournisseur ni droit.
Pas de diaporama requis : changement technique du transport et diagnostic.
