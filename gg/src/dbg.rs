pub(crate) trait Debugger {
    fn set_breakpoint(&mut self, address: u16);
    fn clear_breakpoint(&mut self, address: u16);
    fn default_breakpoints() -> Vec<u16> {
        include_str!("../../external/breakpoints.dat")
            .lines()
            .map(|x| u16::from_str_radix(x.trim_start_matches("0x"), 16).unwrap())
            .collect()
    }
}