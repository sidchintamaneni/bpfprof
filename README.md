# bpfprof

#### Build instructions

`cargo build`

`bpfprof prog [ID]` - samples the BPF program for 30 seconds (default) and prints out the output to the console

`bpfprof prog [ID] -t [time (s)]`

#### Misc

Ignore this for now, since we are not using BPF iter in bpfprof anymore

As I am using a custom kernel I am copying `bpf_core_read.h` and `bpf_helpers.h`
manually to `/usr/include/bpf`.

If you are using a distro kernel then you can get the headers by installing
`libbpf-dev` package.
