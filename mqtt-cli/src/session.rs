pub struct Session {
    debug: bool,
    addr: (String, u16),
}

impl Session {

    pub fn new(debug: bool, addr: (String, u16)) -> Self {
        Self { debug, addr }
    }

    pub fn addr(&self) -> (String, u16) {
        self.addr.clone()
    }

    pub fn debug(&self, msg: String) {
        if self.debug {
            println!("[DEBUG] {}", msg)
        }
    }

    /// FIXME: this is just a static mock for now
    pub fn packet_identifier(&self) -> u16 {
        21
    } 
}