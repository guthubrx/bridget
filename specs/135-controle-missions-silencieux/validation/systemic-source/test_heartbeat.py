"""Oracles purs et transport simulé. Aucun Bridget ni LaunchAgent réel."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch
import heartbeat as h

# Le test isolé utilise la même version de moteur que l'adaptateur testé.
if (Path(__file__).parent / 'scripts/agent_loop.py').is_file():
    h.CANON = Path(__file__).parent / 'scripts/agent_loop.py'
CANON = h.module_canonique()


def tache(**extra):
    return {'task_id':'controle','backend':'existing_bridget','status':'running',
        'agent_target':h.ROOT_UUID,'attempts':1,'dispatched_at':1000,'acknowledged_at':1000,
        'timeout_sec':5000,**extra}


def resultat(status='pass'):
    return {'schema_version':'agent-loop-result-v2','task_id':'controle','terminal':True,'status':status}


def codes(alertes):return {a['code'] for a in alertes}


class CanonSimule:
    def __getattr__(self, name):
        return getattr(CANON, name)
    @staticmethod
    def load_json(path,default=None):return json.loads(path.read_text()) if path.exists() else default
    @staticmethod
    def write_json(path,data):
        path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(data))


class Controle(unittest.TestCase):
    def test_ack_borne_exacte_et_silence_normal(self):
        t=tache(status='dispatched',acknowledged_at='')
        self.assertEqual(h.evaluer({},[t],{},{},{},1599)[1],[])
        self.assertEqual(codes(h.evaluer({},[t],{},{},{},1600)[1]),{'ack-absent'})

    def test_silence_tache_root_et_deadline_non_renouvelee(self):
        t=tache(timeout_sec=1800)
        state,alert=h.evaluer({},[t],{},{},{},2200)
        self.assertIn('tache-silencieuse',codes(alert));self.assertTrue(state['root_task_present'])
        t['dispatched_at']=2500;t['acknowledged_at']=2500
        state,alert=h.evaluer({},[t],{},{},state,2800)
        self.assertIn('deadline-depassee',codes(alert))

    def test_jalon_reel_et_timer_repetition_ne_compte_pas(self):
        p={'controle':{'task_id':'controle','attempt_id':'1','ownerUUID':h.ROOT_UUID,'milestone':'banc-écrit','timestamp':2100}}
        state,alert=h.evaluer({},[tache()],{},p,{},2200);self.assertEqual(alert,[])
        p['controle']['timestamp']=3300
        state,alert=h.evaluer({},[tache()],{},p,state,3300)
        self.assertIn('tache-silencieuse',codes(alert))

    def test_presence_et_mtime_ne_sont_pas_progres(self):
        t=tache(last_seen_secs=0,updated_at=2400)
        self.assertIn('tache-silencieuse',codes(h.evaluer({},[t],{},{},{},2400)[1]))
        p={'controle':{'task_id':'controle','attempt_id':'ancienne','ownerUUID':h.ROOT_UUID,'milestone':'x','timestamp':2400}}
        self.assertIn('tache-silencieuse',codes(h.evaluer({},[t],{},p,{},2400)[1]))

    def test_log_croissant_seul_renouvelle_progression(self):
        p={'controle':{'task_id':'controle','attempt_id':'1','ownerUUID':h.ROOT_UUID,'milestone':'test-natif','timestamp':1100,
            'process_pid':123,'log_path':'/tmp/log-simule','runtime_log_bytes':100}}
        state,_=h.evaluer({},[tache()],{},p,{},1200)
        p['controle']['runtime_log_bytes']=200
        state,alert=h.evaluer({},[tache()],{},p,state,2300);self.assertEqual(alert,[])
        p['controle']['runtime_log_bytes']=50
        state,alert=h.evaluer({},[tache()],{},p,state,3500)
        self.assertIn('tache-silencieuse',codes(alert))

    def test_pass_collecte_sans_cloture_et_alerte600(self):
        t=tache(status='review',collected_at=1500)
        state,alert=h.evaluer({},[t],{'controle':resultat()},{},{},2099)
        self.assertEqual(alert,[]);self.assertTrue(state['collectes']['controle']['awaiting_root_validation'])
        self.assertEqual(t['status'],'review')
        self.assertIn('resultat-non-arbitre',codes(h.evaluer({},[t],{'controle':resultat()},{},state,2100)[1]))

    def test_mauvais_resultats_et_pass_sans_fichier(self):
        for status in ['fail','blocked','review','pass']:
            t=tache(status=status,collected_at=1500)
            self.assertIn('resultat-non-arbitre',codes(h.evaluer({},[t],{},{},{},2100)[1]))
        t=tache(status='review',collected_at=1500)
        self.assertIn('resultat-non-arbitre',codes(h.evaluer({},[t],{'controle':{}},{},{},2100)[1]))

    def test_arbitrage_lie_hash_et_resolution_canonique(self):
        r=resultat();t=tache(status='review',collected_at=1500,disposition_reason='raison seule insuffisante')
        self.assertIn('resultat-non-arbitre',codes(h.evaluer({},[t],{'controle':r},{},{},2100)[1]))
        v={'controle':{'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(r),'verdict':'pass'}}
        self.assertIn('resultat-non-arbitre',codes(h.evaluer({},[t],{'controle':r},{},{},2100,v)[1]))
        t['status']='pass'
        self.assertEqual(h.evaluer({},[t],{'controle':r},{},{},2100,v)[1],[])
        v['controle']['hashresult']='autre'
        self.assertIn('resultat-non-arbitre',codes(h.evaluer({},[t],{'controle':r},{},{},2100,v)[1]))
        t['status']='pass';self.assertIn('resultat-non-arbitre',codes(h.evaluer({},[t],{'controle':r},{},{},2100)[1]))

    def test_fail_root_arbitre_sans_alerte_et_sans_cloture(self):
        r=resultat('review');t=tache(status='fail',collected_at=1500,disposition_reason='root refuse la preuve')
        v={'controle':{'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(r),'verdict':'fail'}}
        state,alert=h.evaluer({},[t],{'controle':r},{},{},2100,v)
        self.assertNotIn('resultat-non-arbitre',codes(alert))
        self.assertFalse(state['collectes']['controle']['awaiting_root_validation'])
        run={'status':'closed','final_inventory':{'remaining':[],'_root_validation_ok':True,'source_epoch':'s','count':1}}
        state,alert=h.evaluer(run,[t],{'controle':r},{},{},2100,v)
        self.assertFalse(state['closed']);self.assertIn('fermeture-non-validee',codes(alert))
        suite=tache(task_id='suite',status='pending',depends_on=['controle'])
        self.assertNotIn('reprise-root-requise',codes(h.evaluer({},[t,suite],{'controle':r},{},{},2100,v)[1]))

    def test_fail_sans_hash_root_raison_ou_resolution_reste_non_arbitre(self):
        r=resultat('review');t=tache(status='fail',collected_at=1500,disposition_reason='root refuse')
        preuve={'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(r),'verdict':'fail'}
        cas=[({},t),({'controle':dict(preuve,hashresult='autre')},t),
            ({'controle':dict(preuve,validatorUUID='autre')},t),
            ({'controle':preuve},dict(t,disposition_reason='   ')),
            ({'controle':preuve},dict(t,status='review')),
            ({'controle':dict(preuve,verdict='pass')},t)]
        for validation,tache_cas in cas:
            with self.subTest(validation=validation,tache=tache_cas):
                self.assertIn('resultat-non-arbitre',codes(h.evaluer({},[tache_cas],{'controle':r},{},{},2100,validation)[1]))

    def test_arbitrage_fail_termine_deadline_tentative_pas_deadline_run(self):
        r=dict(resultat('review'),created_at=1900,worker=h.ROOT_UUID)
        t=tache(status='fail',timeout_sec=800,collected_at=1900,disposition_reason='root refuse après diagnostic')
        v={'controle':{'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(r),'verdict':'fail'}}
        state,alert=h.evaluer({},[t],{'controle':r},{},{},3000,v)
        self.assertNotIn('deadline-depassee',codes(alert))
        self.assertNotIn('resultat-non-arbitre',codes(alert))
        self.assertFalse(state['closed'])
        run={'created_at':1000,'policies':{'total_timeout_sec':1800}}
        self.assertIn('deadline-run-depassee',codes(h.evaluer(run,[t],{'controle':r},{},{},3000,v)[1]))
        for champ,valeur in [('hashresult','invalide'),('validatorUUID','autre')]:
            mauvaise={'controle':dict(v['controle'],**{champ:valeur})}
            self.assertIn('deadline-depassee',codes(h.evaluer({},[t],{'controle':r},{},{},3000,mauvaise)[1]))
        sans_raison=dict(t,disposition_reason='')
        self.assertIn('deadline-depassee',codes(h.evaluer({},[sans_raison],{'controle':r},{},{},3000,v)[1]))

    def test_nouveau_dispatch_apres_fail_arbitre_reste_surveille(self):
        r=dict(resultat('review'),created_at=1900,worker=h.ROOT_UUID)
        t=tache(status='fail',timeout_sec=800,collected_at=1900,disposition_reason='root refuse')
        v={'controle':{'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(r),'verdict':'fail'}}
        state,_=h.evaluer({},[t],{'controle':r},{},{},2100,v)
        nouveau=dict(t,status='dispatched',attempts=2,attempt_id='2',dispatched_at=2500,acknowledged_at='')
        state,alert=h.evaluer({},[nouveau],{'controle':r},{},state,3200,v)
        self.assertTrue(state['collectes']['controle']['awaiting_root_validation'])
        self.assertIn('deadline-depassee',codes(alert))
        self.assertIn('resultat-non-arbitre',codes(alert))
        # Sans vieux résultat, la garde ACK originale reste active au même seuil.
        self.assertIn('ack-absent',codes(h.evaluer({},[nouveau],{},{},{},3200,v)[1]))

    def test_closed_refuse_sans_preuves_et_inventaire(self):
        run={'status':'closed'};t=tache(status='pass',collected_at=1500,disposition_reason='root validé')
        self.assertIn('fermeture-non-validee',codes(h.evaluer(run,[t],{},{},{},2100)[1]))
        run['final_inventory']={'remaining':[],'_root_validation_ok':True,'source_epoch':'sha-controle','count':0}
        self.assertFalse(h.evaluer(run,[t],{'controle':resultat()},{},{},2100)[0]['closed'])
        run['final_inventory']['count']=1
        v={'controle':{'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(resultat()),'verdict':'pass'}}
        state,alert=h.evaluer(run,[t],{'controle':resultat()},{},{},2100,v)
        self.assertTrue(state['closed']);self.assertEqual(alert,[])

    def test_ready_est_evalue_par_agent_et_non_globalement(self):
        ready=tache(task_id='suite',status='ready',agent_target='psycho-2',depends_on=['controle'])
        avant=tache(status='pass',collected_at=1100,disposition_reason='root validé')
        v={'controle':{'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(resultat()),'verdict':'pass'}}
        self.assertIn('reprise-root-requise',codes(h.evaluer({},[avant,ready],{'controle':resultat()},{},{},1200,v)[1]))
        actif=tache(task_id='root-vivant',dispatched_at=1700,acknowledged_at=1700)
        self.assertIn('reprise-root-requise',codes(h.evaluer({},[avant,ready,actif],{'controle':resultat()},{},{},1800,v)[1]))
        actif_psycho2=tache(task_id='psycho-2-vivant',agent_target='psycho-2',dispatched_at=1700,acknowledged_at=1700)
        self.assertNotIn('reprise-root-requise',codes(h.evaluer({},[avant,ready,actif_psycho2],{'controle':resultat()},{},{},1800,v)[1]))
        sanspreuve=tache(status='pass',collected_at=1100)
        self.assertNotIn('reprise-root-requise',codes(h.evaluer({},[sanspreuve,ready],{'controle':resultat()},{},{},1800,v)[1]))

    def test_transport_mock_exact_root_deduplique_et_dryrun(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd=Path(tmp);canon=CanonSimule();sender=Mock()
            canon.write_json(rd/'run.json',{'run_id':'fixture'})
            canon.write_json(rd/'tasks'/'controle.json',tache(status='dispatched',acknowledged_at=''))
            h.passage(rd,canon,maintenant=1599,transport=sender);sender.assert_not_called()
            h.passage(rd,canon,maintenant=1600,transport=sender);self.assertEqual(sender.call_count,1)
            self.assertEqual(sender.call_args.args[0],h.ROOT_UUID)
            h.passage(rd,canon,maintenant=1700,transport=sender);self.assertEqual(sender.call_count,1)
            h.passage(rd,canon,maintenant=2200,dry_run=True,transport=sender);self.assertEqual(sender.call_count,1)
            h.passage(rd,canon,maintenant=2200,transport=sender);self.assertGreater(sender.call_count,1)

    def test_contreexemple_pass_sans_hash_et_review_non_solde_ne_ferment_pas(self):
        r=resultat('review');t=tache(status='review',collected_at=1500,disposition_reason='root')
        v={'controle':{'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(r),'verdict':'pass'}}
        run={'status':'closed','final_inventory':{'remaining':[],'_root_validation_ok':True,'source_epoch':'s','count':1}}
        self.assertFalse(h.evaluer(run,[t],{'controle':r},{},{},2100,v)[0]['closed'])
        t['status']='pass';self.assertTrue(h.evaluer(run,[t],{'controle':r},{},{},2100,v)[0]['closed'])
        r['message']='modifié après validation'
        self.assertFalse(h.evaluer(run,[t],{'controle':r},{},{},2100,v)[0]['closed'])

    def test_contreexemple_closed_dependance_absente_et_superseded(self):
        r=resultat('review');t=tache(status='pass',collected_at=1500,disposition_reason='root',depends_on=['absente'])
        v={'controle':{'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(r),'verdict':'pass'}}
        run={'status':'closed','final_inventory':{'remaining':[],'_root_validation_ok':True,'source_epoch':'s','count':1}}
        self.assertFalse(h.evaluer(run,[t],{'controle':r},{},{},2100,v)[0]['closed'])
        t['depends_on']=[]
        vieux=tache(task_id='ancien',status='superseded',disposition_reason='remplacé',superseded_by='controle')
        self.assertTrue(h.evaluer(run,[t,vieux],{'controle':r},{},{},2100,v)[0]['closed'])
        vieux['superseded_by']='absente'
        self.assertFalse(h.evaluer(run,[t,vieux],{'controle':r},{},{},2100,v)[0]['closed'])

    def test_contreexemple_deadline_json_invalide_tardif_et_attempt(self):
        t=tache(timeout_sec=800,status='review',collected_at=1900)
        mauvais=[{},dict(resultat('review'),created_at=1900,worker=h.ROOT_UUID),
            dict(resultat('review'),created_at=1500,worker=h.ROOT_UUID,attempt_id='autre')]
        for r in mauvais:
            self.assertIn('deadline-depassee',codes(h.evaluer({},[t],{'controle':r},{},{},2000)[1]))
        t['collected_at']=1550
        r=dict(resultat('review'),created_at=1500,worker=h.ROOT_UUID,attempt_id='1')
        self.assertNotIn('deadline-depassee',codes(h.evaluer({},[t],{'controle':r},{},{},2000)[1]))

    def test_contreexemple_ready_cible_apres_worker_review_valide(self):
        r=resultat('review');avant=tache(status='pass',collected_at=1100,disposition_reason='root')
        suite=tache(task_id='suite',status='pending',agent_target=h.ROOT_UUID,depends_on=['controle'])
        v={'controle':{'task_id':'controle','validatorUUID':h.ROOT_UUID,'hashresult':h.empreinte(r),'verdict':'pass'}}
        self.assertIn('reprise-root-requise',codes(h.evaluer({},[avant,suite],{'controle':r},{},{},1200,v)[1]))

    def test_resultat_octets_et_hash_single_lecture(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd=Path(tmp);canon=CanonSimule()
            canon.write_json(rd/'run.json',{'run_id':'fixture'})
            canon.write_json(rd/'tasks'/'controle.json',tache(status='review',collected_at=2000))
            canon.write_json(rd/'results'/'controle.result.json',resultat('review'))
            cible=rd/'results'/'controle.result.json';original=Path.read_bytes;lus=[]
            def lecture(path):
                if path==cible:lus.append(path)
                return original(path)
            with patch.object(Path,'read_bytes',lecture):
                h.passage(rd,canon,maintenant=2001,dry_run=True)
            self.assertEqual(lus,[cible])

    def test_transport_utilise_moteur_commun_avec_recu_idempotent(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd=Path(tmp);canon=CanonSimule()
            canon.write_json(rd/'run.json',{'run_id':'fixture'})
            canon.write_json(rd/'tasks'/'controle.json',tache(status='dispatched',acknowledged_at=''))
            with patch.object(CANON,'bridget_agents',return_value=[{'agent_id':h.ROOT_UUID,'state':'busy'}]), \
                 patch.object(CANON,'send_bridget_message') as appel:
                h.passage(rd,canon,maintenant=1600)
            self.assertEqual(appel.call_args.args[0],h.ROOT_UUID)
            self.assertIn('issuer_scope',appel.call_args.kwargs['replay'])

    def test_collecte_locale_ignore_progress_dans_tasks_et_nenvoie_pas_pass_frais(self):
        with tempfile.TemporaryDirectory() as tmp:
            rd=Path(tmp);canon=CanonSimule();sender=Mock()
            canon.write_json(rd/'run.json',{})
            canon.write_json(rd/'run.json',{'run_id':'fixture'})
            t=tache(status='review',collected_at=2000)
            canon.write_json(rd/'tasks'/'controle.json',t)
            canon.write_json(rd/'tasks'/'controle.progress.json',{'task_id':'faux','status':'running'})
            canon.write_json(rd/'results'/'controle.result.json',resultat())
            h.passage(rd,canon,maintenant=2001,transport=sender);sender.assert_not_called()
            self.assertEqual(canon.load_json(rd/'tasks'/'controle.json'),t)
            self.assertEqual(list(canon.load_json(rd/'psychologie-heartbeat.state.json')['collectes']),['controle'])

if __name__=='__main__':unittest.main()
