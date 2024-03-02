use std::rc::Rc;

use log::error;
use z80::instruction::Instruction;

use crate::bus::{Bus, Passthrough};
use crate::cpu::Cpu;
use crate::error::GgError;

use crate::lua_engine::{HookType, LuaEngine};
use crate::mapper::SegaMapper;
use crate::psg::Psg;
use crate::vdp::{Color, Mode, Vdp};

pub struct SystemState {
    pub frame_ready: bool,
    pub repeat_not_fulfilled: bool,
}

pub struct System {
    pub cpu: Cpu,
    pub bus: Bus,
    pub vdp: Vdp,
    pub psg: Psg,
    lua: Rc<LuaEngine>,
    abort_invalid_io_op: bool,
    last_cycles: usize,
}

impl System {
    pub fn new(lua_script: Option<String>, emulate_sms: bool) -> System {
        // todo: figure out mapper
        let mapper = SegaMapper::new(0);
        let mut bus = Bus::new(mapper);
        let mode = if emulate_sms { Mode::SegaMasterSystem } else { Mode::GameGear };
        let lua = Rc::new(LuaEngine::new(lua_script));

        bus.powerup_reset_banks().unwrap();

        System {
            cpu: Cpu::new(),
            bus,
            vdp: Vdp::new(mode, Rc::clone(&lua)),
            psg: Psg::new(),
            lua,
            abort_invalid_io_op: true,
            last_cycles: 0,
        }
    }

    pub fn load_roms(&mut self, bios: &[u8], cartridge: &[u8]) {
        self.load_bios(bios);
        self.load_cartridge(cartridge);
    }

    pub fn load_bios(&mut self, data: &[u8]) {
        let previous_value = self.enable_bios();
        self.load_rom(Passthrough::Bios, data);
        self.bus.bios_enabled = previous_value;
    }

    pub fn load_cartridge(&mut self, data: &[u8]) {
        self.bus.rom.resize(data.len());

        let previous_value = self.disable_bios();
        self.load_rom(Passthrough::Rom, data);
        self.bus.bios_enabled = previous_value;
    }

    pub fn disable_bios(&mut self) -> bool {
        let previous_value = self.bus.bios_enabled;
        self.bus.bios_enabled = false;
        previous_value
    }

    pub fn enable_bios(&mut self) -> bool {
        let previous_value = self.bus.bios_enabled;
        self.bus.bios_enabled = true;
        previous_value
    }

    pub fn decode_instr_at_pc(&mut self) -> Result<Instruction, String> {
        self.cpu.decode_at_pc(&mut self.bus)
    }

    pub fn tick(&mut self) -> Result<SystemState, GgError> {
        self.lua.create_tables(&self.cpu, &self.vdp, &self.bus);

        // Execute Lua script
        let current_pc_before_tick = self.cpu.registers.pc;
        if self.lua.hook_exists(current_pc_before_tick, HookType::CpuExec) {
            self.lua.execute_hook(current_pc_before_tick, HookType::CpuExec);
        }

        // Process tick for all components
        let mut repeat_not_fulfilled = false;

        let result = self.cpu.tick(&mut self.bus, &mut self.vdp, &mut self.psg);
        match result {
            Err(GgError::IoRequestNotFulfilled) => (),
            Err(GgError::JumpNotTaken) => (),
            Err(GgError::CpuHalted) => (),
            Err(GgError::RepeatNotFulfilled) => repeat_not_fulfilled = true,
            Err(GgError::IoControllerInvalidPort) | Err(GgError::VdpInvalidIoMode) => {
                if self.abort_invalid_io_op {
                    error!("Identified I/O error at address: {:04x}", self.cpu.registers.pc);
                    if self.cpu.registers.pc < 0xc000 {
                        error!(
                            "Real address in ROM: {:08x}",
                            self.bus.translate_address_to_real(self.cpu.registers.pc).unwrap()
                        );
                    }
                    return Err(result.err().unwrap());
                }
            }
            Err(e) => {
                error!("Identified error at address: {:04x}", self.cpu.registers.pc);
                if self.cpu.registers.pc < 0xc000 {
                    error!(
                        "Real address in ROM: {:08x}",
                        self.bus.translate_address_to_real(self.cpu.registers.pc).unwrap()
                    );
                }
                return Err(e);
            }
            _ => (),
        };

        let mut frame_generated = false;

        // https://www.smspower.org/forums/13530-VDPClockSpeed
        let delta = self.cpu.cycles - self.last_cycles;
        if delta > 262 {
            frame_generated = self.vdp.tick();
            self.last_cycles = self.cpu.cycles;
        }

        self.psg.tick();

        // Let the caller know if we reached VBlank to cause a redraw
        Ok(SystemState {
            frame_ready: frame_generated,
            repeat_not_fulfilled,
        })
    }

    pub fn render(&mut self) -> (Color, &Vec<Color>) {
        self.vdp.render()
    }

    pub(crate) fn load_rom(&mut self, rom: Passthrough, data: &[u8]) {
        for i in 0..data.len() {
            self.bus.write_passthrough(&rom, i, data[i]);
        }
    }

    pub fn set_abort_on_io_operation_behavior(&mut self, value: bool) {
        self.abort_invalid_io_op = value;
    }
}

impl std::fmt::Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.cpu)?;

        let value = self.bus.read_word(self.cpu.registers.sp).unwrap();
        write!(f, "RAM @ {:04x}: {:04x}\n", self.cpu.registers.sp, value)?;

        write!(f, "{}\n", self.vdp)?;

        Ok(())
    }
}
