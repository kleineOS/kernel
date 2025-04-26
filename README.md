# KleineOS kernel

My experimentation ground

## TODO

- [ ] Replace panics and unwraps with a Result::Err
    - This might not be as straight forward, as some cases MUST crash ASAP
- [ ] A tiered allocator that will be used with the `alloc` crate
    - This is just a BitMap allocator but with fancier logic
    - Doing this so I wont have to spend a week learning how to do a slab alloc

## Goals of the project

1. A simple FAT driver for filesystem access
2. Userspace with a basic shell
3. Multiprocessing, and a demo to showcase it

---

All code is licensed under the Apache-2.0 license
