#!/usr/bin/env bash
set -euo pipefail

kernel=$1
flags="${2:-default}"

if [[ $flags = "uboot" ]]; then
    if [[ ! -f "target/uboot/u-boot" ]]; then
        echo "could not find u-boot binary, try running \`just build-uboot\`"
        exit 1
    fi
    kernel="target/uboot/u-boot"
    flags="default"
fi

QEMU_FLAGS=(
    -nographic

    -machine $MACHINE

    -bios   default
    -kernel $kernel

    -smp    $CORE_COUNT
    -m      $MEM_SIZE

    -chardev stdio,id=char0,mux=on,logfile=target/serial.log,signal=on
    -serial chardev:char0
    -mon    chardev=char0
)

if [[ ${DISK:-"unset"} != "unset" ]]; then
    QEMU_FLAGS+=(-drive "file=$DISK,if=virtio,format=raw")
fi

case $flags in
    "default")
        $QEMU "${QEMU_FLAGS[@]}"
        ;;
    "debug")
        $QEMU "${QEMU_FLAGS[@]}" -s -S
        ;;
    "gdb")
        $DEBUGGER -ex "symbol-file $kernel"
        ;;
    "objdump")
        $OBJDUMP -d "$kernel"
        ;;
    *)
        echo "undefined flag"
        exit 1
        ;;
esac
