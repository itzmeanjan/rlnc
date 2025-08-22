#!/usr/bin/python

import sys
import re
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from collections import defaultdict
import io

# Store the benchmark results in a multiline string
benchmark_data = """
Timer precision: 23 ns
full_rlnc_encoder                          fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ encode                                                │               │               │               │         │
   ├─ 1.00 MB data split into 4 pieces     41.67 µs      │ 186.9 µs      │ 52.1 µs       │ 53.55 µs      │ 100     │ 100
   │                                       29.29 GiB/s   │ 6.53 GiB/s    │ 23.42 GiB/s   │ 22.79 GiB/s   │         │
   ├─ 1.00 MB data split into 8 pieces     35.05 µs      │ 83.2 µs       │ 47.94 µs      │ 47.69 µs      │ 100     │ 100
   │                                       31.34 GiB/s   │ 13.2 GiB/s    │ 22.91 GiB/s   │ 23.03 GiB/s   │         │
   ├─ 1.00 MB data split into 16 pieces    38.21 µs      │ 47.02 µs      │ 44.66 µs      │ 44.45 µs      │ 100     │ 100
   │                                       27.15 GiB/s   │ 22.06 GiB/s   │ 23.22 GiB/s   │ 23.34 GiB/s   │         │
   ├─ 1.00 MB data split into 32 pieces    34.92 µs      │ 46.84 µs      │ 37.24 µs      │ 37.38 µs      │ 100     │ 100
   │                                       28.83 GiB/s   │ 21.5 GiB/s    │ 27.03 GiB/s   │ 26.93 GiB/s   │         │
   ├─ 1.00 MB data split into 64 pieces    32.41 µs      │ 45.69 µs      │ 33.87 µs      │ 34.45 µs      │ 100     │ 100
   │                                       30.6 GiB/s    │ 21.7 GiB/s    │ 29.28 GiB/s   │ 28.78 GiB/s   │         │
   ├─ 1.00 MB data split into 128 pieces   33.37 µs      │ 60.31 µs      │ 34.88 µs      │ 35.59 µs      │ 100     │ 100
   │                                       29.49 GiB/s   │ 16.32 GiB/s   │ 28.22 GiB/s   │ 27.65 GiB/s   │         │
   ├─ 1.00 MB data split into 256 pieces   33.5 µs       │ 41.11 µs      │ 35.54 µs      │ 35.63 µs      │ 100     │ 100
   │                                       29.27 GiB/s   │ 23.85 GiB/s   │ 27.59 GiB/s   │ 27.52 GiB/s   │         │
   ├─ 1.00 MB data split into 512 pieces   35.53 µs      │ 43.74 µs      │ 36.95 µs      │ 37.08 µs      │ 100     │ 100
   │                                       27.56 GiB/s   │ 22.38 GiB/s   │ 26.5 GiB/s    │ 26.4 GiB/s    │         │
   ├─ 4.00 MB data split into 4 pieces     193 µs        │ 704.3 µs      │ 251.4 µs      │ 282 µs        │ 100     │ 100
   │                                       25.29 GiB/s   │ 6.932 GiB/s   │ 19.41 GiB/s   │ 17.31 GiB/s   │         │
   ├─ 4.00 MB data split into 8 pieces     197.5 µs      │ 1.144 ms      │ 211.6 µs      │ 236.5 µs      │ 100     │ 100
   │                                       22.24 GiB/s   │ 3.839 GiB/s   │ 20.76 GiB/s   │ 18.57 GiB/s   │         │
   ├─ 4.00 MB data split into 16 pieces    171.6 µs      │ 202.3 µs      │ 184.1 µs      │ 184 µs        │ 100     │ 100
   │                                       24.18 GiB/s   │ 20.51 GiB/s   │ 22.53 GiB/s   │ 22.54 GiB/s   │         │
   ├─ 4.00 MB data split into 32 pieces    138.3 µs      │ 820.9 µs      │ 150.2 µs      │ 168.4 µs      │ 100     │ 100
   │                                       29.12 GiB/s   │ 4.907 GiB/s   │ 26.81 GiB/s   │ 23.92 GiB/s   │         │
   ├─ 4.00 MB data split into 64 pieces    141.5 µs      │ 584.8 µs      │ 148.4 µs      │ 159.9 µs      │ 100     │ 100
   │                                       28.03 GiB/s   │ 6.783 GiB/s   │ 26.72 GiB/s   │ 24.8 GiB/s    │         │
   ├─ 4.00 MB data split into 128 pieces   136.8 µs      │ 873.7 µs      │ 146.1 µs      │ 163.5 µs      │ 100     │ 100
   │                                       28.76 GiB/s   │ 4.505 GiB/s   │ 26.94 GiB/s   │ 24.07 GiB/s   │         │
   ├─ 4.00 MB data split into 256 pieces   130.6 µs      │ 731.2 µs      │ 134.8 µs      │ 151 µs        │ 100     │ 100
   │                                       30.01 GiB/s   │ 5.363 GiB/s   │ 29.09 GiB/s   │ 25.97 GiB/s   │         │
   ├─ 4.00 MB data split into 512 pieces   131 µs        │ 256.9 µs      │ 135.8 µs      │ 141.7 µs      │ 100     │ 100
   │                                       29.87 GiB/s   │ 15.23 GiB/s   │ 28.82 GiB/s   │ 27.62 GiB/s   │         │
   ├─ 8.00 MB data split into 4 pieces     430.7 µs      │ 1.895 ms      │ 554.2 µs      │ 611.7 µs      │ 100     │ 100
   │                                       22.67 GiB/s   │ 5.152 GiB/s   │ 17.62 GiB/s   │ 15.96 GiB/s   │         │
   ├─ 8.00 MB data split into 8 pieces     396.4 µs      │ 1.079 ms      │ 459.7 µs      │ 507.9 µs      │ 100     │ 100
   │                                       22.17 GiB/s   │ 8.138 GiB/s   │ 19.11 GiB/s   │ 17.3 GiB/s    │         │
   ├─ 8.00 MB data split into 16 pieces    354.3 µs      │ 814.7 µs      │ 382.4 µs      │ 425.7 µs      │ 100     │ 100
   │                                       23.42 GiB/s   │ 10.18 GiB/s   │ 21.7 GiB/s    │ 19.49 GiB/s   │         │
   ├─ 8.00 MB data split into 32 pieces    287.6 µs      │ 802.1 µs      │ 303.2 µs      │ 360.1 µs      │ 100     │ 100
   │                                       28 GiB/s      │ 10.04 GiB/s   │ 26.56 GiB/s   │ 22.36 GiB/s   │         │
   ├─ 8.00 MB data split into 64 pieces    279.9 µs      │ 784.5 µs      │ 307.7 µs      │ 357.5 µs      │ 100     │ 100
   │                                       28.34 GiB/s   │ 10.11 GiB/s   │ 25.78 GiB/s   │ 22.19 GiB/s   │         │
   ├─ 8.00 MB data split into 128 pieces   286.2 µs      │ 756.6 µs      │ 301.8 µs      │ 355.2 µs      │ 100     │ 100
   │                                       27.5 GiB/s    │ 10.4 GiB/s    │ 26.08 GiB/s   │ 22.16 GiB/s   │         │
   ├─ 8.00 MB data split into 256 pieces   284.4 µs      │ 738.9 µs      │ 299 µs        │ 346.6 µs      │ 100     │ 100
   │                                       27.57 GiB/s   │ 10.61 GiB/s   │ 26.22 GiB/s   │ 22.62 GiB/s   │         │
   ├─ 8.00 MB data split into 512 pieces   271.5 µs      │ 766.7 µs      │ 285.9 µs      │ 335.3 µs      │ 100     │ 100
   │                                       28.82 GiB/s   │ 10.21 GiB/s   │ 27.37 GiB/s   │ 23.34 GiB/s   │         │
   ├─ 16.00 MB data split into 4 pieces    1.256 ms      │ 1.949 ms      │ 1.28 ms       │ 1.327 ms      │ 100     │ 100
   │                                       15.53 GiB/s   │ 10.02 GiB/s   │ 15.25 GiB/s   │ 14.71 GiB/s   │         │
   ├─ 16.00 MB data split into 8 pieces    993.6 µs      │ 1.681 ms      │ 1.137 ms      │ 1.152 ms      │ 100     │ 100
   │                                       17.69 GiB/s   │ 10.45 GiB/s   │ 15.46 GiB/s   │ 15.25 GiB/s   │         │
   ├─ 16.00 MB data split into 16 pieces   943.1 µs      │ 1.515 ms      │ 1.009 ms      │ 1.037 ms      │ 100     │ 100
   │                                       17.6 GiB/s    │ 10.95 GiB/s   │ 16.43 GiB/s   │ 15.99 GiB/s   │         │
   ├─ 16.00 MB data split into 32 pieces   918.7 µs      │ 1.503 ms      │ 964.5 µs      │ 996.1 µs      │ 100     │ 100
   │                                       17.53 GiB/s   │ 10.71 GiB/s   │ 16.7 GiB/s    │ 16.17 GiB/s   │         │
   ├─ 16.00 MB data split into 64 pieces   890.6 µs      │ 1.273 ms      │ 924.4 µs      │ 943.2 µs      │ 100     │ 100
   │                                       17.81 GiB/s   │ 12.46 GiB/s   │ 17.16 GiB/s   │ 16.82 GiB/s   │         │
   ├─ 16.00 MB data split into 128 pieces  893.9 µs      │ 1.474 ms      │ 920 µs        │ 974.1 µs      │ 100     │ 100
   │                                       17.61 GiB/s   │ 10.67 GiB/s   │ 17.11 GiB/s   │ 16.16 GiB/s   │         │
   ├─ 16.00 MB data split into 256 pieces  895.6 µs      │ 1.242 ms      │ 932.7 µs      │ 964.4 µs      │ 100     │ 100
   │                                       17.51 GiB/s   │ 12.62 GiB/s   │ 16.81 GiB/s   │ 16.26 GiB/s   │         │
   ├─ 16.00 MB data split into 512 pieces  883.9 µs      │ 1.247 ms      │ 911.1 µs      │ 928.8 µs      │ 100     │ 100
   │                                       17.71 GiB/s   │ 12.54 GiB/s   │ 17.18 GiB/s   │ 16.85 GiB/s   │         │
   ├─ 32.00 MB data split into 4 pieces    2.205 ms      │ 3.882 ms      │ 2.818 ms      │ 2.842 ms      │ 100     │ 100
   │                                       17.7 GiB/s    │ 10.06 GiB/s   │ 13.85 GiB/s   │ 13.74 GiB/s   │         │
   ├─ 32.00 MB data split into 8 pieces    2.233 ms      │ 2.931 ms      │ 2.489 ms      │ 2.492 ms      │ 100     │ 100
   │                                       15.74 GiB/s   │ 11.99 GiB/s   │ 14.12 GiB/s   │ 14.1 GiB/s    │         │
   ├─ 32.00 MB data split into 16 pieces   2.109 ms      │ 3.066 ms      │ 2.308 ms      │ 2.334 ms      │ 100     │ 100
   │                                       15.74 GiB/s   │ 10.82 GiB/s   │ 14.38 GiB/s   │ 14.22 GiB/s   │         │
   ├─ 32.00 MB data split into 32 pieces   2.138 ms      │ 2.405 ms      │ 2.254 ms      │ 2.258 ms      │ 100     │ 100
   │                                       15.06 GiB/s   │ 13.39 GiB/s   │ 14.29 GiB/s   │ 14.27 GiB/s   │         │
   ├─ 32.00 MB data split into 64 pieces   2.036 ms      │ 2.703 ms      │ 2.159 ms      │ 2.184 ms      │ 100     │ 100
   │                                       15.58 GiB/s   │ 11.74 GiB/s   │ 14.69 GiB/s   │ 14.52 GiB/s   │         │
   ├─ 32.00 MB data split into 128 pieces  2.04 ms       │ 2.199 ms      │ 2.108 ms      │ 2.111 ms      │ 100     │ 100
   │                                       15.43 GiB/s   │ 14.32 GiB/s   │ 14.93 GiB/s   │ 14.91 GiB/s   │         │
   ├─ 32.00 MB data split into 256 pieces  2.083 ms      │ 2.278 ms      │ 2.118 ms      │ 2.125 ms      │ 100     │ 100
   │                                       15.06 GiB/s   │ 13.77 GiB/s   │ 14.81 GiB/s   │ 14.76 GiB/s   │         │
   ├─ 32.00 MB data split into 512 pieces  2.009 ms      │ 2.576 ms      │ 2.103 ms      │ 2.113 ms      │ 100     │ 100
   │                                       15.58 GiB/s   │ 12.15 GiB/s   │ 14.88 GiB/s   │ 14.81 GiB/s   │         │
   ├─ 64.00 MB data split into 4 pieces    7.026 ms      │ 8.87 ms       │ 7.725 ms      │ 7.728 ms      │ 100     │ 100
   │                                       11.11 GiB/s   │ 8.806 GiB/s   │ 10.11 GiB/s   │ 10.1 GiB/s    │         │
   ├─ 64.00 MB data split into 8 pieces    4.468 ms      │ 6.341 ms      │ 5.205 ms      │ 5.191 ms      │ 100     │ 100
   │                                       15.73 GiB/s   │ 11.08 GiB/s   │ 13.5 GiB/s    │ 13.54 GiB/s   │         │
   ├─ 64.00 MB data split into 16 pieces   4.367 ms      │ 5.296 ms      │ 4.793 ms      │ 4.78 ms       │ 100     │ 100
   │                                       15.2 GiB/s    │ 12.53 GiB/s   │ 13.85 GiB/s   │ 13.89 GiB/s   │         │
   ├─ 64.00 MB data split into 32 pieces   4.379 ms      │ 4.998 ms      │ 4.757 ms      │ 4.73 ms       │ 100     │ 100
   │                                       14.71 GiB/s   │ 12.89 GiB/s   │ 13.54 GiB/s   │ 13.62 GiB/s   │         │
   ├─ 64.00 MB data split into 64 pieces   4.438 ms      │ 4.861 ms      │ 4.651 ms      │ 4.649 ms      │ 100     │ 100
   │                                       14.3 GiB/s    │ 13.05 GiB/s   │ 13.64 GiB/s   │ 13.65 GiB/s   │         │
   ├─ 64.00 MB data split into 128 pieces  4.125 ms      │ 4.515 ms      │ 4.345 ms      │ 4.343 ms      │ 100     │ 100
   │                                       15.26 GiB/s   │ 13.94 GiB/s   │ 14.49 GiB/s   │ 14.5 GiB/s    │         │
   ├─ 64.00 MB data split into 256 pieces  4 ms          │ 4.499 ms      │ 4.295 ms      │ 4.287 ms      │ 100     │ 100
   │                                       15.68 GiB/s   │ 13.94 GiB/s   │ 14.6 GiB/s    │ 14.63 GiB/s   │         │
   ╰─ 64.00 MB data split into 512 pieces  4.035 ms      │ 5.133 ms      │ 4.26 ms       │ 4.277 ms      │ 100     │ 100
                                           15.51 GiB/s   │ 12.19 GiB/s   │ 14.69 GiB/s   │ 14.63 GiB/s   │         │
"""

