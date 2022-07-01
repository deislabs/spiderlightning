import os
from time import sleep

"""
This file simulate filesystem operations to test watch interface on kvstore. 
"""
if __name__ == "__main__":
    sleep(1)
    for j in ["", "2"]:
        for i in range(2):
            with open(f"/tmp/my-container/my-key{j}", "w") as file:
                file.write(f"content_{i}")
            sleep(0.1)
