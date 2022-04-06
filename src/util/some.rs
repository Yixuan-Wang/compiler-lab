#[macro_export]
macro_rules! s {
    ($s: literal) => {
        String::from($s)
    };
    (?$s: literal) => {
        Some(String::from($s))
    };
}
