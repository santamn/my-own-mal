use crate::types::{MalError, MalResult, MalVal, Paren};
use std::borrow::Borrow;
use std::collections::LinkedList;
use std::iter::Peekable;

// # やりたいこと
// - read_list/vec/hashmap/hashsetのwhile letの部分を共通化 -> read_seq関数
//   - read_seq関数: readerを受け取り、MalValのイテレータ(= SeqItems)を返す
// - トークン列をRcで包めば部分的にイテレータを取り出せるのではないか
// - SeqItem: 内部的にはトークン列を持っていて、それをMalValに変換するイテレータ
//   - イテレータのイテレータのように、再帰構造になってしまう?
struct SeqItems<I> {
    seq: I, // reader中の()の中身の部分を指すイテレータ
}

impl<I, S> Iterator for SeqItems<I>
where
    I: Iterator<Item = S>,
    S: Borrow<str> + ToString,
{
    type Item = MalVal;

    fn next(&mut self) -> Option<Self::Item> {
        read_form(&mut self.seq.by_ref().peekable()).ok()
    }
}

fn read_form<I, S>(reader: &mut Peekable<I>) -> MalResult
where
    I: Iterator<Item = S>,
    S: Borrow<str> + ToString,
{
    match reader.peek().ok_or(MalError::NoInput)?.borrow() {
        "(" => read_list(reader),
        _ => read_atom(reader.next().unwrap()),
    }
}

// read_seqでリーダーを掘っていく
// read_formを呼ぶと必ず次のフォームが返ってくる
fn read_seq<I, S>(reader: &mut I, paren: Paren) -> Result<SeqItems<I>, MalError>
where
    I: Iterator<Item = S>,
    S: Borrow<str> + ToString,
{
    todo!("read_seq")
}

// listがcollectしてるから意味ない?
fn read_list<I, S>(reader: &mut Peekable<I>) -> MalResult
where
    I: Iterator<Item = S>,
    S: Borrow<str> + ToString,
{
    Ok(MalVal::list(
        read_seq(reader, Paren::Round)?.collect::<LinkedList<MalVal>>(),
    ))
}

fn read_atom<S: Borrow<str>>(atom: S) -> MalResult {
    unimplemented!("read_atom")
}
