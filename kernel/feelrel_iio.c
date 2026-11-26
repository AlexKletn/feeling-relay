// SPDX-License-Identifier: GPL-2.0-only
#include <linux/fs.h>
#include <linux/iio/iio.h>
#include <linux/miscdevice.h>
#include <linux/module.h>
#include <linux/mutex.h>
#include <linux/unaligned.h>
#include <linux/uaccess.h>

enum { MAX_FIELDS = 8 };

struct feelrel_endpoint {
	const char *name;
	const struct iio_chan_spec *channels;
	int num_channels;
	int fields;
	struct iio_dev *device;
	struct miscdevice writer;
	struct mutex lock;
	s32 values[MAX_FIELDS];
	bool has_sample;
	bool writer_open;
};

static struct feelrel_endpoint *writer_endpoint(struct file *file)
{
	return container_of(file->private_data, struct feelrel_endpoint, writer);
}

static int open_writer(struct inode *inode, struct file *file)
{
	struct feelrel_endpoint *endpoint = writer_endpoint(file);
	int ret = 0;

	if (!(file->f_mode & FMODE_WRITE))
		return -EBADF;

	mutex_lock(&endpoint->lock);
	if (endpoint->writer_open)
		ret = -EBUSY;
	else
		endpoint->writer_open = true;
	mutex_unlock(&endpoint->lock);
	return ret;
}

static int close_writer(struct inode *inode, struct file *file)
{
	struct feelrel_endpoint *endpoint = writer_endpoint(file);

	mutex_lock(&endpoint->lock);
	endpoint->has_sample = false;
	endpoint->writer_open = false;
	mutex_unlock(&endpoint->lock);
	return 0;
}

static ssize_t write_sample(struct file *file, const char __user *buffer,
			    size_t length, loff_t *offset)
{
	struct feelrel_endpoint *endpoint = writer_endpoint(file);
	u8 frame[sizeof(endpoint->values)];
	int i;

	if (length != endpoint->fields * sizeof(endpoint->values[0]))
		return -EINVAL;
	if (copy_from_user(frame, buffer, length))
		return -EFAULT;

	mutex_lock(&endpoint->lock);
	for (i = 0; i < endpoint->fields; i++)
		endpoint->values[i] =
			(s32)get_unaligned_le32(frame + i * sizeof(endpoint->values[0]));
	endpoint->has_sample = true;
	mutex_unlock(&endpoint->lock);
	return length;
}

static const struct file_operations writer_ops = {
	.owner = THIS_MODULE,
	.open = open_writer,
	.release = close_writer,
	.write = write_sample,
};

static int read_sample(struct iio_dev *device, const struct iio_chan_spec *channel,
		       int max_len, int *values, int *len, long mask)
{
	struct feelrel_endpoint *endpoint = *(struct feelrel_endpoint **)iio_priv(device);
	int ret = -ENODATA;
	int sample;

	if (max_len < 4)
		return -EINVAL;

	if (mask == IIO_CHAN_INFO_SCALE) {
		values[0] = 0;
		*len = 2;
		if (channel->type == IIO_ANGL) {
			values[1] = 17453293;
			return IIO_VAL_INT_PLUS_NANO;
		}
		values[1] = channel->type == IIO_COLORTEMP ? 1000 : 1;
		return IIO_VAL_INT_PLUS_MICRO;
	}

	mutex_lock(&endpoint->lock);
	if (!endpoint->has_sample)
		goto unlock;

	sample = endpoint->values[channel->address];
	switch (mask) {
	case IIO_CHAN_INFO_RAW:
		if (channel->channel2 == IIO_MOD_QUATERNION) {
			memcpy(values, endpoint->values, 4 * sizeof(*values));
			*len = 4;
			ret = IIO_VAL_INT_MULTIPLE;
		} else if (channel->type != IIO_ANGL || sample >= 0) {
			values[0] = sample;
			*len = 1;
			ret = IIO_VAL_INT;
		}
		break;
	case IIO_CHAN_INFO_PROCESSED:
		if (channel->type == IIO_STEPS) {
			values[0] = sample;
			values[1] = 0;
			*len = 2;
			ret = IIO_VAL_INT_64;
		} else {
			values[0] = sample / 1000;
			values[1] = (sample % 1000) * 1000;
			*len = 2;
			ret = IIO_VAL_INT_PLUS_MICRO;
		}
		break;
	case IIO_CHAN_INFO_INT_TIME:
		sample = endpoint->values[6];
		values[0] = sample / 1000000;
		values[1] = sample % 1000000;
		*len = 2;
		ret = IIO_VAL_INT_PLUS_MICRO;
		break;
	case IIO_CHAN_INFO_HARDWAREGAIN:
		sample = endpoint->values[7];
		values[0] = sample / 1000;
		values[1] = (sample % 1000) * 1000;
		*len = 2;
		ret = IIO_VAL_INT_PLUS_MICRO;
		break;
	default:
		ret = -EINVAL;
	}
unlock:
	mutex_unlock(&endpoint->lock);
	return ret;
}

