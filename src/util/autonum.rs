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
    h: HashMap<Option<String>, Autocount>,
}

impl Autonum {
    pub fn new() -> Autonum {
        let h = HashMap::from([(None, Autocount::new(None))]);
        Autonum { h }
    }

    pub fn gen(&mut self, name: Option<String>) -> String {
        match self.h.get_mut(&name) {
            Some(c) => {
                let s = match &name {
                    Some(n) => format!("{n}_{}", c.gen().unwrap()),
                    None => format!("{}", c.gen().unwrap()),
                };
                s
            }
            None => {
                let s = match &name {
                    Some(n) => format!("{n}_0"),
                    None => "0".to_string(),
                };
                let mut c = Autocount::new(None);
                c.gen().unwrap();
                self.h.insert(name, c);
                s
            }
        }
    }
}

impl Default for Autonum {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Autocount {
    count: usize,
    lim: Option<usize>,
}

impl Autocount {
    pub fn new(lim: Option<usize>) -> Autocount {
        Autocount { count: 0, lim }
    }

    pub fn gen(&mut self) -> Result<usize, &'static str> {
        if let Some(up) = self.lim {
            if up == self.count {
                return Err("Autocount overflow!");
            }
        }
        let cnt = self.count;
        self.count += 1;
        Ok(cnt)
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }
}
