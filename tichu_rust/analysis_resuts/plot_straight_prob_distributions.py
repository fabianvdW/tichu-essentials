import numpy as np
import matplotlib.pyplot as plt

# Data from the problem
counted_straight_lengths = [1271348424, 221667695352, 1089094274744, 1494123765200, 1224371890896, 804372256512,
                            474483929984, 262784360448, 135044874240, 62738923520, 25198854144, 8002732032, 1577058304]
at_least_counted_straight_lenghts = np.cumsum(counted_straight_lengths[::-1])[::-1]
counted_straight_flush_lengths = [170226064036, 3231369839200, 1886228484214, 424635276516, 77497737660, 12663862404,
                                  1846862262, 235769688, 25627140, 2277968, 155316, 7224, 172]
at_least_counted_straight_flush_lenghts = np.cumsum(counted_straight_flush_lengths[::-1])[::-1]

# Total number of hands
total_hands = 5804731963800

# Convert counts to probabilities
prob_straight_lengths = np.array(counted_straight_lengths) / total_hands
prob_straight_flush_lengths = np.array(counted_straight_flush_lengths) / total_hands

prob_at_least_straight = np.cumsum(prob_straight_lengths[::-1])[::-1]
prob_at_least_straight_flush = np.cumsum(prob_straight_flush_lengths[::-1])[::-1]

# X-axis values (starting from 1)
x_indices = np.arange(1, len(prob_at_least_straight) + 1)

# Bar width
width = 0.35

# Generate combined plot with linear scale
fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(14, 12))

# Plot 1: Exact hitting probabilities (linear scale)
ax1.bar(x_indices - width / 2, prob_straight_lengths, width, label='Straight (exact length)')
ax1.bar(x_indices + width / 2, prob_straight_flush_lengths, width, label='Straight Flush (exact length)')
ax1.set_xlabel('Length of Straight')
ax1.set_ylabel('Probability')
ax1.set_title('Exact Probability of Getting Straights and Straight Flushes of Specific Length')
ax1.set_xticks(x_indices)
ax1.legend()
ax1.grid(axis='y', linestyle='--', alpha=0.7)

# Plot 2: Cumulative probabilities (linear scale)
ax2.bar(x_indices - width / 2, prob_at_least_straight, width, label='Straight (at least length X)')
ax2.bar(x_indices + width / 2, prob_at_least_straight_flush, width, label='Straight Flush (at least length X)')
ax2.set_xlabel('Minimum Length of Straight')
ax2.set_ylabel('Probability')
ax2.set_title('Cumulative Probability of Getting Straights and Straight Flushes of At Least X Length')
ax2.set_xticks(x_indices)
ax2.legend()
ax2.grid(axis='y', linestyle='--', alpha=0.7)

plt.tight_layout()
# Save the combined linear scale plots
plt.savefig('probabilities_linear.png', dpi=300, bbox_inches='tight')
plt.close()

# Generate combined plot with logarithmic scale
fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(14, 12))

# Plot 3: Exact hitting probabilities (logarithmic scale)
ax1.bar(x_indices - width / 2, prob_straight_lengths, width, label='Straight (exact length)')
ax1.bar(x_indices + width / 2, prob_straight_flush_lengths, width, label='Straight Flush (exact length)')
ax1.set_xlabel('Length of Straight')
ax1.set_ylabel('Probability (log scale)')
ax1.set_title('Exact Probability of Getting Straights and Straight Flushes of Specific Length (Log Scale)')
ax1.set_xticks(x_indices)
ax1.legend()
ax1.grid(axis='y', linestyle='--', alpha=0.7)
ax1.set_yscale('log')

# Plot 4: Cumulative probabilities (logarithmic scale)
ax2.bar(x_indices - width / 2, prob_at_least_straight, width, label='Straight (at least length X)')
ax2.bar(x_indices + width / 2, prob_at_least_straight_flush, width, label='Straight Flush (at least length X)')
ax2.set_xlabel('Minimum Length of Straight')
ax2.set_ylabel('Probability (log scale)')
ax2.set_title('Cumulative Probability of Getting Straights and Straight Flushes of At Least X Length (Log Scale)')
ax2.set_xticks(x_indices)
ax2.legend()
ax2.grid(axis='y', linestyle='--', alpha=0.7)
ax2.set_yscale('log')

plt.tight_layout()
# Save the combined logarithmic scale plots
plt.savefig('probabilities_log.png', dpi=300, bbox_inches='tight')

# Print Table for Regular Straights
print("REGULAR STRAIGHTS TABLE:")
print("| Length | At Least This Length | Longest Is This Length | Count (At Least)  | Count (Exact Longest) |")
print("|--------|----------------------|------------------------|-------------------|-----------------------|")

for i in range(len(prob_straight_lengths)):
    length = i + 1
    at_least_prob = f"{100 * prob_at_least_straight[i]:.4f}%"
    longest_prob = f"{100 * prob_straight_lengths[i]:.4f}%"

    at_least_count = f"{at_least_counted_straight_lenghts[i]:,}"
    longest_count = f"{counted_straight_lengths[i]:,}"

    print(f"| {length:6d} | {at_least_prob:20s} | {longest_prob:22s} | {at_least_count:17s} | {longest_count:21s} |")

print("\n")

# Print Table for Straight Flushes
print("STRAIGHT FLUSHES TABLE:")
print("| Length | At Least This Length | Longest Is This Length | Count (At Least)  | Count (Exact Longest) |")
print("|--------|----------------------|------------------------|-------------------|-----------------------|")

for i in range(len(prob_straight_flush_lengths)):
    length = i + 1
    at_least_prob = f"{100 * prob_at_least_straight_flush[i]:.4f}%"
    longest_prob = f"{100 * prob_straight_flush_lengths[i]:.4f}%"

    # Calculate the counts
    at_least_count = f"{at_least_counted_straight_flush_lenghts[i]:,}"
    longest_count = f"{counted_straight_flush_lengths[i]:,}"

    print(f"| {length:6d} | {at_least_prob:20s} | {longest_prob:22s} | {at_least_count:17s} | {longest_count:21s} |")

