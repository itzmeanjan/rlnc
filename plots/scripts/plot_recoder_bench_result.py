#!/usr/bin/python

import sys
import re
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from collections import defaultdict
import io

# Store the recoder benchmark results in a multiline string
benchmark_data = """
Timer precision: 15 ns
full_rlnc_recoder                                                    fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ recode                                                                          │               │               │               │         │
   ├─ 1.00 MB data split into 4 pieces, recoding with 2 pieces       26.57 µs      │ 53.18 µs      │ 29.96 µs      │ 30.02 µs      │ 100     │ 100
   │                                                                 27.56 GiB/s   │ 13.77 GiB/s   │ 24.44 GiB/s   │ 24.39 GiB/s   │         │
   ├─ 1.00 MB data split into 8 pieces, recoding with 4 pieces       19.69 µs      │ 38.5 µs       │ 25.16 µs      │ 25.32 µs      │ 100     │ 100
   │                                                                 30.99 GiB/s   │ 15.85 GiB/s   │ 24.25 GiB/s   │ 24.1 GiB/s    │         │
   ├─ 1.00 MB data split into 16 pieces, recoding with 8 pieces      17.68 µs      │ 24.04 µs      │ 20.36 µs      │ 21.33 µs      │ 100     │ 100
   │                                                                 31.06 GiB/s   │ 22.84 GiB/s   │ 26.97 GiB/s   │ 25.74 GiB/s   │         │
   ├─ 1.00 MB data split into 32 pieces, recoding with 16 pieces     18.58 µs      │ 29.34 µs      │ 22.68 µs      │ 21.93 µs      │ 100     │ 100
   │                                                                 27.94 GiB/s   │ 17.69 GiB/s   │ 22.88 GiB/s   │ 23.67 GiB/s   │         │
   ├─ 1.00 MB data split into 64 pieces, recoding with 32 pieces     18.44 µs      │ 23.64 µs      │ 19.41 µs      │ 19.41 µs      │ 100     │ 100
   │                                                                 27.4 GiB/s    │ 21.38 GiB/s   │ 26.03 GiB/s   │ 26.03 GiB/s   │         │
   ├─ 1.00 MB data split into 128 pieces, recoding with 64 pieces    26.66 µs      │ 31.24 µs      │ 27.85 µs      │ 27.96 µs      │ 100     │ 100
   │                                                                 18.88 GiB/s   │ 16.12 GiB/s   │ 18.08 GiB/s   │ 18.01 GiB/s   │         │
   ├─ 1.00 MB data split into 256 pieces, recoding with 128 pieces   54.14 µs      │ 175.9 µs      │ 55.88 µs      │ 57.63 µs      │ 100     │ 100
   │                                                                 9.659 GiB/s   │ 2.972 GiB/s   │ 9.358 GiB/s   │ 9.074 GiB/s   │         │
   ├─ 1.00 MB data split into 512 pieces, recoding with 256 pieces   184.3 µs      │ 263 µs        │ 189.9 µs      │ 191.2 µs      │ 100     │ 100
   │                                                                 3.325 GiB/s   │ 2.329 GiB/s   │ 3.227 GiB/s   │ 3.205 GiB/s   │         │
   ├─ 4.00 MB data split into 4 pieces, recoding with 2 pieces       91.55 µs      │ 218.4 µs      │ 143.9 µs      │ 151.7 µs      │ 100     │ 100
   │                                                                 31.99 GiB/s   │ 13.41 GiB/s   │ 20.34 GiB/s   │ 19.31 GiB/s   │         │
   ├─ 4.00 MB data split into 8 pieces, recoding with 4 pieces       87.03 µs      │ 202.7 µs      │ 109 µs        │ 110.6 µs      │ 100     │ 100
   │                                                                 28.05 GiB/s   │ 12.04 GiB/s   │ 22.39 GiB/s   │ 22.06 GiB/s   │         │
   ├─ 4.00 MB data split into 16 pieces, recoding with 8 pieces      74.56 µs      │ 146.2 µs      │ 84.93 µs      │ 86.94 µs      │ 100     │ 100
   │                                                                 29.46 GiB/s   │ 15.02 GiB/s   │ 25.87 GiB/s   │ 25.27 GiB/s   │         │
   ├─ 4.00 MB data split into 32 pieces, recoding with 16 pieces     68.66 µs      │ 133 µs        │ 75.75 µs      │ 78.11 µs      │ 100     │ 100
   │                                                                 30.22 GiB/s   │ 15.6 GiB/s    │ 27.39 GiB/s   │ 26.57 GiB/s   │         │
   ├─ 4.00 MB data split into 64 pieces, recoding with 32 pieces     70.78 µs      │ 108.5 µs      │ 89.05 µs      │ 87.17 µs      │ 100     │ 100
   │                                                                 28.48 GiB/s   │ 18.57 GiB/s   │ 22.63 GiB/s   │ 23.12 GiB/s   │         │
   ├─ 4.00 MB data split into 128 pieces, recoding with 64 pieces    76.74 µs      │ 104.7 µs      │ 85.82 µs      │ 86.26 µs      │ 100     │ 100
   │                                                                 25.95 GiB/s   │ 19.01 GiB/s   │ 23.2 GiB/s    │ 23.08 GiB/s   │         │
   ├─ 4.00 MB data split into 256 pieces, recoding with 128 pieces   98.12 µs      │ 114.6 µs      │ 101.2 µs      │ 103.8 µs      │ 100     │ 100
   │                                                                 20.37 GiB/s   │ 17.44 GiB/s   │ 19.75 GiB/s   │ 19.25 GiB/s   │         │
   ├─ 4.00 MB data split into 512 pieces, recoding with 256 pieces   230.5 µs      │ 252.9 µs      │ 237.1 µs      │ 238.7 µs      │ 100     │ 100
   │                                                                 9.037 GiB/s   │ 8.237 GiB/s   │ 8.787 GiB/s   │ 8.727 GiB/s   │         │
   ├─ 8.00 MB data split into 4 pieces, recoding with 2 pieces       313.7 µs      │ 429.3 µs      │ 330 µs        │ 331.4 µs      │ 100     │ 100
   │                                                                 18.67 GiB/s   │ 13.64 GiB/s   │ 17.75 GiB/s   │ 17.67 GiB/s   │         │
   ├─ 8.00 MB data split into 8 pieces, recoding with 4 pieces       203.9 µs      │ 360 µs        │ 261.6 µs      │ 267.3 µs      │ 100     │ 100
   │                                                                 23.94 GiB/s   │ 13.56 GiB/s   │ 18.66 GiB/s   │ 18.26 GiB/s   │         │
   ├─ 8.00 MB data split into 16 pieces, recoding with 8 pieces      158.1 µs      │ 210.9 µs      │ 183.1 µs      │ 184.2 µs      │ 100     │ 100
   │                                                                 27.78 GiB/s   │ 20.83 GiB/s   │ 23.98 GiB/s   │ 23.85 GiB/s   │         │
   ├─ 8.00 MB data split into 32 pieces, recoding with 16 pieces     141.8 µs      │ 218.4 µs      │ 181.7 µs      │ 174.4 µs      │ 100     │ 100
   │                                                                 29.26 GiB/s   │ 19 GiB/s      │ 22.84 GiB/s   │ 23.79 GiB/s   │         │
   ├─ 8.00 MB data split into 64 pieces, recoding with 32 pieces     141.1 µs      │ 223 µs        │ 178.4 µs      │ 171.3 µs      │ 100     │ 100
   │                                                                 28.55 GiB/s   │ 18.06 GiB/s   │ 22.58 GiB/s   │ 23.52 GiB/s   │         │
   ├─ 8.00 MB data split into 128 pieces, recoding with 64 pieces    149.3 µs      │ 189.8 µs      │ 155 µs        │ 163.4 µs      │ 100     │ 100
   │                                                                 26.61 GiB/s   │ 20.93 GiB/s   │ 25.63 GiB/s   │ 24.31 GiB/s   │         │
   ├─ 8.00 MB data split into 256 pieces, recoding with 128 pieces   178 µs        │ 261.4 µs      │ 193.9 µs      │ 198 µs        │ 100     │ 100
   │                                                                 22.28 GiB/s   │ 15.17 GiB/s   │ 20.46 GiB/s   │ 20.03 GiB/s   │         │
   ├─ 8.00 MB data split into 512 pieces, recoding with 256 pieces   309.5 µs      │ 513.9 µs      │ 323.8 µs      │ 339.7 µs      │ 100     │ 100
   │                                                                 13.06 GiB/s   │ 7.868 GiB/s   │ 12.48 GiB/s   │ 11.9 GiB/s    │         │
   ├─ 16.00 MB data split into 4 pieces, recoding with 2 pieces      837.9 µs      │ 1.033 ms      │ 932.3 µs      │ 937.6 µs      │ 100     │ 100
   │                                                                 13.98 GiB/s   │ 11.34 GiB/s   │ 12.56 GiB/s   │ 12.49 GiB/s   │         │
   ├─ 16.00 MB data split into 8 pieces, recoding with 4 pieces      577 µs        │ 953.2 µs      │ 634.5 µs      │ 672.3 µs      │ 100     │ 100
   │                                                                 16.92 GiB/s   │ 10.24 GiB/s   │ 15.39 GiB/s   │ 14.52 GiB/s   │         │
   ├─ 16.00 MB data split into 16 pieces, recoding with 8 pieces     441.5 µs      │ 665 µs        │ 511.4 µs      │ 514.6 µs      │ 100     │ 100
   │                                                                 19.9 GiB/s    │ 13.21 GiB/s   │ 17.18 GiB/s   │ 17.07 GiB/s   │         │
   ├─ 16.00 MB data split into 32 pieces, recoding with 16 pieces    414.6 µs      │ 515.6 µs      │ 467.1 µs      │ 456.9 µs      │ 100     │ 100
   │                                                                 20.02 GiB/s   │ 16.09 GiB/s   │ 17.76 GiB/s   │ 18.16 GiB/s   │         │
   ├─ 16.00 MB data split into 64 pieces, recoding with 32 pieces    365.1 µs      │ 764.5 µs      │ 435.6 µs      │ 431.3 µs      │ 100     │ 100
   │                                                                 22.06 GiB/s   │ 10.53 GiB/s   │ 18.49 GiB/s   │ 18.68 GiB/s   │         │
   ├─ 16.00 MB data split into 128 pieces, recoding with 64 pieces   369.5 µs      │ 472.5 µs      │ 394 µs        │ 405.4 µs      │ 100     │ 100
   │                                                                 21.48 GiB/s   │ 16.8 GiB/s    │ 20.15 GiB/s   │ 19.58 GiB/s   │         │
   ├─ 16.00 MB data split into 256 pieces, recoding with 128 pieces  389.1 µs      │ 641.8 µs      │ 455.7 µs      │ 459.2 µs      │ 100     │ 100
   │                                                                 20.31 GiB/s   │ 12.31 GiB/s   │ 17.34 GiB/s   │ 17.21 GiB/s   │         │
   ├─ 16.00 MB data split into 512 pieces, recoding with 256 pieces  527.6 µs      │ 656.8 µs      │ 562 µs        │ 571.2 µs      │ 100     │ 100
   │                                                                 15.09 GiB/s   │ 12.12 GiB/s   │ 14.17 GiB/s   │ 13.94 GiB/s   │         │
   ├─ 32.00 MB data split into 4 pieces, recoding with 2 pieces      1.298 ms      │ 3.089 ms      │ 1.882 ms      │ 1.91 ms       │ 100     │ 100
   │                                                                 18.04 GiB/s   │ 7.584 GiB/s   │ 12.45 GiB/s   │ 12.26 GiB/s   │         │
   ├─ 32.00 MB data split into 8 pieces, recoding with 4 pieces      1.175 ms      │ 1.79 ms       │ 1.474 ms      │ 1.473 ms      │ 100     │ 100
   │                                                                 16.61 GiB/s   │ 10.9 GiB/s    │ 13.24 GiB/s   │ 13.25 GiB/s   │         │
   ├─ 32.00 MB data split into 16 pieces, recoding with 8 pieces     1.14 ms       │ 1.902 ms      │ 1.275 ms      │ 1.286 ms      │ 100     │ 100
   │                                                                 15.41 GiB/s   │ 9.24 GiB/s    │ 13.78 GiB/s   │ 13.66 GiB/s   │         │
   ├─ 32.00 MB data split into 32 pieces, recoding with 16 pieces    1.109 ms      │ 1.432 ms      │ 1.196 ms      │ 1.201 ms      │ 100     │ 100
   │                                                                 14.96 GiB/s   │ 11.59 GiB/s   │ 13.87 GiB/s   │ 13.81 GiB/s   │         │
   ├─ 32.00 MB data split into 64 pieces, recoding with 32 pieces    1.021 ms      │ 1.584 ms      │ 1.098 ms      │ 1.095 ms      │ 100     │ 100
   │                                                                 15.77 GiB/s   │ 10.16 GiB/s   │ 14.67 GiB/s   │ 14.7 GiB/s    │         │
   ├─ 32.00 MB data split into 128 pieces, recoding with 64 pieces   966.9 µs      │ 1.225 ms      │ 1.019 ms      │ 1.033 ms      │ 100     │ 100
   │                                                                 16.41 GiB/s   │ 12.95 GiB/s   │ 15.57 GiB/s   │ 15.36 GiB/s   │         │
   ├─ 32.00 MB data split into 256 pieces, recoding with 128 pieces  1.001 ms      │ 1.523 ms      │ 1.036 ms      │ 1.059 ms      │ 100     │ 100
   │                                                                 15.75 GiB/s   │ 10.35 GiB/s   │ 15.22 GiB/s   │ 14.89 GiB/s   │         │
   ├─ 32.00 MB data split into 512 pieces, recoding with 256 pieces  1.17 ms       │ 1.779 ms      │ 1.38 ms       │ 1.392 ms      │ 100     │ 100
   │                                                                 13.51 GiB/s   │ 8.883 GiB/s   │ 11.45 GiB/s   │ 11.35 GiB/s   │         │
   ├─ 64.00 MB data split into 4 pieces, recoding with 2 pieces      2.351 ms      │ 5.198 ms      │ 4.117 ms      │ 4.112 ms      │ 100     │ 100
   │                                                                 19.93 GiB/s   │ 9.017 GiB/s   │ 11.38 GiB/s   │ 11.39 GiB/s   │         │
   ├─ 64.00 MB data split into 8 pieces, recoding with 4 pieces      3.118 ms      │ 3.636 ms      │ 3.17 ms       │ 3.189 ms      │ 100     │ 100
   │                                                                 12.52 GiB/s   │ 10.74 GiB/s   │ 12.32 GiB/s   │ 12.24 GiB/s   │         │
   ├─ 64.00 MB data split into 16 pieces, recoding with 8 pieces     2.528 ms      │ 4.148 ms      │ 3.158 ms      │ 3.12 ms       │ 100     │ 100
   │                                                                 13.9 GiB/s    │ 8.474 GiB/s   │ 11.12 GiB/s   │ 11.26 GiB/s   │         │
   ├─ 64.00 MB data split into 32 pieces, recoding with 16 pieces    2.791 ms      │ 3.714 ms      │ 3.007 ms      │ 2.997 ms      │ 100     │ 100
   │                                                                 11.89 GiB/s   │ 8.937 GiB/s   │ 11.04 GiB/s   │ 11.07 GiB/s   │         │
   ├─ 64.00 MB data split into 64 pieces, recoding with 32 pieces    2.613 ms      │ 3.098 ms      │ 2.859 ms      │ 2.839 ms      │ 100     │ 100
   │                                                                 12.33 GiB/s   │ 10.4 GiB/s    │ 11.27 GiB/s   │ 11.34 GiB/s   │         │
   ├─ 64.00 MB data split into 128 pieces, recoding with 64 pieces   2.282 ms      │ 2.79 ms       │ 2.545 ms      │ 2.569 ms      │ 100     │ 100
   │                                                                 13.9 GiB/s    │ 11.37 GiB/s   │ 12.46 GiB/s   │ 12.35 GiB/s   │         │
   ├─ 64.00 MB data split into 256 pieces, recoding with 128 pieces  1.92 ms       │ 2.301 ms      │ 1.991 ms      │ 2.014 ms      │ 100     │ 100
   │                                                                 16.41 GiB/s   │ 13.69 GiB/s   │ 15.82 GiB/s   │ 15.64 GiB/s   │         │
   ╰─ 64.00 MB data split into 512 pieces, recoding with 256 pieces  2.052 ms      │ 2.292 ms      │ 2.108 ms      │ 2.134 ms      │ 100     │ 100
                                                                     15.34 GiB/s   │ 13.73 GiB/s   │ 14.93 GiB/s   │ 14.75 GiB/s   │         │
"""

