use log::error;

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::error::GgError;

use crate::lua_engine::{LuaEngine, HookType};
use crate::vdp::{Color, Vdp};

pub struct System {
    pub cpu: Cpu,
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

    pub fn load_roms(&mut self, bios: &[u8], cartridge: &[u8]) {
        self.load_bios(bios);
        self.load_cartridge(cartridge);
    }

    pub fn load_bios(&mut self, data: &[u8]) {
        let previous_value = self.enable_bios();
        self.load_rom(data);
        self.bus.bios_enabled = previous_value;
    }

    pub fn load_cartridge(&mut self, data: &[u8]) {
        let previous_value = self.disable_bios();
        self.load_rom(data);
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

    pub fn tick(&mut self) -> Result<bool, GgError> {
        // Execute pre-tick Lua script
        let current_pc_before_tick = self.cpu.registers.pc;
        if self.lua.hook_exists(current_pc_before_tick, HookType::PreTick) {
            self.lua.create_tables(&self.cpu, &self.vdp, &self.bus);
            self.lua.execute_hook(current_pc_before_tick, HookType::PreTick);
        }

        // Process tick for all components
        let result = self.cpu.tick(&mut self.bus, &mut self.vdp);
        match result {
            Err(GgError::IoRequestNotFulfilled) => (),
            Err(GgError::JumpNotTaken) => (),
            Err(e) => {
                error!("Identified error at address: {:04x}", self.cpu.registers.pc);
                return Err(e);
            },
            _ => ()
        };
        self.vdp.tick(&mut self.bus);

        // Execute post-tick Lua script
        if self.lua.hook_exists(current_pc_before_tick, HookType::PostTick) {
            self.lua.create_tables(&self.cpu, &self.vdp, &self.bus);
            self.lua.execute_hook(current_pc_before_tick, HookType::PostTick);
        }

        // Let the caller know if VRAM is dirty and if we reached VBlank to cause a redraw
        Ok(self.ready_to_redraw())
    }

    pub fn render(&mut self) -> (Color, Vec<Color>) {
        self.vdp.render_background()
    }

    pub(crate) fn load_rom(&mut self, data: &[u8]) {
        for i in 0..data.len() {
            self.bus
                .write(i as u16, data[i])
                .expect("Failed to write to bus while loading into ROM");
        }
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
