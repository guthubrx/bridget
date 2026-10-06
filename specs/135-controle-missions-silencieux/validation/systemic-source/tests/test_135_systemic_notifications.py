import argparse
import contextlib
import importlib.util
import io
import json
import hashlib
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

BASE = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('canonical_135', BASE / 'scripts/agent_loop.py')
a = importlib.util.module_from_spec(spec)
spec.loader.exec_module(a)
domain_path = BASE / 'heartbeat.py'
if not domain_path.is_file():
    domain_path = BASE / 'tests/fixtures/psychologie_heartbeat.py'
domain_spec = importlib.util.spec_from_file_location('psychology_135', domain_path)
h = importlib.util.module_from_spec(domain_spec)
domain_spec.loader.exec_module(h)


class Systemic135Tests(unittest.TestCase):
    def _pending_fixture(self, tmp, *, legacy=False):
        rd = Path(tmp)/'run'
        a.write_json(rd/'run.json', {'run_id':'run'})
        args = argparse.Namespace(root=tmp,run_id='run',dry_run=False,verbose=False,orchestrator_role='')
        event = {'event':'task.silent','task_id':'one','routing':'worker','backend':'existing_bridget','agent_target':'worker','evidence_marker':'first'}
        send = Mock()
        writer = a.write_json
        def crash(path, data):
            if path.parent.name == 'receipts':
                raise KeyboardInterrupt('crash before receipt')
            writer(path, data)
        with patch.object(a,'write_json',side_effect=crash), contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaises(KeyboardInterrupt):
                a.notify_mission_events(args,[event],transport=send,clock='2026-10-06T00:00:00Z')
        state = a.load_json(rd/'heartbeat-state.json')
        batch = state['outbox']['existing_bridget:worker']
        if legacy:
            batch = {key:batch[key] for key in ('id','issued_at','issuer_scope','message')}
            state['outbox']['existing_bridget:worker'] = batch
            a.write_json(rd/'heartbeat-state.json',state)
        return rd, args, event, batch

    def test_pending_batch_precedes_new_fact_without_changing_replay(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd,args,event,batch = self._pending_fixture(tmp)
            second = {**event,'task_id':'two'}
            send = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[event,second],transport=send,clock='2026-10-06T00:01:00Z')
                self.assertEqual(send.call_count,1)
                self.assertEqual(send.call_args.kwargs['replay'],batch)
                self.assertEqual(send.call_args.args[2],batch['message'])
                a.notify_mission_events(args,[event,second],transport=send,clock='2026-10-06T00:02:00Z')
                a.notify_mission_events(args,[event,second],transport=send,clock='2026-10-06T00:03:00Z')
            self.assertEqual(send.call_count,2)
            self.assertIn('task=two ',send.call_args.args[2])
            self.assertNotIn('task=one ',send.call_args.args[2])

    def test_legacy_ambiguous_pending_batch_is_frozen_and_escalates_not_rebuilt(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd,args,event,batch = self._pending_fixture(tmp,legacy=True)
            second = {**event,'task_id':'two'}
            send = Mock()
            role = lambda rd,name: {'backend':'existing_bridget','agent_id':name}
            with contextlib.redirect_stdout(io.StringIO()):
                for minute in (1,2,3):
                    a.notify_mission_events(args,[event,second],transport=send,role_resolver=role,clock=f'2026-10-06T00:0{minute}:00Z')
            worker_calls = [call for call in send.call_args_list if call.args[1]['agent_id']=='worker']
            self.assertEqual(len(worker_calls),1)
            self.assertEqual(worker_calls[0].kwargs['replay'],batch)
            self.assertEqual([call.args[1]['agent_id'] for call in send.call_args_list],['worker','root'])
            decisions = list((rd/'decisions').glob('outbox-recovery-*.json'))
            self.assertEqual(len(decisions),1)
            self.assertEqual(a.load_json(decisions[0])['status'],'open')

    def test_pending_old_proof_does_not_acknowledge_current_proof(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd,args,event,batch = self._pending_fixture(tmp)
            changed = {**event,'evidence_marker':'changed'}
            send = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[changed],transport=send,clock='2026-10-06T00:01:00Z')
                self.assertEqual(send.call_args.kwargs['replay'],batch)
                a.notify_mission_events(args,[changed],transport=send,clock='2026-10-06T00:02:00Z')
                a.notify_mission_events(args,[changed],transport=send,clock='2026-10-06T00:03:00Z')
            self.assertEqual(send.call_count,2)
            self.assertNotEqual(send.call_args.kwargs['replay']['id'],batch['id'])
            self.assertEqual(a.load_json(rd/'heartbeat-state.json')['issues'][a.issue_key(changed)]['evidence_marker'],'changed')

    def test_pending_batch_replays_even_if_issue_resolved(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd,args,event,batch = self._pending_fixture(tmp)
            send = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[],transport=send,clock='2026-10-06T00:01:00Z')
                a.notify_mission_events(args,[],transport=send,clock='2026-10-06T00:02:00Z')
            send.assert_called_once()
            self.assertEqual(send.call_args.kwargs['replay'],batch)
            self.assertEqual(a.load_json(rd/'heartbeat-state.json')['issues'],{})

    def test_pending_recovery_dryrun_does_not_mutate_bytes_or_send(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd,args,event,batch = self._pending_fixture(tmp,legacy=True)
            args.dry_run = True
            before = {p.relative_to(rd):p.read_bytes() for p in rd.rglob('*') if p.is_file()}
            send = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[event,{**event,'task_id':'two'}],transport=send,clock='2026-10-06T00:01:00Z')
            send.assert_not_called()
            self.assertEqual(before,{p.relative_to(rd):p.read_bytes() for p in rd.rglob('*') if p.is_file()})

    def test_legacy_receipt_before_final_crash_does_not_validate_new_fact(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd,args,event,batch = self._pending_fixture(tmp,legacy=True)
            a.write_json(rd/'receipts'/f"notification-{batch['id']}.json",{'state':'accepted'})
            send = Mock()
            role = lambda rd,name: {'backend':'existing_bridget','agent_id':name}
            with contextlib.redirect_stdout(io.StringIO()):
                for minute in (1,2,3):
                    a.notify_mission_events(args,[event,{**event,'task_id':'two'}],transport=send,role_resolver=role,clock=f'2026-10-06T00:0{minute}:00Z')
            self.assertEqual(send.call_count,1)
            self.assertEqual(send.call_args.args[1]['agent_id'],'root')
            self.assertEqual(len(list((rd/'decisions').glob('outbox-recovery-*.json'))),1)

    def test_legacy_finalized_outbox_does_not_block_new_fact_or_new_proof(self):
        for changed_proof in (False,True):
            with self.subTest(changed_proof=changed_proof), tempfile.TemporaryDirectory() as tmp:
                rd,args,event,batch = self._pending_fixture(tmp)
                with contextlib.redirect_stdout(io.StringIO()):
                    a.notify_mission_events(args,[event],transport=Mock(),clock='2026-10-06T00:00:00Z')
                state = a.load_json(rd/'heartbeat-state.json')
                legacy = {key:batch[key] for key in ('id','issued_at','issuer_scope','message')}
                state['outbox']['existing_bridget:worker'] = legacy
                a.write_json(rd/'heartbeat-state.json',state)
                events = [{**event,'evidence_marker':'new'}] if changed_proof else [event,{**event,'task_id':'two'}]
                send = Mock()
                role = lambda rd,name: {'backend':'existing_bridget','agent_id':name}
                with contextlib.redirect_stdout(io.StringIO()):
                    a.notify_mission_events(args,events,transport=send,role_resolver=role,clock='2026-10-06T00:01:00Z')
                self.assertEqual(send.call_count,1)
                self.assertEqual(send.call_args.args[1]['agent_id'],'worker')
                self.assertNotEqual(send.call_args.kwargs['replay']['id'],legacy['id'])
                self.assertEqual(a.load_json(rd/'heartbeat-state.json').get('recovery_blocks',{}),{})
                self.assertFalse(list((rd/'decisions').glob('outbox-recovery-*.json')))

    def test_failed_frozen_replay_escalates_usefully_without_replacing_it(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd,args,event,batch = self._pending_fixture(tmp)
            send = Mock(side_effect=[RuntimeError('unavailable'),None])
            role = lambda rd,name: {'backend':'existing_bridget','agent_id':name}
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[event],transport=send,role_resolver=role,clock='2026-10-06T00:01:00Z')
            self.assertEqual([call.args[1]['agent_id'] for call in send.call_args_list],['worker','orchestrator'])
            self.assertEqual(send.call_args_list[0].kwargs['replay'],batch)
            self.assertEqual(a.load_json(rd/'heartbeat-state.json')['outbox']['existing_bridget:worker'],batch)
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[event],transport=Mock(),role_resolver=role,clock='2026-10-06T00:02:00Z')
            self.assertEqual(a.load_json(rd/'heartbeat-state.json')['issues'][a.issue_key(event)]['reminder_count'],2)

    def test_ambiguous_legacy_recovery_root_is_same_recipient_records_decision(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd,args,event,batch = self._pending_fixture(tmp,legacy=True)
            send = Mock()
            role = lambda rd,name: {'backend':'existing_bridget','agent_id':'worker'}
            with contextlib.redirect_stdout(io.StringIO()):
                for minute in (1,2,3):
                    a.notify_mission_events(args,[event,{**event,'task_id':'two'}],transport=send,role_resolver=role,clock=f'2026-10-06T00:0{minute}:00Z')
            send.assert_called_once()
            self.assertEqual(send.call_args.kwargs['replay'],batch)
            self.assertEqual(len(list((rd/'decisions').glob('outbox-recovery-*.json'))),1)
            self.assertEqual(len(list((rd/'decisions').glob('mission-escalation-*.json'))),1)

    def test_psychology_consumes_canonical_progress_without_replay_reset(self):
        task = {'task_id':'one','backend':'existing_bridget','status':'running','attempts':1,'dispatched_at':1000,'acknowledged_at':1000,'last_progress_at':1500,'last_progress_kind':'source_changed','last_progress_ref':'file:/proof','last_progress_hash':'verified','progress_seen_hashes':['verified']}
        run = {'policies':{'progress_notice_sec':300}}
        state,events = h.evaluer(run,[task],{},{},{},1600)
        self.assertEqual(events,[])
        task['last_progress_at'] = 1800
        state,events = h.evaluer(run,[task],{},{},state,1800)
        self.assertIn('tache-silencieuse',{event['code'] for event in events})

    def test_psychology_configured_ack_is_120_seconds(self):
        task = {'task_id':'one','backend':'existing_bridget','status':'dispatched','dispatched_at':1000}
        run = {'policies':{'ack_notice_sec':120,'progress_notice_sec':300}}
        self.assertEqual(h.evaluer(run,[task],{},{},{},1119)[1],[])
        self.assertIn('ack-absent',{event['code'] for event in h.evaluer(run,[task],{},{},{},1120)[1]})

    def test_busy_agent_is_connected_for_all_three_lookups(self):
        agent = {'agent_id': 'root', 'state': 'busy'}
        self.assertEqual(a.resolve_bridget_target('root', [agent]), agent)
        self.assertTrue(a.session_is_live({'backend': 'existing_bridget', 'agent_id': 'root'}, {}, {'root': agent}))
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)
            a.write_json(rd/'run.json', {})
            a.write_json(rd/'tasks/one.json', {'task_id':'one','status':'running','backend':'existing_bridget','resolved_target':'root','started_at':'2026-10-01T00:00:00Z'})
            with patch.object(a,'bridget_agents_by_id',return_value={'root':agent}):
                self.assertTrue(a.running_task_alerts(rd)[0]['agent_live'])

    def test_legacy_closed_run_does_not_collect_or_send(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)/'run'
            a.write_json(rd/'run.json',{'run_id':'run','status':'closed'})
            args = argparse.Namespace(root=tmp,run_id='run',dry_run=False,verbose=False,orchestrator_role='')
            with patch.object(a,'collect_task_results') as collect, patch.object(a,'send_orchestrator_message') as send, contextlib.redirect_stdout(io.StringIO()):
                a.heartbeat_tick(args)
            collect.assert_not_called()
            send.assert_not_called()

    def test_legacy_terminal_delivery_uses_one_stable_issue_and_no_healthy_pass_notice(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp) / 'run'
            a.write_json(rd / 'run.json', {'run_id':'run'})
            args = argparse.Namespace(root=tmp, run_id='run', dry_run=False, verbose=False, force=False, orchestrator_role='')
            collected = {'event':'task.collected', 'task_id':'one', 'status':'pass'}
            with patch.object(a, 'collect_errors', return_value=[]), patch.object(a, 'cmd_refresh_sessions'), patch.object(a, 'collect_task_results', return_value=[collected]), patch.object(a, 'ready_tasks', return_value=[]), patch.object(a, 'running_task_alerts', return_value=[]), patch.object(a, 'notify_mission_events') as notify:
                a.cmd_heartbeat_legacy(args)
            notify.assert_not_called()

    def test_unreachable_old_issue_new_proof_starts_again_from_worker(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp) / 'run'
            a.write_json(rd/'run.json', {'run_id':'run'})
            args = argparse.Namespace(root=tmp, run_id='run', dry_run=False, verbose=False, orchestrator_role='')
            event = {'event':'task.silent', 'task_id':'one', 'routing':'worker', 'backend':'existing_bridget', 'agent_target':'worker', 'evidence_marker':'old'}
            a.write_json(rd/'heartbeat-state.json', {'issues':{a.issue_key(event):{'recipient_unreachable':True,'evidence_marker':'old','reminder_count':3,'audience':'escalation'}},'outbox':{}})
            send = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[{**event,'evidence_marker':'new'}],role_resolver=lambda rd,role:None,transport=send)
            self.assertEqual(send.call_args.args[1]['agent_id'],'worker')

    def test_psychology_dryrun_no_byte_changed_and_no_transport(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)
            a.write_json(rd/'run.json', {'run_id':'run'})
            a.write_json(rd/'tasks/one.json', {'task_id':'one','backend':'existing_bridget','status':'review','collected_at':1000})
            before = {p.relative_to(rd):p.read_bytes() for p in rd.rglob('*') if p.is_file()}
            sender = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                h.passage(rd,a,maintenant=3000,dry_run=True,transport=sender)
            self.assertEqual(before,{p.relative_to(rd):p.read_bytes() for p in rd.rglob('*') if p.is_file()})
            sender.assert_not_called()

    def test_send_without_receipt_still_uses_client_idempotency_not_registration(self):
        with patch.object(a.subprocess, 'run') as send:
            a.send_bridget_message('root', 'action')
        command = send.call_args.args[0]
        self.assertIn('--id', command)
        self.assertIn('--issuer-scope', command)
        self.assertNotIn('--from', command)

    def test_shared_engine_keeps_relances_then_stops_and_new_proof_reopens(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp) / 'run'
            a.write_json(rd / 'run.json', {'run_id': 'run', 'policies': {'reminder_interval_sec': 600}})
            args = argparse.Namespace(root=tmp, run_id='run', dry_run=False, verbose=False, orchestrator_role='')
            event = {'event': 'task.no_progress', 'task_id': 'one', 'status': 'running', 'evidence_marker': 'proof-a', 'routing': 'worker', 'backend': 'existing_bridget', 'agent_target': 'worker'}
            sender = Mock()
            def role(rd, name):
                return {'backend': 'existing_bridget', 'agent_id': name}
            with contextlib.redirect_stdout(io.StringIO()):
                for seconds in (0, 1, 600, 1200, 1800, 3600):
                    a.notify_mission_events(args, [event], role_resolver=role, transport=sender, clock=f'2026-10-06T00:{seconds // 60:02}:00+00:00' if seconds < 3600 else '2026-10-06T01:00:00+00:00')
                self.assertEqual([call.args[1]['agent_id'] for call in sender.call_args_list], ['worker', 'orchestrator', 'root'])
                a.notify_mission_events(args, [{**event, 'evidence_marker': 'proof-b'}], role_resolver=role, transport=sender, clock='2026-10-06T01:01:00+00:00')
            self.assertEqual(sender.call_count, 4)
            self.assertEqual(sender.call_args.args[1]['agent_id'], 'worker')

    def test_psychology_two_anomalies_one_digest_and_preserves_business_verdict(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)
            a.write_json(rd / 'run.json', {'run_id': 'fixture'})
            task = {'task_id': 'T057', 'backend': 'existing_bridget', 'status': 'review', 'agent_target': h.ROOT_UUID, 'assigned_agent': h.ROOT_UUID, 'dispatched_at': 1000, 'collected_at': 2000, 'timeout_sec': 800}
            result = {'schema_version': 'agent-loop-result-v2', 'task_id': 'T057', 'status': 'review', 'terminal': True, 'created_at': 2000, 'worker': h.ROOT_UUID}
            a.write_json(rd / 'tasks/T057.json', task)
            a.write_json(rd / 'results/T057.result.json', result)
            before = (rd / 'tasks/T057.json').read_bytes(), (rd / 'results/T057.result.json').read_bytes()
            sender = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                first = h.passage(rd, a, maintenant=3000, transport=sender)
                h.passage(rd, a, maintenant=3001, transport=sender)
            self.assertEqual(first['sent'], 1)
            sender.assert_called_once()
            self.assertIn('actions: 2', sender.call_args.args[1])
            self.assertEqual(before, ((rd / 'tasks/T057.json').read_bytes(), (rd / 'results/T057.result.json').read_bytes()))

    def test_psychology_old_alerts_are_migrated_without_realerting_unchanged_state(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)
            a.write_json(rd / 'run.json', {'run_id': 'fixture'})
            task = {'task_id': 'T057', 'backend': 'existing_bridget', 'status': 'review', 'agent_target': h.ROOT_UUID, 'dispatched_at': 1000, 'collected_at': 2000, 'timeout_sec': 800}
            a.write_json(rd / 'tasks/T057.json', task)
            result = {'schema_version':'agent-loop-result-v2','task_id':'T057','status':'review','terminal':True,'created_at':2000,'worker':h.ROOT_UUID}
            a.write_json(rd/'results/T057.result.json',result)
            digest = hashlib.sha256((rd/'results/T057.result.json').read_bytes()).hexdigest()
            old,_ = h.evaluer({},[task],{'T057':result},{},{},2600,result_hashes={'T057':digest})
            old['alertes'] = {'T057:resultat-non-arbitre':{'sent_at':2600},'T057:deadline-depassee':{'sent_at':2600}}
            a.write_json(rd/'psychologie-heartbeat.state.json',old)
            sender = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                h.passage(rd, a, maintenant=3000, transport=sender)
                h.passage(rd, a, maintenant=6000, transport=sender)
            sender.assert_not_called()
            self.assertTrue((rd / 'heartbeat-state.json').exists())

    def test_resolved_then_identical_recurrence_is_delivered_as_new_occurrence(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)/'run'
            a.write_json(rd/'run.json',{'run_id':'run'})
            args = argparse.Namespace(root=tmp,run_id='run',dry_run=False,verbose=False,orchestrator_role='')
            event = {'event':'task.silent','task_id':'one','routing':'worker','backend':'existing_bridget','agent_target':'worker','evidence_marker':'same'}
            send = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[event],transport=send)
                first = send.call_args.kwargs['replay']['id']
                a.notify_mission_events(args,[],transport=send)
                a.notify_mission_events(args,[event],transport=send)
            self.assertEqual(send.call_count,2)
            self.assertNotEqual(first,send.call_args.kwargs['replay']['id'])

    def test_new_occurrence_replay_after_crash_keeps_same_id(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)/'run'
            a.write_json(rd/'run.json',{'run_id':'run'})
            args = argparse.Namespace(root=tmp,run_id='run',dry_run=False,verbose=False,orchestrator_role='')
            event = {'event':'task.silent','task_id':'one','routing':'worker','backend':'existing_bridget','agent_target':'worker','evidence_marker':'same'}
            send = Mock()
            writer = a.write_json
            def crash_after_outbox(path,data):
                if path.parent.name == 'receipts':
                    raise KeyboardInterrupt('simulated crash')
                writer(path,data)
            with contextlib.redirect_stdout(io.StringIO()), patch.object(a,'write_json',side_effect=crash_after_outbox):
                with self.assertRaises(KeyboardInterrupt):
                    a.notify_mission_events(args,[event],transport=send)
            replay = dict(send.call_args.kwargs['replay'])
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[event],transport=send)
            self.assertEqual(send.call_args.kwargs['replay'],replay)

    def test_old_inflight_batch_with_empty_issues_preserves_id_and_body(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)/'run'
            a.write_json(rd/'run.json',{'run_id':'run'})
            args = argparse.Namespace(root=tmp,run_id='run',dry_run=False,verbose=False,orchestrator_role='')
            event = {'event':'task.silent','task_id':'one','routing':'worker','backend':'existing_bridget','agent_target':'worker','evidence_marker':'same'}
            recipient = 'existing_bridget:worker'
            token = hashlib.sha256(json.dumps([str(rd.resolve()),recipient,[(a.issue_key(event),'same',1)]],sort_keys=True).encode()).hexdigest()
            batch = {'id':token,'issued_at':1234,'issuer_scope':'agent-loop-old','message':'Frozen old digest'}
            a.write_json(rd/'heartbeat-state.json',{'issues':{},'outbox':{recipient:batch}})
            send = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                a.notify_mission_events(args,[event],transport=send)
            self.assertEqual(send.call_args.kwargs['replay'],batch)
            self.assertEqual(send.call_args.args[2],'Frozen old digest')

    def test_legacy_terminal_arbitration_uses_same_key_on_subsequent_ticks(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)/'run'
            a.write_json(rd/'run.json',{'run_id':'run','policies':{'reminder_interval_sec':600}})
            a.write_json(rd/'tasks/one.json',{'task_id':'one','status':'review','collected_at':'2026-10-05T00:00:00Z'})
            args = argparse.Namespace(root=tmp,run_id='run',dry_run=False,verbose=False,force=False,orchestrator_role='')
            send = Mock()
            with patch.object(a,'collect_errors',return_value=[]), patch.object(a,'cmd_refresh_sessions'), patch.object(a,'collect_task_results',return_value=[{'event':'task.collected','task_id':'one','status':'review'}]), patch.object(a,'ready_tasks',return_value=[]), patch.object(a,'running_task_alerts',return_value=[]), patch.object(a,'live_session_for_role',return_value={'backend':'existing_bridget','agent_id':'root'}), patch.object(a,'send_orchestrator_message',send), contextlib.redirect_stdout(io.StringIO()):
                a.cmd_heartbeat_legacy(args)
                a.cmd_heartbeat_legacy(args)
            send.assert_called_once()

    def test_legacy_psychology_old_attempt_cannot_silence_new_attempt(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd = Path(tmp)
            a.write_json(rd/'run.json',{'run_id':'run'})
            a.write_json(rd/'tasks/T057.json',{'task_id':'T057','backend':'existing_bridget','status':'running','attempts':2,'dispatched_at':3000,'acknowledged_at':3900,'timeout_sec':800,'agent_target':h.ROOT_UUID})
            a.write_json(rd/'psychologie-heartbeat.state.json',{'alertes':{'T057:deadline-depassee':{'sent_at':2600}},'anomalies':[{'task_id':'T057','code':'deadline-depassee','timeout_sec':800}]})
            send = Mock()
            with contextlib.redirect_stdout(io.StringIO()):
                h.passage(rd,a,maintenant=4000,transport=send)
            send.assert_called_once()


if __name__ == '__main__':
    unittest.main()
