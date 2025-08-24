#!/bin/bash
# filepath: /home/azureuser/personal/dev/os-dev-env/iperf3_benchmark.sh

# Configuration
SERVER_PORT=62229
WARMUP_DURATION=5
TEST_DURATION=10
NUM_RUNS=10
OUTPUT_CSV="iperf3_results.csv"
SLEEP_BETWEEN_RUNS=2

# Get custom label from command line argument
CUSTOM_LABEL="${1:-default}"

# Test parameters
CLIENT_PARAMS="-c localhost -p $SERVER_PORT -u -P 8 -b 1G -l 64 -t $TEST_DURATION"
WARMUP_PARAMS="-c localhost -p $SERVER_PORT -u -P 8 -b 1G -l 64 -t $WARMUP_DURATION"

echo "=== iperf3 UDP Benchmark (Receiver Metrics) ==="
echo "Runs: $NUM_RUNS, Duration: ${TEST_DURATION}s each, Sleep: ${SLEEP_BETWEEN_RUNS}s between runs"
echo "Custom Label: $CUSTOM_LABEL"

# Check server connectivity
if ! timeout 3 bash -c "echo >/dev/tcp/localhost/$SERVER_PORT" 2>/dev/null; then
    echo "Error: Cannot connect to server on port $SERVER_PORT"
    echo "Start server with: iperf3 -s -p $SERVER_PORT"
    exit 1
fi

# Initialize CSV with updated headers
if [ ! -f "$OUTPUT_CSV" ]; then
	echo "run,bitrate_mbps,transfer_mb,label" > "$OUTPUT_CSV"
fi

# Warmup
echo "Running warmup..."
iperf3 $WARMUP_PARAMS > /dev/null 2>&1
echo "Warmup complete"

# Sleep after warmup
echo "Sleeping ${SLEEP_BETWEEN_RUNS}s after warmup..."
sleep $SLEEP_BETWEEN_RUNS

# Main test runs
echo "Starting test runs..."
for ((run=1; run<=NUM_RUNS; run++)); do
    echo "Run $run/$NUM_RUNS"
    
    # Run test and save output
    temp_file="/tmp/iperf3_run_$run.txt"
    iperf3 $CLIENT_PARAMS > "$temp_file" 2>&1
    
    if [ $? -ne 0 ]; then
        echo "Run $run failed"
        if [ $run -lt $NUM_RUNS ]; then
            echo "Sleeping ${SLEEP_BETWEEN_RUNS}s..."
            sleep $SLEEP_BETWEEN_RUNS
        fi
        continue
    fi
    
    # Extract the SUM receiver line specifically
    summary_line=$(grep -E "\[SUM\].*receiver" "$temp_file" | head -1)
    
    if [ -z "$summary_line" ]; then
        echo "Run $run: Could not find SUM receiver line"
        if [ $run -lt $NUM_RUNS ]; then
            echo "Sleeping ${SLEEP_BETWEEN_RUNS}s..."
            sleep $SLEEP_BETWEEN_RUNS
        fi
        continue
    fi
    
    echo "Debug: SUM line = $summary_line"
    
    # Parse transfer - handle both GBytes and MBytes
    transfer_line=$(echo "$summary_line" | grep -oE '[0-9]+(\.[0-9]+)?\s+(G|M)Bytes')
    if echo "$transfer_line" | grep -q "GBytes"; then
        # Convert GBytes to MBytes
        transfer_gb=$(echo "$transfer_line" | grep -oE '[0-9]+(\.[0-9]+)?')
        transfer_mb=$(echo "scale=2; $transfer_gb * 1000" | bc -l 2>/dev/null || echo "0")
    else
        # Already in MBytes
        transfer_mb=$(echo "$transfer_line" | grep -oE '[0-9]+(\.[0-9]+)?')
    fi
    
    # Parse bitrate - handle both Gbits/sec and Mbits/sec
    bitrate_line=$(echo "$summary_line" | grep -oE '[0-9]+(\.[0-9]+)?\s+(G|M)bits/sec')
    if echo "$bitrate_line" | grep -q "Gbits/sec"; then
        # Convert Gbits to Mbits
        bitrate_gbps=$(echo "$bitrate_line" | grep -oE '[0-9]+(\.[0-9]+)?')
        bitrate_mbps=$(echo "scale=2; $bitrate_gbps * 1000" | bc -l 2>/dev/null || echo "0")
    else
        # Already in Mbits/sec
        bitrate_mbps=$(echo "$bitrate_line" | grep -oE '[0-9]+(\.[0-9]+)?')
    fi
    
    if [ -z "$bitrate_mbps" ] || [ -z "$transfer_mb" ]; then
        echo "Run $run: Failed to extract metrics"
        echo "  Transfer line: '$transfer_line' -> '$transfer_mb'"
        echo "  Bitrate line: '$bitrate_line' -> '$bitrate_mbps'"
        if [ $run -lt $NUM_RUNS ]; then
            echo "Sleeping ${SLEEP_BETWEEN_RUNS}s..."
            sleep $SLEEP_BETWEEN_RUNS
        fi
        continue
    fi
    
    # Save to CSV with custom label
    echo "$run,$bitrate_mbps,$transfer_mb,$CUSTOM_LABEL" >> "$OUTPUT_CSV"
    echo "Run $run: $bitrate_mbps Mbps, $transfer_mb MB (receiver throughput) [$CUSTOM_LABEL]"
    
    # Clean up temp file
    rm -f "$temp_file"
    
    # Sleep between runs (except after the last run)
    if [ $run -lt $NUM_RUNS ]; then
        echo "Sleeping ${SLEEP_BETWEEN_RUNS}s..."
        sleep $SLEEP_BETWEEN_RUNS
    fi
done

echo "Complete. Results saved to: $OUTPUT_CSV"
