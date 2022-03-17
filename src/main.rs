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
        }
        cli::CompilerMode::Riscv => {
            let riscv = back::into_riscv(ir)?;
            fs::write(&config.output, riscv)?;
            Ok(())
        }
        _ => unimplemented!()
    }
    // let args: Vec<String> = args().into_iter().collect();

    // let file = String::from_utf8(fs::read(&args[2]).unwrap()).unwrap();

    // let sysy_parser = parser::CompUnitParser::new();
    // let ast = sysy_parser.parse(&file).unwrap();
    // let ir: Ir = ast.try_into().unwrap();
    // let text: String = ir.try_into().unwrap();
    
    // fs::write(&args[4], text).unwrap();
}

#[test]
fn test_parse() {
    let source = r#"
    int main() {
        return 0;
    }
    "#.to_string();
    let ast = front::into_ast(source);
    dbg!(&ast);
    let ir: front::Ir = ast.try_into().unwrap();
    // let text: String = ir.try_into().unwrap();
    let asm: back::Target = ir.try_into().unwrap();
    print!("{}", asm.0);
}
