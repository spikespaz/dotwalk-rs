use std::borrow::Cow;

/// Graph kind determines if `digraph` or `graph` is used as keyword
/// for the graph.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GraphKind {
    Directed,
    Undirected,
}

impl GraphKind {
    /// The keyword to use to introduce the graph.
    /// Determines which edge syntax must be used, and default style.
    pub const fn as_keyword(&self) -> &'static str {
        match *self {
            GraphKind::Directed => "digraph",
            GraphKind::Undirected => "graph",
        }
    }

    /// The edgeop syntax to use for this graph kind.
    pub const fn as_edge_op(&self) -> &'static str {
        match *self {
            GraphKind::Directed => "->",
            GraphKind::Undirected => "--",
        }
    }
}

/// The direction to draw directed graphs (one rank at a time)
/// See https://graphviz.org/docs/attr-types/rankdir/ for descriptions
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RankDir {
    TopBottom,
    LeftRight,
    BottomTop,
    RightLeft,
}

impl RankDir {
    pub const fn as_static_str(self) -> &'static str {
        match self {
            RankDir::TopBottom => "TB",
            RankDir::LeftRight => "LR",
            RankDir::BottomTop => "BT",
            RankDir::RightLeft => "RL",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IdError {
    EmptyName,
    InvalidStartChar(char),
    InvalidChar(char),
}

impl std::error::Error for IdError {}

impl std::fmt::Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdError::EmptyName => write!(f, "Id cannot be empty"),
            IdError::InvalidStartChar(c) => write!(f, "Id cannot begin with '{c}'"),
            IdError::InvalidChar(c) => write!(f, "Id cannot contain '{c}'"),
        }
    }
}

// There is a tension in the design of the labelling API.
//
// For example, I considered making a `Labeller<T>` trait that
// provides labels for `T`, and then making the graph type `G`
// implement `Labeller<Node>` and `Labeller<Edge>`. However, this is
// not possible without functional dependencies. (One could work
// around that, but I did not explore that avenue heavily.)
//
// Another approach that I actually used for a while was to make a
// `Label<Context>` trait that is implemented by the client-specific
// Node and Edge types (as well as an implementation on Graph itself
// for the overall name for the graph). The main disadvantage of this
// second approach (compared to having the `G` type parameter
// implement a Labelling service) that I have encountered is that it
// makes it impossible to use types outside of the current crate
// directly as Nodes/Edges; you need to wrap them in newtype'd
// structs. See e.g., the `No` and `Ed` structs in the examples. (In
// practice clients using a graph in some other crate would need to
// provide some sort of adapter shim over the graph anyway to
// interface with this library).
//
// Another approach would be to make a single `Labeller<N,E>` trait
// that provides three methods (graph_label, node_label, edge_label),
// and then make `G` implement `Labeller<N,E>`. At first this did not
// appeal to me, since I had thought I would need separate methods on
// each data variant for dot-internal identifiers versus user-visible
// labels. However, the identifier/label distinction only arises for
// nodes; graphs themselves only have identifiers, and edges only have
// labels.
//
// So in the end I decided to use the third approach described above.

/// `Id` is a Graphviz `ID`.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Id<'a> {
    pub(crate) name: Cow<'a, str>,
}

impl<'a> std::ops::Deref for Id<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl<'a> Id<'a> {
    /// Creates an `Id` named `name`.
    ///
    /// The caller must ensure that the input conforms to an
    /// identifier format: it must be a non-empty string made up of
    /// alphanumeric or underscore characters, not beginning with a
    /// digit (i.e., the regular expression `[a-zA-Z_][a-zA-Z_0-9]*`).
    ///
    /// (Note: this format is a strict subset of the `ID` format
    /// defined by the DOT language. This function may change in the
    /// future to accept a broader subset, or the entirety, of DOT's
    /// `ID` format.)
    ///
    /// Passing an invalid string (containing spaces, brackets,
    /// quotes, ...) will return an empty `Err` value.
    pub fn new(name: impl Into<Cow<'a, str>>) -> Result<Id<'a>, IdError> {
        let name = name.into();
        match name.chars().next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            Some(c) => return Err(IdError::InvalidStartChar(c)),
            None => return Err(IdError::EmptyName),
        }
        if let Some(c) = name
            .chars()
            .find(|c| !(c.is_ascii_alphanumeric() || *c == '_'))
        {
            return Err(IdError::InvalidChar(c));
        }

        Ok(Id { name })
    }
}

