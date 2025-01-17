use crate::melpow::hash;

use std::convert::TryInto;
use std::fmt;

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

pub type SVec<T> = SmallVec<[T; 32]>;
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Node {
    pub bv: u64,
    pub len: usize,
}

impl Node {
    pub fn new_zero() -> Self {
        Node { bv: 0, len: 0 }
    }

    pub fn new(bv: u64, len: usize) -> Self {
        Node { bv, len }
    }

    pub fn take(self, n: usize) -> Self {
        let mut new = self;
        new.bv &= (1 << n) - 1;
        new.len = n;
        new
    }

    pub fn append(self, n: usize) -> Self {
        let mut nd = self;
        nd.bv |= (n << nd.len) as u64;
        nd.len += 1;
        nd
    }

    pub fn get_bit(self, n: usize) -> u64 {
        self.bv >> n & 1
    }

    pub fn get_parents(self, n: usize) -> SVec<Node> {
        let mut parents = SVec::new();
        if self.len == n {
            let range = 0..n;

            range.into_iter().for_each(|index| {
                if (self.bv >> index) & 1 != 0 {
                    parents.push(self.take(index).append(0))
                }
            });
        } else {
            parents.push(self.append(0));
            parents.push(self.append(1));
        }

        parents
    }

    pub fn uniqid(self) -> u64 {
        (self.len << 56) as u64 | self.bv
    }

    pub fn to_bytes(self) -> [u8; 8] {
        self.uniqid().to_be_bytes()
    }

    pub fn from_bytes(bts: &[u8]) -> Option<Self> {
        let uniqid = u64::from_be_bytes(bts.try_into().ok()?);
        // highest 8 bits is length
        let len = (uniqid >> 56) as usize;
        // lowest 56 bits is the number
        let num = uniqid << 8 >> 8;
        Some(Node { bv: num, len })
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = {
            if self.len == 0 {
                String::from("ε")
            } else {
                (0..self.len)
                    .map(|i| if (self.bv >> i) & 1 != 0 { '1' } else { '0' })
                    .collect()
            }
        };
        write!(f, "{}", str)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = {
            if self.len == 0 {
                String::from("ε")
            } else {
                (0..self.len)
                    .map(|i| if (self.bv >> i) & 1 != 0 { '1' } else { '0' })
                    .collect()
            }
        };
        write!(f, "{}", str)
    }
}

pub fn calc_labels(chi: &[u8], n: usize, f: &mut impl FnMut(Node, &[u8])) {
    calc_labels_helper(chi, n, Node::new_zero(), f, &mut FxHashMap::default());
}

fn calc_labels_helper(
    chi: &[u8],
    n: usize,
    nd: Node,
    f: &mut impl FnMut(Node, &[u8]),
    ell: &mut FxHashMap<Node, SVec<u8>>,
) -> SVec<u8> {
    if nd.len == n {
        let mut lab_gen = hash::Accumulator::new(chi);
        lab_gen.add(&nd.to_bytes());
        let parents = nd.get_parents(n);

        parents.iter().for_each(|parent| {
            lab_gen.add(&ell[parent]);
        });

        let lab = lab_gen.hash();
        f(nd, &lab);
        lab
    } else {
        // left tree
        let l0 = calc_labels_helper(chi, n, nd.append(0), f, ell);
        ell.insert(nd.append(0), l0.clone());
        // right tree
        let l1 = calc_labels_helper(chi, n, nd.append(1), f, ell);
        ell.remove(&nd.append(0));
        // calculate label
        let lab = hash::Accumulator::new(chi)
            .add(&nd.to_bytes())
            .add(&l0)
            .add(&l1)
            .hash();
        f(nd, &lab);
        lab
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    fn print_dag(n: usize, b: Node) {
        println!("digraph G {{");
        println!("rankdir = BT;");
        println!("graph [splines=line];");
        println!("subgraph {{");
        print_dag_helper(n, b, &mut HashSet::new());
        println!("}}\n}}");
    }

    fn print_dag_helper(n: usize, b: Node, printed: &mut HashSet<(usize, Node)>) {
        if printed.contains(&(n, b)) {
            return;
        }
        printed.insert((n, b));

        b.get_parents(n).iter().for_each(|parent| {
            if parent.len <= b.len {
                println!("\"{}\" -> \"{}\" [constraint=false]", parent, b)
            } else {
                println!("\"{}\" -> \"{}\"", parent, b)
            }
            print_dag_helper(n, *parent, printed)
        });
    }

    #[test]
    fn test_dag() {
        let n = 4;
        let root = Node::new_zero();
        print_dag(n, root)
    }
}
