#include <stddef.h>
#include <stdint.h>

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    if (size >= 3 &&
        data[0] == 'U' &&
        data[1] == 'D' &&
        data[2] == 'S')
    {
        __builtin_trap();
    }

    return 0;
}
