use std::io;

const HEADER_SIZE: usize = 7;
const CLIENT_REQUEST: u16 = 0x0020;
const REPORT: u16 = 0x0021;
const REQUEST: u8 = 0;
const RESPONSE: u8 = 2;
const INDICATION: u8 = 4;

pub(crate) struct QmiPacket<'a> {
    kind: u8,
    transaction: u16,
    message: u16,
    fields: &'a [u8],
}

impl<'a> QmiPacket<'a> {
    pub fn sns_request(transaction: u16, payload: &[u8]) -> io::Result<Vec<u8>> {
        let body_size = u16::try_from(payload.len().saturating_add(5))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "SNS request too large"))?;
        let mut bytes = Vec::with_capacity(HEADER_SIZE + usize::from(body_size));
        bytes.push(REQUEST);
        bytes.extend_from_slice(&transaction.to_le_bytes());
        bytes.extend_from_slice(&CLIENT_REQUEST.to_le_bytes());
        bytes.extend_from_slice(&body_size.to_le_bytes());
        bytes.push(1);
        bytes.extend_from_slice(&(body_size - 3).to_le_bytes());
        bytes.extend_from_slice(&(body_size - 5).to_le_bytes());
        bytes.extend_from_slice(payload);
        Ok(bytes)
    }

    pub fn decode(packet: &'a [u8]) -> io::Result<Self> {
        let [
            kind,
            tx_low,
            tx_high,
            msg_low,
            msg_high,
            size_low,
            size_high,
            fields @ ..,
        ] = packet
        else {
            return Err(invalid_packet());
        };
        if usize::from(u16::from_le_bytes([*size_low, *size_high])) != fields.len() {
            return Err(invalid_packet());
        }
        Ok(Self {
            kind: *kind,
            transaction: u16::from_le_bytes([*tx_low, *tx_high]),
            message: u16::from_le_bytes([*msg_low, *msg_high]),
            fields,
        })
    }

    pub fn sns_payload(&self, transaction: Option<u16>) -> io::Result<Option<&'a [u8]>> {
        let field = match (self.kind, self.message) {
            (INDICATION, REPORT) => self.field(2)?,
            (RESPONSE, CLIENT_REQUEST)
                if transaction.is_none_or(|expected| self.transaction == expected) =>
            {
                if let Some([status_low, status_high, error_low, error_high]) = self.field(2)?
                    && u16::from_le_bytes([*status_low, *status_high]) != 0
                {
                    let code = u16::from_le_bytes([*error_low, *error_high]);
                    return Err(io::Error::other(format!(
                        "SNS request rejected by QMI (error {code})"
                    )));
                }
                self.field(1)?
            }
            _ => None,
        };
        let Some(field) = field else {
            return Ok(None);
        };
        let [low, high, payload @ ..] = field else {
            return Err(invalid_packet());
        };
        if usize::from(u16::from_le_bytes([*low, *high])) != payload.len() {
            return Err(invalid_packet());
        }
        Ok(Some(payload))
    }

    fn field(&self, wanted: u8) -> io::Result<Option<&'a [u8]>> {
        let mut fields = self.fields;
        while !fields.is_empty() {
            let [tag, low, high, rest @ ..] = fields else {
                return Err(invalid_packet());
            };
            let size = usize::from(u16::from_le_bytes([*low, *high]));
            let (value, remaining) = rest.split_at_checked(size).ok_or_else(invalid_packet)?;
            if *tag == wanted {
                return Ok(Some(value));
            }
            fields = remaining;
        }
        Ok(None)
    }
}

fn invalid_packet() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "malformed QMI packet")
}

#[cfg(test)]
#[path = "qmi.tests.rs"]
mod tests;
