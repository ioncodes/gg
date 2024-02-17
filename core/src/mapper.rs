use crate::memory::Memory;

pub trait Mapper {
    fn read_from_bank(&self, bank: usize, offset: u16) -> u8;
    fn write_to_bank(&mut self, bank: usize, offset: u16, value: u8);
    fn resize(&mut self, new_size: usize);
    fn memory(&self) -> &Memory<usize>;
    fn memory_mut(&mut self) -> &mut Memory<usize>;
    fn name(&self) -> String;

    fn read(&self, address: usize) -> u8 {
        let bank = (address / 0x4000) as usize;
        let addr = (address % 0x4000) as u16;
        self.read_from_bank(bank, addr)
    }

    fn write(&mut self, address: usize, value: u8) {
        let bank = (address / 0x4000) as usize;
        let addr = (address % 0x4000) as u16;
        self.write_to_bank(bank, addr, value);
    }
}

pub struct SegaMapper {
    pub rom: Memory<usize>,
}

impl SegaMapper {
    pub fn new(size: usize) -> SegaMapper {
        SegaMapper {
            rom: Memory::new(size, 0x0000),
        }
    }
}

impl Mapper for SegaMapper {
    fn read_from_bank(&self, bank: usize, offset: u16) -> u8 {
        let addr = (bank * 0x4000) + offset as usize;
        self.rom.read(addr)
    }

    fn write_to_bank(&mut self, bank: usize, offset: u16, value: u8) {
        let addr = (bank * 0x4000) + offset as usize;
        self.rom.write(addr as usize, value);
    }

    fn resize(&mut self, new_size: usize) {
        self.rom.resize(new_size);
    }

    fn memory(&self) -> &Memory<usize> {
        &self.rom
    }

    fn memory_mut(&mut self) -> &mut Memory<usize> {
        &mut self.rom
    }

    fn name(&self) -> String {
        String::from("Sega Mapper")
    }
}
