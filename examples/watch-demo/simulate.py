import os
from time import sleep

"""
This file simulate filesystem operations to test watch interface on kvstore. 
"""
if __name__ == "__main__":
    p = os.environ["TMPDIR"]
    sleep(1)
    for j in ["", "2"]:
        for i in range(2):
            # FIXME: This only works on unix machines, as the "/tmp" directory doesn't exist on Windows
            with open(f"{p}/my-container/my-key{j}", "w") as file:
                file.write(f"content_{i}")
            sleep(0.1)
