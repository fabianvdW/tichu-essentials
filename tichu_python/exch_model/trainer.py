import torch
import torch.nn.functional as F
import time

def train_model(model, train_loader, val_loader, epochs=100, learning_rate=0.001):
    """
    Train model with data already on GPU
    """
    device = torch.device("cuda")
    model = model.to(device)

    optimizer = torch.optim.Adam(model.parameters(), lr=learning_rate)
    scheduler = torch.optim.lr_scheduler.ReduceLROnPlateau(
        optimizer, mode='min', factor=0.5, patience=5
    )

    best_val_loss = float('inf')

    for epoch in range(epochs):
        # Training phase
        model.train()
        train_loss = 0
        start = time.time()
        correct = 0
        total = 0

        for batch_features, batch_labels in train_loader:
            outputs = model(batch_features)
            _, predicted = outputs.max(1)
            total += batch_labels.size(0)
            correct += predicted.eq(batch_labels.to(dtype=torch.long)).sum().item()
            loss = F.cross_entropy(outputs, batch_labels.to(dtype=torch.long))

            optimizer.zero_grad()
            loss.backward()
            optimizer.step()

            train_loss += loss.item()
        avg_train_loss = train_loss / len(train_loader)
        accuracy = 100. * correct / total
        print(f"Training in epoch {epoch}, time: {time.time() - start}")
        print(f'Train Accuracy: {accuracy:.2f}%')

        # Validation phase
        model.eval()
        val_loss = 0
        correct = 0
        total = 0

        with torch.no_grad():
            for batch_features, batch_labels in val_loader:
                outputs = model(batch_features)
                loss = F.cross_entropy(outputs, batch_labels.to(dtype=torch.long))
                val_loss += loss.item()

                _, predicted = outputs.max(1)
                total += batch_labels.size(0)
                correct += predicted.eq(batch_labels.to(dtype=torch.long)).sum().item()

        avg_val_loss = val_loss / len(val_loader)
        accuracy = 100. * correct / total

        scheduler.step(avg_val_loss)

        if avg_val_loss < best_val_loss:
            best_val_loss = avg_val_loss
            torch.save(model.state_dict(), 'best_model.pt')

        print(f'Epoch {epoch + 1}/{epochs}:')
        print(f'Train Loss: {avg_train_loss:.4f}')
        print(f'Val Loss: {avg_val_loss:.4f}')
        print(f'Val Accuracy: {accuracy:.2f}%')
        print('-' * 50)
