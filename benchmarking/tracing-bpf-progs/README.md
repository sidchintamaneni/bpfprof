# Benchmarking Data

## Experimental setup

Running iperf3 on a virtualization setup.

Host System - Azure VM (kernel - 6.11.0-1018-azure, OS - ubuntu 24.04.03 LTS, 16 CPUs).
Guest System - QEMU (x86_64, kernel - 6.17.0-rc3, custom config - pretty light weight, 8 CPUs).
*Qemu runs inside a docker container.

Running standard iperf3 on Host system as client and Qemu as the server.

Measuring the overhead of different types of tracing BPF program types with and
without BPF_ENABLE_STATS.

Different tracing program types:
- Tracepoints
- Kprobes (interrupt based)
- Kprobes (fprobe based)
- BPF Trampolines (fentry/ fexit)

Attached BPF program
1. null prog (which returns 0)
2. BPF prog with a per-cpu map operation (reads pid from helper and write it
   into a per-cpu map)

#### iperf command on the client side

**Sending tiny packets as fast as possible**

```
iperf3 -c localhost -p 62229 -u -P 8 -b 1G -l 64
```

- The idea is that this will stress out the `udp_recvmsg` function where BPF
  program is attached.

```
./iperf_client.sh "experiment_label"
```

#### BPF commands on the server side
- Attached the BPF programs individually

To enable BPF_STATS_ENABLED
```
root@q:/os-dev-env# echo 1 > /proc/sys/kernel/bpf_stats_enabled   
root@q:/os-dev-env# cat /proc/sys/kernel/bpf_stats_enabled 
```

#### Results

not as expected :(

Configuration             Bitrate (Mbps)    Transfer (MB)     Samples   
-------------------------------------------------------------------------------------
baseline                  18.6±1.2       23.8±0.9        10        
fentry-map                19.9±1.1       25.4±0.8        10        
fentry-map-stat           19.5±0.9       25.6±1.2        10        
fentry-null               20.1±1.2       25.6±1.1        10        
fentry-null-stat          19.6±1.6       25.4±1.2        10        
kprobe-fprobe-map         20.6±1.3       25.7±1.1        10        
kprobe-fprobe-map-stat    19.4±1.3       25.1±0.6        10        
kprobe-fprobe-null        19.7±1.5       25.7±1.0        10        
kprobe-fprobe-null-stat   20.5±1.3       26.1±1.6        10        

=====================================================================================
PERFORMANCE IMPACT vs BASELINE
=====================================================================================
Configuration             Bitrate Impact (%)   Transfer Impact (%) 
-------------------------------------------------------------------------------------
fentry-map                   +6.7%                +6.9%
fentry-map-stat              +4.7%                +7.7%
fentry-null                  +8.2%                +7.7%
fentry-null-stat             +5.3%                +6.7%
kprobe-fprobe-map           +10.4%                +8.4%
kprobe-fprobe-map-stat       +4.1%                +5.6%
kprobe-fprobe-null           +5.5%                +8.3%

