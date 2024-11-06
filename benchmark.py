# /usr/bin/env python3
# A simple scrip to benchmark
import subprocess
import time
import os
import platform
from datetime import datetime

def measure_command_time(command):
    start_time = time.time()
    result = subprocess.run(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    end_time = time.time()
    elapsed_time = end_time - start_time
    # print(f"Command Output:\n{result.stdout.decode()}")
    if result.stderr:
        return ("Error", f"{command} Error:\n{result.stderr.decode()}")

    return ("Ok", round(elapsed_time, 4))

# Bytes -> MB
def file_size_mb(file_path):
    return round(os.stat(file_path).st_size / (1024 * 1024), 4)

# Speed
def calculate_speed(file_size_MB, time_Second):
    if time_Second == "Error":
        return "Error"
    else:
        return round(file_size_MB / time_Second, 4)

def get_git_commit_hash():
    try:
        result = subprocess.run(
            ['git', 'rev-parse', '--short', 'HEAD'],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print("Error running git command:", e)
        return None

if __name__ == "__main__":
    file_to_test = "tests/image.jpg"
    commands = {
        "sha256sum": "sha256sum " + file_to_test,
        "ezcheck(ring)": " target/release/ezcheck calculate sha256 -f " + file_to_test
        # "ezcheck(hashes)": " target/release/ezcheck calculate sha256 -f win98.iso"
    }

    file_size_MB = file_size_mb(file_to_test)
    git_hash = get_git_commit_hash()

    # It runs slowly at the first time after being compiled.
    measure_command_time("target/release/ezcheck -V")

    results = []
    for name, command in commands.items():
        r = measure_command_time(command)
        if r[0] == 'Ok':
            results.append([name, r[1]])
        else:
            print(r[1])
            results.append([name, "Error"])

    sorted_results = sorted(results, key=lambda x: (x[1] == 'Error', x[1]))


    print("+{:-^53}+".format("BENCHMARK-RESULTS"))
    print("|  Test file: {: <40}|".format(file_to_test))
    print("|  File size: {: <40}|".format(str(file_size_MB) + " MB"))
    print("|   Git hash: {: <40}|".format(git_hash))
    print("+-----------------+-----------------+-----------------+")
    print("|{: ^17}|{: ^17}|{: ^17}|".format("Command", "Speed(MB/s)", "Time(s)"))
    print("+-----------------+-----------------+-----------------+")

    for r in sorted_results:
        print("|{: ^17}|{: ^17}|{: ^17}|".format(r[0], calculate_speed(file_size_MB, r[1]), r[1]))

    print("+-----------------+-----------------+-----------------+")
    print()
    print("Platform version: " + platform.version())
    print("Run time: " + datetime.now().strftime("%Y-%m-%d %H:%M:%S"))
