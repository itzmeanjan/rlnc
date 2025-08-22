#!/usr/bin/python

import sys
import re
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from collections import defaultdict
import io

# Store the decoder benchmark results in a multiline string
benchmark_data = """
Timer precision: 22 ns
full_rlnc_decoder                          fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ decode                                                │               │               │               │         │
   ├─ 1.00 MB data split into 4 pieces     166.8 µs      │ 533.5 µs      │ 172.9 µs      │ 179 µs        │ 100     │ 100
   │                                       5.852 GiB/s   │ 1.83 GiB/s    │ 5.647 GiB/s   │ 5.454 GiB/s   │         │
   ├─ 1.00 MB data split into 8 pieces     314.4 µs      │ 740.8 µs      │ 323.6 µs      │ 329.1 µs      │ 100     │ 100
   │                                       3.105 GiB/s   │ 1.318 GiB/s   │ 3.017 GiB/s   │ 2.966 GiB/s   │         │
   ├─ 1.00 MB data split into 16 pieces    589.7 µs      │ 649.3 µs      │ 611.5 µs      │ 611.2 µs      │ 100     │ 100
   │                                       1.656 GiB/s   │ 1.504 GiB/s   │ 1.597 GiB/s   │ 1.598 GiB/s   │         │
   ├─ 1.00 MB data split into 32 pieces    1.158 ms      │ 1.472 ms      │ 1.214 ms      │ 1.269 ms      │ 100     │ 100
   │                                       864 MiB/s     │ 679.6 MiB/s   │ 823.9 MiB/s   │ 788.3 MiB/s   │         │
   ├─ 1.00 MB data split into 64 pieces    2.482 ms      │ 2.682 ms      │ 2.527 ms      │ 2.532 ms      │ 100     │ 100
   │                                       404.4 MiB/s   │ 374.3 MiB/s   │ 397.1 MiB/s   │ 396.4 MiB/s   │         │
   ├─ 1.00 MB data split into 128 pieces   5.314 ms      │ 5.938 ms      │ 5.751 ms      │ 5.697 ms      │ 100     │ 100
   │                                       191.1 MiB/s   │ 171 MiB/s     │ 176.6 MiB/s   │ 178.2 MiB/s   │         │
   ├─ 1.00 MB data split into 256 pieces   15.38 ms      │ 15.95 ms      │ 15.53 ms      │ 15.54 ms      │ 100     │ 100
   │                                       69.06 MiB/s   │ 66.62 MiB/s   │ 68.42 MiB/s   │ 68.36 MiB/s   │         │
   ├─ 1.00 MB data split into 512 pieces   64.87 ms      │ 69 ms         │ 65.06 ms      │ 65.11 ms      │ 100     │ 100
   │                                       19.27 MiB/s   │ 18.12 MiB/s   │ 19.22 MiB/s   │ 19.2 MiB/s    │         │
   ├─ 4.00 MB data split into 4 pieces     1.13 ms       │ 2.426 ms      │ 1.158 ms      │ 1.188 ms      │ 100     │ 100
   │                                       3.453 GiB/s   │ 1.609 GiB/s   │ 3.372 GiB/s   │ 3.285 GiB/s   │         │
   ├─ 4.00 MB data split into 8 pieces     1.776 ms      │ 3.342 ms      │ 1.8 ms        │ 1.818 ms      │ 100     │ 100
   │                                       2.198 GiB/s   │ 1.168 GiB/s   │ 2.169 GiB/s   │ 2.148 GiB/s   │         │
   ├─ 4.00 MB data split into 16 pieces    2.964 ms      │ 4.957 ms      │ 2.994 ms      │ 3.018 ms      │ 100     │ 100
   │                                       1.317 GiB/s   │ 806.9 MiB/s   │ 1.304 GiB/s   │ 1.294 GiB/s   │         │
   ├─ 4.00 MB data split into 32 pieces    5.395 ms      │ 7.445 ms      │ 5.831 ms      │ 5.843 ms      │ 100     │ 100
   │                                       741.5 MiB/s   │ 537.4 MiB/s   │ 686 MiB/s     │ 684.6 MiB/s   │         │
   ├─ 4.00 MB data split into 64 pieces    10.33 ms      │ 12.51 ms      │ 11.56 ms      │ 11.58 ms      │ 100     │ 100
   │                                       387.4 MiB/s   │ 319.8 MiB/s   │ 346.1 MiB/s   │ 345.4 MiB/s   │         │
   ├─ 4.00 MB data split into 128 pieces   23.05 ms      │ 28.16 ms      │ 23.74 ms      │ 23.87 ms      │ 100     │ 100
   │                                       174.1 MiB/s   │ 142.5 MiB/s   │ 169.1 MiB/s   │ 168.1 MiB/s   │         │
   ├─ 4.00 MB data split into 256 pieces   45.81 ms      │ 50.14 ms      │ 48.56 ms      │ 48.53 ms      │ 100     │ 100
   │                                       88.67 MiB/s   │ 81.02 MiB/s   │ 83.64 MiB/s   │ 83.7 MiB/s    │         │
   ├─ 4.00 MB data split into 512 pieces   132.2 ms      │ 133.9 ms      │ 133.2 ms      │ 133.1 ms      │ 100     │ 100
   │                                       32.13 MiB/s   │ 31.73 MiB/s   │ 31.9 MiB/s    │ 31.91 MiB/s   │         │
   ├─ 8.00 MB data split into 4 pieces     2.37 ms       │ 2.726 ms      │ 2.393 ms      │ 2.405 ms      │ 100     │ 100
   │                                       3.296 GiB/s   │ 2.865 GiB/s   │ 3.263 GiB/s   │ 3.247 GiB/s   │         │
   ├─ 8.00 MB data split into 8 pieces     4.152 ms      │ 6.829 ms      │ 4.327 ms      │ 4.367 ms      │ 100     │ 100
   │                                       1.881 GiB/s   │ 1.143 GiB/s   │ 1.805 GiB/s   │ 1.788 GiB/s   │         │
   ├─ 8.00 MB data split into 16 pieces    6.636 ms      │ 9.798 ms      │ 6.814 ms      │ 6.849 ms      │ 100     │ 100
   │                                       1.177 GiB/s   │ 816.4 MiB/s   │ 1.146 GiB/s   │ 1.14 GiB/s    │         │
   ├─ 8.00 MB data split into 32 pieces    11.24 ms      │ 15.13 ms      │ 11.89 ms      │ 11.93 ms      │ 100     │ 100
   │                                       711.8 MiB/s   │ 528.6 MiB/s   │ 672.8 MiB/s   │ 670.1 MiB/s   │         │
   ├─ 8.00 MB data split into 64 pieces    22.75 ms      │ 24.32 ms      │ 23.56 ms      │ 23.59 ms      │ 100     │ 100
   │                                       351.6 MiB/s   │ 328.9 MiB/s   │ 339.5 MiB/s   │ 339.2 MiB/s   │         │
   ├─ 8.00 MB data split into 128 pieces   45.81 ms      │ 47.69 ms      │ 46.86 ms      │ 46.88 ms      │ 100     │ 100
   │                                       174.9 MiB/s   │ 168 MiB/s     │ 171 MiB/s     │ 170.9 MiB/s   │         │
   ├─ 8.00 MB data split into 256 pieces   97.82 ms      │ 109.9 ms      │ 98.97 ms      │ 99.63 ms      │ 100     │ 100
   │                                       82.41 MiB/s   │ 73.3 MiB/s    │ 81.46 MiB/s   │ 80.92 MiB/s   │         │
   ├─ 8.00 MB data split into 512 pieces   224 ms        │ 227.9 ms      │ 225.3 ms      │ 225.3 ms      │ 100     │ 100
   │                                       36.82 MiB/s   │ 36.19 MiB/s   │ 36.6 MiB/s    │ 36.61 MiB/s   │         │
   ├─ 16.00 MB data split into 4 pieces    5.53 ms       │ 6.627 ms      │ 5.642 ms      │ 5.698 ms      │ 100     │ 100
   │                                       2.825 GiB/s   │ 2.357 GiB/s   │ 2.769 GiB/s   │ 2.741 GiB/s   │         │
   ├─ 16.00 MB data split into 8 pieces    9.49 ms       │ 10.87 ms      │ 9.551 ms      │ 9.593 ms      │ 100     │ 100
   │                                       1.646 GiB/s   │ 1.436 GiB/s   │ 1.635 GiB/s   │ 1.628 GiB/s   │         │
   ├─ 16.00 MB data split into 16 pieces   16.78 ms      │ 19.14 ms      │ 17.13 ms      │ 17.17 ms      │ 100     │ 100
   │                                       953.1 MiB/s   │ 835.9 MiB/s   │ 933.9 MiB/s   │ 931.3 MiB/s   │         │
   ├─ 16.00 MB data split into 32 pieces   28 ms         │ 31.84 ms      │ 28.2 ms       │ 28.3 ms       │ 100     │ 100
   │                                       571.4 MiB/s   │ 502.4 MiB/s   │ 567.2 MiB/s   │ 565.3 MiB/s   │         │
   ├─ 16.00 MB data split into 64 pieces   50.23 ms      │ 54.53 ms      │ 50.5 ms       │ 50.73 ms      │ 100     │ 100
   │                                       318.5 MiB/s   │ 293.4 MiB/s   │ 316.8 MiB/s   │ 315.4 MiB/s   │         │
   ├─ 16.00 MB data split into 128 pieces  99.62 ms      │ 105.7 ms      │ 100.3 ms      │ 100.7 ms      │ 100     │ 100
   │                                       160.7 MiB/s   │ 151.3 MiB/s   │ 159.5 MiB/s   │ 158.9 MiB/s   │         │
   ├─ 16.00 MB data split into 256 pieces  204.9 ms      │ 214.1 ms      │ 207.2 ms      │ 207.4 ms      │ 100     │ 100
   │                                       78.36 MiB/s   │ 75.02 MiB/s   │ 77.5 MiB/s    │ 77.42 MiB/s   │         │
   ├─ 16.00 MB data split into 512 pieces  450.7 ms      │ 488.6 ms      │ 456.3 ms      │ 457.1 ms      │ 100     │ 100
   │                                       36.05 MiB/s   │ 33.25 MiB/s   │ 35.61 MiB/s   │ 35.54 MiB/s   │         │
   ├─ 32.00 MB data split into 4 pieces    20.24 ms      │ 22.47 ms      │ 20.6 ms       │ 20.73 ms      │ 100     │ 100
   │                                       1.543 GiB/s   │ 1.39 GiB/s    │ 1.516 GiB/s   │ 1.506 GiB/s   │         │
   ├─ 32.00 MB data split into 8 pieces    28.68 ms      │ 32.32 ms      │ 29.03 ms      │ 29.15 ms      │ 100     │ 100
   │                                       1.089 GiB/s   │ 990 MiB/s     │ 1.076 GiB/s   │ 1.071 GiB/s   │         │
   ├─ 32.00 MB data split into 16 pieces   45.35 ms      │ 47.58 ms      │ 45.62 ms      │ 45.8 ms       │ 100     │ 100
   │                                       705.5 MiB/s   │ 672.5 MiB/s   │ 701.3 MiB/s   │ 698.6 MiB/s   │         │
   ├─ 32.00 MB data split into 32 pieces   77.51 ms      │ 79.93 ms      │ 77.81 ms      │ 78.01 ms      │ 100     │ 100
   │                                       412.8 MiB/s   │ 400.3 MiB/s   │ 411.2 MiB/s   │ 410.1 MiB/s   │         │
   ├─ 32.00 MB data split into 64 pieces   128.7 ms      │ 143.5 ms      │ 130.2 ms      │ 130.6 ms      │ 100     │ 100
   │                                       248.6 MiB/s   │ 222.8 MiB/s   │ 245.7 MiB/s   │ 244.9 MiB/s   │         │
   ├─ 32.00 MB data split into 128 pieces  232.1 ms      │ 264.3 ms      │ 237.6 ms      │ 237.9 ms      │ 100     │ 100
   │                                       137.8 MiB/s   │ 121.1 MiB/s   │ 134.6 MiB/s   │ 134.5 MiB/s   │         │
   ├─ 32.00 MB data split into 256 pieces  465.6 ms      │ 571.5 ms      │ 472.9 ms      │ 474.9 ms      │ 100     │ 100
   │                                       68.86 MiB/s   │ 56.09 MiB/s   │ 67.79 MiB/s   │ 67.5 MiB/s    │         │
   ├─ 32.00 MB data split into 512 pieces  970.2 ms      │ 1.253 s       │ 1.002 s       │ 1.006 s       │ 100     │ 100
   │                                       33.23 MiB/s   │ 25.71 MiB/s   │ 32.18 MiB/s   │ 32.04 MiB/s   │         │
   ├─ 64.00 MB data split into 4 pieces    46.93 ms      │ 51.19 ms      │ 48.15 ms      │ 48.31 ms      │ 100     │ 100
   │                                       1.331 GiB/s   │ 1.22 GiB/s    │ 1.297 GiB/s   │ 1.293 GiB/s   │         │
   ├─ 64.00 MB data split into 8 pieces    62.61 ms      │ 71.21 ms      │ 64.31 ms      │ 64.5 ms       │ 100     │ 100
   │                                       1022 MiB/s    │ 898.6 MiB/s   │ 995.1 MiB/s   │ 992.1 MiB/s   │         │
   ├─ 64.00 MB data split into 16 pieces   98.03 ms      │ 119 ms        │ 99.73 ms      │ 100.1 ms      │ 100     │ 100
   │                                       652.8 MiB/s   │ 537.5 MiB/s   │ 641.7 MiB/s   │ 639.1 MiB/s   │         │
   ├─ 64.00 MB data split into 32 pieces   169.4 ms      │ 204 ms        │ 170.8 ms      │ 174.4 ms      │ 100     │ 100
   │                                       377.6 MiB/s   │ 313.7 MiB/s   │ 374.5 MiB/s   │ 366.9 MiB/s   │         │
   ├─ 64.00 MB data split into 64 pieces   306.6 ms      │ 379.2 ms      │ 310.3 ms      │ 323.6 ms      │ 100     │ 100
   │                                       208.7 MiB/s   │ 168.7 MiB/s   │ 206.2 MiB/s   │ 197.7 MiB/s   │         │
   ├─ 64.00 MB data split into 128 pieces  522.2 ms      │ 641.4 ms      │ 543.5 ms      │ 549.5 ms      │ 100     │ 100
   │                                       122.5 MiB/s   │ 99.8 MiB/s    │ 117.7 MiB/s   │ 116.4 MiB/s   │         │
   ├─ 64.00 MB data split into 256 pieces  1.022 s       │ 1.208 s       │ 1.044 s       │ 1.049 s       │ 96      │ 96
   │                                       62.65 MiB/s   │ 53 MiB/s      │ 61.36 MiB/s   │ 61.04 MiB/s   │         │
   ╰─ 64.00 MB data split into 512 pieces  2.074 s       │ 2.336 s       │ 2.121 s       │ 2.129 s       │ 47      │ 47
                                           30.97 MiB/s   │ 27.49 MiB/s   │ 30.28 MiB/s   │ 30.17 MiB/s   │         │
"""

