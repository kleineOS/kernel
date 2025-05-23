set export := true

@default: run-dbg

@run-dbg *FLAGS:
    cargo build
    just runner target/riscv64-bare/debug/kernel {{ FLAGS }}

# default runner for cargo, not meant to be used directly
[private]
@runner kernel *FLAGS: create-disk
    .cargo/runner.sh {{ kernel }} {{ FLAGS }}

# create a 64MiB fat32 disk image
create-disk: build-tools
    #!/usr/bin/env bash
    # I will not use global variables here, dont want to accidently dd /dev/sda
    if [[ ! -f disk.img ]]; then
        dd if=/dev/zero of=disk.img bs=1M count=64
        mkfs.fat -F32 disk.img
        # copying this for testing reasons
        mcopy -i disk.img build.rs ::
        mcopy -i disk.img linker.ld ::
    fi

# view the contents of the created disk
@ls-disk:
    mdir -i disk.img ::

# build u-boot
[working-directory: "target/uboot"]
build-uboot: build-tools-full (build-dir "uboot")
    #!/usr/bin/env bash
    if [[ ! -f u-boot-spl.bin || ! -f u-boot.itb || ! -f u-boot ]]; then
        test -f $UBOOT_TAR || wget $UBOOT_URL
        test -d $UBOOT_DIR || tar xvf $UBOOT_TAR

        cd $UBOOT_DIR
        make qemu-riscv64_spl_defconfig
        make -j$(nproc)

        cp spl/u-boot-spl.bin ..
        cp u-boot.itb ..
        cp u-boot ..
    fi

[working-directory: "target/uboot"]
clean-uboot:
    rm u-boot-spl.bin u-boot.itb

# depend on a sub-directory of the build folder
[private]
@build-dir DIR:
    mkdir -p target/{{DIR}}

[private]
@build-tools-full: build-tools
    which wget tar nproc dd mkfs.fat mcopy swig > /dev/null

[private]
@build-tools:
    which $QEMU $OBJDUMP $OBJDUMP > /dev/null

# tools we use (can differ on other distros)
QEMU := env("QEMU", "qemu-system-riscv64")
DEBUGGER := env("DEBUGGER", "rust-gdb")
OBJDUMP := env("OBJDUMP", "riscv64-linux-gnu-objdump")
# VM config
CORE_COUNT := "4"
MEM_SIZE := "256M"
MACHINE := "virt,aclint=on,aia=aplic-imsic,accel=tcg"

# setting a different disk will NOT actually change the target disk. I have
# hardcoded "disk.img" because I do not want to deal with accidental dd
# (destroy disk) operations. HOWEVER, you can do `DISK= cargo run` to disable
# the mounting of this disk to the VM
DISK := env("DISK", "disk.img")

UBOOT_URL := "https://ftp.denx.de/pub/u-boot/u-boot-2025.04.tar.bz2"
UBOOT_TAR := "u-boot-2025.04.tar.bz2"
UBOOT_DIR := "u-boot-2025.04"
OPENSBI := "/usr/share/qemu/opensbi-riscv64-generic-fw_dynamic.bin"
CROSS_COMPILE := "riscv64-linux-gnu-"