def parse_and_save_plot(data: str, output_filename: str):
    """
    Parses the benchmark data and saves the median throughput plot to a file.
    """
    results = defaultdict(list)
    
    config_pattern = re.compile(r'([\d\.]+\s*(?:MB|GB|KB))\s+data split into\s+(\d+)\s+pieces')
    
    throughput_pattern = re.compile(
        r'([\d\.]+\s+GiB/s)\s+│\s+'
        r'([\d\.]+\s+GiB/s)\s+│\s+'
        r'([\d\.]+\s+GiB/s)\s+│\s+'
        r'([\d\.]+\s+GiB/s)'
    )

    lines = io.StringIO(data).readlines()
    
    for i, line in enumerate(lines):
        config_match = config_pattern.search(line)
        if config_match:
            data_size = config_match.group(1)
            num_pieces = int(config_match.group(2))
            
            if i + 1 < len(lines):
                throughput_line = lines[i+1]
                tp_match = throughput_pattern.search(throughput_line)
                
                if tp_match:
                    median_throughput_str = tp_match.group(3).replace('GiB/s', '').strip()
                    median_throughput = float(median_throughput_str)
                    results[data_size].append((num_pieces, median_throughput))

    # --- Plotting Section ---
    _, ax = plt.subplots(figsize=(12, 7))

    for data_size, values in sorted(results.items()):
        values.sort()
        pieces = [v[0] for v in values]
        throughputs = [v[1] for v in values]
        ax.plot(pieces, throughputs, marker='o', linestyle='-', label=f'{data_size} data')

    # --- Formatting the Plot ---
    ax.set_xscale('log', base=2)
    ax.xaxis.set_major_formatter(mticker.ScalarFormatter())
    ax.set_xticks([4, 8, 16, 32, 64, 128, 256, 512])

    ax.set_title('RLNC Encoder Median Throughput vs. Number of Pieces', fontsize=16)
    ax.set_xlabel('Number of Pieces (log scale)', fontsize=12)
    ax.set_ylabel('Median Throughput (GiB/s)', fontsize=12)

    ax.legend(title='Total Data Size')
    ax.grid(True, which='both', linestyle='--', linewidth=0.5)

    plt.tight_layout()

    # --- Save the plot to a file instead of showing it ---
    plt.savefig(output_filename, dpi=300) # dpi for higher resolution
    plt.close() # Close the figure to free up memory

    print(f"Plot successfully saved to {output_filename}")

if __name__ == '__main__':
    # Run the function to generate and save the image
    output_filename = sys.argv.pop() if len(sys.argv) == 2 else "rlnc_encoder_median_throughput.png"
    parse_and_save_plot(benchmark_data, output_filename)
