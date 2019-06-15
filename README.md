# NAND2Tetris in Rust

いわゆる NAND2Tetris(https://www.nand2tetris.org/ )「コンピュータシステムの理論と実装」をRustで実装したものです。といっても道半ばで止まっていますが…orz

一番おもしろい（？）のは machine/ プロジェクトです。
machine/given.rs で下記のように NAND と FlipFlop が定義されています。

```
pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}

pub struct Flipflop { bit: bool }

impl Flipflop {
    pub fn new() -> Self {
        Flipflop{ bit: false }
    }
    pub fn out(&self) -> bool {
        self.bit
    }
    pub fn clock(&mut self, a: bool) {
        self.bit = a;
    }
}
```

この NAND と FlipFlop がもっとも基本となる素子で、唯一 Rust の「高級な（？）」機能が使われている箇所です。
これ以外の箇所は：

* if 式などの条件分岐は一切使っていません。
* for などのループ構文は一切使っていません。
* 足し算などの演算子は一切使っていません。

この制約のもとで、NAND と FlipFlop のみを利用して ALU や RAM をシミュレートしています。
