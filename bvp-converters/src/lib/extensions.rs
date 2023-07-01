use std::hash::Hash;

#[derive(Clone, Copy, Debug)]
pub enum Extension {
    ExtFormatMono
}

impl Extension {
    pub fn to_string(&self) -> String {
        return match self {
            Extension::ExtFormatMono => "EXT_format_mono".to_string()
        }
    }
}

impl PartialEq for Extension {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for Extension {}

impl Hash for Extension {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}