# Yixuan Wang 的 Sysy 编译器实现

## 生命周期

```rust
'a: {
    program
    'b: {
        ast
        'c: {
            ctx: &'c Context<'a>
        }
    }
}
```
