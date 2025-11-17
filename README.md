# bpfprof

#### Build instructions

```
cargo build
```

## Usage

### List all BPF programs

```bash
sudo ./bpfprof list
```

### Profile specific program(s)

```bash
# Profile single program for 30 seconds (default)
sudo ./bpfprof prog id <PROG_ID>

# Profile with custom duration
sudo ./bpfprof prog id <PROG_ID> -t <TIME_SECONDS>

# Profile with custom sampling frequency (default: 997 Hz)
sudo ./bpfprof prog id <PROG_ID> -f <FREQUENCY>

# Profile multiple programs
sudo ./bpfprof prog id <ID1> <ID2> <ID3> -t 30

# Export results to CSV
sudo ./bpfprof prog id <PROG_ID> -o results.csv
```

### Profile all BPF programs

```bash
# Profile all programs in the kernel
sudo ./bpfprof prog all

# Profile all with custom duration and export
sudo ./bpfprof prog all -t 10 -o all_programs.csv
```

#### TODO

- [ ] Add the ability to generate relavant graphs
- [ ] Add relavant comments and do a `cargo fmt` on the proj
- [ ] Refactor the code
