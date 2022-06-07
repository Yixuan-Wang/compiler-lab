use std::env::args;

/// 命令行参数解析的 helper struct
pub struct Config {
    pub mode: CompilerMode,
    pub input: String,
    pub output: String,
}

pub enum CompilerMode {
    Koopa,
    Riscv,
}

impl Config {
    pub fn new() -> Config {
        let args: Vec<_> = args().collect();
        let mut mode = CompilerMode::Koopa;
        let mut input = String::new();
        let mut output = String::new();
        for (idx, arg) in args.iter().enumerate() {
            if idx == 0 {
                continue;
            }
            if arg.starts_with('-') {
                match arg.as_str() {
                    "-koopa" => {
                        mode = CompilerMode::Koopa;
                        input.push_str(args.get(idx + 1).expect("Missing input path!"))
                    }
                    "-riscv" => {
                        mode = CompilerMode::Riscv;
                        input.push_str(args.get(idx + 1).expect("Missing input path!"))
                    }
                    "-o" => output.push_str(args.get(idx + 1).expect("Missing output path!")),
                    _ => unimplemented!(),
                }
            }
        }
        Config {
            mode,
            input,
            output,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
