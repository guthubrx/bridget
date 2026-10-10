import json
R='/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage'
V=R+'/specs/149-sous-agents-lineage/validation'
T='/Volumes/8TB2/50-repos-archives/validation-cache'
base={
 "schema":"native149-%s-receipt/1",
 "session":"149-sous-agents-lineage",
 "producedBy":"testeur Sonnet 5.5 (claudeAgent), ronde r9",
 "producedAt":"2026-10-10",
 "worktree":R,
 "headCommit":"6807c22b7ada683f757486a6382aeda170ec68ab",
 "worktreeClean":False,
 "sourceFingerprint":{"value":"77d5d15503ae8abca96b817b6ec2191d7f502a248d891cc574e76fe9e0c217bf","fileCount":201,"previousR8":"1b8ed0269943968a12a1a859776f7bf938da1353cf66fcd6e2a7fe4c10dbe4f9","previousR8FileCount":200,"stableDuringBuilds":True,"files":"native-r9-fingerprint-before-final-build.json et native-r9-fingerprint-after-build.json (identiques)"},
 "productionSourceFingerprint":{"value":"b2b87458cf3cec7989debeb91352b838c417c3eb48329d57099bec4c9a0d5f29","fileCount":111,"previousR8":"3943009ca82db59d850d913b55b18c7a0a17c91c3143ffaeb39ee6cdd600526d","note":"identique au début et à la fin de la ronde : aucun fichier de production modifié par le testeur"},
 "sourceChanges":{
  "productionChangedByOthersSinceR8":[
   {"path":"crates/bridget-daemon/src/daemon/native_delegation.rs","sha256R8":"ea4e4316a796f62a74a035aebd13fac18f8b201c1df73ad14fb4d2808ad97ee6","sha256R9":"7343c6df46ed89fcaeeedbf997146d8bf64e0048466ca4bbe9392bacc63e82bc"},
   {"path":"crates/bridget-daemon/src/daemon.rs","sha256R8":"7815b19971f4eaa7051c90d2dd1f90ee3c5642566f906de8c047dd91f947c28b","sha256R9":"6baee3bad12b89e5de81ca18e82f3fb03818bd65b69b5cd3bb48761b0fbc51ca"},
   {"path":"crates/bridget-daemon/src/wrapper.rs","sha256R8":"28e92599eee1bb03017fdc8f5c730c4137609e668d862178bd05642dcc7c8a0b","sha256R9":"49f3a304d9504fe66882ce282d14d5119132ace4a0e4ddd02b017154ad5b29d1"},
   {"path":"crates/bridget-daemon/src/managed_process.rs","sha256R8":"non relevé en r8","sha256R9":"dde9324a45e6c5e223a23b4d245319a9cf40e9cd0d50b0220ffeeb522f93d197"}],
  "productionUnchangedSinceR8":["crates/bridget-transport/src/codex_app_server.rs","crates/bridget-transport/src/managed_session.rs"],
  "testsAddedByTester":[
   {"path":"crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs","sha256R9":"ec2f67ad6886dc8038738295be5349cd9c615d08e7dba04c34894b21b64c975a","note":"début octet-identique à r8, +17 tests en fin de fichier (E2 3, F3 5, O4 9)"},
   {"path":"crates/bridget-transport/tests/native_alias149_test.rs","sha256R9":"e2bae01c27deb5c5e3f2ee690bd84f7d166ac237f8da7a602783ed34e34f1f76","note":"nouveau, 2 tests"},
   {"path":"crates/bridget-daemon/tests/fixtures/codex_interactive_090.py","sha256R9":"4cd16daf1b9b371b604181dbee733bb646769a54ac3c70c04cbc0b23f26591ea","note":"fixture de recette réelle adaptée à l'alias hors BRIDGET_HOME"}],
  "productionFilesTouchedByTester":[]},
 "env":{"CARGO_TARGET_DIR":T+"/bridget149-target","CARGO_BUILD_JOBS":"2","CARGO_INCREMENTAL":"0","CARGO_NET_OFFLINE":"true (--offline --locked)","umask":"077","TMPDIR":"/tmp/b149t-r9 (0700, hors /Users/moi)","charge machine":"load average 35 à 268 pendant la ronde (autres processus)"},
 "lockfile":{"cargoUpdate":False,"newDependency":False,"cargoLockSha256":"df6a077c8d74cbb959156b57d06ae2738c9ad4872076d3abd8a5c48ef702c7b3","cargoTomlSha256":"52293175462770fc5d52722391c952428c9c3a40dad2472e527042042d3aab42"},
 "tests":{
  "commands":["cargo test -p bridget-transport --offline --locked --no-fail-fast","cargo test -p bridget-daemon --offline --locked --no-fail-fast (interrompu par la limite de 30 min dans sc005_attach_budget)","cargo test -p bridget-daemon --offline --locked --no-fail-fast --test <11 exécutables restants>","cargo test -p bridget-daemon --doc","rejeu isolé des 3 tests échoués sous charge"],
  "countingMethod":"native-r9-tool-tally-blocks.py : dernier 'test result' de chaque exécutable. L'outil r8 comptait des lignes imbriquées : r8 corrigé = daemon 1491, transport 331.",
  "byCrate":{
   "bridget-transport":{"executables":11,"passed":333,"failed":0,"ignored":2},
   "bridget-daemon":{"passed":1507,"failed":0,"ignored":58,"detail":"lib 1230 + 3 rejoués isolés ; 66 intégrations 188 ; 11 restants 84 ; sc005 : 2 ok observés ; doc 0","transientFailuresUnderLoad":3,"notConcluded":1}},
  "totalPassed":1840,"totalFailedFinal":0,"totalIgnored":60,"newTests":19,
  "notConcluded":["sc005_attach_budget::sc005_deux_vues_reelles_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent (passait en r8 en 33,8 s ; non terminé sous charge 100-260)"],
  "transientFailures":[
   "t3code::tests::spec105_question_suivie_relayee_une_fois_puis_arret (EAGAIN WouldBlock)",
   "wrapper::reconnect_tests::lignes_illisibles_et_trop_grandes_signalent_puis_laissent_progresser (borne de test)",
   "wrapper::reconnect_tests::spec_024_enregistrement_tmux_ecrit_protocole_et_canal_separes (< 3 s)"],
  "transientFailuresRerun":"3 sur 3 PASS isolés (native-r9-flakes-rerun.log)",
  "logs":{"transport":V+"/native-r9-final-transport.log","daemonLibAndIntegration":V+"/native-r9-final-daemon.log","daemonRest":V+"/native-r9-final-daemon-rest.log","daemonDoc":V+"/native-r9-final-daemon-doc.log","flakesRerun":V+"/native-r9-flakes-rerun.log","aliasRecipes":V+"/native-r9-alias-recipes-no-tui.log","aliasRecipeControl":V+"/native-r9-alias-recipe-control-r8-binary.log","aliasRecipeNewThread":V+"/native-r9-alias-recipe-new-thread.log"}},
 "mutationOracles":{"tool":V+"/native-r9-mutate.py","method":"copie jetable /tmp/b149t-r9/mut (supprimée), cible Cargo séparée (supprimée), motif unique par mutation","summaries":[V+"/native-r9-mutations-summary.txt",V+"/native-r9-mutations-summary-2.txt"],"killed":["E1","E2","E3","E4","F1","F2","F3","O1","O2","O3","O4","O5","O6","O7","O8","I1","I2","I3","A1","A2","A3","W1 (recette réelle)"],"survivorsAfterHardening":[],"firstPassSurvivors":["E4 (variante de test ajoutée)","O7 (délai d'observation ajouté)"],"witnessesGreen":["R0","A0"]},
 "supersedes":V+"/native149-%s-receipt-r8.json (conservé)",
}
def mk(prof,path,stable,sha,size,build,dur,paired):
    d=json.loads(json.dumps(base))
    d["schema"]=d["schema"]%prof
    d["supersedes"]=d["supersedes"]%prof
    d["binary"]={"profile":prof,"path":path,"stableCopy":stable,"stableCopySha256Prefix":sha[:12],"sha256":sha,"sizeBytes":size,"version":"bridget 0.1.3","arch":"Mach-O 64-bit executable arm64","installed":False,"mode":"0700","codesign":"codesign -v OK (signature ad hoc privée)","immutableFlag":"chflags uchg posé sur la copie","noProductionSourceNewerThanBinary":True,"rootTargetRelease":"non utilisé","installPath":"aucun","claimsT036_T039":False,"builtAt":"2026-10-10T21:36-21:46","buildCommand":build,"buildDuration":dur,"pairedWith":paired,"immutablePreviousCopies":{"debug r8":"bridget-833a030545c3","debug r6":"bridget-620e729fca53","debug r5":"bridget-ec6b18b5d468","release r8":"bridget-0a29ad9b2cdb","release r6":"bridget-abfb346e23cc (SHA relus : inchangés)"}}
    return d
dbg=mk('debug',T+'/bridget149-target/debug/bridget',T+'/bridget149-debug/bridget-0fb06ae920e6','0fb06ae920e6e21b0e49c91debec6032e749902c158821615a4749327f769b24',54187512,'cargo build -p bridget-daemon --bins --offline --locked','1 min 38 s',T+'/bridget149-release/bridget-abc850858975')
rel=mk('release',T+'/bridget149-target/release/bridget',T+'/bridget149-release/bridget-abc850858975','abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8',17517232,'cargo build --release -p bridget-daemon --bins --offline --locked','10 min 00 s (machine chargée)',T+'/bridget149-debug/bridget-0fb06ae920e6')
rel["binary"]["releaseCandidateOnly"]=True
json.dump(dbg,open(V+'/native149-debug-receipt-r9.json','w'),indent=1,ensure_ascii=False)
json.dump(rel,open(V+'/native149-release-receipt-r9.json','w'),indent=1,ensure_ascii=False)
print("ok")