#define VECTOR_AXIS(kind, axis, index) {					\
	.type = kind,							\
	.modified = 1,							\
	.channel2 = IIO_MOD_ ## axis,					\
	.address = index,						\
	.info_mask_separate = BIT(IIO_CHAN_INFO_RAW),			\
	.info_mask_shared_by_type = BIT(IIO_CHAN_INFO_SCALE),		\
}

#define VECTOR_CHANNELS(name, kind)					\
static const struct iio_chan_spec name[] = {				\
	VECTOR_AXIS(kind, X, 0),						\
	VECTOR_AXIS(kind, Y, 1),						\
	VECTOR_AXIS(kind, Z, 2),						\
}

VECTOR_CHANNELS(acceleration, IIO_ACCEL);
VECTOR_CHANNELS(angular_velocity, IIO_ANGL_VEL);
VECTOR_CHANNELS(magnetic_field, IIO_MAGN);

static const struct iio_chan_spec gravity[] = {
	VECTOR_AXIS(IIO_GRAVITY, X, 0),
	VECTOR_AXIS(IIO_GRAVITY, Y, 1),
	VECTOR_AXIS(IIO_GRAVITY, Z, 2),
	VECTOR_AXIS(IIO_ACCEL, LINEAR_X, 3),
	VECTOR_AXIS(IIO_ACCEL, LINEAR_Y, 4),
	VECTOR_AXIS(IIO_ACCEL, LINEAR_Z, 5),
};

static const struct iio_chan_spec rotation[] = {
	VECTOR_AXIS(IIO_ROT, QUATERNION, 0),
};

static const struct iio_chan_spec orientation[] = {
	{
		.type = IIO_ANGL,
		.info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
	},
};

static const struct iio_chan_spec illuminance[] = {
	{ .type = IIO_LIGHT, .info_mask_separate = BIT(IIO_CHAN_INFO_PROCESSED) },
};

static const struct iio_chan_spec steps[] = {
	{ .type = IIO_STEPS, .info_mask_separate = BIT(IIO_CHAN_INFO_PROCESSED) },
};

#define COLOR_AXIS(color, index) {					\
	.type = IIO_INTENSITY, .modified = 1, .channel2 = IIO_MOD_LIGHT_ ## color, \
	.address = index, .info_mask_separate = BIT(IIO_CHAN_INFO_RAW),	\
	.info_mask_shared_by_type = BIT(IIO_CHAN_INFO_INT_TIME) |	\
				    BIT(IIO_CHAN_INFO_HARDWAREGAIN),	\
}

static const struct iio_chan_spec rgb[] = {
	{
		.type = IIO_COLORTEMP,
		.info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
	},
	COLOR_AXIS(CLEAR, 1),
	COLOR_AXIS(RED, 2),
	COLOR_AXIS(GREEN, 3),
	COLOR_AXIS(BLUE, 4),
	{
		.type = IIO_INTENSITY, .indexed = 1, .channel = 0, .address = 5,
		.info_mask_separate = BIT(IIO_CHAN_INFO_RAW),
	},
};

