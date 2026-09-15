#include "nrc_oracle.h"

#include <stdlib.h>
#include <string.h>

static void expect_response(const uint8_t *actual, size_t actual_len,
                            const uint8_t *expected, size_t expected_len) {
    if (actual_len != expected_len) {
        abort();
    }

    if (expected_len > 0U && memcmp(actual, expected, expected_len) != 0) {
        abort();
    }
}

void check_nrc_oracle(const uint8_t *request, size_t request_len,
                      const uint8_t *response, size_t response_len) {
    if (request_len == 0U) {
        if (response_len != 0U) {
            abort();
        }
        return;
    }

    if (request[0] != 0x22U) {
        const uint8_t expected[] = {0x7F, request[0], 0x11};
        expect_response(response, response_len, expected, sizeof(expected));
        return;
    }

    if (request_len != 3U) {
        const uint8_t expected[] = {0x7F, 0x22, 0x13};
        expect_response(response, response_len, expected, sizeof(expected));
        return;
    }

    if (request[1] == 0xF1U && request[2] == 0x90U) {
        const uint8_t expected[] = {0x62, 0xF1, 0x90, 0x01};
        expect_response(response, response_len, expected, sizeof(expected));
        return;
    }

    const uint8_t expected[] = {0x7F, 0x22, 0x31};
    expect_response(response, response_len, expected, sizeof(expected));
}
