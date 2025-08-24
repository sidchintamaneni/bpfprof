#include <linux/bpf.h>
#include <linux/types.h>
#include <bpf/bpf_helpers.h>


SEC("fentry/udpv6_recvmsg")
int bpf_prog_udp(void *ctx)
{
    return 0;
}

char LISENSE[] SEC("license") = "Dual BSD/GPL";
