"""Mutations jetables r8 : copie /tmp/b149t-r8/mut uniquement, jamais l'arbre de travail.
Chaque mutation remplace un motif unique dans la copie, relance les tests ciblés, puis restaure."""
import sys, subprocess, shutil, os, pathlib
SRC = pathlib.Path('/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage')
MUT = pathlib.Path('/tmp/b149t-r8/mut')
V = SRC / 'specs/149-sous-agents-lineage/validation'
ND = 'crates/bridget-daemon/src/daemon/native_delegation.rs'
DM = 'crates/bridget-daemon/src/daemon.rs'
WR = 'crates/bridget-daemon/src/wrapper.rs'
CX = 'crates/bridget-transport/src/codex_app_server.rs'
MS = 'crates/bridget-transport/src/managed_session.rs'
F2 = ['--lib', '--offline', '--', 'native149_f2']
WS = ['--test', 'native_wrapper_stop149_test', '--offline']
TS = ['-p', 'bridget-transport', '--test', 'native_stop149_test', '--offline']
M = {
 'F0-temoin-sans-mutation': ([], F2),
 'F1-prepare_restart-sans-fermeture-execution': ([(ND, 'if matches!(snapshot.state.as_str(), "queued" | "starting" | "running"', 'if false && matches!(snapshot.state.as_str(), "queued" | "starting" | "running"')], F2),
 'F2-prepare_restart-rouvre-les-terminaux': ([(ND, 'if matches!(snapshot.state.as_str(), "queued" | "starting" | "running"', 'if true || matches!(snapshot.state.as_str(), "queued" | "starting" | "running"')], F2),
 'F3-agent-different-accepte': ([(ND, 'if target.target_agent != task.child {', 'if false && target.target_agent != task.child {')], F2),
 'F4-autorite-sans-controle-du-corps': ([(ND, 'task.result.as_deref() != Some(message.body.as_str())\n        || message.body.trim()', 'false\n        || message.body.trim()')], F2),
 'F5-autorite-sans-controle-destinataire': ([(ND, 'message.from != task.child || message.to != task.owner', 'message.from != task.child')], F2),
 'F6-autorite-sans-revocation': ([(ND, 'if st.delegation_store.revoked_agent(&task.owner)?\n        || st.delegation_store.revoked_agent(&task.root_owner_agent_id)?\n        || st.delegation_store.revoked(&task.owner_instance)? {\n        return Err("native_result_authority_revoked".into());', 'if false {\n        return Err("native_result_authority_revoked".into());')], F2),
 'F7-proprietaire-hors-ligne-faux-result_sent': ([(ND, 'task.error = Some("native_result_owner_offline".into());', 'task.result_sent = true; task.error = Some("native_result_owner_offline".into());')], F2),
 'F8-garde-expediteur-par-defaut-relache': ([(DM, '} else if !human_from_bare_client && !sender_is_authorized(st, conn_id, &message.from) {', '} else if false && !human_from_bare_client && !sender_is_authorized(st, conn_id, &message.from) {')], F2),
 'F9-autorite-sans-controle-du-proprietaire-vivant': ([(ND, '!= Some((task.owner.clone(), task.owner_instance.clone()))\n        || !can_own_task(st, &task, &task.owner, &task.owner_instance)? {', '!= Some((task.owner.clone(), task.owner_instance.clone())) && false {')], F2),
 'F10-attente-descendants-ignore-les-fermes': ([(ND, 'child.child_instance.as_deref() == Some(link.child_instance_id.as_str())\n                                && child.state == "failed"', 'child.child_instance.as_deref() == Some(link.child_instance_id.as_str())\n                                && false && child.state == "failed"')], F2),
 'O1-wrapper-sans-gestionnaire-sigterm': ([(WR, '.map(|_| NativeTerminationSignal::new()).transpose()?;', '.map(|_| NativeTerminationSignal::new()).transpose()?.filter(|_| false);')], WS),
 'O2-wrapper-natif-sans-stop_native_mission': ([(WR, 'transport.stop_native_mission(&bootstrap.mission_id);', '')], WS),
 'O3-wrapper-natif-se-reconnecte-sur-eof': ([(WR, 'Ok(0) => {\n                if native_bootstrap.is_some() {\n                    break;\n                }', 'Ok(0) => {\n                if false && native_bootstrap.is_some() {\n                    break;\n                }')], WS),
 'O4-wrapper-sigterm-sans-sortie-de-boucle': ([(WR, 'let read_result = reader.read_line(&mut line);\n        if native_termination.as_ref().is_some_and(NativeTerminationSignal::requested) {\n            break;\n        }', 'let read_result = reader.read_line(&mut line);')], WS),
 'T0-temoin-transport': ([], TS),
 'T1-codex-stop_native-sans-interrupt': ([(CX, 'if let Some((thread_id, turn_id)) = turn {\n            let _ = request_with_timeout(', 'if let Some((thread_id, turn_id)) = turn.filter(|_| false) {\n            let _ = request_with_timeout(')], TS),
 'T2-codex-stop_native-sans-fermeture-admission': ([(CX, 'self.alive.store(false, Ordering::SeqCst);\n        let (queue, wake) = &*self.queue;\n        queue.lock().unwrap_or_else(|e| e.into_inner()).closed = true;\n        wake.notify_all();\n        if let Some((thread_id', 'let (_queue, wake) = &*self.queue;\n        wake.notify_all();\n        if let Some((thread_id')], TS),
 'T3-codex-stop_native-interrompt-un-autre-tour': ([(CX, '.as_ref().filter(|detail| detail.message_id == message_id)', '.as_ref()')], TS),
 'T4-defaut-managed-stop_native-sans-annulation': ([(MS, 'self.cancel_delivery(message_id, "wrapper natif arrêté");\n        self.stop();', 'self.stop();')], TS),
}
names = sys.argv[1:] or list(M)
env = dict(os.environ, TMPDIR='/tmp/b149t-r8', CARGO_TARGET_DIR='/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target-mut', CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0')
os.umask(0o077)
touched = set()
for name in names:
    muts, args = M[name]
    # copie fraîche des fichiers concernés (production et tests)
    for f in {ND, DM, WR, CX, MS} | touched:
        shutil.copy(SRC / f, MUT / f)
    for t in ('crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs', 'crates/bridget-daemon/tests/native_wrapper_stop149_test.rs', 'crates/bridget-transport/tests/native_stop149_test.rs'):
        if (SRC / t).exists():
            shutil.copy(SRC / t, MUT / t)
    for f, old, new in muts:
        t = (MUT / f).read_text()
        assert t.count(old) == 1, (name, old, t.count(old))
        (MUT / f).write_text(t.replace(old, new))
    cmd = ['cargo', 'test', '-p', 'bridget-daemon'] + args if args is not TS else ['cargo', 'test'] + args
    log = V / f'native-r8-mutation-{name}.log'
    with open(log, 'w') as out:
        r = subprocess.run(cmd, cwd=MUT, env=env, stdout=out, stderr=subprocess.STDOUT)
    txt = log.read_text()
    res = [l for l in txt.splitlines() if l.startswith('test ') and ('... ' in l)]
    ko = [l for l in res if 'FAILED' in l]
    print(name, 'rc=', r.returncode, 'tests=', len(res), 'echecs=', len(ko), flush=True)
    for l in ko: print('    ECHEC', l.split('::')[-1], flush=True)
