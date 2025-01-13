import torch
import torch.nn.functional as F
import time

def train_model(model, train_loader, val_loader, epochs=100, learning_rate=0.001, save_file="best_model.pt"):
    """
    Train model with data already on GPU
    """
    device = torch.device("cuda")
    model = model.to(device)

    optimizer = torch.optim.Adam(model.parameters(), lr=learning_rate)
    scheduler = torch.optim.lr_scheduler.ReduceLROnPlateau(
        optimizer, mode='min', factor=0.5, patience=5
    )

    criterion = torch.nn.MSELoss()
    best_val_loss = float('inf')

    for epoch in range(epochs):
        # Training phase
        model.train()
        train_loss = 0
        start = time.time()

        for batch_features, batch_labels in train_loader:
            outputs = model(batch_features)
            loss = criterion(outputs, batch_labels.float())

            optimizer.zero_grad()
            loss.backward()
            optimizer.step()

            train_loss += loss.item()
        avg_train_loss = train_loss / len(train_loader)
        print(f"Training in epoch {epoch}, time: {time.time() - start}")

        # Validation phase
        model.eval()
        val_loss = 0
        val_mae = 0  # Track Mean Absolute Error as well

        with torch.no_grad():
            for batch_features, batch_labels in val_loader:
                outputs = model(batch_features)
                loss = criterion(outputs, batch_labels.float())
                val_loss += loss.item()

                # Calculate MAE
                mae = torch.abs(outputs - batch_labels.float()).mean()
                val_mae += mae.item()

        avg_val_loss = val_loss / len(val_loader)
        avg_val_mae = val_mae / len(val_loader)

        scheduler.step(avg_val_loss)

        if avg_val_loss < best_val_loss:
            best_val_loss = avg_val_loss
            torch.save(model.state_dict(), save_file)

        print(f'Epoch {epoch + 1}/{epochs}:')
        print(f'Train Loss (MSE): {avg_train_loss:.4f}')
        print(f'Val Loss (MSE): {avg_val_loss:.4f}')
        print(f'Val MAE: {avg_val_mae:.4f}')
        print('-' * 50)
