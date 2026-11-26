use crate::StreamProfile;

#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MeasurementProfile {
    pub illuminance: Option<String>,
    pub color: Option<StreamProfile>,
    pub gravity: Option<StreamProfile>,
    pub rotation: Option<StreamProfile>,
    pub game_rotation: Option<StreamProfile>,
    pub geomagnetic_rotation: Option<StreamProfile>,
    pub orientation: Option<String>,
    pub pedometer: Option<String>,
    pub significant_motion: Option<String>,
    pub psmd: Option<String>,
}