def parse_and_save_plot(data: str, output_filename: str):
    """
    Parses the decoder benchmark data and saves the median throughput plot to a file.
    Handles throughput values in both GiB/s and MiB/s.
    """
    results = defaultdict(list)
    
    config_pattern = re.compile(r'([\d\.]+\s*(?:MB|GB|KB))\s+data split into\s+(\d+)\s+pieces')
    
    # Regex to capture throughput values with either GiB/s or MiB/s units
    throughput_pattern = re.compile(
        r'([\d\.]+\s+(?:GiB|MiB)/s)\s+│\s+'  # Group 1: fastest
        r'([\d\.]+\s+(?:GiB|MiB)/s)\s+│\s+'  # Group 2: slowest
        r'([\d\.]+\s+(?:GiB|MiB)/s)\s+│\s+'  # Group 3: median
        r'([\d\.]+\s+(?:GiB|MiB)/s)'         # Group 4: mean
    )

    lines = io.StringIO(data).readlines()
    
    for i, line in enumerate(lines):
        config_match = config_pattern.search(line)
        if config_match:
            data_size = config_match.group(1).strip()
            num_pieces = int(config_match.group(2))
            
            if i + 1 < len(lines):
                throughput_line = lines[i+1]
                tp_match = throughput_pattern.search(throughput_line)
                
                if tp_match:
                    median_throughput_str = tp_match.group(3).strip()
                    
                    # Convert value to GiB/s
                    if "MiB/s" in median_throughput_str:
                        value = float(median_throughput_str.replace("MiB/s", "").strip())
                        median_throughput_gib = value / 1024
                    else:
                        value = float(median_throughput_str.replace("GiB/s", "").strip())
                        median_throughput_gib = value
                        
                    results[data_size].append((num_pieces, median_throughput_gib))

    # --- Plotting Section ---
    fig, ax = plt.subplots(figsize=(12, 7))

    for data_size, values in sorted(results.items()):
        values.sort()
        pieces = [v[0] for v in values]
        throughputs = [v[1] for v in values]
        ax.plot(pieces, throughputs, marker='o', linestyle='-', label=f'{data_size} data')

    # --- Formatting the Plot ---
    ax.set_xscale('log', base=2)
    ax.xaxis.set_major_formatter(mticker.ScalarFormatter())
    ax.set_xticks([4, 8, 16, 32, 64, 128, 256, 512])

    ax.set_title('RLNC Decoder Median Throughput vs. Number of Pieces', fontsize=16)
    ax.set_xlabel('Number of Pieces (log scale)', fontsize=12)
    ax.set_ylabel('Median Throughput (GiB/s)', fontsize=12)

    ax.legend(title='Total Data Size')
    ax.grid(True, which='both', linestyle='--', linewidth=0.5)

    plt.tight_layout()

    # --- Save the plot to a file ---
    plt.savefig(output_filename, dpi=300)
    plt.close()

    print(f"Plot successfully saved to {output_filename}")

if __name__ == '__main__':
    # Run the function to generate and save the image
    output_filename = sys.argv.pop() if len(sys.argv) == 2 else "rlnc_decoder_median_throughput.png"
    parse_and_save_plot(benchmark_data, output_filename)
