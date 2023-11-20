#include <linux/init.h>
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/device.h>
#include <linux/fs.h>
#include <linux/uaccess.h>
#include <linux/mutex.h>

#define DEVNAME		"hello"
#define BUFSIZE		4096

static int param = 0;
module_param(param, int, S_IRUGO);

static int major = 0;
static struct class *dev_class = NULL;
static struct device *dev_device = NULL;

static DEFINE_MUTEX(dev_mutex);

static char dev_buf[BUFSIZE] = { 0 };

static int dev_open(struct inode *inode, struct file *filp)
{
	printk(KERN_INFO "Open\n");
	return 0;
}

static int dev_close(struct inode *inode, struct file *filp)
{
	printk(KERN_INFO "Close\n");
	return 0;
}

static ssize_t dev_read(struct file *filp, char __user *buf,
			size_t len, loff_t *off)
{
	int ret;

	if (mutex_lock_interruptible(&dev_mutex))
		return -EINTR;

	if (len + *off >= BUFSIZE)
		len = BUFSIZE - *off;

	ret = copy_to_user(buf, dev_buf + *off, len);
	*off += len - ret;

	mutex_unlock(&dev_mutex);

	return len - ret;
}

static ssize_t dev_write(struct file *filp, const char __user *buf,
			 size_t len, loff_t *off)
{
	int ret;

	if (mutex_lock_interruptible(&dev_mutex))
		return -EINTR;

	if (len + *off >= BUFSIZE)
		len = BUFSIZE - *off;

	ret = copy_from_user(dev_buf + *off, buf, len);
	*off += len - ret;

	mutex_unlock(&dev_mutex);

	return len - ret;
}

static struct file_operations fops = {
	.open    = dev_open,
	.release = dev_close,
	.read    = dev_read,
	.write   = dev_write,
};

static int __init hellodev_init(void)
{
	int ret;

	printk(KERN_INFO "Loading with param %d\n", param);

	major = register_chrdev(major, DEVNAME, &fops);
	if (major < 0) {
		ret = major;
		goto exit;
	}

	dev_class = class_create(THIS_MODULE, DEVNAME);
	if (IS_ERR(dev_class)) {
		ret = PTR_ERR(dev_class);
		goto err_class_create;
	}

	dev_device = device_create(dev_class, NULL, MKDEV(major, param),
				   NULL, DEVNAME "%d", param);
	if (IS_ERR(dev_device)) {
		ret = PTR_ERR(dev_device);
		goto err_device_create;
	}

	return 0;

err_device_create:
	class_unregister(dev_class);
	class_destroy(dev_class);
err_class_create:
	unregister_chrdev(major, DEVNAME);
exit:
	return ret;
}

static void __exit hellodev_exit(void)
{
	printk(KERN_INFO "Unloading\n");

	device_destroy(dev_class, MKDEV(major, param));
	class_unregister(dev_class);
	class_destroy(dev_class);
	unregister_chrdev(major, DEVNAME);
}

module_init(hellodev_init);
module_exit(hellodev_exit);

MODULE_LICENSE("GPL");
