use super::*;

#[test]
fn wraps_requests_and_extracts_sns_reports() {
    let packet = QmiPacket::sns_request(2, b"abc").unwrap();
    assert_eq!(
        packet,
        [0, 2, 0, 0x20, 0, 8, 0, 1, 5, 0, 3, 0, b'a', b'b', b'c']
    );
    let indication = [4, 0, 0, 0x21, 0, 8, 0, 2, 5, 0, 3, 0, b'a', b'b', b'c'];
    let decoded = QmiPacket::decode(&indication).unwrap();
    assert_eq!(
        decoded.sns_payload(Some(2)).unwrap(),
        Some(b"abc".as_slice())
    );
    assert!(QmiPacket::decode(&indication[..14]).is_err());
}
