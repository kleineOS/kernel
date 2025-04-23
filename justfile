set export

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

# list all the options
@list:
    just --list

# default runner for cargo, not meant to be used directly
[private]
@runner kernel *FLAGS:
    .cargo/runner.sh {{ kernel }} {{ FLAGS }}

[working-directory: "target"]
@build-uboot:
    echo "$PWD"
