#!/bin/bash

echo "Generating notifications..."
for i in {1..10}; do
    osascript -e "display notification \"Test Notification $i\" with title \"Test Notification $i\""
done

echo "Waiting 1 second..."
sleep 1

echo "Running..."
cargo run --release
