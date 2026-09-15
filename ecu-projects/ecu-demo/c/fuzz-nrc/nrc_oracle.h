#pragma once

#include <stddef.h>
#include <stdint.h>

void check_nrc_oracle(const uint8_t *request, size_t request_len,
                      const uint8_t *response, size_t response_len);
