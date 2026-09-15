#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

#define STATELESS_ECU_FUZZING
#include "../stateless_ecu_mvp.c"

#include "nrc_oracle.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    Response response = handle_request(data, size);

    if (response.len > sizeof(response.data)) {
        abort();
    }

    check_nrc_oracle(data, size, response.data, response.len);
    return 0;
}
