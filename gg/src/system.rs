use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::error::GgError;
use crate::lua_engine::{LuaEngine, HookType};
use crate::vdp::Vdp;

pub(crate) struct System {
    pub(crate) cpu: Cpu,
    pub(crate) bus: Bus,
    pub(crate) vdp: Vdp,
    lua: LuaEngine,
    lua_cached: bool,
}

impl System {
    pub(crate) fn new(lua_script: Option<String>) -> System {
        System {
            cpu: Cpu::new(),
            bus: Bus::new(),
            vdp: Vdp::new(),
            lua: LuaEngine::new(lua_script),
            lua_cached: false
        }
    }

    pub(crate) fn load_rom(&mut self, data: &[u8], is_bios: bool) {
        if !is_bios {
            self.bus.bios_enabled = false;
        }

        for i in 0..data.len() {
            self.bus
                .write(i as u16, data[i])
                .expect("Failed to write to bus while loading into ROM");
        }

        if !is_bios {
            self.bus.bios_enabled = true;
        }
    }

    pub(crate) fn run(&mut self) {
        loop {
            // Execute pre-tick Lua script
            let current_pc_before_tick = self.cpu.registers.pc;
            if self.lua.hook_exists(current_pc_before_tick, HookType::PreTick) {
                if !self.lua_cached {
                    self.lua.create_tables(&self.cpu, &self.vdp, &self.bus);
                    self.lua_cached = true;
                }

                self.lua.execute_hook(current_pc_before_tick,HookType::PreTick);
            }

            // Process tick for all components
            let result = self.cpu.tick(&mut self.bus);
            match result {
                Err(GgError::OpcodeNotImplemented { opcode: _ }) => panic!("{}", self),
                Err(GgError::DecoderError { msg }) => panic!("Decoder error: {}\n{}", msg, self),
                _ => {}
            };
            self.vdp.tick(&mut self.bus);

            // Execute other components here (e.g. VDP or I/O interaction)
            self.bus.io.process_default();

            // Execute post-tick Lua script
            if self.lua.hook_exists(current_pc_before_tick, HookType::PostTick) {
                self.lua.create_tables(&self.cpu, &self.vdp, &self.bus);
                self.lua.execute_hook(current_pc_before_tick, crate::lua_engine::HookType::PostTick);
            }
        }
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
