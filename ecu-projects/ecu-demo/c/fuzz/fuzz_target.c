#include <stddef.h>
#include <stdint.h>

#define STATELESS_ECU_FUZZING
#include "../stateless_ecu_mvp.c"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    Response response = handle_request(data, size);

    if (response.len > sizeof(response.data)) {
        __builtin_trap();
    }

    return 0;
}
