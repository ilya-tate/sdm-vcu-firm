import json
import struct
import subprocess
import sys

from elftools.elf.elffile import ELFFile

QEMU = "timeout -s KILL 20 qemu-system-arm -M olimex-stm32-h405 -kernel /tests.elf -display none"
PANIC_CODE = 134


def get_tests():
    elf = ELFFile(open("/tests.elf", "rb"))
    entries = elf.get_section_by_name(".embedded_test")
    rodata = elf.get_section_by_name(".rodata")
    tests = []

    for symbol in elf.get_section_by_name(".symtab").iter_symbols():
        if not symbol.name.startswith('{"disambiguator"'):
            continue

        test = json.loads(symbol.name)
        entry = symbol["st_value"] - entries["sh_addr"]
        test["addr"], mod, length = struct.unpack_from("<3I", entries.data(), entry)
        mod -= rodata["sh_addr"]
        test["mod"] = rodata.data()[mod : mod + length].decode().removeprefix("tests::")
        tests.append(test)

    return sorted(tests, key=lambda test: (test["mod"], test["name"]))


def run(test):
    args = f" -semihosting-config enable=on,arg=run_addr,arg={test['addr']}"
    return subprocess.call((QEMU + args).split())


with open("/tests.elf", "wb") as file:
    file.write(sys.stdin.buffer.read())

filters = [arg for arg in sys.argv[2:] if arg != "--no-board"]
passed = failed = skipped = 0
mod = None

for test in get_tests():
    name = test["name"]
    if filters and not any(f in f"{test['mod']}::{name}" for f in filters):
        continue

    if test["mod"] != mod:
        mod = test["mod"]
        print(f"\n{mod}")

    if test["ignored"] or mod.startswith("pcb::"):
        print(f"\tskipped\t{name}")
        skipped += 1
        continue

    code = run(test)
    if code == (PANIC_CODE if test["should_panic"] else 0):
        print(f"\tok\t{name}")
        passed += 1
    else:
        print(f"\tFAILED\t{name} (exit code {code})")
        failed += 1

print(f"\n{passed} passed,\n{failed} failed,\n{skipped} skipped")

if failed or not passed:
    sys.exit(1)
