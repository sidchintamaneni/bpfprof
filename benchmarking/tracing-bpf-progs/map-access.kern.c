#include <linux/bpf.h>
#include <linux/types.h>
#include <bpf/bpf_helpers.h>

struct {
    __uint(type, BPF_MAP_TYPE_PERCPU_ARRAY);
    __type(key, __u32);
    __type(value, __u64);
    __uint(max_entries, 1);
} udp_counter_map SEC(".maps");

SEC("fentry/udpv6_recvmsg")
int bpf_prog_udp(void *ctx)
{
	 __u32 key = 0;
    __u64 *counter;

    counter = bpf_map_lookup_elem(&udp_counter_map, &key);
    if (counter) {
        __sync_fetch_and_add(counter, 1);
    }     

	return 0;
}

char LISENSE[] SEC("license") = "Dual BSD/GPL";
