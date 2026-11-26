use super::*;

#[test]
fn accepts_only_matching_service_announcements_with_an_endpoint() {
    let service = 400;
    let mut packet = lookup(NEW_SERVER, service);
    assert!(service_endpoint(&packet, NEW_SERVER, service).is_none());
    packet[12..16].copy_from_slice(&7u32.to_le_bytes());
    packet[16..20].copy_from_slice(&41u32.to_le_bytes());

    assert_eq!(
        service_endpoint(&packet, NEW_SERVER, service),
        Some(QrtrAddress::new(7, 41))
    );
    assert!(service_endpoint(&packet, NEW_SERVER, service + 1).is_none());
    assert!(service_endpoint(&packet[..19], NEW_SERVER, service).is_none());
    assert!(removed_service(&packet, service).is_none());
    packet[..4].copy_from_slice(&DEL_SERVER.to_le_bytes());
    assert_eq!(
        removed_service(&packet, service),
        Some(QrtrAddress::new(7, 41))
    );
    assert!(removed_service(&packet, service + 1).is_none());
    assert!(service_endpoint(&packet, NEW_SERVER, service).is_none());
}
