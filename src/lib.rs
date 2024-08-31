//! Generate files suitable for use with [Graphviz](https://www.graphviz.org/)
//!
//! The `render` function generates output (e.g., an `output.dot` file) for
//! use with [Graphviz](https://www.graphviz.org/) by walking a labeled
//! graph. (Graphviz can then automatically lay out the nodes and edges
//! of the graph, and also optionally render the graph as an image or
//! other [output formats](https://www.graphviz.org/docs/outputs), such as SVG.)
//!
//! Rather than impose some particular graph data structure on clients,
//! this library exposes two traits that clients can implement on their
//! own structs before handing them over to the rendering function.
//!
//! Note: This library does not yet provide access to the full
//! expressiveness of the [DOT language](https://www.graphviz.org/doc/info/lang.html).
//! For example, there are many [attributes](https://www.graphviz.org/doc/info/attrs.html)
//! related to providing layout hints (e.g., left-to-right versus top-down, which
//! algorithm to use, etc). The current intention of this library is to
//! emit a human-readable .dot file with very regular structure suitable
//! for easy post-processing.
//!
//! # Examples
//!
//! The first example uses a very simple graph representation: a list of
//! pairs of ints, representing the edges (the node set is implicit).
//! Each node label is derived directly from the int representing the node,
//! while the edge labels are all empty strings.
//!
//! This example also illustrates how to use `Cow<[T]>` to return
//! an owned vector or a borrowed slice as appropriate: we construct the
//! node vector from scratch, but borrow the edge list (rather than
//! constructing a copy of all the edges from scratch).
//!
//! The output from this example renders five nodes, with the first four
//! forming a diamond-shaped acyclic graph and then pointing to the fifth
//! which is cyclic.
//!
//! ```rust
//! use std::io::Write;
//!
//! use dotwalk as dot;
//!
//! type Nd = isize;
//! type Ed = (isize, isize);
//! struct Edges(Vec<Ed>);
//!
//! pub fn render_to<W: Write>(output: &mut W) {
//!     let edges = Edges(vec![(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 4)]);
//!     dot::render(&edges, output).unwrap()
//! }
//!
//! impl<'a> dot::Labeller<'a> for Edges {
//!     type Node = Nd;
//!     type Edge = Ed;
//!     type Subgraph = ();
//!
//!     fn graph_id(&'a self) -> dot::Id<'a> {
//!         dot::Id::new("example1").unwrap()
//!     }
//!
//!     fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
//!         dot::Id::new(format!("N{}", *n)).unwrap()
//!     }
//! }
//!
//! impl<'a> dot::GraphWalk<'a> for Edges {
//!     type Node = Nd;
//!     type Edge = Ed;
//!     type Subgraph = ();
//!
//!     fn nodes(&self) -> dot::Nodes<'a, Nd> {
//!         // (assumes that |N| \approxeq |E|)
//!         let &Edges(ref v) = self;
//!         let mut nodes = Vec::with_capacity(v.len());
//!         for &(s, t) in v {
//!             nodes.push(s);
//!             nodes.push(t);
//!         }
//!         nodes.sort();
//!         nodes.dedup();
//!         nodes.into()
//!     }
//!
//!     fn edges(&'a self) -> dot::Edges<'a, Ed> {
//!         let &Edges(ref edges) = self;
//!         (&edges[..]).into()
//!     }
//!
//!     fn source(&self, e: &Ed) -> Nd {
//!         e.0
//!     }
//!
//!     fn target(&self, e: &Ed) -> Nd {
//!         e.1
//!     }
//! }
//!
//! # pub fn main() { render_to(&mut Vec::new()) }
//! ```
//!
//! ```no_run
//! # pub fn render_to<W:std::io::Write>(output: &mut W) { unimplemented!() }
//! pub fn main() {
//!     use std::fs::File;
//!     let mut f = File::create("example1.dot").unwrap();
//!     render_to(&mut f)
//! }
//! ```
//!
//! Output from first example (in `example1.dot`):
//!
//! ```dot
//! digraph example1 {
//!     N0[label="N0"];
//!     N1[label="N1"];
//!     N2[label="N2"];
//!     N3[label="N3"];
//!     N4[label="N4"];
//!     N0 -> N1[label=""];
//!     N0 -> N2[label=""];
//!     N1 -> N3[label=""];
//!     N2 -> N3[label=""];
//!     N3 -> N4[label=""];
//!     N4 -> N4[label=""];
//! }
//! ```
//!
//! The second example illustrates using `node_label` and `edge_label` to
//! add labels to the nodes and edges in the rendered graph. The graph
//! here carries both `nodes` (the label text to use for rendering a
//! particular node), and `edges` (again a list of `(source,target)`
//! indices).
//!
//! This example also illustrates how to use a type (in this case the edge
//! type) that shares substructure with the graph: the edge type here is a
//! direct reference to the `(source,target)` pair stored in the graph's
//! internal vector (rather than passing around a copy of the pair
//! itself). Note that this implies that `fn edges(&'a self)` must
//! construct a fresh `Vec<&'a (usize,usize)>` from the `Vec<(usize,usize)>`
//! edges stored in `self`.
//!
//! Since both the set of nodes and the set of edges are always
//! constructed from scratch via iterators, we use the `collect()` method
//! from the `Iterator` trait to collect the nodes and edges into freshly
//! constructed growable `Vec` values (rather than using `Cow` as in the
//! first example above).
//!
//! The output from this example renders four nodes that make up the
//! Hasse-diagram for the subsets of the set `{x, y}`. Each edge is
//! labeled with the &sube; character (specified using the HTML character
//! entity `&sube`).
//!
//! ```rust
//! use std::io::Write;
//!
//! use dotwalk as dot;
//!
//! type Nd = usize;
//! type Ed<'a> = &'a (usize, usize);
//! struct Graph {
//!     nodes: Vec<&'static str>,
//!     edges: Vec<(usize, usize)>,
//! }
//!
//! pub fn render_to<W: Write>(output: &mut W) {
//!     let nodes = vec!["{x,y}", "{x}", "{y}", "{}"];
//!     let edges = vec![(0, 1), (0, 2), (1, 3), (2, 3)];
//!     let graph = Graph {
//!         nodes: nodes,
//!         edges: edges,
//!     };
//!
//!     dot::render(&graph, output).unwrap()
//! }
//!
//! impl<'a> dot::Labeller<'a> for Graph {
//!     type Node = Nd;
//!     type Edge = Ed<'a>;
//!     type Subgraph = ();
//!
//!     fn graph_id(&'a self) -> dot::Id<'a> {
//!         dot::Id::new("example2").unwrap()
//!     }
//!     fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
//!         dot::Id::new(format!("N{}", n)).unwrap()
//!     }
//!     fn node_label(&self, n: &Nd) -> dot::Text<'_> {
//!         dot::Text::label(self.nodes[*n])
//!     }
//!     fn edge_label(&self, _: &Ed<'_>) -> dot::Text<'_> {
//!         dot::Text::label("&sube;")
//!     }
//! }
//!
//! impl<'a> dot::GraphWalk<'a> for Graph {
//!     type Node = Nd;
//!     type Edge = Ed<'a>;
//!     type Subgraph = ();
//!
//!     fn nodes(&self) -> dot::Nodes<'a, Nd> {
//!         (0..self.nodes.len()).collect()
//!     }
//!     fn edges(&'a self) -> dot::Edges<'a, Ed<'a>> {
//!         self.edges.iter().collect()
//!     }
//!     fn source(&self, e: &Ed<'_>) -> Nd {
//!         e.0
//!     }
//!     fn target(&self, e: &Ed<'_>) -> Nd {
//!         e.1
//!     }
//! }
//!
//! # pub fn main() { render_to(&mut Vec::new()) }
//! ```
//!
//! ```no_run
//! # pub fn render_to<W:std::io::Write>(output: &mut W) { unimplemented!() }
//! pub fn main() {
//!     use std::fs::File;
//!     let mut f = File::create("example2.dot").unwrap();
//!     render_to(&mut f)
//! }
//! ```
//!
//! The third example is similar to the second, except now each node and
//! edge now carries a reference to the string label for each node as well
//! as that node's index. (This is another illustration of how to share
//! structure with the graph itself, and why one might want to do so.)
//!
//! The output from this example is the same as the second example: the
//! Hasse-diagram for the subsets of the set `{x, y}`.
//!
//! ```rust
//! use std::io::Write;
//!
//! use dotwalk as dot;
//!
//! type Nd<'a> = (usize, &'a str);
//! type Ed<'a> = (Nd<'a>, Nd<'a>);
//! struct Graph {
//!     nodes: Vec<&'static str>,
//!     edges: Vec<(usize, usize)>,
//! }
//!
//! pub fn render_to<W: Write>(output: &mut W) {
//!     let nodes = vec!["{x,y}", "{x}", "{y}", "{}"];
//!     let edges = vec![(0, 1), (0, 2), (1, 3), (2, 3)];
//!     let graph = Graph {
//!         nodes: nodes,
//!         edges: edges,
//!     };
//!
//!     dot::render(&graph, output).unwrap()
//! }
//!
//! impl<'a> dot::Labeller<'a> for Graph {
//!     type Node = Nd<'a>;
//!     type Edge = Ed<'a>;
//!     type Subgraph = ();
//!
//!     fn graph_id(&'a self) -> dot::Id<'a> {
//!         dot::Id::new("example3").unwrap()
//!     }
//!     fn node_id(&'a self, n: &Nd<'a>) -> dot::Id<'a> {
//!         dot::Id::new(format!("N{}", n.0)).unwrap()
//!     }
//!     fn node_label(&self, n: &Nd<'_>) -> dot::Text<'_> {
//!         let &(i, _) = n;
//!         dot::Text::label(self.nodes[i])
//!     }
//!     fn edge_label(&self, _: &Ed<'_>) -> dot::Text<'_> {
//!         dot::Text::label("&sube;")
//!     }
//! }
//!
//! impl<'a> dot::GraphWalk<'a> for Graph {
//!     type Node = Nd<'a>;
//!     type Edge = Ed<'a>;
//!     type Subgraph = ();
//!
//!     fn nodes(&'a self) -> dot::Nodes<'a, Nd<'a>> {
//!         self.nodes.iter().map(|s| &s[..]).enumerate().collect()
//!     }
//!     fn edges(&'a self) -> dot::Edges<'a, Ed<'a>> {
//!         self.edges
//!             .iter()
//!             .map(|&(i, j)| ((i, &self.nodes[i][..]), (j, &self.nodes[j][..])))
//!             .collect()
//!     }
//!     fn source(&self, e: &Ed<'a>) -> Nd<'a> {
//!         e.0
//!     }
//!     fn target(&self, e: &Ed<'a>) -> Nd<'a> {
//!         e.1
//!     }
//! }
//!
//! # pub fn main() { render_to(&mut Vec::new()) }
//! ```
//!
//! ```no_run
//! # pub fn render_to<W:std::io::Write>(output: &mut W) { unimplemented!() }
//! pub fn main() {
//!     use std::fs::File;
//!     let mut f = File::create("example3.dot").unwrap();
//!     render_to(&mut f)
//! }
//! ```
//!
//! For this fourth example, we take the first one and add subgraphs:
//!
//! ```rust
//! use std::borrow::Cow;
//! use std::io::Write;
//!
//! use dotwalk as dot;
//!
//! type Nd = isize;
//! type Ed = (isize, isize);
//! type Su = usize;
//! struct Edges(Vec<Ed>);
//!
//! pub fn render_to<W: Write>(output: &mut W) {
//!     let edges = Edges(vec![(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 4)]);
//!     dot::render(&edges, output).unwrap()
//! }
//!
//! impl<'a> dot::Labeller<'a> for Edges {
//!     type Node = Nd;
//!     type Edge = Ed;
//!     type Subgraph = Su;
//!
//! #   fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example4").unwrap() }
//! #
//! #   fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
//! #       dot::Id::new(format!("N{}", *n)).unwrap()
//! #   }
//!     // ...
//!
//!     fn subgraph_id(&'a self, s: &Su) -> Option<dot::Id<'a>> {
//!         dot::Id::new(format!("cluster_{}", s)).ok()
//!     }
//! }
//!
//! impl<'a> dot::GraphWalk<'a> for Edges {
//!     type Node = Nd;
//!     type Edge = Ed;
//!     type Subgraph = Su;
//!
//! #   fn nodes(&self) -> dot::Nodes<'a,Nd> {
//! #       // (assumes that |N| \approxeq |E|)
//! #       let &Edges(ref v) = self;
//! #       let mut nodes = Vec::with_capacity(v.len());
//! #       for &(s,t) in v {
//! #           nodes.push(s); nodes.push(t);
//! #       }
//! #       nodes.sort();
//! #       nodes.dedup();
//! #       Cow::Owned(nodes)
//! #   }
//! #
//! #   fn edges(&'a self) -> dot::Edges<'a,Ed> {
//! #       let &Edges(ref edges) = self;
//! #       Cow::Borrowed(&edges[..])
//! #   }
//! #
//! #   fn source(&self, e: &Ed) -> Nd { e.0 }
//! #
//! #   fn target(&self, e: &Ed) -> Nd { e.1 }
//!     // ...
//!
//!     fn subgraphs(&'a self) -> dot::Subgraphs<'a, Su> {
//!         Cow::Borrowed(&[0, 1])
//!     }
//!
//!     fn subgraph_nodes(&'a self, s: &Su) -> dot::Nodes<'a, Nd> {
//!         let subgraph = if *s == 0 { vec![0, 1, 2] } else { vec![3, 4] };
//!
//!         Cow::Owned(subgraph)
//!     }
//! }
//! # pub fn main() { render_to(&mut Vec::new()) }
//! ```
//!
//! ```no_run
//! # pub fn render_to<W:std::io::Write>(output: &mut W) { unimplemented!() }
//! pub fn main() {
//!     use std::fs::File;
//!     let mut f = File::create("example4.dot").unwrap();
//!     render_to(&mut f)
//! }
//! ```
//!
//! The corresponding output:
//!
//! ```dot
//! digraph example4 {
//!     subgraph cluster_0 {
//!         label="";
//!         N0;
//!         N1;
//!         N2;
//!     }
//!
//!     subgraph cluster_1 {
//!         label="";
//!         N3;
//!         N4;
//!     }
//!
//!     N0[label="{x,y}"];
//!     N1[label="{x}"];
//!     N2[label="{y}"];
//!     N3[label="{}"];
//!     N0 -> N1[label="&sube;"];
//!     N0 -> N2[label="&sube;"];
//!     N1 -> N3[label="&sube;"];
//!     N2 -> N3[label="&sube;"];
//! }
//! ```
//!
//! # References
//!
//! * [Graphviz](https://www.graphviz.org/)
//!
//! * [DOT language](https://www.graphviz.org/doc/info/lang.html)

