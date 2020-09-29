use {
    paste::paste,
    core::marker::PhantomData,

    crate::pac::{
        SMC, smc,
    },
    crate::clock::{
        StaticMemoryControllerClock,
        Enabled,
    },
    crate::gpio::*,
};

// Chip Select Mode
pub struct Uninitialized;
pub struct Configured;

#[derive(Copy, Clone)]
pub enum WaitMode {
    Frozen,
    Ready
}

#[derive(Copy, Clone)]
pub enum PageSize {
    FourBytes,
    EightBytes,
    SixteenBytes,
    ThirtyTwoBytes,
}

#[derive(Copy, Clone)]
pub enum AccessMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub struct ChipSelectConfiguration {
    // 27.16 in SAM4E datasheet for details.

    // Setup parameters
    pub nwe_setup_length: u8,
    pub ncs_write_setup_length: u8,
    pub nrd_setup_length: u8,
    pub ncs_read_setup_length: u8,

    // Pulse parameters
    pub nwe_pulse_length: u8,
    pub ncs_write_pulse_length: u8,
    pub nrd_pulse_length: u8,
    pub ncs_read_pulse_length: u8,

    pub nwe_total_cycle_length: u16,
    pub nrd_total_cycle_length: u16,

    pub access_mode: AccessMode,

    pub wait_mode: Option<WaitMode>,    // If some(), wait mode is as specified, otherwise disabled.

    pub data_float_time: u8,

    pub tdf_optimization: bool,
    pub page_size: Option<PageSize>,     // If some(), page mode is enabled with the given size.
}

macro_rules! chip_select {
    (
        $ChipSelectType:ident,
        $cs:expr
    ) => {
        pub struct $ChipSelectType<MODE> {
            _mode: PhantomData<MODE>,
        }

        paste! {
            impl<MODE> $ChipSelectType<MODE> {
                pub(crate) fn setup(&mut self) -> &smc::[<SETUP $cs>] {
                    unsafe { &(*SMC::ptr()).[<setup $cs>] }
                }

                pub(crate) fn pulse(&mut self) -> &smc::[<PULSE $cs>] {
                    unsafe { &(*SMC::ptr()).[<pulse $cs>] }
                }

                pub(crate) fn cycle(&mut self) -> &smc::[<CYCLE $cs>] {
                    unsafe { &(*SMC::ptr()).[<cycle $cs>] }
                }

                pub(crate) fn mode(&mut self) -> &smc::[<MODE $cs>] {
                    unsafe { &(*SMC::ptr()).[<mode $cs>] }
                }

                pub fn into_configured_state(mut self, config: &ChipSelectConfiguration) -> $ChipSelectType<Configured> {
                    self.setup().write(|w| unsafe { 
                        w.nwe_setup().bits(config.nwe_setup_length).
                          ncs_wr_setup().bits(config.ncs_write_setup_length).
                          nrd_setup().bits(config.nrd_setup_length).
                          ncs_rd_setup().bits(config.ncs_read_setup_length)
                    });  

                    self.pulse().write(|w| unsafe {
                        w.nwe_pulse().bits(config.nwe_pulse_length).
                          ncs_wr_pulse().bits(config.ncs_write_pulse_length).
                          nrd_pulse().bits(config.nrd_pulse_length).
                          ncs_rd_pulse().bits(config.ncs_read_pulse_length)
                    });

                    self.cycle().write(|w| unsafe {
                        w.nwe_cycle().bits(config.nwe_total_cycle_length).
                          nrd_cycle().bits(config.nrd_total_cycle_length)
                    });

                    // WARNING: Mode register *must* be writen after the above registers in order to
                    // 'validate' the new configuration.    See 27.11.3.1 in datasheet.
                    self.mode().write(|w| unsafe {
                        match config.access_mode {
                            AccessMode::ReadOnly => w.read_mode().set_bit(),
                            AccessMode::WriteOnly => w.write_mode().set_bit(),
                            AccessMode::ReadWrite => w.read_mode().set_bit().write_mode().set_bit(),
                        };

                        if let Some(wait_mode) = config.wait_mode {
                            let mode = match wait_mode {
                                WaitMode::Frozen => 2,
                                WaitMode::Ready => 3,
                            };
                            w.exnw_mode().bits(mode);
                        }
                        else {
                            w.exnw_mode().bits(0);
                        }
    
                        w.tdf_cycles().bits(config.data_float_time);
    
                        if (config.tdf_optimization) {
                            w.tdf_mode().set_bit();
                        }
                        else {
                            w.tdf_mode().clear_bit();
                        }
                        
                        if let Some(page_size) = config.page_size {
                            let value = match page_size {
                                PageSize::FourBytes => 0,
                                PageSize::EightBytes => 1,
                                PageSize::SixteenBytes => 2,
                                PageSize::ThirtyTwoBytes => 3,
                            };

                            w.pmen().set_bit().ps().bits(value);
                        }
                        else {
                            w.pmen().clear_bit().ps().bits(0);
                        }

                        w
                    });

                    $ChipSelectType { _mode: PhantomData }
                }
            }
        }
    }
}

chip_select!(ChipSelect0, 0);
chip_select!(ChipSelect1, 1);
chip_select!(ChipSelect2, 2);
chip_select!(ChipSelect3, 3);

pub struct StaticMemoryController {
    clock: PhantomData<StaticMemoryControllerClock<Enabled>>,

    ncs1: PhantomData<NCS1>,
    ncs3: PhantomData<NCS3>,

    nrd: PhantomData<Pc11<PfA>>,
    nwe: PhantomData<Pc8<PfA>>,

