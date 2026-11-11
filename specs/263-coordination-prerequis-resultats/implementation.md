# Correction 263

Version 0.8.29, identité 263-coordination-resultats-r1. Corrections portables, sans chemin personnel ni nom de machine imposé dans le produit.

## Causes vérifiées

- Guidage de l’assembleur : un résultat avait été remis quatre secondes après le dernier steer accepté. Les deux refus suivants concernaient donc une ronde déjà finie. Le nouveau refus indique la tâche existante, sa lecture et la continuation native éventuelle. Il ne transforme pas le motif hors projet en autorisation.
- Vue locale : le registre du worker attesté était sélectionné pour shared hors Git seulement. La préparation locale commit_view pouvait consulter un autre profil. Le même registre attesté est maintenant utilisé pour cette vue. Identités, empreintes et mappings restent vérifiés ; aucun registre n’est réinitialisé.
- Web : les définitions réellement lancées pour les deux missions discovery interdisaient WebSearch, WebFetch et Bash. La machine n’était pas la cause. Le contrat MCP et le catalogue expliquent cette limite avant admission. Le catalogue ne certifie pas les outils web, les données ou les hôtes SSH externes. Ces ressources ne sont pas transférées avec le dépôt ; une contrainte de destination doit être explicite quand elle est nécessaire.
- Résultat incomplet : le pilote effaçait les réponses achevées en attendant les tâches de fond. Il conserve désormais ces textes et les réunit une seule fois, en O(total des textes), avant le résultat durable. L’attente des tâches de fond et l’échéance initiale restent applicables.
- Journal après arrêt : la façade suivait un flux actif et rendait seulement AgentStopped. Elle explique maintenant la portée de cette lecture et l’accès à l’historique conservé dans la vue native, sans proposer de relancer une mission pour le lire.
- Limite de préparation : trois contrôles perdaient leur cause connue. Ils rendent désormais operation_capacity ou metadata_capacity sous le contrat déjà négocié. Le reçu ancien ne conservait pas les compteurs permettant de départager ces contrôles. La tâche annulée n’est pas rejouée pour reconstituer cette observation.

## Faits non établis

Les deux envois suspectés conservent le destinataire attendu et des corps non vides. Le journal de l’autre enfant contient une seule entrée reçue, non vide, de 5083 octets. Aucun défaut de routage ni perte du corps n’est démontré. Les preuves techniques privées conservent les identifiants et empreintes ; aucun texte métier n’est publié.

La coupure de transport signalée précède la maintenance de 262. Les traces conservées ne permettent pas d’en établir le type d’erreur système. Le client journalise maintenant l’étape connect/write/read, le type fermé de l’erreur et la date. Aucun chemin, compte, secret ou corps n’est ajouté à ce diagnostic. La reprise conserve les identifiants et les droits.

## Revue et source exécutable

Revue indépendante en lecture seule : tâche 87bb84ef-ff37-472e-9318-45e4d1f288dc, clé bridget-263-result-message-review-20261111-r1. Le défaut de conservation est confirmé. Le signalement de message vide n’est pas confirmé. La mission de revue a rendu son résultat ; l’annulation propriétaire est enregistrée et l’arrêt de son enfant est confirmé. Sa vue reste conservée selon le contrôle natif.

La décision D1 du programme 260 reste respectée. build-runtime.py reconstruit la base active 75259217a72b3177cc0f50ba58c1fc3f97e5e22e avec runtime.patch. La source obtenue reproduit exactement les 23 fichiers attendus. Les extractions réservées à une activation ultérieure restent hors du binaire.

## Validation et activation

À compléter avec les reçus de qualification et de chargement réels avant clôture.

Les sept avertissements Linux préexistants de la base ont été corrigés sans changement de comportement : conversions entières explicites communes aux plateformes et conditions let regroupées. Ces ajustements ne déclenchent aucune extraction réservée au programme 260.

La qualification de distribution a refusé le premier binaire Linux avant activation sur Debian 12 : il exigeait glibc 2.39. Le candidat commun est reconstruit sur Debian 12. Le script check-linux-abi.py impose désormais glibc 2.36 au plus à make release sur Linux ; une absence de preuve n’est pas un succès. Trois tests et un refus réel du premier binaire qualifient cette garde.
