#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "server.h"
#include "tp.h"

typedef struct {
    UDSTp_t tp;

    const uint8_t *data;
    size_t size;
    int delivered;
} FuzzTp;


static ssize_t fuzz_send(
    struct UDSTp *hdl,
    uint8_t *buf,
    size_t len,
    UDSSDU_t *info)
{
    return (ssize_t)len;
}


static ssize_t fuzz_recv(
    struct UDSTp *hdl,
    uint8_t *buf,
    size_t bufsize,
    UDSSDU_t *info)
{
    FuzzTp *tp = (FuzzTp *)hdl;

    if (tp->delivered)
        return 0;

    size_t len = tp->size;

    if (len > bufsize)
        len = bufsize;

    memcpy(buf, tp->data, len);

    if (info) {
        memset(info, 0, sizeof(*info));

        info->A_Mtype = UDS_A_MTYPE_DIAG;
        info->A_TA_Type = UDS_A_TA_TYPE_PHYSICAL;
    }

    tp->delivered = 1;

    return (ssize_t)len;
}


static UDSTpStatus_t fuzz_poll(struct UDSTp *hdl)
{
    return UDS_TP_IDLE;
}


static UDSErr_t server_event(
    UDSServer_t *srv,
    UDSEvent_t event,
    void *arg)
{
    return UDS_PositiveResponse;
}


int LLVMFuzzerTestOneInput(
    const uint8_t *data,
    size_t size)
{
    if (size == 0)
        return 0;

    FuzzTp tp = {0};

    tp.tp.send = fuzz_send;
    tp.tp.recv = fuzz_recv;
    tp.tp.poll = fuzz_poll;

    tp.data = data;
    tp.size = size;

    UDSServer_t server;

    UDSServerInit(&server);

    server.tp = &tp.tp;
    server.fn = server_event;

    UDSServerPoll(&server);

    return 0;
}
