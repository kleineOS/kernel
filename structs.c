#include <stdint.h>

// debug symbols of this file will be used for for debugging with gdb
// gcc -g -c % -o structs.o

struct vendor_dev {
    uint16_t vendor_id;
    uint16_t device_id;
};