/// The text for a graphviz label on a node or edge.
#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Text<'a> {
    /// This kind of label preserves the text directly as is.
    ///
    /// Occurrences of backslashes (`\`) are escaped, and thus appear
    /// as backslashes in the rendered label.
    Label(Cow<'a, str>),

    /// This kind of label uses the graphviz label escString type:
    /// <https://www.graphviz.org/docs/attr-types/escString>
    ///
    /// Occurrences of backslashes (`\`) are not escaped; instead they
    /// are interpreted as initiating an escString escape sequence.
    ///
    /// Escape sequences of particular interest: in addition to `\n`
    /// to break a line (centering the line preceding the `\n`), there
    /// are also the escape sequences `\l` which left-justifies the
    /// preceding line and `\r` which right-justifies it.
    Esc(Cow<'a, str>),

    /// This uses a graphviz [HTML string label][html]. The string is
    /// printed exactly as given, but between `<` and `>`. **No
    /// escaping is performed.**
    ///
    /// [html]: https://www.graphviz.org/doc/info/shapes.html#html
    Html(Cow<'a, str>),
}

impl<'a> Text<'a> {
    pub fn label(s: impl Into<Cow<'a, str>>) -> Self {
        Self::Label(s.into())
    }

    pub fn esc(s: impl Into<Cow<'a, str>>) -> Self {
        Self::Esc(s.into())
    }

    pub fn html(s: impl Into<Cow<'a, str>>) -> Self {
        Self::Html(s.into())
    }

    /// Renders text as string suitable for a label in a .dot file.
    /// This includes quotes or suitable delimiters.
    pub fn to_escaped_string(&self) -> String {
        match self {
            Self::Label(s) => format!("\"{}\"", s.escape_default()),
            Self::Esc(s) => format!("\"{}\"", Text::escape_str(s)),
            Self::Html(s) => format!("<{s}>"),
        }
    }

    pub fn into_inner(self) -> Cow<'a, str> {
        match self {
            Text::Label(s) | Text::Esc(s) | Text::Html(s) => s,
        }
    }

    pub(crate) fn escape_char(c: char, mut f: impl FnMut(char)) {
        match c {
            // not escaping \\, since Graphviz escString needs to
            // interpret backslashes; see EscStr above.
            '\\' => f(c),
            _ => {
                for c in c.escape_default() {
                    f(c)
                }
            }
        }
    }
    pub(crate) fn escape_str(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            Text::escape_char(c, |c| out.push(c));
        }
        out
    }
}

/// The style for a node or edge.
/// See <https://www.graphviz.org/docs/attr-types/style/> for descriptions.
/// Note that some of these are not valid for edges.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Style {
    None,
    Solid,
    Dashed,
    Dotted,
    Bold,
    Rounded,
    Diagonals,
    Filled,
    Striped,
    Wedged,
}

impl Style {
    pub const fn as_static_str(self) -> &'static str {
        match self {
            Style::None => "",
            Style::Solid => "solid",
            Style::Dashed => "dashed",
            Style::Dotted => "dotted",
            Style::Bold => "bold",
            Style::Rounded => "rounded",
            Style::Diagonals => "diagonals",
            Style::Filled => "filled",
            Style::Striped => "striped",
            Style::Wedged => "wedged",
        }
    }
}

/// Arrow modifier that determines if the shape is empty or filled.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ShapeFill {
    Open,
    Filled,
}

impl ShapeFill {
    pub const fn as_static_str(self) -> &'static str {
        match self {
            ShapeFill::Open => "o",
            ShapeFill::Filled => "",
        }
    }
}

/// Arrow modifier that determines if the shape is clipped.
/// For example `Side::Left` means only left side is visible.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Both,
}

impl Side {
    pub const fn as_static_str(self) -> &'static str {
        match self {
            Side::Left => "l",
            Side::Right => "r",
            Side::Both => "",
        }
    }
}

/// This enumeration represents all possible arrow edge
/// as defined in [grapviz documentation](http://www.graphviz.org/content/arrow-shapes).
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ArrowVertex {
    /// No arrow will be displayed
    None,
    /// Arrow that ends in a triangle. Basically a normal arrow.
    /// NOTE: there is error in official documentation, this supports both fill and side clipping
    Normal(ShapeFill, Side),
    /// Arrow ending in a small square box
    Box(ShapeFill, Side),
    /// Arrow ending in a three branching lines also called crow's foot
    Crow(Side),
    /// Arrow ending in a curve
    Curve(Side),
    /// Arrow ending in an inverted curve
    ICurve(ShapeFill, Side),
    /// Arrow ending in an diamond shaped rectangular shape.
    Diamond(ShapeFill, Side),
    /// Arrow ending in a circle.
    Dot(ShapeFill),
    /// Arrow ending in an inverted triangle.
    Inv(ShapeFill, Side),
    /// Arrow ending with a T shaped arrow.
    Tee(Side),
    /// Arrow ending with a V shaped arrow.
    Vee(Side),
}

impl ArrowVertex {
    /// Constructor which returns no arrow.
    pub fn none() -> ArrowVertex {
        ArrowVertex::None
    }

    /// Constructor which returns normal arrow.
    pub fn normal() -> ArrowVertex {
        ArrowVertex::Normal(ShapeFill::Filled, Side::Both)
    }

    /// Constructor which returns a regular box arrow.
    pub fn boxed() -> ArrowVertex {
        ArrowVertex::Box(ShapeFill::Filled, Side::Both)
    }

    /// Constructor which returns a regular crow arrow.
    pub fn crow() -> ArrowVertex {
        ArrowVertex::Crow(Side::Both)
    }

