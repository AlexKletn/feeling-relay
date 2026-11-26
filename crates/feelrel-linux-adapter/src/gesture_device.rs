use std::io;

use evdev::{AttributeSet, BusType, InputId, MiscCode, MiscEvent, uinput::VirtualDevice};
use feelrel_ports::{Gesture, Motion};

use crate::MotionCodes;

use super::device_name::validate_device_name;

pub struct GestureDevice {
    device: VirtualDevice,
    wake: i32,
    tilt: i32,
    motion: Option<MotionCodes>,
}

impl GestureDevice {
    pub fn create(name: &str, wake: i32, tilt: i32) -> io::Result<Self> {
        Self::create_with_motion(name, wake, tilt, None)
    }

    pub fn create_with_motion(
        name: &str,
        wake: i32,
        tilt: i32,
        motion: Option<MotionCodes>,
    ) -> io::Result<Self> {
        validate_device_name(name)?;
        let mut values = vec![wake, tilt];
        if let Some(motion) = motion {
            values.extend([motion.significant, motion.moving, motion.stationary]);
        }
        values.sort_unstable();
        if values[0] <= 0 || values.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "event codes must be distinct positive integers",
            ));
        }
        let mut codes = AttributeSet::new();
        codes.insert(MiscCode::MSC_GESTURE);
        let device = VirtualDevice::builder()?
            .name(name)
            .input_id(InputId::new(BusType::BUS_VIRTUAL, 0, 0, 0))
            .with_msc(&codes)?
            .build()?;
        Ok(Self {
            device,
            wake,
            tilt,
            motion,
        })
    }

    pub fn publish(&mut self, gesture: Gesture) -> io::Result<()> {
        let value = match gesture {
            Gesture::Wake => self.wake,
            Gesture::Tilt => self.tilt,
        };
        self.emit(value)
    }

    pub fn publish_motion(&mut self, motion: Motion) -> io::Result<()> {
        if let Some(codes) = self.motion {
            let value = match motion {
                Motion::Significant => codes.significant,
                Motion::Moving => codes.moving,
                Motion::Stationary => codes.stationary,
            };
            self.emit(value)?;
        }
        Ok(())
    }

    fn emit(&mut self, value: i32) -> io::Result<()> {
        self.device
            .emit(&[*MiscEvent::new(MiscCode::MSC_GESTURE, value)])
    }
}
