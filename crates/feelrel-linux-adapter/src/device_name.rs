use std::io;

pub(super) fn validate_device_name(name: &str) -> io::Result<()> {
    if name.is_empty() || name.len() > 78 || name.as_bytes().contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid uinput device name",
        ));
    }
    Ok(())
}
