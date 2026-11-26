#![cfg(target_os = "linux")]

mod adapter;
mod connection;
mod event_queue;
mod measurement_kind;
mod measurement_profile;
mod measurement_subscription;
mod measurements;
mod motion_subscription;
mod profile;
mod qmi;
mod qrtr;
mod report_profile;
mod sensors;
mod sns;
mod stream_profile;

pub use adapter::SlpiAdapter;
pub use connection::SlpiConnection;
pub use measurement_profile::MeasurementProfile;
pub use profile::SlpiProfile;
pub use report_profile::ReportProfile;
pub use sns::Suid;
pub use stream_profile::StreamProfile;
