import sys
import re

MESEN2_REGEX = r"([0-9A-F]+)  "
GG_REGEX = r"(rom|ram):([0-9a-f]+)->"

def compare_traces(trace, mesen_trace):
    for (gg, mesen) in zip(trace, mesen_trace):
        gg_pc = re.search(GG_REGEX, gg).group(2)
        mesen_pc = re.search(MESEN2_REGEX, mesen).group(1).lower()
        if gg_pc != mesen_pc:
            print(f"PC mismatch: {gg_pc} != {mesen_pc}")
            break

def main():
    trace = sys.argv[1]
    mesen_trace = sys.argv[2]
    print(f"Comparing files: {trace}, {mesen_trace}")

    trace = open(trace, "r").read().splitlines()
    trace = [line for line in trace if line.startswith("[TRACE emu::cpu] [rom:")]
    mesen_trace = open(mesen_trace, "r").read().splitlines()
    compare_traces(trace, mesen_trace)


if __name__ == "__main__":
    main()