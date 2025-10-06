import json
import matplotlib.pyplot as plt
import collections
import argparse
import re


def parse_and_group_data(
    file_path: str, unit: str
) -> dict[str, dict[str, dict[str, list[int | float]]]] | None:
    """
    Parses a JSON Lines file, groups data by group_name (method), then by size, and calculates throughput.

    Args:
        file_path (str): The path to the input .jsonl file.
        unit (str): The desired output unit ('MB/s' or 'GB/s').

    Returns:
        dict: A dictionary grouped by group_name, then by size, with sorted benchmark data.
    """
    grouped_data = collections.defaultdict(lambda: collections.defaultdict(list))

    try:
        with open(file_path, "r") as f:
            for line in f:
                if not line.strip():
                    continue
                try:
                    data = json.loads(line)
                    id_parts = data.get("id", "").split("/")
                    if len(id_parts) != 3:
                        continue

                    group_name = id_parts[0]
                    size_str = id_parts[1]
                    pieces_str = id_parts[2]

                    # Extract size and pieces using regex
                    size_match = re.match(r"([\d.]+)MB", size_str)
                    pieces_match = re.match(r"(\d+)-pieces", pieces_str)
                    if not size_match or not pieces_match:
                        continue

                    size = size_match.group(1)  # e.g., '1.0', '16.0', '32.0'
                    pieces = int(pieces_match.group(1))

                    # --- Throughput Calculation ---
                    bytes_processed = data["throughput"][0]["per_iteration"]
                    time_ns = data["typical"]["estimate"]

                    if time_ns > 0:
                        bytes_per_second = bytes_processed / (time_ns * 1e-9)
                        if unit == "MB/s":
                            throughput = bytes_per_second / (1024**2)
                        elif unit == "GB/s":
                            throughput = bytes_per_second / (1024**3)
                        else:
                            throughput = 0  # Should not happen with arg choices
                    else:
                        throughput = 0

                    grouped_data[group_name][size].append((pieces, throughput))

                except (json.JSONDecodeError, KeyError, IndexError, ValueError):
                    print(
                        f"Warning: Skipping malformed or incomplete record: {line.strip()}"
                    )

    except FileNotFoundError:
        print(f"Error: The file '{file_path}' was not found.")
        return None

    # --- Sort the data within each group ---
    sorted_grouped_data = {}
    for group_name, sizes_data in grouped_data.items():
        sorted_grouped_data[group_name] = {}
        # Sort sizes numerically
        for size in sorted(sizes_data, key=float):
            data_list = sizes_data[size]
            # Sort by pieces (integer)
            data_list.sort(key=lambda x: x[0])

            # Unzip the sorted data back into separate lists
            pieces, throughputs = zip(*data_list)
            sorted_grouped_data[group_name][size] = {
                "pieces": list(pieces),
                "throughputs": list(throughputs),
            }

    return sorted_grouped_data


def generate_plot(
    group_name: str,
    data_dict: dict[str, dict[str, list[int | float]]],
    unit: str,
    title_format: str,
):
    """
    Generates and saves a single line plot for a group_name, with lines for each size.

    Args:
        group_name (str): The group_name of the plot (e.g., 'encode').
        data_dict (dict): Dictionary with sizes as keys and {'pieces': list, 'throughputs': list} as values.
        unit (str): The unit for the y-axis label.
        title_format (str): The format string for the title.
    """
    if not data_dict:
        print(f"Skipping plot for '{group_name}' due to no data.")
        return

    plt.figure(figsize=(12, 7))

    # --- Create the Line Plots for each size ---
    for size, data in data_dict.items():
        if not data["pieces"] or not data["throughputs"]:
            continue
        plt.plot(
            data["pieces"],
            data["throughputs"],
            marker="o",
            linestyle="-",
            label=f"{size}MB",
        )

    # --- Plot Formatting ---
    plt.ylabel(f"Throughput ({unit})", fontsize=12)
    plt.xlabel("Number of Pieces", fontsize=12)
    plt.title(
        title_format.format(group_name=group_name), fontsize=16, fontweight="bold"
    )
    plt.grid(True, which="both", linestyle="--", linewidth=0.5)
    plt.legend()
    plt.tight_layout()

    # --- Save the Plot ---
    output_filename = f"benchmark_{group_name}.png"
    plt.savefig(output_filename)
    plt.close()

    print(f"✅ Successfully generated plot: {output_filename}")


def main():
    """Main function to drive the script."""
    parser = argparse.ArgumentParser(
        description="Parse a benchmark JSON Lines file and generate performance line plots grouped by group_name."
    )
    parser.add_argument(
        "filepath", type=str, help="Path to the newline-delimited JSON file (.jsonl)."
    )
    parser.add_argument(
        "-u",
        "--unit",
        type=str,
        default="MB/s",
        choices=["MB/s", "GB/s"],
        help="The unit for throughput on the y-axis (default: MB/s).",
    )
    parser.add_argument(
        "--title-format",
        type=str,
        default="Benchmark Results for: {group_name}",
        help="The format string for the plot title, with optional {group_name} placeholder (default: 'Benchmark Results for: {group_name}').",
    )
    args = parser.parse_args()

    grouped_data = parse_and_group_data(args.filepath, args.unit)

    if not grouped_data:
        print("No data was parsed. Exiting.")
        return

    for group_name in grouped_data:
        generate_plot(
            group_name, grouped_data[group_name], args.unit, args.title_format
        )


if __name__ == "__main__":
    main()
