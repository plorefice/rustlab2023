ifneq ($(KERNELRELEASE),)
# kbuild part of makefile
obj-m  := hello_c.o

else
# normal makefile
KDIR ?= ../../linux

default:
	$(MAKE) -C $(KDIR) M=$$PWD LLVM=1
	$(MAKE) -C $(KDIR) M=$$PWD LLVM=1 INSTALL_MOD_PATH=../initramfs modules_install

endif