    /// Constructor which returns a regular curve arrow.
    pub fn curve() -> ArrowVertex {
        ArrowVertex::Curve(Side::Both)
    }

    /// Constructor which returns an inverted curve arrow.
    pub fn icurve() -> ArrowVertex {
        ArrowVertex::ICurve(ShapeFill::Filled, Side::Both)
    }

    /// Constructor which returns a diamond arrow.
    pub fn diamond() -> ArrowVertex {
        ArrowVertex::Diamond(ShapeFill::Filled, Side::Both)
    }

    /// Constructor which returns a circle shaped arrow.
    pub fn dot() -> ArrowVertex {
        ArrowVertex::Dot(ShapeFill::Filled)
    }

    /// Constructor which returns an inverted triangle arrow.
    pub fn inv() -> ArrowVertex {
        ArrowVertex::Inv(ShapeFill::Filled, Side::Both)
    }

    /// Constructor which returns a T shaped arrow.
    pub fn tee() -> ArrowVertex {
        ArrowVertex::Tee(Side::Both)
    }

    /// Constructor which returns a V shaped arrow.
    pub fn vee() -> ArrowVertex {
        ArrowVertex::Vee(Side::Both)
    }

    /// Function which renders given ArrowShape into a String for displaying.
    pub fn to_dot_string(&self) -> String {
        let mut res = String::new();

        match *self {
            Self::Box(fill, side)
            | Self::ICurve(fill, side)
            | Self::Diamond(fill, side)
            | Self::Inv(fill, side)
            | Self::Normal(fill, side) => {
                res.push_str(fill.as_static_str());
                match side {
                    Side::Left | Side::Right => res.push_str(side.as_static_str()),
                    Side::Both => {}
                };
            }
            Self::Dot(fill) => res.push_str(fill.as_static_str()),
            Self::Crow(side) | Self::Curve(side) | Self::Tee(side) | Self::Vee(side) => {
                match side {
                    Side::Left | Side::Right => res.push_str(side.as_static_str()),
                    Side::Both => {}
                }
            }
            Self::None => {}
        };
        match *self {
            Self::None => res.push_str("none"),
            Self::Normal(_, _) => res.push_str("normal"),
            Self::Box(_, _) => res.push_str("box"),
            Self::Crow(_) => res.push_str("crow"),
            Self::Curve(_) => res.push_str("curve"),
            Self::ICurve(_, _) => res.push_str("icurve"),
            Self::Diamond(_, _) => res.push_str("diamond"),
            Self::Dot(_) => res.push_str("dot"),
            Self::Inv(_, _) => res.push_str("inv"),
            Self::Tee(_) => res.push_str("tee"),
            Self::Vee(_) => res.push_str("vee"),
        };
        res
    }
}

/// This structure holds all information that can describe an arrow connected to
/// either start or end of an edge.
///
/// <https://graphviz.org/doc/info/arrows.html>
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Arrow {
    pub arrows: Vec<ArrowVertex>,
}

impl Arrow {
    /// Return `true` if this is a default arrow.
    pub fn is_default(&self) -> bool {
        self.arrows.is_empty()
    }

    /// Arrow constructor which returns an empty arrow
    pub fn none() -> Arrow {
        Arrow {
            arrows: vec![ArrowVertex::None],
        }
    }

    /// Arrow constructor which returns a regular triangle arrow, without modifiers
    pub fn normal() -> Arrow {
        Arrow {
            arrows: vec![ArrowVertex::normal()],
        }
    }

    /// Function which converts given arrow into a renderable form.
    pub fn to_dot_string(&self) -> String {
        let mut cow = String::new();
        for arrow in &self.arrows {
            cow.push_str(&arrow.to_dot_string());
        }
        cow
    }
}

impl Default for Arrow {
    /// Arrow constructor which returns a default arrow
    fn default() -> Arrow {
        Arrow { arrows: vec![] }
    }
}

impl From<ArrowVertex> for Arrow {
    fn from(vertex: ArrowVertex) -> Self {
        Arrow {
            arrows: vec![vertex],
        }
    }
}

macro_rules! impl_arrow_from_vertex_array {
    ( $($n:literal)+ ) => {
        $(
            impl From<[ArrowVertex; $n]> for Arrow {
                fn from(shape: [ArrowVertex; $n]) -> Arrow {
                    Arrow {
                        arrows: shape.to_vec(),
                    }
                }
            }
        )+
    };
}

impl_arrow_from_vertex_array!(1 2 3 4);

/// <https://graphviz.org/docs/attr-types/portPos/>
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum CompassPoint {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    Center,
}

impl CompassPoint {
    pub const fn as_static_str(&self) -> &'static str {
        use CompassPoint as C;
        match self {
            C::North => ":n",
            C::NorthEast => ":ne",
            C::East => ":e",
            C::SouthEast => ":se",
            C::South => ":s",
            C::SouthWest => ":sw",
            C::West => ":w",
            C::NorthWest => ":nw",
            C::Center => ":c",
        }
    }
}
