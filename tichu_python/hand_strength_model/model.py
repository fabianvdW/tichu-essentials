import torch
import torch.nn as nn
import torch.nn.functional as F
import itertools


class ColorInvariantConv(nn.Module):
    def __init__(self, in_channels, out_channels, kernel_size):
        super().__init__()
        self.conv = nn.Conv2d(in_channels, out_channels, kernel_size)

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



def predict(model, X):
    model.eval()
    device = next(model.parameters()).device
    X = torch.from_numpy(X).to(device)

    with torch.no_grad():
        output = model(X)

    return output.cpu().numpy()
