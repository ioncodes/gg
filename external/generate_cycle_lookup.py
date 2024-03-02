import re

no_prefix = open("./external/libzel/tables/no_prefix.spec").read().splitlines()
cb_prefix = open("./external/libzel/tables/cb_prefix.spec").read().splitlines()
dd_prefix = open("./external/libzel/tables/dd_prefix.spec").read().splitlines()
ed_prefix = open("./external/libzel/tables/ed_prefix.spec").read().splitlines()
fd_prefix = open("./external/libzel/tables/fd_prefix.spec").read().splitlines()
ddcb_prefix = open("./external/libzel/tables/ddcb_prefix.spec").read().splitlines()
fdcb_prefix = open("./external/libzel/tables/fdcb_prefix.spec").read().splitlines()
opcodes = [item for row in [no_prefix, cb_prefix, dd_prefix, ed_prefix, fd_prefix, ddcb_prefix, fdcb_prefix] for item in row]

pattern = re.compile(r"([A-F0-9]{2})([A-F0-9]{2})?( d )?([A-F0-9]{2})?.+\t([0-9]{1,2})")

table = {}

for opcode in opcodes:
    matches = pattern.search(opcode)
    idx = 1
    data = [matches.group(1)]
    if matches.group(2) is not None:
        idx += 1
        data.append(matches.group(2))
        if matches.group(3) is not None:
            idx += 1
            if matches.group(3) == " d ":
                idx += 1
                data.append("_")
                data.append(matches.group(4))
    data = data + ["_"] * (4 - len(data))
    table[tuple(data)] = matches.groups()[-1]

for (data, cycles) in table.items():
    s = "("
    if data[0] == "_":
        s += f"_, "
    else:
        s += f"Some(0x{data[0].lower()}), "
    if data[1] == "_":
        s += f"_, "
    else:
        s += f"Some(0x{data[1].lower()}), "
    if data[2] == "_":
        s += f"_, "
    else:
        s += f"Some(0x{data[2].lower()}), "
    try:
        if data[3] == "_":
            s += f"_) => "
        else:
            s += f"Some(0x{data[3].lower()})) => "
    except:
        pass
    s += f"{cycles},"
    print(s)