use std::fs;

use compiler::{cli, front, back};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let config = cli::Config::new();
    let source = String::from_utf8(fs::read(&config.input)?)?;
    let ir = front::into_ir(source);
    match &config.mode {
        cli::CompilerMode::Koopa => {
            let koopa = front::into_ir_text(ir)?;
            fs::write(&config.output, koopa)?;
            Ok(())
        },
        cli::CompilerMode::Riscv => {
            let riscv = back::into_riscv(ir)?;
            fs::write(&config.output, riscv)?;
            Ok(())
        },
    }
}

#[cfg(test)]
mod test {
    use std::fs::{self, read_dir};
    use crate::{front, back};

    fn read_test_file() -> String {
        let s = String::from_utf8(fs::read("this.test.sysy").unwrap()).unwrap();
        s
    }

    #[test]
    fn ast() {
        let source = read_test_file();
        let ast = front::into_ast(source);
        dbg!(&ast);
    }

    #[test]
    fn koopa() {
        let source = read_test_file();
        let koopa = front::into_ir(source);
        let koopa = front::into_ir_text(koopa).unwrap();
        print!("{}", koopa);
    }

    #[test]
    fn riscv() {
        let source = read_test_file();
        let koopa = front::into_ir(source);
        let riscv = back::into_riscv(koopa).unwrap();
        print!("{}", riscv);
    }
}