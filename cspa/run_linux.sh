#!/bin/bash

# File to store individual run times
TIMES_FILE="times.txt"
# File to store the average time
AVERAGE_FILE="average.txt"

# Ensure the times file is empty or create it if it doesn't exist
> "$TIMES_FILE"

# Run the Rust program 500 times
for i in {1..500}
do
    # Time the execution and append to times file
    { time ./rust_program ; } 2>> "$TIMES_FILE"
done

# Process the times and compute the average
awk '
    /real/ {
        # Extract minutes and seconds and convert to total seconds
        split($2, time, "m")
        seconds = time[1] * 60 + time[2]
        total += seconds
        count++
    }
    END {
        # Calculate and print the average
        printf "Average Time: %.3f seconds\n", total/count
    }
' "$TIMES_FILE" | tee -a "$AVERAGE_FILE"