static int read_label(struct iio_dev *device, const struct iio_chan_spec *channel,
		      char *buffer)
{
	static const char * const labels[] = { "cct", "clear", "red", "green", "blue", "white" };

	return sysfs_emit(buffer, "%s\n", labels[channel->address]);
}

static const struct iio_info color_info = {
	.read_raw_multi = read_sample,
	.read_label = read_label,
};

static const struct iio_info sensor_info = {
	.read_raw_multi = read_sample,
};

#define ENDPOINT(device_name, writer_name, channel_array, frame_fields) { \
	.name = device_name,						\
	.channels = channel_array,					\
	.num_channels = ARRAY_SIZE(channel_array),			\
	.fields = frame_fields,					\
	.writer = {							\
		.minor = MISC_DYNAMIC_MINOR,				\
		.name = writer_name,					\
		.mode = 0200,						\
		.fops = &writer_ops,					\
	},								\
}

static struct feelrel_endpoint endpoints[] = {
	ENDPOINT("feelrel-accelerometer", "feelrel-iio-accel", acceleration, 3),
	ENDPOINT("feelrel-gyroscope", "feelrel-iio-gyro", angular_velocity, 3),
	ENDPOINT("feelrel-magnetometer", "feelrel-iio-magn", magnetic_field, 3),
	ENDPOINT("feelrel-ambient-light", "feelrel-iio-light", illuminance, 1),
	ENDPOINT("feelrel-rgb", "feelrel-iio-rgb", rgb, 8),
	ENDPOINT("feelrel-gravity", "feelrel-iio-gravity", gravity, 6),
	ENDPOINT("feelrel-rotation", "feelrel-iio-rotation", rotation, 4),
	ENDPOINT("feelrel-game-rotation", "feelrel-iio-game-rotation", rotation, 4),
	ENDPOINT("feelrel-geomagnetic-rotation", "feelrel-iio-geomagnetic-rotation", rotation, 4),
	ENDPOINT("feelrel-orientation", "feelrel-iio-orientation", orientation, 1),
	ENDPOINT("feelrel-pedometer", "feelrel-iio-steps", steps, 1),
};

static int register_endpoint(struct feelrel_endpoint *endpoint)
{
	struct iio_dev *device;
	int ret;

	mutex_init(&endpoint->lock);
	device = iio_device_alloc(NULL, sizeof(struct feelrel_endpoint *));
	if (!device)
		return -ENOMEM;

	*(struct feelrel_endpoint **)iio_priv(device) = endpoint;
	device->name = endpoint->name;
	device->modes = INDIO_DIRECT_MODE;
	device->channels = endpoint->channels;
	device->num_channels = endpoint->num_channels;
	device->info = endpoint->channels == rgb ? &color_info : &sensor_info;

	ret = iio_device_register(device);
	if (ret)
		goto free_device;

	ret = misc_register(&endpoint->writer);
	if (!ret) {
		endpoint->device = device;
		return 0;
	}

	iio_device_unregister(device);
free_device:
	iio_device_free(device);
	return ret;
}

static void unregister_endpoint(struct feelrel_endpoint *endpoint)
{
	misc_deregister(&endpoint->writer);
	iio_device_unregister(endpoint->device);
	iio_device_free(endpoint->device);
}

static int __init feelrel_iio_init(void)
{
	int i, ret;

	for (i = 0; i < ARRAY_SIZE(endpoints); i++) {
		ret = register_endpoint(&endpoints[i]);
		if (ret) {
			while (i > 0)
				unregister_endpoint(&endpoints[--i]);
			return ret;
		}
	}
	return 0;
}

static void __exit feelrel_iio_exit(void)
{
	int i;

	for (i = ARRAY_SIZE(endpoints); i > 0; i--)
		unregister_endpoint(&endpoints[i - 1]);
}

module_init(feelrel_iio_init);
module_exit(feelrel_iio_exit);

MODULE_DESCRIPTION("IIO sensors fed by feeling-relay");
MODULE_LICENSE("GPL");
