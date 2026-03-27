use super::*;
use crate::wire::WireSession;

pub(crate) struct WireObjectGraph {
    pub(crate) patch_object: Value,
    pub(crate) patch_id: String,
    pub(crate) revision_object: Value,
    pub(crate) revision_id: String,
}

pub(crate) fn registered_session(signing_key: &SigningKey, sender: &str) -> WireSession {
    let sender_key = sender_public_key(signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer(sender, &sender_key)
        .expect("known peer should register");
    session
}

pub(crate) fn patch_revision_graph(
    signing_key: &SigningKey,
    sender: &str,
    base_revision: &str,
) -> WireObjectGraph {
    let patch_object = signed_patch_object_message(signing_key, sender, base_revision);
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("signed patch OBJECT should include object_id")
        .to_owned();
    let revision_object =
        signed_revision_object_message(signing_key, sender, &[], &[patch_id.as_str()]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed revision OBJECT should include object_id")
        .to_owned();

    WireObjectGraph {
        patch_object,
        patch_id,
        revision_object,
        revision_id,
    }
}
