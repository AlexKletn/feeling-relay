use std::io;

use prost::encoding::{self, DecodeContext, WireType};

pub(crate) fn float_values(payload: &[u8], field: u32) -> io::Result<Vec<f32>> {
    let mut values = Vec::new();
    visit(payload, field, |wire, bytes, context| {
        encoding::float::merge_repeated(wire, &mut values, bytes, context)
    })?;
    Ok(values)
}

pub(crate) fn integer(payload: &[u8], field: u32) -> io::Result<Option<u64>> {
    let mut value = None;
    visit(payload, field, |wire, bytes, context| {
        let mut word = 0;
        encoding::uint64::merge(wire, &mut word, bytes, context)?;
        value = Some(word);
        Ok(())
    })?;
    Ok(value)
}

fn visit(
    mut payload: &[u8],
    field: u32,
    mut read: impl FnMut(WireType, &mut &[u8], DecodeContext) -> Result<(), prost::DecodeError>,
) -> io::Result<()> {
    let decode = |source| io::Error::new(io::ErrorKind::InvalidData, source);
    while !payload.is_empty() {
        let (tag, wire) = encoding::decode_key(&mut payload).map_err(decode)?;
        let context = DecodeContext::default();
        if tag == field {
            read(wire, &mut payload, context).map_err(decode)?;
        } else {
            encoding::skip_field(wire, tag, &mut payload, context).map_err(decode)?;
        }
    }
    Ok(())
}
