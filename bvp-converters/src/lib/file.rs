use std::{rc::Rc, path::Path, fs};

pub struct File {
    pub name: String,
    pub data: Rc<Vec<u8>>,
    pub mime: Option<String>
}

impl File {
    pub fn new(name: String, data: Rc<Vec<u8>>, mime: Option<String>) -> Self {
        return Self { name, data, mime };
    }

    pub fn write(&self) -> Result<(), String> {
        let path = Path::new(&self.name);
        let prefix = path.parent().unwrap();
        match fs::create_dir_all(prefix) {
            Ok(_) => (),
            Err(_) => {
                return Err(format!("Could not create path {:?}", prefix));
            },
        };
        match fs::write(&self.name, self.data.as_slice()) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!("Error writing file {}: {}", self.name, e));
            },
        };

        return Ok(());
    }
}