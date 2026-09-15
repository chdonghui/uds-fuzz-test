#include <stdio.h>
#include <stdint.h>

void print_hex(const uint8_t *data, size_t len) {
    for (size_t i = 0; i < len; i++) {
        printf("%02X", data[i]);

        if (i + 1 < len) {
            printf(" ");
        }
    }

    printf("\n");
}

typedef struct {
    uint8_t data[8];
    size_t len;
} Response;

Response handle_request(const uint8_t *req, size_t req_len) {
    Response resp = {0};

    /* 1. 빈 요청 검사 */
    if (req_len == 0) {
        return resp;
    }

    /* 2. SID 0x22인지 검사 */
    if (req[0] != 0x22) {
        resp.data[0] = 0x7F;
        resp.data[1] = req[0];
        resp.data[2] = 0x11;
        resp.len = 3;

        return resp;
    }

    /* 3. 요청 길이가 3인지 검사 */
    if (req_len != 3) {
        resp.data[0] = 0x7F;
        resp.data[1] = 0x22;
        resp.data[2] = 0x13;
        resp.len = 3;

        return resp;
    }

    /* 4. DID가 F1 90인지 검사 */

    if (req[1] != 0xF1 || req[2] != 0x90) {
        resp.data[0] = 0x7F;
        resp.data[1] = 0x22;
        resp.data[2] = 0x31;
        resp.len = 3;

        return resp;
    }

    /* 5. resp.data에 응답을 넣고 resp.len 설정 */

    resp.data[0] = 0x62;
    resp.data[1] = 0xF1;
    resp.data[2] = 0x90;
    resp.data[3] = 0x01;
    resp.len = 4;

    return resp;
}

#ifndef STATELESS_ECU_FUZZING

int main() {
    uint8_t request[] = {0x22, 0x00, 0x00};
    // uint8_t request[] = {0x22, 0xF1, 0x90};

    Response response = handle_request(request, sizeof(request));

    printf("요청: ");
    print_hex(request, sizeof(request));
    printf("응답: ");
    print_hex(response.data, response.len);
    return 0;
}

#endif
