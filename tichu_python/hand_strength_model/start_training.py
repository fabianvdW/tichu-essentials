import numpy as np
import pickle
from data_loader import prepare_data_gpu, create_data_loaders
from model import HandStrengthNet, HandStrengthNet2
from trainer import train_model


def save_dict(dictionary, filename):
    with open(filename, 'wb') as file:
        pickle.dump(dictionary, file)


# Loading the dictionary from a file
def load_dict(filename):
    with open(filename, 'rb') as file:
        return pickle.load(file)


if __name__ == '__main__':
    db_as_np = np.load("db_as_np_filtered.npy")
    incoming_card_labels = np.load("labels_filtered.npy")
    # Prepare data
    X_train, X_val, y_train, y_val = prepare_data_gpu(db_as_np, incoming_card_labels)

    # Create data loaders
    train_loader, val_loader = create_data_loaders(
        X_train, y_train, X_val, y_val, batch_size=4096
    )

    # Initialize model
    model = HandStrengthNet2().cuda()

    # Train the model
    train_model(
        model=model,
        train_loader=train_loader,
        val_loader=val_loader,
        epochs=100,
        learning_rate=0.001,
        save_file="best_model2.pt"
    )
