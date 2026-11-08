import sys
import time

with open("/home/shabd-vasudeva/pyre-ide/python_debug.log", "w") as f:
    f.write("Python script started successfully!\n")

print("Pyre IDE Engine Test: Starting...", flush=True)
for i in range(3):
    print(f"Counter: {i}", flush=True)
    time.sleep(0.5)

print("Pyre IDE Engine Test: Finished successfully!", flush=True)