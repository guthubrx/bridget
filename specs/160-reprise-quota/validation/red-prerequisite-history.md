# Disposition ROOT des rouges160

**Aucune tâche rouge n’est validée par ce rapport.** Les vraies interfaces opt-in et quota restent à raccorder par les fondations. ROOT devra rejouer les mêmes11tests après T009/T010/T011, avant le code natif160.

| Étape ROOT | RC / durée | Résultat | Étape atteinte |
|---|---|---|---|
| T003-T008-T032-T033-T037-eleven-RED-first | 101 / 5.25s | [] | COMPILATION_ONLY_NOT_RED_BUSINESS :4erreurs Task.agent_type absent / StoredSendDelivery.message_bytes privé ;0tests exécutés. |
| T003-T008-T032-T033-T037-eleven-RED-compiled | 101 / 14.43s | [['FAILED', '0', '11', '0']] | FIXTURE_SERDE_ONLY_NOT_RED_BUSINESS :0P11F avant admission dans NativeDelegationRequest, unknownresume_on_limit ; aucun handler160atteint. |
| T003-T008-T032-T033-T037-eleven-RED-base-legacy-protocol | 101 / 9.7s | [['FAILED', '0', '11', '0']] | PREFOUNDATION_LEGACY_PROTOCOL :0P11F ; fallbacksupprime seulementresume_on_limit, doncNone/inactif. Certains handlers/horloges152atteints, pasoptin160réel. T004/T008/T032schemasNONATTEINTS. T037erreurlecturefil directflatten. |
| T037-RED-existing-double-notice-flat-wire-oracle | 101 / 14.75s | [['FAILED', '0', '1', '0']] | LEGACY_NOFACT_DOUBLE_NOTICE_OBSERVED :0P1F ; fauxchampoptintrue devientNone commefalse. Deuxavis réellementobservés danslesdeuxcas. Écartàexigence160 mais pasRED d’un vraiSome(true) avantT009. |

La préparation initiale a supposé à tort que Delegate ignorait les champs inconnus. La source réelle porte deny_unknown_fields. Les11échecs du second run arrivent au décodage de resume_on_limit. Aucun handler n’est atteint. Task.agent_type était aussi inexistant et message_bytes était privé ; ROOT a corrigé les fixtures avec Task.request et stored_send_message existants.

Le fallback ROOT traite uniquement l’erreur unknown field resume_on_limit. Il retire cette clé pour l’ancienne base. Toute autre erreur reste propagée. AprèsT009, le mêmeJSON doit être décodé en Some(true), sans fallback. Task lui-même ne porte pas deny_unknown_fields ; avantT010 les données quota JSON sont ignorées.

Le run legacy atteint les premières assertions152 : T003 taskfailed/pendingfalse/parent265octets ; T005pause600, deadlinegel600 et suivigel600 (pas2400 observés) ; T006restart_resumes1 contre0 attendu160 ; T007quota_waits absent. T004/T008/T032 s’arrêtent explicitement à schémaquotaNONATTEINT. Aucune frontière de relance quota n’est revendiquée.

T037 a d’abord mal compté le direct : Deliver porte son body à plat. ROOT a corrigé la lecture plate ou message.body. Le rejeu constate réellement, dans le premiercas no_fact, direct1+durable1 pourfalse, puis direct1+durable1 pourtrue demandé. Ce true devientNone sur l’ancienprotocole. C’est une preuve du double avis existant ; ce n’est pas encore un RED de vraie mission opt-in160. Les autres bornes du même test ne sont pas atteintes.

Tous les reçus attestent des sources stables pendant leur run. Les trois exécutions compilées ont une seule cibleparentlib et aucun résumé enfant. Les quatre étapes sont conservées, sans transformer une erreur de compilation ou de fixture en RED métier.

Les chemins, commandes, SHA, noms exacts, lignes de panic et limites sont dans :
/Users/moi/.cache/bridget-code160-red-preparation-20261012/ROOT-red-disposition.json

Preuves ROOT :

/Users/moi/.cache/bridget-tests160-20261012/T003-T008-T032-T033-T037-eleven-RED-first.json
SHA 4cfc80dc7b5b4b6dd6f0df2c448e5bea713e194e456a77326ae255cb5301833f
/Users/moi/.cache/bridget-tests160-20261012/T003-T008-T032-T033-T037-eleven-RED-first.log
SHA 864ae2a88b3bba4dd05148c6668a48c37b860d07639b085f0c9c764b7e4c3ae3

/Users/moi/.cache/bridget-tests160-20261012/T003-T008-T032-T033-T037-eleven-RED-compiled.json
SHA 005c4df3b8a6396e5435327ca756deb118f579a46ebe74c9c0eb3e7b2ad55b57
/Users/moi/.cache/bridget-tests160-20261012/T003-T008-T032-T033-T037-eleven-RED-compiled.log
SHA 74ec980584b148608547d5441c0af828b6a027cfc27bf33802d1786e95f6bb71

/Users/moi/.cache/bridget-tests160-20261012/T003-T008-T032-T033-T037-eleven-RED-base-legacy-protocol.json
SHA f6180455293ff7367751ccbaf43b0e0b8838ef7739c9995fde149cb4db864b6e
/Users/moi/.cache/bridget-tests160-20261012/T003-T008-T032-T033-T037-eleven-RED-base-legacy-protocol.log
SHA eac91bc530eeca323c00435ff84ebc626de38babf199b91f324b514443c51da1

/Users/moi/.cache/bridget-tests160-20261012/T037-RED-existing-double-notice-flat-wire-oracle.json
SHA 4c5b2de9c737164aa3011f269edfb262dabce4c8f5a74c03bf1d74415563cce5
/Users/moi/.cache/bridget-tests160-20261012/T037-RED-existing-double-notice-flat-wire-oracle.log
SHA 13ed51f3f824fd64cf58ae4f7ad1b060d74b13998746853f4f0bf060b17bb4b6

Aucun WT, test, Cargo, Git, SQL, service, tâche ou patch historique n’a été modifié par ce rapport.
