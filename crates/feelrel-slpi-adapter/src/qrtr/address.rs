pub(super) const CONTROL_PORT: u32 = 0xffff_fffe;
pub(super) const AF_QIPCRTR: libc::c_int = 42;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct QrtrAddress {
    pub family: libc::sa_family_t,
    pub node: u32,
    pub port: u32,
}

impl QrtrAddress {
    pub fn control(node: u32) -> Self {
        Self::new(node, CONTROL_PORT)
    }

    pub fn new(node: u32, port: u32) -> Self {
        Self {
            family: AF_QIPCRTR as libc::sa_family_t,
            node,
            port,
        }
    }
}
