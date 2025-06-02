# KleineOS kernel

KleineOS (lit. Small Operating System) is a simple kernel that runs on the RISC-V ISA. It is written using Rust, leveraging increased memory safety along with superior design patterns to provide a safe and modular base for writing other applications on.

As it stands, kleineOS is just a hobby project. While I do have some long-term visions, I do not yet know how they should be manifest into the codebase.

## Goals of the project

I would consider this hobby project "done" when I have:
1. A Gemini (protocol not AI) server
2. A Gemini (protocol not AI) client

I would also like to make this happen, if I can pull it off:
1. A mechanism to reload the kernel without reloading the userspace

## Building

The core build system for kleineOS uses [`just`](https://github.com/casey/just), `bash` and `cargo`. You also need QEMU (with RISC-V support) to actually run the kernel.

For developing for kleineOS, I recommend the RISC-V GNU toolchain. It provides utilities with RISC-V support, such as `objdump`, `gdb`, `as`, etc.

Some systems have different binary names for some of the tools we rely on. You can modify the variables set at the bottom of the [`justfile`](./justfile) to your liking. The variables you might need to modify on some systems are:
```just
# tools we use (can differ on other distros)
QEMU := env("QEMU", "qemu-system-riscv64")
DEBUGGER := env("DEBUGGER", "rust-gdb")
OBJDUMP := env("OBJDUMP", "riscv64-linux-gnu-objdump")
```

Once everything is set, you can run the `cargo run` command. This should fetch all the dependencies, compile the code and run a QEMU virtual machine to execute the kernel. You should see some logs from OpenSBI before the first line of code in our OS is executed.

```sh
$ cargo run
# ... collapsed OpenSBI output ...

^w^ welcome to my operating system
DEBUG: KERNEL STARTING ON HART#0
```

## Architecture support

I do not have any plans of extending architecture support. I am very inexperienced when it comes to osdev, and juggling between multiple CPUs will put too much strain on my peanut noggin. Instead, I will (try to) put `// TODO-ARCH-RISCV` comments wherever I am hardcoding RISC-V specific behaviour. Once I have enough of these comments, I will abstract all arch specific stuff in an `arch` module, and make it generic over a trait (if such a thing is possible even).

## Contributing

Contributions are welcome! Do keep in mind that this is an educational project for me, so I might end up asking too many questions about your code. Do not submit patches if you cannot explain it! If you want to find things to contribute, run `grep -rE  "TODO|HACK|FIXME" **/*/*.rs` in the root of this project.

## TODO list
This is the TODO for what I am working on right now.

- [ ] VirtIO block driver spec from the PDF

---

All code is licensed under the Apache-2.0 license

Copyright Kunal Dandekar (2025)
