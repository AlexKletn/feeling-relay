# feeling-relay

[Русская версия](README.ru.md) · [Contributing](CONTRIBUTING.md) · [GPL-2.0-only](LICENSE)

A Rust workspace for exposing the measurements and events provided by
Qualcomm's sensor coprocessor to Linux.

## Arch Linux package

Build the two packages from the current checkout.

```sh
tar --transform='s,^,feeling-relay-0.1.0/,' -czf feeling-relay-0.1.0.tar.gz \
    Cargo.toml Cargo.lock crates kernel systemd LICENSE README.md README.ru.md
makepkg -s
```

`feeling-relay` installs the daemon and systemd unit.
`feeling-relay-dkms` provides the optional IIO module source. DKMS needs the
headers for the target kernel. The package does not load the module.

Start `feeling-relay.service` once to create the commented profile at
`/etc/feeling-relay/feeling-relay.toml`. Adapt it for the device, set
`enabled = true`, then start or enable the service. The generated profile
is disabled and the daemon exits successfully until it is enabled.
Existing profiles without an `enabled` key stay enabled.

## Sensor ports

`feelrel-ports` defines accelerometer, gyroscope, magnetometer, cover and gesture ports. Angular velocity is in rad/s and magnetic field in µT.
Accelerometer readings are in metres per second squared in the sensor's
coordinate frame. The cover exposes its latest known state and a separate
stream of changes. Wake and tilt are discrete events.

Polling a port never waits for the next event. `Ok(None)` means there is
no event queued, or the adapter has yet to receive a cover state.
Timestamps are monotonic durations in the source's clock domain.
Readings from different clock domains require explicit synchronization.

## SLPI source

`SlpiAdapter` uses one QRTR socket for the enabled subscriptions. Supply
channel names, the accelerometer rate and the SLPI clock frequency in
`SlpiProfile`. Await `pump()` on a Tokio runtime with I/O and time enabled
and drain the corresponding sensor ports. If a consumer falls behind, its
bounded queue reports the number of dropped events on the next read.

## Linux input

`feelrel-linux-adapter` publishes cover state as `EV_SW / SW_LID`
and gestures as `EV_MSC / MSC_GESTURE`. Supply distinct positive integer
codes for wake and tilt and configure consumers to use those same codes.
Both devices require access to `/dev/uinput`. The kernel timestamps input
events.

## Running the daemon

Run `feeling-relay [profile.toml]` on Linux with access to QRTR and
`/dev/uinput`. The profile supplies the SLPI subscription settings and
the names and gesture codes of both virtual input devices.

The daemon reconnects after a QRTR service removal or a gap in
accelerometer samples. `sample_timeout_ms` is optional and defaults to
5000 ms. Set it above the expected sample interval.

## Linux IIO

Build the optional kernel module against the target kernel with
`make -C /path/to/kernel/build M="$PWD/kernel" modules`. It requires
`CONFIG_IIO`. Load `feelrel_iio` and set
`iio_accelerometer_device = "/dev/feelrel-iio-accel"` at the top level of
the daemon profile to enable accelerometer publication.

The IIO device `feelrel-accelerometer` exposes the latest sample as
`in_accel_{x,y,z}_raw` with the shared scale `0.000001` m/s² per unit.
Readings are unavailable until a producer sends a sample and become
unavailable again when the producer disconnects. Buffered capture and
sample timestamps are outside this direct-read interface.

Add optional `[slpi.gyroscope]` and `[slpi.magnetometer]` tables, each
with `channel` and `hz`. Set `iio_gyroscope_device = "/dev/feelrel-iio-gyro"`
and `iio_magnetometer_device = "/dev/feelrel-iio-magn"` at the top level
to enable direct IIO readings for those streams. IIO uses rad/s for
angular velocity and gauss for magnetic field.

The optional `[slpi.measurements]` section selects illuminance, RGB,
gravity, fused/game/geomagnetic rotation, device orientation, pedometer,
significant motion and PSMD channels. Continuous channels take a channel
name and sample rate. State and event channels take a channel name.
When the kernel module is loaded, set `iio_measurements_directory = "/dev"`
to expose the additional measurements and orientation. Motion events use
the custom `MSC_GESTURE` values declared in `[input.motion]`.

PSMD uses a QRTR connection for each mode because firmware reports moving
and stationary with the same empty payload. The two subscriptions are
renewed alongside the other channels when the daemon reconnects.

The required `[slpi.reports]` section describes the selected firmware's SNS
reports. Vector reports specify the message number, the protobuf field
and the zero-based order of values. Optional `scales` convert individual
values to the port's units. Cover, orientation and step reports specify
their integer fields and accepted values or message numbers. Gesture
message numbers and PSMD mode values also come from the profile.
