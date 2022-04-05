use super::*;

/*

||:
  res = 1
  if (l == 0) res = (r != 0)

&&
  res = 0
  if (l != 0) res = (r != 0)

*/

impl<'f> Generate<'f> for ast::LAndExp {
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        match self {
            Self::Unary(p) => p.generate(ctx),
            Self::Binary(l, r) => {
                let res_name = ctx.variable_namer.gen(Some("%lazy_land".to_string()));
                let res = ctx.add_value(val!(alloc(ir::Type::get_i32())), Some(res_name));
                ctx.insert_inst(res, ctx.curr());

                let zero = ctx.zero;
                let init_res = ctx.add_value(val!(store(zero, res)), None);
                ctx.insert_inst(init_res, ctx.curr());

                let block_right_name = ctx.inst_namer.gen(Some("lazy_land_right".to_string()));
                let block_skip_name = ctx.inst_namer.gen(Some("lazy_land_skip".to_string()));

                let block_right = ctx.add_block(&block_right_name);
                let block_skip = ctx.add_block(&block_skip_name);

                {
                    let l = l.generate(ctx);
                    let gate = ctx.add_mid_value(val!(binary(ir::BinaryOp::NotEq, l, zero)));
                    ctx.insert_inst(gate, ctx.curr());
                    let branch = ctx.add_value(val!(branch(gate, block_right, block_skip)), None);
                    ctx.insert_inst(branch, ctx.curr());
                    ctx.seal_block(ctx.curr());
                }

                {
                    ctx.insert_block(block_right);
                    ctx.set_curr(block_right);
                    let r = r.generate(ctx);
                    let r_is_zero = ctx.add_mid_value(val!(binary(ir::BinaryOp::NotEq, r, zero)));
                    ctx.insert_inst(r_is_zero, ctx.curr());
                    let store_res = ctx.add_value(val!(store(r_is_zero, res)), None);
                    ctx.insert_inst(store_res, ctx.curr());
                    let jump = ctx.add_value(val!(jump(block_skip)), None);
                    ctx.insert_inst(jump, ctx.curr());
                    ctx.seal_block(ctx.curr());
                }
                

                ctx.insert_block(block_skip);
                ctx.set_curr(block_skip);
                
                let load_res = ctx.add_mid_value(val!(load(res)));
                ctx.insert_inst(load_res, ctx.curr());
                load_res
            }
        }
    }
}

impl<'f> Generate<'f> for ast::LOrExp {
    type Val = ir::Value;
    fn generate(&self, ctx: &'f mut Context) -> Self::Val {
        match self {
            Self::Unary(p) => p.generate(ctx),
            Self::Binary(l, r) => {
                let res_name = ctx.variable_namer.gen(Some("%lazy_lor".to_string()));
                let res = ctx.add_value(val!(alloc(ir::Type::get_i32())), Some(res_name));
                ctx.insert_inst(res, ctx.curr());

                let one = ctx.one;
                let zero = ctx.zero;

                let init_res = ctx.add_value(val!(store(one, res)), None);
                ctx.insert_inst(init_res, ctx.curr());

                let block_right_name = ctx.inst_namer.gen(Some("lazy_lor_right".to_string()));
                let block_skip_name = ctx.inst_namer.gen(Some("lazy_lor_skip".to_string()));

                let block_right = ctx.add_block(&block_right_name);
                let block_skip = ctx.add_block(&block_skip_name);

                {
                    let l = l.generate(ctx);
                    let gate = ctx.add_mid_value(val!(binary(ir::BinaryOp::Eq, l, zero)));
                    ctx.insert_inst(gate, ctx.curr());
                    let branch = ctx.add_value(val!(branch(gate, block_right, block_skip)), None);
                    ctx.insert_inst(branch, ctx.curr());
                    ctx.seal_block(ctx.curr());
                }

                {
                    ctx.insert_block(block_right);
                    ctx.set_curr(block_right);
                    let r = r.generate(ctx);
                    let r_is_zero = ctx.add_mid_value(val!(binary(ir::BinaryOp::NotEq, r, zero)));
                    ctx.insert_inst(r_is_zero, ctx.curr());
                    let store_res = ctx.add_value(val!(store(r_is_zero, res)), None);
                    ctx.insert_inst(store_res, ctx.curr());
                    let jump = ctx.add_value(val!(jump(block_skip)), None);
                    ctx.insert_inst(jump, ctx.curr());
                    ctx.seal_block(ctx.curr());
                }
                

                ctx.insert_block(block_skip);
                ctx.set_curr(block_skip);
                
                let load_res = ctx.add_mid_value(val!(load(res)));
                ctx.insert_inst(load_res, ctx.curr());
                load_res
            }
        }
    }
}