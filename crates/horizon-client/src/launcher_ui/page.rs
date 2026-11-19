#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Play,
    Account,
    Mods,
    Packs,
    Shaders,
    Controls,
    Installed,
    VoltSettings,
    Settings,
}
impl Page {
    pub const ALL: [Self; 9] = [
        Self::Play,
        Self::Account,
        Self::Mods,
        Self::Packs,
        Self::Shaders,
        Self::Controls,
        Self::Installed,
        Self::VoltSettings,
        Self::Settings,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Play => "Play",
            Self::Account => "Account",
            Self::Mods => "Mods",
            Self::Packs => "Packs",
            Self::Shaders => "Shaders",
            Self::Controls => "Controls",
            Self::Installed => "Installed",
            Self::VoltSettings => "Volt Settings",
            Self::Settings => "Settings",
        }
    }
    pub fn is_catalog(self) -> bool {
        matches!(self, Self::Packs | Self::Shaders)
    }
}
