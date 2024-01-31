use log::{debug, info};

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::error::GgError;
use crate::lua_engine::{LuaEngine, HookType};
use crate::vdp::{Color, Vdp};

pub struct System {
    pub(crate) cpu: Cpu,
    pub(crate) bus: Bus,
    pub(crate) vdp: Vdp,
    lua: LuaEngine
}

impl System {
    pub fn new(lua_script: Option<String>) -> System {
        System {
            cpu: Cpu::new(),
            bus: Bus::new(),
            vdp: Vdp::new(),
            lua: LuaEngine::new(lua_script)
        }
    }

    pub fn load_rom(&mut self, data: &[u8], is_bios: bool) {
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

    pub fn tick(&mut self) -> bool {
        // Execute pre-tick Lua script
        let current_pc_before_tick = self.cpu.registers.pc;
        if self.lua.hook_exists(current_pc_before_tick, HookType::PreTick) {
            self.lua.create_tables(&self.cpu, &self.vdp, &self.bus);
            self.lua.execute_hook(current_pc_before_tick, HookType::PreTick);
        }

        // Process tick for all components
        let result = self.cpu.tick(&mut self.bus);
        match result {
            Err(GgError::OpcodeNotImplemented { opcode: _ }) => panic!("{}", self),
            Err(GgError::DecoderError { msg }) => panic!("Decoder error: {}\n{}", msg, self),
            Err(GgError::BreakpointHit) => {
                debug!("{}", self);

                use std::io;
                use std::io::prelude::*;

                info!("Press any key to continue...");

                let mut stdin = io::stdin();
                let _ = stdin.read(&mut [0u8]).unwrap();

                self.cpu.resume_execution();

                return self.ready_to_redraw();
            },
            _ => {}
        };
        self.vdp.tick(&mut self.bus);

        // Execute other components here (e.g. VDP or I/O interaction)
        self.bus.io.process_default();

        // Execute post-tick Lua script
        if self.lua.hook_exists(current_pc_before_tick, HookType::PostTick) {
            self.lua.create_tables(&self.cpu, &self.vdp, &self.bus);
            self.lua.execute_hook(current_pc_before_tick, HookType::PostTick);
        }

        // Let the caller know if VRAM is dirty and if we reached VBlank to cause a redraw
        self.ready_to_redraw()
    }

    pub fn render(&mut self) -> (Color, Vec<Vec<Color>>) {
        self.vdp.render_background()
    }

    fn ready_to_redraw(&self) -> bool {
        self.vdp.vram_dirty && self.vdp.is_vblank()
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
