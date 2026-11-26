use std::io;

use feelrel_linux_adapter::{GestureDevice, LidDevice, MotionCodes};
use feelrel_ports::{CoverPort, CoverState, GesturePort, MotionPort};

use crate::config::InputConfig;

pub(super) struct InputRelay<'a> {
    lid_name: &'a str,
    lid: Option<LidDevice>,
    gestures: GestureDevice,
}

impl<'a> InputRelay<'a> {
    pub fn open(config: &'a InputConfig) -> io::Result<Self> {
        let motion = config.motion.as_ref().map(|codes| MotionCodes {
            significant: codes.significant,
            moving: codes.moving,
            stationary: codes.stationary,
        });
        let gestures = GestureDevice::create_with_motion(
            &config.gesture_device,
            config.wake_code,
            config.tilt_code,
            motion,
        )?;
        Ok(Self {
            lid_name: &config.lid_device,
            lid: None,
            gestures,
        })
    }

    pub fn publish(
        &mut self,
        source: &mut (
                 impl CoverPort<Error = io::Error>
                 + GesturePort<Error = io::Error>
                 + MotionPort<Error = io::Error>
             ),
    ) -> io::Result<()> {
        while let Some(change) = source.next_cover_change()? {
            self.publish_cover(change.value)?;
        }
        if let Some(current) = source.cover_state()? {
            self.publish_cover(current.value)?;
        }
        while let Some(gesture) = source.next_gesture()? {
            self.gestures.publish(gesture.value)?;
        }
        while let Some(motion) = source.next_motion()? {
            self.gestures.publish_motion(motion.value)?;
        }
        Ok(())
    }

    fn publish_cover(&mut self, state: CoverState) -> io::Result<()> {
        if self.lid.is_none() {
            self.lid = Some(LidDevice::create(self.lid_name)?);
        }
        if let Some(lid) = &mut self.lid {
            lid.publish(state)?;
        }
        Ok(())
    }
}
