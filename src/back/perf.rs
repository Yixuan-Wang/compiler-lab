pub mod peephole {
    use crate::back::risc::{RiscInst, RiscItem};
    use std::borrow::Cow;

    pub trait Peephole {
        fn remove_unnecessary_load_store(self) -> Self;
    }

    impl Peephole for Vec<RiscItem> {
        fn remove_unnecessary_load_store(self) -> Self
        {
            use RiscInst::*;

            self
                .iter()
                .take(1)
                .map(RiscItem::clone)
                .chain(
                    self
                    .windows(2)
                    .map(|window| {
                        let (item_1, item_2) =
                            unsafe { (window.get_unchecked(0), window.get_unchecked(1)) };
                        if let (RiscItem::Inst(inst_1), RiscItem::Inst(inst_2)) = (item_1, item_2) {
                            match (inst_1, inst_2) {
                                (
                                    Lw(rs_l, imm_l, rd_l),
                                    Sw(rs_s, imm_s, rd_s),
                                ) if rs_l == rs_s && imm_l == imm_s && rd_l == rd_s 
                                    => None,
                                (
                                    Sw(rs_s, imm_s, rd_s),
                                    Lw(rs_l, imm_l, rd_l),
                                ) if rd_l == rd_s && imm_l == imm_s => {
                                    if rs_s == rs_l {
                                        None
                                    } else {
                                        Some(Cow::Owned(
                                            RiscItem::Inst(Mv(*rs_l, *rs_s)),
                                        ))
                                    }
                                }
                                _ => Some(Cow::Borrowed(item_2)),
                            }
                        } else {
                            Some(Cow::Borrowed(item_2))
                        }
                    })
                    .flatten()
                    .map(|x| (*x).clone())
                )
                .collect()
        }
    }
}

#[cfg(test)]
mod test_this {
    use std::borrow::Cow;

    #[test]
    fn test_this() {
        let x = [1, 2, 3, 4, 5, 6];
        let v: Vec<_> = x.into_iter().map(Cow::Owned).map(filter).collect();

        dbg!(v);
    }

    fn filter(i: Cow<i32>) -> Cow<i32> {
        if *i % 2 == 0 {
            i
        } else {
            Cow::Owned(-*i)
        }
    }
}
