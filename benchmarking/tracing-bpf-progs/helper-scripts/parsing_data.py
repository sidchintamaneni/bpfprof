import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import seaborn as sns

def create_benchmark_charts(csv_file='iperf3_results.csv'):
    """
    Generate bar charts with standard deviation from iperf3 benchmark results
    """
    
    # Read the CSV data
    try:
        df = pd.read_csv(csv_file)
        print(f"Loaded data from {csv_file}")
    except FileNotFoundError:
        print(f"CSV file {csv_file} not found. Please check the file path.")
        return
    
    # Group by label and calculate statistics
    stats = df.groupby('label').agg({
        'bitrate_mbps': ['mean', 'std', 'count'],
        'transfer_mb': ['mean', 'std', 'count']
    }).round(2)
    
    # Flatten column names
    stats.columns = ['_'.join(col).strip() for col in stats.columns]
    stats = stats.reset_index()
    
    # Set up the plotting style
    plt.style.use('seaborn-v0_8')
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 8))
    
    # Color palette
    colors = plt.cm.Set3(np.linspace(0, 1, len(stats)))
    
    # Plot 1: Bitrate (Mbps)
    x_pos = np.arange(len(stats))
    bars1 = ax1.bar(x_pos, stats['bitrate_mbps_mean'], 
                    yerr=stats['bitrate_mbps_std'],
                    capsize=5, color=colors, alpha=0.8, edgecolor='black')
    
    ax1.set_xlabel('BPF Program Configuration', fontsize=12, fontweight='bold')
    ax1.set_ylabel('Bitrate (Mbps)', fontsize=12, fontweight='bold')
    ax1.set_title('iperf3 Bitrate Performance Comparison\nwith Standard Deviation', 
                  fontsize=14, fontweight='bold')
    ax1.set_xticks(x_pos)
    ax1.set_xticklabels(stats['label'], rotation=45, ha='right')
    ax1.grid(True, alpha=0.3)
    
    # Add value labels on bars
    for i, (bar, mean, std) in enumerate(zip(bars1, stats['bitrate_mbps_mean'], stats['bitrate_mbps_std'])):
        ax1.text(bar.get_x() + bar.get_width()/2, bar.get_height() + std + 50,
                f'{mean:.0f}±{std:.1f}', ha='center', va='bottom', fontweight='bold', fontsize=9)
    
    # Plot 2: Transfer (MB)
    bars2 = ax2.bar(x_pos, stats['transfer_mb_mean'], 
                    yerr=stats['transfer_mb_std'],
                    capsize=5, color=colors, alpha=0.8, edgecolor='black')
    
    ax2.set_xlabel('BPF Program Configuration', fontsize=12, fontweight='bold')
    ax2.set_ylabel('Transfer (MB)', fontsize=12, fontweight='bold')
    ax2.set_title('iperf3 Transfer Performance Comparison\nwith Standard Deviation', 
                  fontsize=14, fontweight='bold')
    ax2.set_xticks(x_pos)
    ax2.set_xticklabels(stats['label'], rotation=45, ha='right')
    ax2.grid(True, alpha=0.3)
    
    # Add value labels on bars
    for i, (bar, mean, std) in enumerate(zip(bars2, stats['transfer_mb_mean'], stats['transfer_mb_std'])):
        ax2.text(bar.get_x() + bar.get_width()/2, bar.get_height() + std + 0.5,
                f'{mean:.1f}±{std:.1f}', ha='center', va='bottom', fontweight='bold', fontsize=9)
    
    plt.tight_layout()
    plt.savefig('bpf_benchmark_results.png', dpi=300, bbox_inches='tight')
    plt.show()
    
    # Print summary statistics
    print("\n" + "="*85)
    print("BENCHMARK SUMMARY STATISTICS")
    print("="*85)
    print(f"{'Configuration':<25} {'Bitrate (Mbps)':<17} {'Transfer (MB)':<17} {'Samples':<10}")
    print("-"*85)
    
    for _, row in stats.iterrows():
        print(f"{row['label']:<25} "
              f"{row['bitrate_mbps_mean']:.1f}±{row['bitrate_mbps_std']:.1f}{'':>6} "
              f"{row['transfer_mb_mean']:.1f}±{row['transfer_mb_std']:.1f}{'':>7} "
              f"{int(row['bitrate_mbps_count']):<10}")
    
    # Calculate performance impact relative to baseline
    baseline_bitrate = stats[stats['label'] == 'baseline']['bitrate_mbps_mean'].iloc[0]
    baseline_transfer = stats[stats['label'] == 'baseline']['transfer_mb_mean'].iloc[0]
    
    print("\n" + "="*85)
    print("PERFORMANCE IMPACT vs BASELINE")
    print("="*85)
    print(f"{'Configuration':<25} {'Bitrate Impact (%)':<20} {'Transfer Impact (%)':<20}")
    print("-"*85)
    
    for _, row in stats.iterrows():
        if row['label'] != 'baseline':
            bitrate_impact = ((row['bitrate_mbps_mean'] - baseline_bitrate) / baseline_bitrate) * 100
            transfer_impact = ((row['transfer_mb_mean'] - baseline_transfer) / baseline_transfer) * 100
            print(f"{row['label']:<25} {bitrate_impact:>+7.1f}%{'':>12} {transfer_impact:>+7.1f}%")

