import torch
from torch.utils.data import TensorDataset, DataLoader
from sklearn.model_selection import train_test_split


def prepare_data_gpu(X, y, val_split=0.2, random_state=42):
    """
    Prepare data for training by splitting into train/val and moving directly to GPU
    """
    # Split the data
    X_train, X_val, y_train, y_val = train_test_split(
        X, y, test_size=val_split, random_state=random_state
    )

    device = torch.device("cuda")
    X_train = torch.from_numpy(X_train).view(-1, 4, 14).to(device)
    y_train = torch.from_numpy(y_train).to(device)
    X_val = torch.from_numpy(X_val).view(-1, 4, 14).to(device)
    y_val = torch.from_numpy(y_val).to(device)

    return X_train, X_val, y_train, y_val


def create_data_loaders(X_train, y_train, X_val, y_val, batch_size=32):
    """
    Create train and validation data loaders
    """
    train_dataset = TensorDataset(X_train, y_train)
    val_dataset = TensorDataset(X_val, y_val)

    train_loader = DataLoader(
        train_dataset,
        batch_size=batch_size,
        shuffle=True,
    )

    val_loader = DataLoader(
        val_dataset,
        batch_size=batch_size,
        shuffle=False,
    )

    return train_loader, val_loader
