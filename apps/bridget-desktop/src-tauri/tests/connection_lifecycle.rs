use bridget_desktop::connection::{
    ConnectionError, RelayProbe, RemoteTransport, connect_remote, transition,
};
use bridget_desktop::profile::{
    ConnectionProfile, ConnectionSession, ConnectionState, ProfileCapability, RelayEndpoint,
    SshIdentityRef,
};

struct AvailableRelay;

impl RelayProbe for AvailableRelay {
    fn check(&mut self, _: u16, _: &RelayEndpoint) -> Result<(), ConnectionError> {
        Ok(())
    }
}

struct ManagedTunnel;

impl RemoteTransport for ManagedTunnel {
    fn discover_endpoint(
        &mut self,
        _: &ConnectionProfile,
    ) -> Result<RelayEndpoint, ConnectionError> {
        RelayEndpoint::new(17888, "fixture-first".into())
            .map_err(|_| ConnectionError::EndpointInvalid)
    }

    fn open_tunnel(&mut self, _: &ConnectionProfile, _: u16) -> Result<u16, ConnectionError> {
        Ok(39001)
    }

    fn close(&mut self) {}
}

#[test]
fn une_session_fermee_ne_redevient_pas_active_et_l_autre_reste_intacte() {
    let profile = ConnectionProfile::Ssh {
        id: "first".into(),
        label: "Premier".into(),
        host: "cartae.app".into(),
        port: 2222,
        user: "moi".into(),
        identity: SshIdentityRef::Agent,
        host_fingerprint: Some("SHA256:abcdefghijklmnopqrstuvwxyz123456".into()),
        capabilities: vec![ProfileCapability::Ui],
    };
    let mut transport = ManagedTunnel;
    let mut probe = AvailableRelay;
    let mut first = connect_remote(&profile, &mut transport, &mut probe).expect("connected");
    transition(&mut first, ConnectionState::Closed).expect("close");
    assert!(transition(&mut first, ConnectionState::Reconnecting).is_err());

    let second = ConnectionSession::disconnected("second");
    assert_eq!(second.state, ConnectionState::Disconnected);
}
