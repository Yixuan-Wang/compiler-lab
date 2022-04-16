use koopa::ir;

use crate::{front::symtab::FuncTab, ty};

fn decl_func<'a: 'b, 'b>(
    program: &'a mut ir::Program,
    name: &'static str,
    params_ty: &'b [ir::Type],
    ret_ty: ir::Type,
) -> ir::Function {
    let func_data = ir::FunctionData::new_decl(name.to_string(), params_ty.into(), ret_ty);
    let func = program.new_func(func_data);
    func
}

/*
 * decl @getint(): i32
 * decl @getch(): i32
 * decl @getarray(*i32): i32
 * decl @putint(i32)
 * decl @putch(i32)
 * decl @putarray(i32, *i32)
 * decl @starttime()
 * decl @stoptime()
 */

pub fn with_prelude(program: &mut ir::Program, func_tab: &mut FuncTab) {
    let lib_funcs = [
        ("@getint", vec![], ty!(i32)),
        ("@getch", vec![], ty!(i32)),
        ("@getarray", vec![ty!(*i32)], ty!(i32)),
        ("@putint", vec![ty!(i32)], ty!(())),
        ("@putch", vec![ty!(i32)], ty!(())),
        ("@putarray", vec![ty!(i32), ty!(*i32)], ty!(())),
        ("@starttime", vec![], ty!(())),
        ("@stoptime", vec![], ty!(())),
    ];
    let lib_func_names: Vec<_> = lib_funcs
        .iter()
        .map(|(h, ..)| unsafe { h.get_unchecked(1..) }.to_string())
        .collect();
    lib_funcs
        .into_iter()
        .map(|(n, ref p, r)| decl_func(program, n, p, r))
        .zip(lib_func_names)
        .for_each(|(h, n)| {
            func_tab.insert(n, h);
        });
}
