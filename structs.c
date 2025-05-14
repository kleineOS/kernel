#include <stdint.h>

// debug symbols of this file will be used for for debugging with gdb
// gcc -g -c % -o structs.o

struct vendor_dev {
    uint16_t vendor_id;
    uint16_t device_id;
};

struct virtio_pci_common_cfg {
    /* About the whole device */
    uint32_t device_feature_select_le;
    uint32_t divice_feature_le;
    uint32_t driver_feature_select_le;
    uint32_t driver_feature_le;
    uint16_t config_msix_vector_le;
    uint16_t num_queues_le;
    uint8_t device_status;
    uint8_t config_generation;

    /* About a specific virtqueue */
    uint16_t queue_select_le;
    uint16_t queue_size_le;
    uint16_t queue_msix_vector_le;
    uint16_t queue_enable_le;
    uint16_t queue_notify_off_le;
    uint64_t queue_desc_le;
    uint64_t queue_driver_le;
    uint64_t queue_device_le;
    uint16_t queue_notif_config_data_le;
    uint16_t queue_reset_le;

    /* About the administration virtqueue */
    uint16_t admin_queue_index_le;
    uint16_t admin_queue_num_le;
};