def create_side_by_side_comparison(csv_file='iperf3_results.csv'):
    """
    Create a side-by-side comparison chart
    """
    df = pd.read_csv(csv_file)
    
    # Calculate statistics
    stats = df.groupby('label').agg({
        'bitrate_mbps': ['mean', 'std'],
        'transfer_mb': ['mean', 'std']
    }).round(2)
    
    stats.columns = ['_'.join(col).strip() for col in stats.columns]
    stats = stats.reset_index()
    
    # Create grouped bar chart with dual y-axis
    fig, ax1 = plt.subplots(figsize=(14, 8))
    
    x = np.arange(len(stats))
    width = 0.35
    
    # Plot bitrate on primary y-axis
    bars1 = ax1.bar(x - width/2, stats['bitrate_mbps_mean'], width, 
                    yerr=stats['bitrate_mbps_std'], label='Bitrate (Mbps)', 
                    capsize=5, alpha=0.8, color='skyblue')
    
    ax1.set_xlabel('BPF Program Configuration', fontweight='bold')
    ax1.set_ylabel('Bitrate (Mbps)', fontweight='bold', color='blue')
    ax1.tick_params(axis='y', labelcolor='blue')
    
    # Create secondary y-axis for transfer
    ax2 = ax1.twinx()
    bars2 = ax2.bar(x + width/2, stats['transfer_mb_mean'], width,
                    yerr=stats['transfer_mb_std'], label='Transfer (MB)',
                    capsize=5, alpha=0.8, color='lightcoral')
    
    ax2.set_ylabel('Transfer (MB)', fontweight='bold', color='red')
    ax2.tick_params(axis='y', labelcolor='red')
    
    ax1.set_title('iperf3 Performance: Bitrate vs Transfer Comparison', fontweight='bold', fontsize=14)
    ax1.set_xticks(x)
    ax1.set_xticklabels(stats['label'], rotation=45, ha='right')
    ax1.grid(True, alpha=0.3)
    
    # Add legends
    ax1.legend(loc='upper left')
    ax2.legend(loc='upper right')
    
    plt.tight_layout()
    plt.savefig('bpf_comparison_chart.png', dpi=300, bbox_inches='tight')
    plt.show()

def create_bitrate_only_chart(csv_file='iperf3_results.csv'):
    """
    Create a focused chart showing only bitrate performance in Mbps
    """
    df = pd.read_csv(csv_file)
    
    # Calculate statistics
    stats = df.groupby('label').agg({
        'bitrate_mbps': ['mean', 'std']
    }).round(2)
    
    stats.columns = ['mean', 'std']
    stats = stats.reset_index()
    
    # Create chart
    fig, ax = plt.subplots(figsize=(12, 8))
    
    x_pos = np.arange(len(stats))
    colors = plt.cm.viridis(np.linspace(0, 1, len(stats)))
    
    bars = ax.bar(x_pos, stats['mean'], 
                  yerr=stats['std'],
                  capsize=7, color=colors, alpha=0.8, 
                  edgecolor='black', linewidth=1.5)
    
    ax.set_xlabel('BPF Program Configuration', fontsize=14, fontweight='bold')
    ax.set_ylabel('Bitrate (Mbps)', fontsize=14, fontweight='bold')
    ax.set_title('BPF Program Performance Impact on Network Throughput\n(Higher is Better)', 
                 fontsize=16, fontweight='bold')
    ax.set_xticks(x_pos)
    ax.set_xticklabels(stats['label'], rotation=45, ha='right', fontsize=12)
    ax.grid(True, alpha=0.3, axis='y')
    
    # Add value labels on bars
    for i, (bar, mean, std) in enumerate(zip(bars, stats['mean'], stats['std'])):
        ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + std + 200,
                f'{mean:.0f}±{std:.1f}', ha='center', va='bottom', 
                fontweight='bold', fontsize=11)
    
    plt.tight_layout()
    plt.savefig('bpf_bitrate_performance.png', dpi=300, bbox_inches='tight')
    plt.show()

if __name__ == "__main__":
    # Run the analysis
    create_benchmark_charts()
    create_side_by_side_comparison()
    create_bitrate_only_chart()
