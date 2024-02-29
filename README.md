## gg
WIP GameGear (and possibly Sega Master System) emulator. Most of the information I either figured out by reverse engineering
the hardware and software myself or by using the excellent docs found on [smspower.org](https://www.smspower.org/). This project is  
messed up in some cases and should not be used as direct reference for your own implementation. I mostly implemented everything on a
"fuck around and find out" basis, implementing features where needed to get things working. Also, there is no PSG support!

<details>
  <summary>Demos</summary>

  | Sonic The Hedgehog 2 Demo                                                                         | Lucky Dime Caper Starring Donald Duck Demo                                                        | Pac-Man Demo                                                                                      | Earthworm Jim Demo |
  | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | ---- |
  | <video src="https://github.com/ioncodes/gg/assets/18533297/afd20259-47d0-4dd6-86b8-abe68bc0a4e6"> | <video src="https://github.com/ioncodes/gg/assets/18533297/9ac07dca-3b54-43b2-a881-163980319dff"> | <video src="https://github.com/ioncodes/gg/assets/18533297/1549887c-45f5-4eec-9201-9e4de168b32e"> | <video src="https://github.com/ioncodes/gg/assets/18533297/3812319b-5b0e-4841-99c7-e3f084279b67"> |
</details>

## Compatibility List
| **Title**                                                                     | **CRC32**  | **Status** |
| ----------------------------------------------------------------------------- | :--------: | :--------: |
| [BIOS] Sega Game Gear (USA) (Majesco)                                         | `0ebea9d4` |     👌      |
| Pac-Man (USA)                                                                 | `b318dd37` |     👌      |
| Sonic The Hedgehog 2 (U) [!]                                                  | `95a18ec7` |     👌      |
| Sonic & Tails (Japan) (En)                                                    | `8ac0dade` |     👌      |
| Sonic & Tails 2 (Japan)                                                       | `496bce64` |     👌      |
| Sonic Labyrinth (World)                                                       | `5550173b` |     👌      |
| Sonic The Hedgehog - Triple Trouble (USA, Europe, Brazil) (Beta) (1994-08-08) | `80eb7cfb` |     👌      |
| Lucky Dime Caper Starring Donald Duck, The (USA, Europe)                      | `07a7815a` |     👌      |
| Earthworm Jim (Europe)                                                        | `691ae339` |     🐥      |
| Batman Returns (World)                                                        | `7ac4a3ca` |     🐥      |
| Ecco the Dolphin (Japan)                                                      | `a32eb9d5` |     🐥      |
| Ecco - The Tides of Time (USA, Europe, Brazil)                                | `e2f3b203` |     🐥      |
| GG Shinobi II, The ~ Shinobi II - The Silent Fury (World)                     | `6201c694` |     🐥      |
| Sonic The Hedgehog (U) (V1.0) [!]                                             | `3e31cb8c` |     🐣      |
| Tom and Jerry - The Movie (USA, Europe)                                       | `5cd33ff2` |     🐞      |
| Asterix and the Great Rescue (Europe) (En,Fr,De,Es,It)                        | `328c5cc8` |     🐞      |
| Shinobi (USA, Europe)                                                         | `30f1c984` |     🐞      |

* 👌: No known issues
* 🐥: Playable with a few bugs
* 🐣: In-Game, but only limited playability
* 🐞: Bugged/Broken

Note: This rating is completely subjective.

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
test result: FAILED. 1388 passed; 222 failed; 0 ignored; 0 measured; 0 filtered out; finished in 66.36s
```
