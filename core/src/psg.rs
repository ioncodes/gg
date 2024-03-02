use log::{debug, error};

use crate::bus::io::Controller;
use crate::error::GgError;

struct Latch {
    data: u16,
    is_volume: bool,
    channel: u8,
}

#[derive(Copy, Clone)]
struct Channel {
    volume: u8,
    tone_or_noise: u16,
    output: bool,
    counter: u16,
}

pub struct Psg {
    channels: [Channel; 4],
    latch: Option<Latch>,
}

impl Psg {
    pub(crate) fn new() -> Psg {
        Psg {
            channels: [Channel {
                volume: 0,
                tone_or_noise: 0,
                output: false,
                counter: 0,
            }; 4],
            latch: None,
        }
    }

    pub(crate) fn tick(&mut self) {
        for channel in self.channels.iter_mut() {
            if channel.counter == 0 {
                let _frequency = 3579545.0 / (32 * channel.tone_or_noise) as f32;
            } else {
                channel.counter -= 1;
            }
        }
    }
}

impl Controller for Psg {
    fn read_io(&mut self, port: u8) -> Result<u8, GgError> {
        debug!("PSG read from port {:02X}", port);
        Ok(0)
    }

    fn write_io(&mut self, port: u8, value: u8) -> Result<(), GgError> {
        debug!("PSG write to port {:02X} with value {:08b}", port, value);

        match value & 0b1000_0000 > 0 {
            true => {
                let data = (value & 0b0000_1111) as u16;
                let latch = Latch {
                    channel: value & 0b0110_0000 >> 5,
                    is_volume: (value & 0b0001_0000) > 0,
                    data,
                };

                if latch.is_volume {
                    self.channels[latch.channel as usize].volume |= (data & 0b0000_0000_0000_1111) as u8;
                } else {
                    self.channels[latch.channel as usize].tone_or_noise |= data & 0b0000_0000_0000_1111;
                    self.channels[latch.channel as usize].counter = self.channels[latch.channel as usize].tone_or_noise;
                    self.channels[latch.channel as usize].output = !self.channels[latch.channel as usize].output;
                }

                self.latch = Some(latch);
            }
            false => {
                if let Some(latch) = &self.latch {
                    let mut data = (value & 0b0011_1111) as u16;
                    data |= (latch.data as u16) << 6;

                    if latch.is_volume {
                        self.channels[latch.channel as usize].volume = (data & 0b1111_1111) as u8;
                    } else {
                        self.channels[latch.channel as usize].tone_or_noise |= data;
                        self.channels[latch.channel as usize].counter = self.channels[latch.channel as usize].tone_or_noise;
                        self.channels[latch.channel as usize].output = !self.channels[latch.channel as usize].output;
                    }

                    self.latch = None;
                } else {
                    error!("PSG write to data port without a latch");
                }
            }
        }

        Ok(())
    }
}
