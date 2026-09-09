#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "server.h"
#include "tp.h"

#define POLL_COUNT 4
#define POLL_STEP_MS 10000U
#define DOWNLOAD_BLOCK_LENGTH 512U

typedef struct {
    UDSTp_t tp;

    const uint8_t *request;
    size_t request_len;
    int delivered;

    uint8_t response[UDS_TP_MTU];
    size_t response_len;
    size_t response_count;
} FuzzTp;

static uint32_t fuzz_time_ms;

// 실제 시간을 기다리지 않고 서버의 응답 타이머를 진행한다.
uint32_t UDSMillis(void) {
    return fuzz_time_ms;
}

static UDSTpSize_t fuzz_send(struct UDSTp *hdl, const uint8_t *buf, size_t len,
                             const UDSSDU_t *info) {
    FuzzTp *tp = (FuzzTp *)hdl;
    (void)info;

    if (len > sizeof(tp->response)) {
        return -1;
    }

    memcpy(tp->response, buf, len);
    tp->response_len = len;
    tp->response_count++;
    return (UDSTpSize_t)len;
}

static UDSTpSize_t fuzz_recv(struct UDSTp *hdl, uint8_t *buf, size_t bufsize,
                             UDSSDU_t *info) {
    FuzzTp *tp = (FuzzTp *)hdl;

    if (tp->delivered) {
        return 0;
    }
    if (tp->request_len > bufsize) {
        return -1;
    }

    memcpy(buf, tp->request, tp->request_len);

    if (info != NULL) {
        memset(info, 0, sizeof(*info));
        info->A_Mtype = UDS_A_MTYPE_DIAG;
        info->A_TA_Type = UDS_A_TA_TYPE_PHYSICAL;
    }

    tp->delivered = 1;
    return (UDSTpSize_t)tp->request_len;
}

static UDSTpStatus_t fuzz_poll(struct UDSTp *hdl) {
    (void)hdl;
    return UDS_TP_IDLE;
}

// 실제 ECU 애플리케이션 대신 서비스별 최소 정상 응답값을 제공한다.
static UDSErr_t server_event(UDSServer_t *server, UDSEvent_t event, void *arg) {
    static const uint8_t response_data[] = {0x01, 0x02, 0x03, 0x04};

    switch (event) {
    case UDS_EVT_DiagSessCtrl: {
        UDSDiagSessCtrlArgs_t *args = (UDSDiagSessCtrlArgs_t *)arg;
        args->p2_ms = 50;
        args->p2_star_ms = 5000;
        break;
    }
    case UDS_EVT_EcuReset: {
        UDSECUResetArgs_t *args = (UDSECUResetArgs_t *)arg;
        args->powerDownTimeMillis = 10;
        break;
    }
    case UDS_EVT_ReadDataByIdent: {
        UDSRDBIArgs_t *args = (UDSRDBIArgs_t *)arg;
        (void)args->copy(server, response_data, sizeof(response_data));
        break;
    }
    case UDS_EVT_ReadMemByAddr: {
        UDSReadMemByAddrArgs_t *args = (UDSReadMemByAddrArgs_t *)arg;
        (void)args->copy(server, response_data, sizeof(response_data));
        break;
    }
    case UDS_EVT_SecAccessRequestSeed: {
        UDSSecAccessRequestSeedArgs_t *args = (UDSSecAccessRequestSeedArgs_t *)arg;
        (void)args->copySeed(server, response_data, sizeof(response_data));
        break;
    }
    case UDS_EVT_RoutineCtrl: {
        UDSRoutineCtrlArgs_t *args = (UDSRoutineCtrlArgs_t *)arg;
        (void)args->copyStatusRecord(server, response_data, sizeof(response_data));
        break;
    }
    case UDS_EVT_RequestDownload: {
        UDSRequestDownloadArgs_t *args = (UDSRequestDownloadArgs_t *)arg;
        args->maxNumberOfBlockLength = DOWNLOAD_BLOCK_LENGTH;
        break;
    }
    case UDS_EVT_RequestUpload: {
        UDSRequestUploadArgs_t *args = (UDSRequestUploadArgs_t *)arg;
        args->maxNumberOfBlockLength = DOWNLOAD_BLOCK_LENGTH;
        break;
    }
    case UDS_EVT_TransferData: {
        UDSTransferDataArgs_t *args = (UDSTransferDataArgs_t *)arg;
        (void)args->copyResponse(server, response_data, sizeof(response_data));
        break;
    }
    case UDS_EVT_RequestTransferExit: {
        UDSRequestTransferExitArgs_t *args = (UDSRequestTransferExitArgs_t *)arg;
        (void)args->copyResponse(server, response_data, sizeof(response_data));
        break;
    }
    case UDS_EVT_RequestFileTransfer: {
        UDSRequestFileTransferArgs_t *args = (UDSRequestFileTransferArgs_t *)arg;
        args->maxNumberOfBlockLength = DOWNLOAD_BLOCK_LENGTH;
        break;
    }
    case UDS_EVT_Custom: {
        UDSCustomArgs_t *args = (UDSCustomArgs_t *)arg;
        (void)args->copyResponse(server, response_data, sizeof(response_data));
        break;
    }
    default:
        break;
    }

    return UDS_PositiveResponse;
}

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    if (size == 0 || size > UDS_TP_MTU) {
        return 0;
    }

    FuzzTp tp = {0};
    tp.tp.send = fuzz_send;
    tp.tp.recv = fuzz_recv;
    tp.tp.poll = fuzz_poll;
    tp.request = data;
    tp.request_len = size;

    UDSServer_t server;
    fuzz_time_ms = 0;

    if (UDSServerInit(&server) != UDS_OK) {
        return 0;
    }

    server.tp = &tp.tp;
    server.fn = server_event;

    // 첫 poll에서 요청을 처리하고 이후 poll에서 타이머와 응답 송신을 진행한다.
    for (int i = 0; i < POLL_COUNT; i++) {
        UDSServerPoll(&server);
        fuzz_time_ms += POLL_STEP_MS;
    }

    return 0;
}
