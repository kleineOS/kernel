set export

DISK := "disk.img"
# tools we use (can differ on other distros)
QEMU := "qemu-system-riscv64"
DEBUGGER := "rust-gdb"
OBJDUMP := "riscv64-linux-gnu-objdump"
# VM config
CORE_COUNT := "4"
MEM_SIZE := "256M"
MACHINE := "virt,aclint=on,aia=aplic-imsic"

@default: run-dbg

@run-dbg *FLAGS:
    cargo build
    just runner target/riscv64-bare/debug/kernel {{ FLAGS }}

# default runner for cargo, not meant to be used directly
[private]
@runner kernel *FLAGS: create-disk
    .cargo/runner.sh {{ kernel }} {{ FLAGS }}

# create a 64MiB fat32 disk image
@create-disk:
    #!/usr/bin/env bash
    if [[ ! -f "$DISK" ]]; then
        dd if=/dev/zero of=disk.img bs=1M count=64
        mkfs.fat -F32 disk.img
        # copying this for testing reasons
        mcopy -i disk.img build.rs ::
        mcopy -i disk.img linker.ld ::
    fi

# view the contents of the created disk
@ls-disk:
    mdir -i disk.img ::
