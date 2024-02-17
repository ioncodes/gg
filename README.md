## gg
WIP GameGear (and possibly Sega Master System) emulator. Most of the information I either figured out by reverse engineering
the hardware and software myself or by using the excellent docs found on [smspower.org](https://www.smspower.org/).

## Compatibility List
| **Title**                                                | **CRC32**  | **Status** |
| -------------------------------------------------------- | :--------: | :--------: |
| [BIOS] Sega Game Gear (USA) (Majesco)                    | `0ebea9d4` |     👌      |
| Sonic The Hedgehog 2 (U) [!]                             | `95a18ec7` |     🐣      |
| Lucky Dime Caper Starring Donald Duck, The (USA, Europe) | `07a7815a` |     🐣      |
| Asterix and the Great Rescue (Europe) (En,Fr,De,Es,It)   | `328c5cc8` |     🐞      |
| Shinobi (USA, Europe)                                    | `30f1c984` |     🐞      |

* 👌: Playable
* 🐣: In-Game, but not playable
* 🐞: Bugged/Broken

## Testing
Currently the Z80 implementation can be tested using [ZEXDOC/ZEXALL](https://github.com/maxim-zhao/zexall-smsjsm) and using the JSON unit tests 
provided by [jsmoo](https://github.com/raddad772/jsmoo/tree/main/misc/tests/GeneratedTests/z80/v1). However, some features are ignored/disabled/not implemented.

## ZEXDOC & ZEXALL
`zexdoc` is built into the emulator and can be executed by passing the `--cpu-test` flag.

## JSON Tests
`cargo test` in the workspace folder will launch all unit tests. The current implementation measures only registers and RAM content. Status at the time of writing:  
```
test result: FAILED. 1359 passed; 248 failed; 3 ignored; 0 measured; 0 filtered out; finished in 64.65s
```