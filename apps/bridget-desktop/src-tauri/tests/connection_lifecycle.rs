use bridget_desktop::connection::{ConnectionError, RelayProbe, connect_local, transition};
use bridget_desktop::profile::{
    ConnectionProfile, ConnectionSession, ConnectionState, ProfileCapability, RelayEndpoint,
};

struct AvailableRelay;

impl RelayProbe for AvailableRelay {
    fn check(&mut self, _: u16, _: &RelayEndpoint) -> Result<(), ConnectionError> {
        Ok(())
    }
}

#[test]
fn une_session_fermee_ne_redevient_pas_active_et_l_autre_reste_intacte() {
    let profile = ConnectionProfile::Local {
        id: "first".into(),
        label: "Premier".into(),
        relay_port: 17888,
        capabilities: vec![ProfileCapability::Ui],
    };
    let mut probe = AvailableRelay;
    let mut first = connect_local(
        &profile,
        RelayEndpoint::new(17888, "fixture-first".into()).expect("endpoint"),
        &mut probe,
    )
    .expect("connected");
    transition(&mut first, ConnectionState::Closed).expect("close");
    assert!(transition(&mut first, ConnectionState::Reconnecting).is_err());

    let second = ConnectionSession::disconnected("second");
    assert_eq!(second.state, ConnectionState::Disconnected);
}