    d0: PhantomData<Pc0<PfA>>,
    d1: PhantomData<Pc1<PfA>>,
    d2: PhantomData<Pc2<PfA>>,
    d3: PhantomData<Pc3<PfA>>,
    d4: PhantomData<Pc4<PfA>>,
    d5: PhantomData<Pc5<PfA>>,
    d6: PhantomData<Pc6<PfA>>,
    d7: PhantomData<Pc7<PfA>>,

    a0: PhantomData<Pc18<PfA>>,
    a1: PhantomData<Pc19<PfA>>,
    a2: PhantomData<Pc20<PfA>>,
    a3: PhantomData<Pc21<PfA>>,
    a4: PhantomData<Pc22<PfA>>,
    a5: PhantomData<Pc23<PfA>>,
    a6: PhantomData<Pc24<PfA>>,
    a7: PhantomData<Pc25<PfA>>,
    a8: PhantomData<Pc26<PfA>>,
    a9: PhantomData<Pc27<PfA>>,
    a10: PhantomData<Pc28<PfA>>,
    a11: PhantomData<Pc29<PfA>>,
    a12: PhantomData<Pc30<PfA>>,
    a13: PhantomData<Pc31<PfA>>,
    a14: PhantomData<Pa18<PfC>>,
    a15: PhantomData<Pa19<PfC>>,
    a16: PhantomData<Pa20<PfC>>,
    a17: PhantomData<Pa0<PfC>>,
    a18: PhantomData<Pa1<PfC>>,
    a19: PhantomData<Pa23<PfC>>,
    a20: PhantomData<Pa24<PfC>>,
    a21: PhantomData<Pc16<PfA>>,
    a22: PhantomData<Pc17<PfA>>,
    a23: PhantomData<Pa25<PfC>>,

    pub chip_select0: ChipSelect0<Uninitialized>,
    pub chip_select1: ChipSelect1<Uninitialized>,
    pub chip_select2: ChipSelect2<Uninitialized>,
    pub chip_select3: ChipSelect3<Uninitialized>,
}

pub enum NCS1 {
    C15(Pc15<PfA>),

    #[cfg(feature = "atsam4e")]
    D18(Pd18<PfA>),
}

pub enum NCS3 {
    C12(Pc12<PfA>),

    #[cfg(feature = "atsam4e")]
    D19(Pd19<PfA>),
}

impl StaticMemoryController {
    pub fn new(
        _clock: StaticMemoryControllerClock<Enabled>,

        _ncs1: NCS1,
        _ncs3: NCS3,

        _nrd: Pc11<PfA>,
        _nwe: Pc8<PfA>,
    
        _d0: Pc0<PfA>,
        _d1: Pc1<PfA>,
        _d2: Pc2<PfA>,
        _d3: Pc3<PfA>,
        _d4: Pc4<PfA>,
        _d5: Pc5<PfA>,
        _d6: Pc6<PfA>,
        _d7: Pc7<PfA>,

        _a0: Pc18<PfA>,
        _a1: Pc19<PfA>,
        _a2: Pc20<PfA>,
        _a3: Pc21<PfA>,
        _a4: Pc22<PfA>,
        _a5: Pc23<PfA>,
        _a6: Pc24<PfA>,
        _a7: Pc25<PfA>,
        _a8: Pc26<PfA>,
        _a9: Pc27<PfA>,
        _a10: Pc28<PfA>,
        _a11: Pc29<PfA>,
        _a12: Pc30<PfA>,
        _a13: Pc31<PfA>,
        _a14: Pa18<PfC>,
        _a15: Pa19<PfC>,
        _a16: Pa20<PfC>,
        _a17: Pa0<PfC>,
        _a18: Pa1<PfC>,
        _a19: Pa23<PfC>,
        _a20: Pa24<PfC>,
        _a21: Pc16<PfA>,
        _a22: Pc17<PfA>,
        _a23: Pa25<PfC>,
    ) -> Self {
        StaticMemoryController {
            clock: PhantomData,

            ncs1: PhantomData,
            ncs3: PhantomData,

            nrd: PhantomData,
            nwe: PhantomData,
        
            d0: PhantomData,
            d1: PhantomData,
            d2: PhantomData,
            d3: PhantomData,
            d4: PhantomData,
            d5: PhantomData,
            d6: PhantomData,
            d7: PhantomData,

            a0: PhantomData,
            a1: PhantomData,
            a2: PhantomData,
            a3: PhantomData,
            a4: PhantomData,
            a5: PhantomData,
            a6: PhantomData,
            a7: PhantomData,
            a8: PhantomData,
            a9: PhantomData,
            a10: PhantomData,
            a11: PhantomData,
            a12: PhantomData,
            a13: PhantomData,
            a14: PhantomData,
            a15: PhantomData,
            a16: PhantomData,
            a17: PhantomData,
            a18: PhantomData,
            a19: PhantomData,
            a20: PhantomData,
            a21: PhantomData,
            a22: PhantomData,
            a23: PhantomData,    

            chip_select0: ChipSelect0::<Uninitialized>{ _mode: PhantomData, },
            chip_select1: ChipSelect1::<Uninitialized>{ _mode: PhantomData, },
            chip_select2: ChipSelect2::<Uninitialized>{ _mode: PhantomData, },
            chip_select3: ChipSelect3::<Uninitialized>{ _mode: PhantomData, },
        }
    }

    pub fn base_address(&self, chip_select: u8) -> usize {
        match chip_select {
            0 => 0x6000_0000,
            1 => 0x6100_0000,
            2 => 0x6200_0000,
            3 => 0x6300_0000,
            _ => panic!("Unrecognized chip select provided: {}", chip_select),
        }
    }
}
