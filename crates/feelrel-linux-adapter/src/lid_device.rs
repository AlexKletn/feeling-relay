use std::io;

use evdev::{AttributeSet, BusType, InputId, SwitchCode, SwitchEvent, uinput::VirtualDevice};
use feelrel_ports::CoverState;

use super::device_name::validate_device_name;

pub struct LidDevice {
    device: VirtualDevice,
    published: Option<CoverState>,
}

impl LidDevice {
    pub fn create(name: &str) -> io::Result<Self> {
        validate_device_name(name)?;
        let mut switches = AttributeSet::new();
        switches.insert(SwitchCode::SW_LID);
        let device = VirtualDevice::builder()?
            .name(name)
            .input_id(InputId::new(BusType::BUS_VIRTUAL, 0, 0, 0))
            .with_switches(&switches)?
            .build()?;
        Ok(Self {
            device,
            published: None,
        })
    }

    pub fn publish(&mut self, state: CoverState) -> io::Result<()> {
        if self.published != Some(state) {
            let event =
                SwitchEvent::new(SwitchCode::SW_LID, i32::from(state == CoverState::Closed));
            self.device.emit(&[*event])?;
            self.published = Some(state);
        }
        Ok(())
    }
}
