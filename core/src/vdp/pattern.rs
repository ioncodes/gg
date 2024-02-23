use crate::vdp::Color;

#[derive(Debug)]
pub(crate) struct Pattern {
    pub(crate) data: [[Color; 8]; 8],
}

impl Pattern {
    pub(crate) fn new() -> Pattern {
        Pattern {
            data: [[(0, 0, 0, 0); 8]; 8],
        }
    }

    pub(crate) fn set_pixel(&mut self, x: u8, y: u8, color: Color) {
        self.data[y as usize][x as usize] = color;
    }

    pub(crate) fn get_pixel(&self, x: u8, y: u8) -> Color {
        self.data[y as usize][x as usize]
    }

    pub(crate) fn flip_vertical(&mut self) {
        for y in 0..4 {
            for x in 0..8 {
                let temp = self.data[y as usize][x as usize];
                self.data[y as usize][x as usize] = self.data[7 - y as usize][x as usize];
                self.data[7 - y as usize][x as usize] = temp;
            }
        }
    }

    pub(crate) fn flip_horizontal(&mut self) {
        for y in 0..8 {
            for x in 0..4 {
                let temp = self.data[y as usize][x as usize];
                self.data[y as usize][x as usize] = self.data[y as usize][7 - x as usize];
                self.data[y as usize][7 - x as usize] = temp;
            }
        }
    }
}
