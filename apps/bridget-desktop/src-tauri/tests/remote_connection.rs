use bridget_desktop::connection::{ConnectionError, RelayProbe, RemoteTransport, connect_remote};
use bridget_desktop::profile::{
    ConnectionProfile, ProfileCapability, RelayEndpoint, SshIdentityRef,
};
use std::cell::RefCell;

fn remote_profile() -> ConnectionProfile {
    ConnectionProfile::Ssh {
        id: "integration-remote".into(),
        label: "Relais factice".into(),
        host: "relay.example.test".into(),
        port: 2222,
        user: "operator".into(),
        identity: SshIdentityRef::Agent,
        host_fingerprint: Some("SHA256:abcdefghijklmnopqrstuvwxyz123456".into()),
        capabilities: vec![ProfileCapability::Ui],
    }
}

struct FakeTransport {
    calls: RefCell<Vec<&'static str>>,
}

impl RemoteTransport for FakeTransport {
    fn discover_endpoint(
        &mut self,
        _: &ConnectionProfile,
    ) -> Result<RelayEndpoint, ConnectionError> {
        self.calls.borrow_mut().push("discover");
        RelayEndpoint::new(17888, "test-relay-token".into())
            .map_err(|_| ConnectionError::EndpointInvalid)
    }

    fn open_tunnel(
        &mut self,
        _: &ConnectionProfile,
        remote_port: u16,
    ) -> Result<u16, ConnectionError> {
        assert_eq!(remote_port, 17888);
        self.calls.borrow_mut().push("tunnel");
        Ok(39174)
    }

    fn close(&mut self) {
        self.calls.borrow_mut().push("close");
    }
}

struct FakeRelay {
    calls: RefCell<Vec<&'static str>>,
    available: bool,
}

impl RelayProbe for FakeRelay {
    fn check(&mut self, local_port: u16, _: &RelayEndpoint) -> Result<(), ConnectionError> {
        assert_eq!(local_port, 39174);
        self.calls.borrow_mut().push("http");
        if self.available {
            Ok(())
        } else {
            Err(ConnectionError::RelayUnavailable)
        }
    }
}

#[test]
fn le_traj_distant_est_decouverte_tunnel_http_avant_connected() {
    let mut transport = FakeTransport {
        calls: RefCell::new(vec![]),
    };
    let mut relay = FakeRelay {
        calls: RefCell::new(vec![]),
        available: true,
    };
    let session = connect_remote(&remote_profile(), &mut transport, &mut relay).expect("session");
    assert_eq!(format!("{:?}", session.state), "Connected");
    assert_eq!(transport.calls.borrow().as_slice(), &["discover", "tunnel"]);
    assert_eq!(relay.calls.borrow().as_slice(), &["http"]);
}

#[test]
fn une_erreur_de_relais_ferme_le_tunnel_sans_divulguer_le_jeton() {
    let mut transport = FakeTransport {
        calls: RefCell::new(vec![]),
    };
    let mut relay = FakeRelay {
        calls: RefCell::new(vec![]),
        available: false,
    };
    let error =
        connect_remote(&remote_profile(), &mut transport, &mut relay).expect_err("échec relais");
    assert!(matches!(error, ConnectionError::RelayUnavailable));
    assert_eq!(
        transport.calls.borrow().as_slice(),
        &["discover", "tunnel", "close"]
    );
    assert!(!error.to_string().contains("test-relay-token"));
}
