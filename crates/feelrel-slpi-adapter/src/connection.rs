use std::io;
use std::time::Duration;

use prost::Message;
use tokio::time::timeout;

use crate::qmi::QmiPacket;
use crate::qrtr::{QrtrAddress, QrtrSocket, discover_service, removed_service};
use crate::sns::{EventBatch, Suid, lookup_packet, lookup_result};

const SNS_SERVICE: u32 = 400;
pub(super) const MAX_QMI_DATAGRAM: usize = u16::MAX as usize + 7;

pub struct SlpiConnection {
    socket: QrtrSocket,
    endpoint: QrtrAddress,
    control: QrtrAddress,
    next_transaction: u16,
}

impl SlpiConnection {
    pub async fn connect(discovery_timeout: Duration) -> io::Result<Self> {
        let socket = QrtrSocket::open()?;
        let control = QrtrAddress::control(socket.local_address()?.node);
        let endpoint = timeout(
            discovery_timeout,
            discover_service(&socket, control, SNS_SERVICE),
        )
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "SLPI SNS service not found"))??;
        Ok(Self {
            socket,
            endpoint,
            control,
            next_transaction: 1,
        })
    }

    pub async fn discover_channel(&mut self, channel: &str, wait: Duration) -> io::Result<Suid> {
        timeout(wait, async {
            let transaction = self.send_request(&lookup_packet(channel)).await?;
            let mut buffer = vec![0; MAX_QMI_DATAGRAM];
            loop {
                let received = self.receive(&mut buffer).await?;
                let packet = QmiPacket::decode(&buffer[..received])?;
                if let Some(payload) = packet.sns_payload(Some(transaction))?
                    && let Some(sensor) = lookup_result(payload, channel)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
                {
                    return Ok(sensor);
                }
            }
        })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                format!("SNS channel {channel:?} did not respond"),
            )
        })?
    }

    pub(super) async fn send_request(&mut self, payload: &[u8]) -> io::Result<u16> {
        let transaction = self.next_transaction;
        self.next_transaction = transaction.wrapping_add(1).max(1);
        let packet = QmiPacket::sns_request(transaction, payload)?;
        self.send(&packet).await?;
        Ok(transaction)
    }

    pub(super) async fn receive_batch(
        &mut self,
        buffer: &mut [u8],
    ) -> io::Result<Option<EventBatch>> {
        let received = self.receive(buffer).await?;
        QmiPacket::decode(&buffer[..received])?
            .sns_payload(None)?
            .map(|payload| {
                EventBatch::decode(payload)
                    .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))
            })
            .transpose()
    }

    pub async fn send(&self, datagram: &[u8]) -> io::Result<()> {
        self.socket.send_to(self.endpoint, datagram).await
    }

    pub async fn receive(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        loop {
            let (source, received) = self.socket.recv_from(buffer).await?;
            if source == self.control
                && removed_service(&buffer[..received], SNS_SERVICE) == Some(self.endpoint)
            {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionReset,
                    "SLPI SNS service disappeared",
                ));
            }
            if source == self.endpoint {
                return Ok(received);
            }
        }
    }
}
