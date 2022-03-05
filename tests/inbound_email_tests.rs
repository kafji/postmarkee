use postmarkee::InboundEmail;

#[test]
fn test_deserialize() {
    let serialized = include_str!("./postmark_inbound_payload_example.json");

    let email: InboundEmail = serde_json::from_str(serialized).unwrap();

    insta::assert_debug_snapshot!(email);
}

#[test]
fn test_serialize() {
    let email: InboundEmail = {
        let serialized = include_str!("./postmark_inbound_payload_example.json");
        serde_json::from_str(serialized).unwrap()
    };

    let serialized = serde_json::to_string_pretty(&email).unwrap();

    insta::assert_display_snapshot!(serialized);
}
