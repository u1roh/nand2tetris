# NAND2Tetris in Rust

いわゆる NAND2Tetris(https://www.nand2tetris.org/ )「コンピュータシステムの理論と実装」をRustで実装したものです。といっても道半ばで止まっていますが…orz

## NAND2Tetris とは

MITの講義です。コンピュータのミニチュアモデルをシミュレートするプログラムをまるっと組み上げることで、コンピュータ全体の仕組みを俯瞰して理解できるようになろうというものです。浅く広く学ぶものなので深い知識は得られませんが、コンピュータのアーキテクチャ全体を俯瞰できるようになります。

タイトルは「NANDからテトリスまで」という意味ですが、実は最後に作るのは何故かテトリスとは違ってPongという別のゲームを作る内容になっています。まあ私はそこまで到達しませんでしたが…。

## `machine/` プロジェクト

一番おもしろい（？）のは `machine/` プロジェクトだと思っています。本来の NAND2Tetris ではハードウェアは HDL (Hardware Description Language) を使って記述するのですが、私は Rust で書いてみました。ただし Rust のほんの一部の機能しか使っていません。物理的な電子回路をシミュレートするのが目的なので、0 と 1 の電気信号を結線して組み立てるだけのプログラムで、それを超える「高級な」命令は一切使わないという縛りの元で書いています。

まず `machine/given.rs` には下記のように NAND と FlipFlop が「所与のものとして」定義されています。

```rust
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

この NAND と FlipFlop がもっとも基本となる素子で、唯一 Rust の「高級な」機能が使われている箇所です。ここだけは `&&` などといった「高級な」命令を利用して実装されていますが、それ以外は NAND と FlipFlop をブラックボックスとして利用するだけで、0 と 1 の電気信号を結線する以上のことは行っていません。

これ以外の箇所は：

* プリミティブ型は `bool` しか用いていません。（`bool` の固定長配列は使っていますが）
* if 式などの条件分岐は一切使っていません。
* for などのループ構文は一切使っていません。
* 足し算などの演算子は一切使っていません。

この制約のもとで、NAND と FlipFlop のみを利用して ALU や RAM をシミュレートしています。ただひたすら、素子の出力を別の素子に繋ぎ、FlipFlop に clock信号を送るだけです。これを作ることで、コンピュータという壮大なピタゴラ装置の一端を垣間見ることができた気分になれます。

※ ただし `machine/src/blackbox.rs` には `ROM32K`、`Screen`、`Keyboard` といった外部機器をブラックボックスとしてシミュレートするためのものが用意されており、これらは「高級な」機能を利用して実装されています。

* `machine/src/gate.rs` ... `not`, `and`, `xor` などといった論理ゲートを NAND から組み立てています。
* `machine/src/adder.rs` ... 論理ゲートから16bit加算器を作ります。
* `machine/src/alu.rs` ... 加算器と論理ゲートからALUを作ります。
* `machine/src/ram.rs` ... FlipFlop を使ってレジスタを作り、レジスタを使ってRAMを作ります。
* `machine/src/inst.rs` ... CPU の命令セットを定義しています。ここはコンピュータの動きそのものをシミュレートするコードではなく、生のマシン語ではあまりに可読性が悪いのでビット列の命令とアセンブリ言語的な命令を対応付けています。
* `machine/src/cpu.rs` ... CPU を作ります。
* `machine/src/lib.rs` ... CPU と RAM と、ブラックボックスの ROM や Screen、Keyboard をつなげてマシンを作ります。
