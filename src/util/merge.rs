/// 合并一个迭代器中符合某种条件的元素
///
/// `P`: 合并条件和提取值 `T`
/// `A`: 累加
/// `F`: 从值返回原迭代器元素 `B`
///
/// ```
/// let v: Vec<Result<u32, i32>> = vec![Ok(1), Err(2), Err(3), Ok(4), Err(5), Ok(6)];
/// let merge = Merge::new(
///     v.into_iter(),
///     |elem| elem.err(),
///     |acc, step| acc + step,
///     |acc| Err(acc),
/// );
/// let v: Vec<_> = merge.collect();
/// assert_eq!(v, [Ok(1u32), Err(5), Ok(4u32), Err(5), Ok(6u32)]);
/// ```
///
pub struct Merge<I, P, A, F, T, B> {
    iter: I,
    p: P,
    a: A,
    f: F,
    acc: Option<T>,
    buff: Option<B>,
}

pub fn merge<I, P, A, F, T, B>(iter: I, p: P, a: A, f: F) -> Merge<I, P, A, F, T, B>
where
    I: Iterator<Item = B>,
    T: Default,
    P: Fn(&I::Item) -> Option<T>,
    A: Fn(&T, &T) -> T,
    F: Fn(T) -> I::Item,
{
    Merge {
        iter,
        p,
        a,
        f,
        acc: None,
        buff: None,
    }
}

impl<I, P, A, F, T, B> Iterator for Merge<I, P, A, F, T, B>
where
    I: Iterator<Item = B>,
    T: Default,
    P: Fn(&I::Item) -> Option<T>,
    A: Fn(&T, &T) -> T,
    F: Fn(T) -> I::Item,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buff.is_some() {
            return self.buff.take();
        }
        let ret = loop {
            let item = self.iter.next();

            let item = if item.is_none() {
                return Some((self.f)(self.acc.take()?));
            } else {
                item.unwrap()
            };

            if let Some(ref t) = (self.p)(&item) {
                let new = (self.a)(&self.acc.take().unwrap_or_default(), t);
                self.acc.replace(new);
            } else {
                if self.acc.is_some() {
                    self.buff = Some(item);
                    break (self.f)(self.acc.take().unwrap());
                } else {
                    break item;
                }
            }
        };
        Some(ret)
    }
}
