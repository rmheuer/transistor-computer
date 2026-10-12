/*

   Reverser  Shift4  Shift2  Shift1  Reverser
0         a       b       c       d         e  0
1  a      a  b    b  c    c  d    d  e      e  1
2  a      a  b    b  c    c  d    d  e      e  2
3  a      a  b    b  c    c  d    d  e      e  3
4  a      a  b    b  c    c  d    d  e      e  4
5  a      a  b    b  c    c  d    d  e      e  5
6  a      a  b    b  c    c  d    d  e      e  6
7  a      a  b    b  c    c  d    d  e      e  7

 */

use int_enum::IntEnum;
use itertools::Itertools;
use priority_queue::PriorityQueue;

enum Mapping {
    Identity,
    Reverser,
    Shift4,
    Shift2,
    Shift1,
}

impl Mapping {
    // Maps input index to set of outputs that require it
    #[rustfmt::skip]
    pub fn connections(self) -> &'static [&'static [u8]; 8] {
        match self {
            Self::Identity => &[&[0],    &[1],    &[2],    &[3],    &[4],    &[5],    &[6],    &[7]],
	    Self::Reverser => &[&[0, 7], &[1, 6], &[2, 5], &[3, 4], &[3, 4], &[2, 5], &[1, 6], &[0, 7]],
	    Self::Shift4   => &[&[0, 4], &[1, 5], &[2, 6], &[3, 7], &[0, 4], &[1, 5], &[2, 6], &[3, 4, 5, 6, 7]],
	    Self::Shift2   => &[&[0, 6], &[1, 7], &[0, 2], &[1, 3], &[2, 4], &[3, 5], &[4, 6], &[5, 6, 7]],
	    Self::Shift1   => &[&[0, 7], &[0, 1], &[1, 2], &[2, 3], &[3, 4], &[4, 5], &[5, 6], &[6, 7]],
	}
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntEnum)]
enum Layer {
    Start = 0,
    StartToReverser = 1,
    ReverserToShift4 = 2,
    Shift4ToShift2 = 3,
    Shift2ToShift1 = 4,
    Shift1ToReverser = 5,
    ReverserToEnd = 6,
    End = 7,
}
impl Layer {
    pub const COUNT: usize = 8;

    pub fn next(self) -> Self {
        Self::try_from((self as u8) + 1).expect("no next layer")
    }

    pub fn next_mapping(self) -> Mapping {
        match self {
            Self::Start => Mapping::Identity,
            Self::StartToReverser => Mapping::Reverser,
            Self::ReverserToShift4 => Mapping::Shift4,
            Self::Shift4ToShift2 => Mapping::Shift2,
            Self::Shift2ToShift1 => Mapping::Shift1,
            Self::Shift1ToReverser => Mapping::Reverser,
            Self::ReverserToEnd => Mapping::Identity,
            Self::End => panic!("No mapping from end"),
        }
    }
}

