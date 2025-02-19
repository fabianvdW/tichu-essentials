import torch
import torch.nn as nn
import numpy as np
import itertools


class ColorInvariantConv(nn.Module):
    def __init__(self, in_channels, out_channels, kernel_size, use_padding=False):
        super().__init__()

        # Calculate padding if enabled
        padding = "same" if use_padding else (0, 0)

        self.conv = nn.Conv2d(
            in_channels,
            out_channels,
            kernel_size,
            padding=padding
        )

        # Generate all possible color permutations (4! = 24)
        self.color_perms = list(itertools.permutations(range(4)))

    def forward(self, x):
        # x shape: [batch_size, in_channels, 4, 13]

        # Apply convolution to each color permutation
        outputs = []
        for perm in self.color_perms:
            # Permute the colors (dim=2 is the color dimension)
            out = self.conv(x[:, :, perm, :]).relu()
            outputs.append(out)

        # Average over all permutations
        # Stack along a new dimension and then mean over it
        return torch.stack(outputs).mean(0)


class HandStrengthNet(nn.Module):
    def __init__(self):
        super().__init__()

        # Define the filter configurations
        self.filter_configs = [
            ((4, 5), 4),  # Streets across colors length 5
            ((4, 6), 4),  # Streets across colors length 6
            ((4, 7), 4),  # Streets across colors length 7
            ((1, 5), 4),  # Single color streets
            ((4, 2), 4),  # Pair streets length 2
            ((4, 3), 4),  # Pair streets length 3
            ((4, 4), 4),  # Pair streets length 4
            ((4, 1), 4),  # Same value across colors
        ]

        # Create ColorInvariantConv layers for each filter type
        self.conv_layers = nn.ModuleList([
            ColorInvariantConv(1, n_filters, size)
            for size, n_filters in self.filter_configs
        ])

        # Dense layer for special cards
        self.special_and_exch_dense = nn.Linear(4 + 17 + 17, 64)

        # Calculate total features
        self.total_features = sum([
            n_filters * (5 - size[0]) * (14 - size[1])
            for size, n_filters in self.filter_configs
        ]) + 64

        # Dense layers
        self.fc1 = nn.Linear(self.total_features, 256)
        self.fc2 = nn.Linear(256, 128)
        self.fc3 = nn.Linear(128, 128)
        self.fc_out = nn.Linear(128, 1)

        self.dropout = nn.Dropout(0.2)

        self.batch_float = None

    def forward(self, x):
        #[Batch_size, 90]
        # Split input into regular and special cards
        cards = x[:, :56].view(-1, 4, 14)
        regular_cards = cards[:, :, 1:].float()  # [batch_size, 4, 13]
        special_cards = cards[:, :, 0].float()  # [batch_size, 4]
        exch_cards = x[:, 56:]

        # Add channel dimension for convolution
        regular_cards = regular_cards.unsqueeze(1)  # [batch_size, 1, 4, 13]

        # Process regular cards through each filter type
        conv_outputs = []
        for conv in self.conv_layers:
            out = conv(regular_cards)
            conv_outputs.append(out.view(out.size(0), -1))  # Flatten each output

        # Process special cards
        special_features = self.special_and_exch_dense(torch.cat([special_cards, exch_cards], dim=1)).relu()

        # Combine all features
        combined = torch.cat(conv_outputs + [special_features], dim=1)

        # Final dense layers
        x = self.fc1(combined).relu()
        x = self.dropout(x)
        x = self.fc2(x).relu()
        x = self.dropout(x)
        x = self.fc3(x).relu()
        return self.fc_out(x).squeeze(-1)


class DoubleConvBlock(nn.Module):
    def __init__(self, in_channels, mid_channels, out_channels, kernel_size):
        super().__init__()

        # First layer is always ColorInvariantConv, with optional padding
        self.conv1 = ColorInvariantConv(in_channels, mid_channels, kernel_size, use_padding=True)

        # Second conv layer with no padding to maintain original output dimensions
        self.conv2 = nn.Conv2d(
            mid_channels,
            out_channels,
            kernel_size=kernel_size,
        )

    def forward(self, x):
        x = self.conv1(x)  # ReLU is already applied in ColorInvariantConv
        return self.conv2(x).relu()

class HandStrengthNet2(nn.Module):
    def __init__(self):
        super().__init__()

        # Define the filter configurations
        self.filter_configs = [
            ((4, 5), 16, 8),  # Streets across colors length 5 (mid_channels, out_channels)
            ((4, 6), 8, 4),   # Streets across colors length 6
            ((4, 7), 8, 4),   # Streets across colors length 7
            ((1, 5), 8, 4),   # Single color streets
            ((4, 2), 16, 8),  # Pair streets length 2
            ((4, 3), 8, 4),   # Pair streets length 3
            ((4, 4), 8, 4),   # Pair streets length 4
            ((4, 1), 16, 8),  # Same value across colors
        ]

        # Create ColorInvariantConv layers for each filter type
        self.conv_layers = nn.ModuleList([
            DoubleConvBlock(1, mid_channels, out_channels, size)
            for size, mid_channels, out_channels in self.filter_configs
        ])

        # Dense layer for special cards
        self.special_and_exch_dense = nn.Linear(4 + 17 + 17, 64)

        # Calculate total features
        self.total_features = sum([
            n_filters * (5 - size[0]) * (14 - size[1])
            for size, _, n_filters in self.filter_configs
        ]) + 64

        # Dense layers
        self.fc1 = nn.Linear(self.total_features, 256)
        self.fc2 = nn.Linear(256, 128)
        self.fc3 = nn.Linear(128, 128)
        self.fc_out = nn.Linear(128, 1)

        self.dropout = nn.Dropout(0.2)

        self.batch_float = None

    def forward(self, x):
        #[Batch_size, 90]
        # Split input into regular and special cards
        cards = x[:, :56].view(-1, 4, 14)
        regular_cards = cards[:, :, 1:].float()  # [batch_size, 4, 13]
        special_cards = cards[:, :, 0].float()  # [batch_size, 4]
        exch_cards = x[:, 56:]

        # Add channel dimension for convolution
        regular_cards = regular_cards.unsqueeze(1)  # [batch_size, 1, 4, 13]

        # Process regular cards through each filter type
        conv_outputs = []
        for conv in self.conv_layers:
            out = conv(regular_cards)
            conv_outputs.append(out.view(out.size(0), -1))  # Flatten each output

        # Process special cards
        special_features = self.special_and_exch_dense(torch.cat([special_cards, exch_cards], dim=1)).relu()

        # Combine all features
        combined = torch.cat(conv_outputs + [special_features], dim=1)

        # Final dense layers
        x = self.fc1(combined).relu()
        x = self.dropout(x)
        x = self.fc2(x).relu()
        x = self.dropout(x)
        x = self.fc3(x).relu()
        return self.fc_out(x).squeeze(-1)

def predict(model, X, max_batch_size=65536):
    model.eval()
    device = next(model.parameters()).device
    X = torch.from_numpy(X)

    outputs = []
    # Calculate number of samples and create batches
    n_samples = len(X)

    with torch.no_grad():
        for start_idx in range(0, n_samples, max_batch_size):
            end_idx = min(start_idx + max_batch_size, n_samples)
            batch = X[start_idx:end_idx].to(device)
            batch_output = model(batch)
            outputs.append(batch_output.cpu().numpy())

    # Concatenate all batch outputs
    return np.concatenate(outputs, axis=0)
