# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [v0.6.0](https://github.com/ide4cj/cangjie-zed/compare/765f1bff3c5a6ea919073ec42e27c45197fa7943..v0.6.0) - 2026-10-26
#### Features
- give *.cj.macrocall the Cangjie language, for cjls's Expand macro (#13) - ([d3b9bfd](https://github.com/ide4cj/cangjie-zed/commit/d3b9bfd92a4a8ee2a332f41a406badb1b76c0908)) - FilaCo
- cjls's semantic tokens go by the names highlights.scm gives (cjls's D105) (#14) - ([29a850c](https://github.com/ide4cj/cangjie-zed/commit/29a850c232dfb6899e51148cae9e9af3aa54105d)) - FilaCo
#### Bug Fixes
- (**cjls**) update cjls to v0.6.0 (#27) - ([0524b57](https://github.com/ide4cj/cangjie-zed/commit/0524b570abf9444a3b919e99f8d51cdd6cf68b34)) - renovate[bot]
#### Documentation
- record binary source readonly API limitation (#24) - ([da45881](https://github.com/ide4cj/cangjie-zed/commit/da45881be1e46ad16db6c6bb242ea745cad5b01d)) - FilaCo
#### Tests
- separate arrange act and assert in client tests (#25) - ([0aed057](https://github.com/ide4cj/cangjie-zed/commit/0aed05721c3b095586713af3d2b70df17555887a)) - FilaCo
#### Continuous Integration
- highlights.scm made again in the PR that moves the grammar's pin (#23) - ([00cf33e](https://github.com/ide4cj/cangjie-zed/commit/00cf33eacfc8a21412423fbff7df9123b9243854)) - FilaCo
- the release on the pinned cjls's minor with a patch of its own (cjls's D132) (#18) - ([0dad852](https://github.com/ide4cj/cangjie-zed/commit/0dad8529bdd65ba8fbbb1595fb65688138e02fb5)) - FilaCo
- the server's settings made again in the PR that moves the pin, never checked against the cjls under test (#15) - ([a8e60b1](https://github.com/ide4cj/cangjie-zed/commit/a8e60b1b8ec680371fbf870390bb1982161821b6)) - FilaCo
#### Miscellaneous Chores
- (**deps**) update https://github.com/ide4cj/tree-sitter-cangjie digest to 2df278c (#26) - ([bf01227](https://github.com/ide4cj/cangjie-zed/commit/bf01227b4ed4219ee0f332f6918735f96276e6a6)) - renovate[bot]
- (**deps**) update huacnlee/zed-extension-action action to v2 (#22) - ([c6ec691](https://github.com/ide4cj/cangjie-zed/commit/c6ec691e6adfd310464f748730aebc56b0cc0f19)) - renovate[bot]
- (**deps**) update dependency node to v24 (#21) - ([cd403bf](https://github.com/ide4cj/cangjie-zed/commit/cd403bf2d97f896be5ceee00ceb5000d05c51c88)) - renovate[bot]
- (**deps**) update https://github.com/ide4cj/tree-sitter-cangjie digest to 36b9a1c (#19) - ([02abb3f](https://github.com/ide4cj/cangjie-zed/commit/02abb3f403cc612c791a8ea83bc2cfe1e048260c)) - renovate[bot]
- (**deps**) update actions/setup-python action to v7 (#20) - ([1523836](https://github.com/ide4cj/cangjie-zed/commit/1523836a71a889cde756671500c5b26b1798fe0c)) - renovate[bot]
- (**deps**) update actions/setup-node action to v7 (#12) - ([765f1bf](https://github.com/ide4cj/cangjie-zed/commit/765f1bff3c5a6ea919073ec42e27c45197fa7943)) - renovate[bot]

- - -

## [v0.2.0](https://github.com/ide4cj/cangjie-zed/compare/be67cc7a7e78dc858b942f056b12070ac380649d..v0.2.0) - 2026-10-12
#### Features
- lsp.cjls.settings reach cjls, as its initialization options and section cjls (#8) - ([6a4673e](https://github.com/ide4cj/cangjie-zed/commit/6a4673ea6f403952a4451cbf829e690da4cfb77d)) - FilaCo
#### Bug Fixes
- (**cjls**) update cjls to v0.5.0 (#11) - ([0a26b5e](https://github.com/ide4cj/cangjie-zed/commit/0a26b5e478801e4f602723eede95237a08f907bf)) - renovate[bot]
- tree-sitter-cangjie at c03359d, the head of its master (#7) - ([be67cc7](https://github.com/ide4cj/cangjie-zed/commit/be67cc7a7e78dc858b942f056b12070ac380649d)) - FilaCo
#### Miscellaneous Chores
- (**deps**) update actions/create-github-app-token action to v3 (#10) - ([d372733](https://github.com/ide4cj/cangjie-zed/commit/d372733052500a8e3c7c6851a91909f84ff103bf)) - renovate[bot]
- (**deps**) update actions/checkout action to v7 (#9) - ([95ce646](https://github.com/ide4cj/cangjie-zed/commit/95ce6465938cc690cec94dcaed512180f57f0743)) - renovate[bot]

- - -

## [v0.1.1](https://github.com/ide4cj/cangjie-zed/compare/ce09223570f03badb808dd66acdcbffd1c6881f7..v0.1.1) - 2026-10-09
#### Bug Fixes
- (**cjls**) update cjls to v0.4.0 (#6) - ([ea1e2fe](https://github.com/ide4cj/cangjie-zed/commit/ea1e2fe31aac5f44945432b5d1b90c8521016d19)) - FilaCo
- cjls's nightly is its release nightly-build (#3) - ([0670cbf](https://github.com/ide4cj/cangjie-zed/commit/0670cbfa82f1aac46ed0a1e094952ca3c5b68954)) - FilaCo
- take tree-sitter-cangjie from ide4cj, at 388ea32, the head of its master (#2) - ([ce09223](https://github.com/ide4cj/cangjie-zed/commit/ce09223570f03badb808dd66acdcbffd1c6881f7)) - FilaCo
#### Documentation
- folds from cjls, turned on by document_folding_ranges (#4) - ([0031196](https://github.com/ide4cj/cangjie-zed/commit/0031196314bdafc1895a24093b07c1629745bfdf)) - FilaCo

- - -

## [v0.1.0](https://github.com/ide4cj/cangjie-zed/compare/1c468f78ef1bd8bab762d425234dc39ddb0acb61..v0.1.0) - 2026-10-02
#### Features
- Cangjie in Zed: cjls downloaded within its pinned minor, tree-sitter highlighting (#1) - ([8038de4](https://github.com/ide4cj/cangjie-zed/commit/8038de4fac3b75c65a0399bfe2cdf335177d463a)) - FilaCo
#### Miscellaneous Chores
- Cangjie for Zed, the repository - ([1c468f7](https://github.com/ide4cj/cangjie-zed/commit/1c468f78ef1bd8bab762d425234dc39ddb0acb61)) - FilaCo

- - -

