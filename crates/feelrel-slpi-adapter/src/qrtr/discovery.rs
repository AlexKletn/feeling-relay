use std::io;

use super::{QrtrAddress, QrtrSocket, address::CONTROL_PORT};

const NEW_LOOKUP: u32 = 10;
const NEW_SERVER: u32 = 4;
const DEL_SERVER: u32 = 5;

pub(crate) async fn discover_service(
    socket: &QrtrSocket,
    control: QrtrAddress,
    service: u32,
) -> io::Result<QrtrAddress> {
    socket
        .send_to(control, &lookup(NEW_LOOKUP, service))
        .await?;
    let mut buffer = [0; 20];
    loop {
        let (source, received) = socket.recv_from(&mut buffer).await?;
        if source == control
            && let Some(address) = service_endpoint(&buffer[..received], NEW_SERVER, service)
        {
            return Ok(address);
        }
    }
}

fn lookup(command: u32, service: u32) -> [u8; 20] {
    let mut packet = [0; 20];
    packet[..4].copy_from_slice(&command.to_le_bytes());
    packet[4..8].copy_from_slice(&service.to_le_bytes());
    packet
}

pub(crate) fn removed_service(packet: &[u8], service: u32) -> Option<QrtrAddress> {
    service_endpoint(packet, DEL_SERVER, service)
}

fn service_endpoint(packet: &[u8], expected_command: u32, service: u32) -> Option<QrtrAddress> {
    let [command, announced, _, node, port] = <&[u8; 20]>::try_from(packet)
        .ok()?
        .as_chunks::<4>()
        .0
        .try_into()
        .ok()?;
    let command = u32::from_le_bytes(command);
    let announced = u32::from_le_bytes(announced);
    let port = u32::from_le_bytes(port);
    (command == expected_command && announced == service && port != 0 && port != CONTROL_PORT)
        .then(|| QrtrAddress::new(u32::from_le_bytes(node), port))
}

#[cfg(test)]
#[path = "discovery.tests.rs"]
mod tests;