#[derive(Clone, Copy)]
struct PermutationIndex(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeIndex(usize);

#[derive(Debug, Default, Clone)]
struct SearchNode {
    cost_so_far: Option<u8>,
    came_from: Option<NodeIndex>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EdgeKind {
    Falling,
    Rising,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    index: u8,
    kind: EdgeKind,
}

#[derive(Debug)]
struct SearchResult {
    positions: Vec<Vec<u8>>,
}

struct Searcher {
    permutations: Vec<Vec<u8>>,
    nodes: Vec<SearchNode>,

    edges_scratch: Vec<Edge>,
    results: Vec<SearchResult>,
}

impl Searcher {
    pub fn new() -> Self {
        let permutations = (0..8).permutations(8).collect_vec();
        let nodes = vec![Default::default(); permutations.len() * Layer::COUNT];

        Self {
            permutations,
            nodes,
            edges_scratch: Vec::with_capacity(16),
            results: Vec::new(),
        }
    }

    pub fn node(&self, permutation: PermutationIndex, layer: Layer) -> NodeIndex {
        NodeIndex(permutation.0 + (layer as u8) as usize * self.permutations.len())
    }

    // Permutation is the arrangement of outputs
    pub fn node_permutation(&self, index: NodeIndex) -> PermutationIndex {
        PermutationIndex(index.0 % self.permutations.len())
    }

    pub fn node_layer(&self, index: NodeIndex) -> Layer {
        Layer::try_from((index.0 / self.permutations.len()) as u8).expect("invalid node index")
    }

    pub fn count_overlaps(
        parent_perm: &[u8],
        child_perm: &[u8],
        connections: &[&[u8]],
        edges_scratch: &mut Vec<Edge>,
    ) -> u8 {
        edges_scratch.clear();
        for input_index in 0..8 {
            let input_position = parent_perm[input_index];

            let mut min = input_position;
            let mut max = input_position;

            for &output_index in connections[input_index] {
                let output_position = child_perm[output_index as usize];
                min = min.min(output_position);
                max = max.max(output_position);
            }

            if min != max {
                edges_scratch.push(Edge {
                    index: min,
                    kind: EdgeKind::Rising,
                });
                edges_scratch.push(Edge {
                    index: max,
                    kind: EdgeKind::Falling,
                });
            }
        }
        edges_scratch.sort();

        //println!("{edges_scratch:#?}");

        let mut max_overlapping = 0;
        let mut current_overlapping = 0;
        for edge in edges_scratch {
            if edge.kind == EdgeKind::Rising {
                current_overlapping += 1;
                max_overlapping = max_overlapping.max(current_overlapping);
            } else {
                current_overlapping -= 1;
            };
        }

        max_overlapping
    }

    fn check_child(
        &mut self,
        parent: NodeIndex,
        child: NodeIndex,
        frontier: &mut PriorityQueue<NodeIndex, std::cmp::Reverse<u8>>,
    ) {
        let parent_perm = &self.permutations[self.node_permutation(parent).0];
        let child_perm = &self.permutations[self.node_permutation(child).0];

        let parent_layer = self.node_layer(parent);
        let child_layer = self.node_layer(child);
        let connections = parent_layer.next_mapping().connections();

        let parent_to_child_cost = Self::count_overlaps(
            parent_perm,
            child_perm,
            connections,
            &mut self.edges_scratch,
        );

        let parent_cost = self.nodes[parent.0].cost_so_far.unwrap();
        let new_cost = parent_cost + parent_to_child_cost;

        let child_node = &mut self.nodes[child.0];
        let heuristic = if (child_layer as usize) < Layer::COUNT - 2 {
            (Layer::COUNT - (child_layer as usize) - 2) * 2
        } else {
            0
        };

        if match child_node.cost_so_far {
            Some(cost) => new_cost < cost,
            None => true,
        } {
            child_node.cost_so_far = Some(new_cost);
            frontier.push(child, std::cmp::Reverse(new_cost + heuristic as u8));
            child_node.came_from = Some(parent);
        }
    }

    pub fn run_search(mut self) -> Vec<SearchResult> {
        let mut frontier = PriorityQueue::new();

        let start = self.node(PermutationIndex(0), Layer::Start);
        self.nodes[start.0] = SearchNode {
            cost_so_far: Some(0),
            came_from: None,
        };
        frontier.push(start, std::cmp::Reverse(0));

        while let Some((current, est_final_cost)) = frontier.pop() {
            let current_layer = self.node_layer(current);

            println!(
                "@{} {:?}[{}] -> {:?}",
                est_final_cost.0,
                current_layer,
                self.node_permutation(current).0,
                self.nodes[current.0]
            );

            if current_layer == Layer::End {
                // Reached end
                let result = self.trace_result();
                self.results.push(SearchResult { positions: result });
                continue;
            }

            if current_layer == Layer::ReverserToEnd {
                // Only one option
                self.check_child(
                    current,
                    self.node(PermutationIndex(0), Layer::End),
                    &mut frontier,
                );
            } else {
                // All possible permutations are options
                for permutation_idx in 0..self.permutations.len() {
                    self.check_child(
                        current,
                        self.node(PermutationIndex(permutation_idx), current_layer.next()),
                        &mut frontier,
                    );
                }
            }
        }

        self.results
    }

    fn trace_result(&self) -> Vec<Vec<u8>> {
        let mut result = Vec::new();

        let mut node = self.node(PermutationIndex(0), Layer::End);
        while let Some(parent) = self.nodes[node.0].came_from {
            let perm = self.permutations[self.node_permutation(parent).0].clone();
            result.push(invert_permutation(&perm));
            node = parent;
        }
        result.reverse();

        result
    }
}

fn invert_permutation(perm_in: &[u8]) -> Vec<u8> {
    let mut out = vec![0; 8];
    for i in 0..8 {
        out[perm_in[i] as usize] = i as u8;
    }
    out
}

fn main() {
    let search = Searcher::new();
    let result = search.run_search();

    println!("Finished! Results:");
    for r in result {
        println!("  {r:?}");
    }
}
