use std::collections::HashMap;

/* #[macro_export]
macro_rules! auton {
    ($a:expr , $e:expr) => {
        $a.gen(Some(String::from(e)))
    };
    ($a:expr ,@ $e:expr) => {
        $a.gen(Some(String::from("@").push_str($e)))
    };
    ($a:expr ,% $e:expr) => {
        $a.gen(Some(String::from("%").push_str($e)))
    };
    ($a:expr) => {
        $a.gen(None)
    };
} */

pub struct Autonum {
    h: HashMap<Option<String>, usize>,
}

impl Autonum {
    pub fn new() -> Autonum {
        let h = HashMap::from([
            (None, 0)
        ]);
        Autonum { h }
    }

    pub fn gen(&mut self, name: Option<String>) -> String {
        match self.h.get_mut(&name) {
            Some(c) => {
                let s = match &name {
                    Some(n) => format!("%{n}_{c}"),
                    None => format!("%{c}"),
                };
                *c += 1;
                s
            },
            None => {
                let s = match &name {
                    Some(n) => format!("%{n}_0"),
                    None => format!("%0"),
                };
                self.h.insert(name, 1);
                s
            },
        }
    }
}