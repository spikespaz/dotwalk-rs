# dotwalk-rs

Traits for traversing through your own structure and generating [DOT]
language ([reference](1)) files for graphs to be rendered by another tool.

[DOT]: https://en.wikipedia.org/wiki/DOT_%28graph_description_language%29
[1]: https://graphviz.org/doc/info/lang.html

# Contribution History

This crate is extracted from the Rust compiler source tree.
It is based on [`compiler/rustc_graphviz`] at commit [`fa18140`].
The crate has been renamed and moved from [`src/librustc_graphviz`] and previously [`src/libgraphviz`].
All commit history for relevant files is preserved in this repository.

You can create your own filtered repository from newer Rust source by running these commands:
```sh
git clone git@github.com:rust-lang/rust.git rustc_graphviz
cd rustc_graphviz
git filter-repo \
	--path src/libgraphviz \
	--path src/librustc_graphviz \
	--path compiler/rustc_graphviz \
	--subdirectory-filter src/libgraphviz \
	--subdirectory-filter src/librustc_graphviz \
	--subdirectory-filter compiler/rustc_graphviz \
	--prune-empty always \
	--prune-degenerate always
```

[`fa18140`]: https://github.com/rust-lang/rust/tree/6cf068db566de080dfa7ed24a216ea3aed2b98ce
[`compiler/rustc_graphviz`]: https://github.com/rust-lang/rust/tree/6cf068db566de080dfa7ed24a216ea3aed2b98ce/compiler/rustc_graphviz
[`src/librustc_graphviz`]: https://github.com/rust-lang/rust/tree/db534b3ac286cf45688c3bbae6aa6e77439e52d2/src/librustc_graphviz
[`src/libgraphviz`]: https://github.com/rust-lang/rust/tree/eeaf497b2a6bc065874e3d3367b1f3023c5bb3d3/src/libgraphviz

# License

This project is dial-licensed under either MIT or Apache-2.0, at your option.
This licensing the same as the Rust Project, from which this fork is based.

See [COPYRIGHT](./COPYRIGHT) for more information.
