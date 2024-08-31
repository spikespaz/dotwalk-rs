use std::io;
use std::io::Write;

use crate::{Edges, GraphKind, GraphWalk, Labeller, Nodes, Style, Subgraphs};

/// Renders graph `g` into the writer `w` in DOT syntax.
/// (Simple wrapper around `render_opts` that passes a default set of options.)
pub fn render<'a, N, E, S, G, W>(g: &'a G, w: &mut W) -> io::Result<()>
where
    N: Clone + 'a,
    E: Clone + 'a,
    S: Clone + 'a,
    G: Labeller<'a, Node = N, Edge = E, Subgraph = S>
        + GraphWalk<'a, Node = N, Edge = E, Subgraph = S>,
    W: Write,
{
    render_opts(g, w, &[])
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RenderOption {
    NoEdgeLabels,
    NoNodeLabels,
    NoEdgeStyles,
    NoEdgeColors,
    NoNodeStyles,
    NoNodeColors,

    Fontname(String),
    DarkTheme,
    NoArrows,
}

/// Renders graph `g` into the writer `w` in DOT syntax.
/// (Main entry point for the library.)
pub fn render_opts<'a, N, E, S, G, W>(
    g: &'a G,
    w: &mut W,
    options: &[RenderOption],
) -> io::Result<()>
where
    N: Clone + 'a,
    E: Clone + 'a,
    S: Clone + 'a,
    G: Labeller<'a, Node = N, Edge = E, Subgraph = S>
        + GraphWalk<'a, Node = N, Edge = E, Subgraph = S>,
    W: Write,
{
    writeln!(w, "{} {} {{", g.kind().as_keyword(), *g.graph_id())?;

    if g.kind() == GraphKind::Directed {
        if let Some(rankdir) = g.rank_dir() {
            writeln!(w, "    rankdir=\"{}\";", rankdir.as_static_str())?;
        }
    }

    for (name, value) in g.graph_attrs().iter() {
        writeln!(w, "    {name}={value}")?;
    }

    // Global graph properties
    let mut graph_attrs = Vec::new();
    let mut content_attrs = Vec::new();
    let font;
    let fontname = options.iter().find_map(|option| {
        if let RenderOption::Fontname(fontname) = option {
            Some(fontname)
        } else {
            None
        }
    });
    if let Some(fontname) = fontname {
        font = format!(r#"fontname="{fontname}""#);
        graph_attrs.push(&font[..]);
        content_attrs.push(&font[..]);
    }
    if options.contains(&RenderOption::DarkTheme) {
        graph_attrs.push(r#"bgcolor="black""#);
        graph_attrs.push(r#"fontcolor="white""#);
        content_attrs.push(r#"color="white""#);
        content_attrs.push(r#"fontcolor="white""#);
    }
    if !(graph_attrs.is_empty() && content_attrs.is_empty()) {
        writeln!(w, r#"    graph[{}];"#, graph_attrs.join(" "))?;
        let content_attrs_str = content_attrs.join(" ");
        writeln!(w, r#"    node[{content_attrs_str}];"#)?;
        writeln!(w, r#"    edge[{content_attrs_str}];"#)?;
    }

    render_subgraphs(w, g, &g.subgraphs(), options)?;
    render_nodes(w, g, &g.nodes(), options)?;
    render_edges(w, g, &g.edges(), options)?;

    writeln!(w, "}}")
}

pub fn render_nodes<'a, N, E, S, G, W>(
    w: &mut W,
    graph: &'a G,
    nodes: &Nodes<'a, N>,
    options: &[RenderOption],
) -> io::Result<()>
where
    W: Write,
    N: Clone + 'a,
    E: Clone + 'a,
    S: Clone + 'a,
    G: Labeller<'a, Node = N, Edge = E, Subgraph = S>
        + GraphWalk<'a, Node = N, Edge = E, Subgraph = S>,
{
    let mut text = Vec::new();
    for n in nodes.iter() {
        write!(text, "    {}", *graph.node_id(n)).unwrap();

        if !options.contains(&RenderOption::NoNodeLabels) {
            let label = &graph.node_label(n).to_escaped_string();
            write!(text, "[label={label}]").unwrap();
        }

        let style = graph.node_style(n);
        if !options.contains(&RenderOption::NoNodeStyles) && style != Style::None {
            write!(text, "[style=\"{}\"]", style.as_static_str()).unwrap();
        }

        if !options.contains(&RenderOption::NoNodeColors) {
            if let Some(color) = graph.node_color(n) {
                write!(text, "[color={}]", color.to_escaped_string()).unwrap();
            }
        }

        if let Some(shape) = graph.node_shape(n) {
            write!(text, "[shape={}]", &shape.to_escaped_string()).unwrap();
        }

        for (name, value) in graph.node_attrs(n).into_iter() {
            write!(text, "[{name}={value}]").unwrap();
        }

        writeln!(text, ";").unwrap();

        w.write_all(&text)?;
        text.clear();
    }
    Ok(())
}

pub fn render_subgraphs<'a, N, E, S, G, W>(
    w: &mut W,
    graph: &'a G,
    subgraphs: &Subgraphs<'a, S>,
    options: &[RenderOption],
) -> io::Result<()>
where
    W: Write,
    N: Clone + 'a,
    E: Clone + 'a,
    S: Clone + 'a,
    G: Labeller<'a, Node = N, Edge = E, Subgraph = S>
        + GraphWalk<'a, Node = N, Edge = E, Subgraph = S>,
{
    let mut text = Vec::new();
    for s in subgraphs.iter() {
        write!(text, "subgraph").unwrap();

        if let Some(id) = graph.subgraph_id(s) {
            write!(text, " {}", *id).unwrap();
        }

        writeln!(text, " {{").unwrap();

        if !options.contains(&RenderOption::NoNodeLabels) {
            let label = &graph.subgraph_label(s).to_escaped_string();
            writeln!(text, "    label={label};").unwrap();
        }

        let style = graph.subgraph_style(s);
        let var_name = style != Style::None;
        if !options.contains(&RenderOption::NoNodeStyles) && var_name {
            writeln!(text, "    style=\"{}\";", style.as_static_str()).unwrap();
        }

        if !options.contains(&RenderOption::NoNodeColors) {
            if let Some(color) = graph.subgraph_color(s) {
                writeln!(text, "    color={};", color.to_escaped_string()).unwrap();
            }
        }

        if let Some(shape) = graph.subgraph_shape(s) {
            writeln!(text, "    shape={};", &shape.to_escaped_string()).unwrap();
        }

        for (name, value) in graph.subgraph_attrs(s).into_iter() {
            writeln!(text, "    {name}={value};").unwrap();
        }

        for n in graph.subgraph_nodes(s).iter() {
            writeln!(text, "    {};", *graph.node_id(n)).unwrap();
        }

        writeln!(text, "}}").unwrap();

        w.write_all(&text)?;
        text.clear();
    }
    Ok(())
}

pub fn render_edges<'a, N, E, S, G, W>(
    w: &mut W,
    graph: &'a G,
    edges: &Edges<'a, E>,
    options: &[RenderOption],
) -> io::Result<()>
where
    W: Write,
    N: Clone + 'a,
    E: Clone + 'a,
    S: Clone + 'a,
    G: Labeller<'a, Node = N, Edge = E, Subgraph = S>
        + GraphWalk<'a, Node = N, Edge = E, Subgraph = S>,
{
    let mut text = Vec::new();
    for e in edges.iter() {
        let start_arrow = graph.edge_start_arrow(e);
        let end_arrow = graph.edge_end_arrow(e);
        let start_port = graph
            .edge_start_port(e)
            .map(|p| format!(":{}", p.name))
            .unwrap_or_default();
        let end_port = graph
            .edge_end_port(e)
            .map(|p| format!(":{}", p.name))
            .unwrap_or_default();
        let start_point = graph
            .edge_start_point(e)
            .map(|p| p.as_static_str())
            .unwrap_or("");
        let end_point = graph
            .edge_end_point(e)
            .map(|p| p.as_static_str())
            .unwrap_or("");

        write!(w, "    ")?;

        let source_id = graph.node_id(&graph.source(e));
        let target_id = graph.node_id(&graph.target(e));

        write!(
            text,
            "{}{}{} {} {}{}{}",
            *source_id,
            start_port,
            start_point,
            graph.kind().as_edge_op(),
            *target_id,
            end_port,
            end_point,
        )
        .unwrap();

        if !options.contains(&RenderOption::NoEdgeLabels) {
            let label = graph.edge_label(e).to_escaped_string();
            write!(text, "[label={label}]").unwrap();
        }

        let style = graph.edge_style(e);
        if !options.contains(&RenderOption::NoEdgeStyles) && style != Style::None {
            write!(text, "[style=\"{}\"]", style.as_static_str()).unwrap();
        }

        if !options.contains(&RenderOption::NoEdgeColors) {
            if let Some(color) = graph.edge_color(e) {
                write!(text, "[color={}]", color.to_escaped_string()).unwrap();
            }
        }

        if !options.contains(&RenderOption::NoArrows)
            && (!start_arrow.is_default() || !end_arrow.is_default())
        {
            write!(text, "[").unwrap();
            if !end_arrow.is_default() {
                write!(text, "arrowhead=\"{}\"", end_arrow.to_dot_string()).unwrap();
            }
            if !start_arrow.is_default() {
                if *text.last().unwrap() != b'[' {
                    write!(text, " ").unwrap();
                }
                write!(
                    text,
                    "dir=\"both\" arrowtail=\"{}\"",
                    start_arrow.to_dot_string()
                )
                .unwrap();
            }
            write!(text, "]").unwrap();
        }

        for (name, value) in graph.edge_attrs(e).into_iter() {
            write!(text, "{name}={value}").unwrap();
        }

        writeln!(text, ";").unwrap();

        w.write_all(&text)?;
        text.clear();
    }
    Ok(())
}
