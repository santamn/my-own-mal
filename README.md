# My Own Mal

[mal (Make-A-Lisp)](https://github.com/kanaka/mal)の自分用の実装です。

## Rust

### 参考にしたもの

- [mal-rust](https://github.com/seven1m/mal-rust/tree/master/rust/src)
- [mal/impls/rust](https://github.com/kanaka/mal/blob/master/impls/rust)

### Build

impls/rustディレクトリで

```
$ make stepX
```

### Run the REPL

トップレベルで
```
$ impls/rust/target/release/stepX_XXX
```

### Test

トップレベルで
```
$ make test^rust^stepX
```

## 今後やりたい言語

- Clojure
- Go
- Haskell
- Julia
- Scala
- Scheme
- Zig