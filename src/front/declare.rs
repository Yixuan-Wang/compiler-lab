use koopa::ir::{self, builder_traits::*, Program};

use std::iter::zip;

use crate::{WrapProgram, front::context::GlobalContext, ty};

use super::{ast, context::Context, gen::Generate, symtab::{FuncTab, ValTab}};


/// [`Declare`] 处理 AST 中的条目（[`ast::Item`]）：全局常量、变量声明和函数，并为每一个函数生成上下文（[`Context`]）
pub trait Declare<'a> {
    fn declare(&self, program: &'a mut Program, func_tab: &'a mut FuncTab, global_val_tab: &'a mut ValTab);
}

impl<'a> Declare<'a> for ast::Item {
    fn declare(&self, program: &'a mut Program, func_tab: &'a mut FuncTab, global_val_tab: &'a mut ValTab) {
        use ast::ItemKind::*;
        use koopa::ir::{ValueKind, TypeKind};
        match &self.kind {
            Global(decls) => {
                use crate::front::{
                    gen::eval::Eval,
                    ast::SymKind
                };
                for d in decls {
                    let mut ctx = GlobalContext::new(program, global_val_tab);
                    match d.kind {
                        SymKind::Const => {
                            dbg!(d.exp.as_ref());
                            let val = d.exp.as_ref().unwrap().eval(&ctx).unwrap();
                            let const_val = ctx.add_global_value(val!(integer(val)), None);
                            let alloc = ctx.add_global_value(
                                val!(global_alloc(const_val)),
                                Some(format!("@{}", d.ident)),
                            );
                            ctx.register_global_value(&d.ident, alloc);
                        }
                        SymKind::Var => {
                            let v = match &d.exp {
                                Some(e) => match e.eval(&ctx) {
                                    Some(v) => ctx.add_global_value(val!(integer(v)), None),
                                    None => todo!(),
                                },
                                None => ctx.add_global_value(val!(zero_init(ty!(i32))), None),
                            };
                            let alloc = ctx.add_global_value(
                                val!(global_alloc(v)),
                                Some(format!("@{}", d.ident)),
                            );
                            ctx.register_global_value(&d.ident, alloc);
                        }
                    };
                }
            },
            Func(f) => {
                let func_data =
                    ir::FunctionData::with_param_names(
                        format!("@{}", f.ident),
                        f.params.iter().map(|p| (
                            Some(format!("@_{}", p.ident)),
                            (&p.ty).into()
                        )).collect(),
                        (&f.output).into());
                let func = program.new_func(func_data);
                func_tab.insert(f.ident.clone(), func);
                
                let mut ctx = Context::new(program, func_tab, global_val_tab, func);

                let param_values = ctx.this_func().params().to_owned();
                zip(f.params.iter(), param_values).for_each(|pair| pair.generate(&mut ctx));
                ctx.table_mut().push_scope();
                f.block.generate(&mut ctx);

                // 保证最后一个基本块有 return
                let insts = ctx.bb_node(ctx.curr()).insts();
                if (insts.back_key().is_some()
                    && !matches!(
                        ctx.value(*insts.back_key().unwrap()).kind(),
                        ValueKind::Return(_) | ValueKind::Jump(_) | ValueKind::Branch(..)
                    ))
                    || insts.back_key().is_none()
                {
                    let implicit_val = match ctx.this_func().ty().kind() {
                        TypeKind::Function(_, ret_ty) => {
                            match ret_ty.kind() {
                                TypeKind::Int32 => Some(ctx.zero),
                                TypeKind::Unit => None,
                                TypeKind::Function(..) => unreachable!(),
                                _ => unimplemented!()
                            }
                        },
                        _ => unreachable!(),
                    };
                    let implicit_ret = ctx.add_value(val!(ret(implicit_val)), None);
                    ctx.bb_node_mut(ctx.curr())
                        .insts_mut()
                        .push_key_back(implicit_ret)
                        .unwrap();
                }
            }
        };
    }
}
