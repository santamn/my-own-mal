# My Own Mal

[mal (Make-A-Lisp)](https://github.com/kanaka/mal)の自分用の実装です。

## Rust

step5(step5_tco)まで実装しました。

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

## Malの機能について

### 型

- Nil: `nil`
- Bool: `true`, `false`
- Integer: `1`, `2`, `3`, ...
- Symbol: `a`, `b`, `c`, ...
- String: `"abc"`, `"def"`, ...
- List: `()`, `(1 2 3)`, `(a b c)`, ...
- Vector: `[]`, `[1 2 3]`, `[a b c]`, ...
- Hashmap: `{}`, `{"a" 1 "b" 2 "c" 3}`, ...
- Function: `(fn [a b] (+ a b))`, ...

### 変数・関数定義・if・do・let*・eval

- `(def! x 3)`
- `(def f (fn* [a b] (+ a b)))`
- `(if true 1 2)`
- `(do (def! x 3) (def! y 4) (+ x y))`
- `(let* [x 3 y 4] (+ x y))`
- `(eval (list + 1 2))`

### できないこと

マクロは未実装。

### 言い訳

step6のatomというデータ型(OCamlのrefに似ている)を実装する予定でしたが、step5で末尾最適化を行ったことで環境がバグり散らかしてしまいました。そのため、上記のstep4までの機能で採点をお願いします。

## 今後やりたい言語

- Clojure
- Go
- Haskell
- Julia
- Scala
- Scheme
- Zig