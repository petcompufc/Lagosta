use godot::prelude::*;
use std::fmt::Display;

#[derive(GodotConvert, Var, Export, Default, Clone, Debug, Copy)]
#[godot(via=u8)]
#[repr(u8)]
pub enum OCIModalidade {
    #[default]
    IniA = 0,
    IniB = 1,
    Prog = 2,
}

#[derive(GodotConvert, Var, Export, Default, Clone, Debug)]
#[godot(via=u8)]
#[repr(u8)]
pub enum OCIFase {
    #[default]
    Fase1 = 0,
    Fase2 = 1,
    Fase3 = 2,
}

impl OCIModalidade {
    pub fn char(&self) -> &str {
        match self {
            Self::IniA => "a",
            Self::IniB => "b",
            Self::Prog => "b",
        }
    }
}

impl OCIFase {
    pub fn char(&self) -> &str {
        match self {
            Self::Fase1 => "1",
            Self::Fase2 => "2",
            Self::Fase3 => "3",
        }
    }
}

impl Display for OCIModalidade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IniA => write!(f, "Iniciação A"),
            Self::IniB => write!(f, "Iniciação B"),
            Self::Prog => write!(f, "Programação"),
        }
    }
}

impl Display for OCIFase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fase1 => write!(f, "1"),
            Self::Fase2 => write!(f, "2"),
            Self::Fase3 => write!(f, "3"),
        }
    }
}
