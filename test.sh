#!/bin/bash

echo "Generating notifications..."
for i in {1..10}; do
    osascript -e "display notification \"Test Notification $i\" with title \"Test Notification $i\""
done

echo "Letting them show up..."
sleep 0.4

echo "Running..."
RUST_LOG=trace cargo run --release
