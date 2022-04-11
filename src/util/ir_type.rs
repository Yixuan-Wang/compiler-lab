#[macro_export]
macro_rules! ty {
    (i32) => {
        koopa::ir::types::Type::get_i32()
    };
    (()) => {
        koopa::ir::types::Type::get_unit()
    };
    ([ $base:tt ; $len:literal ]) => {
        koopa::ir::types::Type::get_array(ty!($base), $len)
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