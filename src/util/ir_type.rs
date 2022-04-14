#[macro_export]
macro_rules! ty {
    (i32) => {
        koopa::ir::types::Type::get_i32()
    };
    (()) => {
        koopa::ir::types::Type::get_unit()
    };
    ([ $base:tt ; $len:expr ]) => {
        koopa::ir::types::Type::get_array(ty!($base), $len)
    };
    ([ $base:expr ; $len:expr ]) => {
        koopa::ir::types::Type::get_array($base, $len)
    };
    (*$base:tt) => {
        koopa::ir::types::Type::get_pointer(ty!($base))
    };
    (fn ( $($t:tt),* )) => {
        koopa::ir::types::Type::get_function(vec![$(ty!($t)),*], ty!(()))
    };
    (fn ( $($t:tt),* ) -> $ret:tt) => {
        koopa::ir::types::Type::get_function(vec![$(ty!($t)),*], ty!($ret))
    };
}

pub fn to_array_ty(iter: impl Iterator<Item = usize>) -> koopa::ir::types::Type {
    iter
    .fold(ty!(i32), |ty, d: usize| koopa::ir::Type::get_array(ty, d))
}

pub fn ref_to_array_ty<'a>(iter: impl Iterator<Item = &'a usize>) -> koopa::ir::types::Type {
    iter
    .fold(ty!(i32), |ty, d: &usize| koopa::ir::Type::get_array(ty, *d))
}
