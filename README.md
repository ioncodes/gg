## gg
WIP GameGear (and possibly Sega Master System) emulator. Most of the information I either figured out by reverse engineering
the hardware and software myself or by using the excellent docs found on [smspower.org](https://www.smspower.org/).  

<details>
  <summary>Demos</summary>

  | Sonic The Hedgehog 2 Demo                                                                         | Lucky Dime Caper Starring Donald Duck Demo                                                        | Pac-Man Demo                                                                                      |
  | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
  | <video src="https://github.com/ioncodes/gg/assets/18533297/707c1d23-8182-4dd6-8960-2e86f79d7ee6"> | <video src="https://github.com/ioncodes/gg/assets/18533297/a9638e21-5365-4515-a29d-4701d761c3a2"> | <video src="https://github.com/ioncodes/gg/assets/18533297/1549887c-45f5-4eec-9201-9e4de168b32e"> |
</details>

## Compatibility List
| **Title**                                                | **CRC32**  | **Status** |
| -------------------------------------------------------- | :--------: | :--------: |
| [BIOS] Sega Game Gear (USA) (Majesco)                    | `0ebea9d4` |     👌      |
| Pac-Man (USA)                                            | `b318dd37` |     👌      |
| Sonic The Hedgehog 2 (U) [!]                             | `95a18ec7` |     🐥      |
| Lucky Dime Caper Starring Donald Duck, The (USA, Europe) | `07a7815a` |     🐥      |
| Sonic The Hedgehog (U) (V1.0) [!]                        | `3e31cb8c` |     🐣      |
| Batman Returns (World)                                   | `7ac4a3ca` |     🐣      |
| Asterix and the Great Rescue (Europe) (En,Fr,De,Es,It)   | `328c5cc8` |     🐞      |
| Shinobi (USA, Europe)                                    | `30f1c984` |     🐞      |

* 👌: No known issues
* 🐥: Playable with a few bugs
* 🐣: In-Game, but only limited playability
* 🐞: Bugged/Broken

## Running
It is strongly recommended to run the emulator in release mode, no matter what.
```
cargo run --release -- --bios bios.gg --rom game.gg
```

It is possible to dump debug and/or trace information either to stderr or a file:

```
Usage: gg.exe [OPTIONS] --bios <BIOS> --rom <ROM>

Options:
  -b, --bios <BIOS>
  -r, --rom <ROM>
  -l, --lua <LUA>
  -c, --cpu-test
  -l, --log-level <LOG_LEVEL>  [default: info]
  -l, --log-to-file
  -h, --help                   Print help
```

## Debugging
The emulator features a debugger built around [egui and eframe](https://github.com/emilk/egui). It is very simple and hosts the following features:

* Memory Viewer (ROM, RAM, SRAM, VRAM, CRAM)
* Display CPU memory address mappings (ROM / RAM banks)
* "Resume", "Break On" and "Step" debugger controls
* Disassembly & Trace
* View CPU and VDP infromation such as registers
* SDSC Debug Console

There's more features that are CLI only:
* Lua scripting (pretick/posttick hooks with access to CPU & VDP state and memory)
* Debug and trace logging ("debug", "trace")

## Testing
Currently the Z80 implementation can be tested using [ZEXDOC/ZEXALL](https://github.com/maxim-zhao/zexall-smsjsm) and using the JSON unit tests 
provided by [jsmoo](https://github.com/raddad772/jsmoo/tree/main/misc/tests/GeneratedTests/z80/v1). However, some features are ignored/disabled/not implemented.

## ZEXDOC & ZEXALL
`zexdoc` is built into the emulator and can be executed by passing the `--cpu-test` flag.

## JSON Tests
`cargo test` in the workspace folder will launch all unit tests. The current implementation measures only registers and RAM content. Status at the time of writing:  
```
test result: FAILED. 1375 passed; 232 failed; 3 ignored; 0 measured; 0 filtered out; finished in 32.60s
```