// tidy-alphabetical-start
#![doc(test(attr(allow(unused_variables), deny(warnings))))]
#![warn(unreachable_pub)]
// tidy-alphabetical-end

pub mod render;
pub mod types;

use std::borrow::Cow;
use std::collections::HashMap;

pub use render::{render, render_opts};
pub use types::*;

/// Each instance of a type that implements `Label<C>` maps to a
/// unique identifier with respect to `C`, which is used to identify
/// it in the generated .dot file. They can also provide more
/// elaborate (and non-unique) label text that is used in the graphviz
/// rendered output.

/// The graph instance is responsible for providing the DOT compatible
/// identifiers for the nodes and (optionally) rendered labels for the nodes and
/// edges, as well as an identifier for the graph itself.
pub trait Labeller<'a> {
    type Node;
    type Edge;
    type Subgraph;

    /// Must return a DOT compatible identifier naming the graph.
    fn graph_id(&'a self) -> Id<'a>;

    /// A list of attributes to apply to the graph
    fn graph_attrs(&'a self) -> HashMap<&str, &str> {
        HashMap::default()
    }

    /// Maps `n` to a unique identifier with respect to `self`. The
    /// implementor is responsible for ensuring that the returned name
    /// is a valid DOT identifier.
    fn node_id(&'a self, n: &Self::Node) -> Id<'a>;

    /// Maps `n` to one of the [graphviz `shape` names][1]. If `None`
    /// is returned, no `shape` attribute is specified.
    ///
    /// [1]: https://www.graphviz.org/doc/info/shapes.html
    fn node_shape(&'a self, _node: &Self::Node) -> Option<Text<'a>> {
        None
    }

    /// Maps `n` to a label that will be used in the rendered output.
    /// The label need not be unique, and may be the empty string; the
    /// default is just the output from `node_id`.
    fn node_label(&'a self, n: &Self::Node) -> Text<'a> {
        Text::Label(self.node_id(n).name)
    }

    /// Maps `e` to a label that will be used in the rendered output.
    /// The label need not be unique, and may be the empty string; the
    /// default is in fact the empty string.
    fn edge_label(&'a self, _e: &Self::Edge) -> Text<'a> {
        Text::Label("".into())
    }

    /// Maps `n` to a style that will be used in the rendered output.
    fn node_style(&'a self, _n: &Self::Node) -> Style {
        Style::None
    }

    /// Return an explicit rank dir to use for directed graphs.
    ///
    /// Return 'None' to use the default (generally "TB" for directed graphs).
    fn rank_dir(&'a self) -> Option<RankDir> {
        None
    }

    /// Maps `n` to one of the [graphviz `color` names][1]. If `None`
    /// is returned, no `color` attribute is specified.
    ///
    /// [1]: https://graphviz.gitlab.io/_pages/doc/info/colors.html
    fn node_color(&'a self, _node: &Self::Node) -> Option<Text<'a>> {
        None
    }

    /// Maps `n` to a set of arbritrary node attributes.
    fn node_attrs(&'a self, _n: &Self::Node) -> HashMap<&str, &str> {
        HashMap::default()
    }

    /// Maps `e` to arrow style that will be used on the end of an edge.
    /// Defaults to generic arrow style.
    fn edge_end_arrow(&'a self, _e: &Self::Edge) -> Arrow {
        Arrow::default()
    }

    /// Maps `e` to arrow style that will be used on the end of an edge.
    /// Defaults to generic arrow style.
    fn edge_start_arrow(&'a self, _e: &Self::Edge) -> Arrow {
        Arrow::default()
    }

    /// Maps `e` to a style that will be used in the rendered output.
    fn edge_style(&'a self, _e: &Self::Edge) -> Style {
        Style::None
    }

    /// Maps `e` to one of the [graphviz `color` names][1]. If `None`
    /// is returned, no `color` attribute is specified.
    ///
    /// [1]: https://graphviz.gitlab.io/_pages/doc/info/colors.html
    fn edge_color(&'a self, _e: &Self::Edge) -> Option<Text<'a>> {
        None
    }

    /// Maps `e` to a set of arbritrary edge attributes.
    fn edge_attrs(&'a self, _e: &Self::Edge) -> HashMap<&str, &str> {
        HashMap::default()
    }

    /// Maps `e` to the compass point that the edge will start from.
    /// Defaults to the default point
    fn edge_start_point(&'a self, _e: &Self::Edge) -> Option<CompassPoint> {
        None
    }

    /// Maps `e` to the compass point that the edge will end at.
    /// Defaults to the default point
    fn edge_end_point(&'a self, _e: &Self::Edge) -> Option<CompassPoint> {
        None
    }

    /// Maps `e` to the port that the edge will start from.
    fn edge_start_port(&'a self, _: &Self::Edge) -> Option<Id<'a>> {
        None
    }

    /// Maps `e` to the port that the edge will end at.
    fn edge_end_port(&'a self, _: &Self::Edge) -> Option<Id<'a>> {
        None
    }

    /// The kind of graph, defaults to `Kind::Digraph`.
    fn kind(&self) -> GraphKind {
        GraphKind::Directed
    }

    /// Maps `s` to a unique subgraph identifier.
    /// Prefix this identifier by `cluster_` to draw this subgraph in its own distinct retangle.
    fn subgraph_id(&'a self, _s: &Self::Subgraph) -> Option<Id<'a>> {
        None
    }

    /// Maps `s` to the corresponding subgraph label.
    fn subgraph_label(&'a self, _s: &Self::Subgraph) -> Text<'a> {
        Text::Label("".into())
    }

    /// Maps `s` to the corresponding subgraph style (default to `Style::None`).
    fn subgraph_style(&'a self, _s: &Self::Subgraph) -> Style {
        Style::None
    }

    /// Maps `s` to the corresponding subgraph shape.
    /// If `None` is returned (default), no `shape` attribute is specified.
    fn subgraph_shape(&'a self, _s: &Self::Subgraph) -> Option<Text<'a>> {
        None
    }

    /// Maps `s` to the corresponding subgraph color (default to `Style::None`).
    /// If `None` is returned (default), no `color` attribute is specified.
    fn subgraph_color(&'a self, _s: &Self::Subgraph) -> Option<Text<'a>> {
        None
    }

    /// Maps `s` to a set of arbritrary node attributes.
    fn subgraph_attrs(&'a self, _n: &Self::Subgraph) -> HashMap<&str, &str> {
        HashMap::default()
    }
}

/// Escape tags in such a way that it is suitable for inclusion in a
/// Graphviz HTML label.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('\"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', "<br align=\"left\"/>")
}

pub type Nodes<'a, N> = Cow<'a, [N]>;
pub type Edges<'a, E> = Cow<'a, [E]>;
pub type Subgraphs<'a, S> = Cow<'a, [S]>;

// (The type parameters in GraphWalk should be associated items,
// when/if Rust supports such.)

/// GraphWalk is an abstraction over a graph = (nodes,edges)
/// made up of node handles `N` and edge handles `E`, where each `E`
/// can be mapped to its source and target nodes.
///
/// The lifetime parameter `'a` is exposed in this trait (rather than
/// introduced as a generic parameter on each method declaration) so
/// that a client impl can choose `N` and `E` that have substructure
/// that is bound by the self lifetime `'a`.
///
/// The `nodes` and `edges` method each return instantiations of
/// `Cow<[T]>` to leave implementors the freedom to create
/// entirely new vectors or to pass back slices into internally owned
/// vectors.
pub trait GraphWalk<'a> {
    type Node: Clone;
    type Edge: Clone;
    type Subgraph: Clone;

    /// Returns all the nodes in this graph.
    fn nodes(&'a self) -> Nodes<'a, Self::Node>;
    /// Returns all of the edges in this graph.
    fn edges(&'a self) -> Edges<'a, Self::Edge>;
    /// The source node for `edge`.
    fn source(&'a self, edge: &Self::Edge) -> Self::Node;
    /// The target node for `edge`.
    fn target(&'a self, edge: &Self::Edge) -> Self::Node;

    /// Returns all the subgraphs in this graph.
    fn subgraphs(&'a self) -> Subgraphs<'a, Self::Subgraph> {
        std::borrow::Cow::Borrowed(&[])
    }

    /// Returns all the subgraphs in this graph.
    fn subgraph_nodes(&'a self, _s: &Self::Subgraph) -> Nodes<'a, Self::Node> {
        std::borrow::Cow::Borrowed(&[])
    }
}

#[cfg(test)]
mod tests;
