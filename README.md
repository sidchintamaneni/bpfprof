# bpfprof

#### Build instructions

As I am using a custom kernel I am copying `bpf_core_read.h` and `bpf_helpers.h`
manually to `/usr/include/bpf`.

If you are using a distro kernel then you can get the headers by installing
`libbpf-dev` package.
