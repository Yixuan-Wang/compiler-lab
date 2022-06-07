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

/// 自动编号
pub struct Autonum {
    h: HashMap<String, Autocount>,
    temp: Autocount,
}

impl Autonum {
    pub fn new() -> Autonum {
        Autonum {
            h: HashMap::new(),
            temp: Autocount::new(0, None),
        }
    }

    pub fn gen(&mut self, name: &str) -> String {
        let s = name.to_string();
        match self.h.get_mut(&s) {
            Some(c) => format!("{name}_{}", c.gen().unwrap()),
            None => {
                let mut c = Autocount::new(0, None);
                c.gen().unwrap();
                self.h.insert(s, c);
                format!("{name}_0")
            }
        }
    }

    pub fn gen_temp(&mut self) -> String {
        format!("{}", self.temp.gen().unwrap())
    }
}

impl Default for Autonum {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Autocount {
    count: i32,
    start: i32,
    lim: Option<i32>,
}

impl Autocount {
    pub fn new(start: i32, lim: Option<i32>) -> Autocount {
        Autocount {
            count: start,
            start,
            lim,
        }
    }

    pub fn gen(&mut self) -> Result<i32, &'static str> {
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
        self.count = self.start;
    }
}