def parse_and_save_plot(data: str, output_filename: str):
    """
    Parses the recoder benchmark data and saves the median throughput plot to a file.
    """
    results = defaultdict(list)
    
    # Updated regex to match the recoder's specific output format
    config_pattern = re.compile(r'([\d\.]+\s*(?:MB|GB|KB))\s+data split into\s+(\d+)\s+pieces,')
    
    # Regex to capture all four throughput columns remains the same
    throughput_pattern = re.compile(
        r'([\d\.]+\s+GiB/s)\s+│\s+'  # Group 1: fastest
        r'([\d\.]+\s+GiB/s)\s+│\s+'  # Group 2: slowest
        r'([\d\.]+\s+GiB/s)\s+│\s+'  # Group 3: median
        r'([\d\.]+\s+GiB/s)'         # Group 4: mean
    )

    lines = io.StringIO(data).readlines()
    
    for i, line in enumerate(lines):
        config_match = config_pattern.search(line)
        if config_match:
            data_size = config_match.group(1).strip()
            # The x-axis is the number of pieces the data was split into
            num_pieces = int(config_match.group(2))
            
            if i + 1 < len(lines):
                throughput_line = lines[i+1]
                tp_match = throughput_pattern.search(throughput_line)
                
                # Check if the pattern matched and extract the median value (Group 3)
                if tp_match:
                    median_throughput_str = tp_match.group(3).replace('GiB/s', '').strip()
                    median_throughput = float(median_throughput_str)
                    results[data_size].append((num_pieces, median_throughput))

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

    # Updated title for the recoder plot
    ax.set_title('RLNC Recoder Median Throughput vs. Number of Split Pieces', fontsize=16)
    ax.set_xlabel('Number of Split Pieces (log scale)', fontsize=12)
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
    output_filename = sys.argv.pop() if len(sys.argv) == 2 else "rlnc_recoder_median_throughput.png"
    parse_and_save_plot(benchmark_data, output_filename)
