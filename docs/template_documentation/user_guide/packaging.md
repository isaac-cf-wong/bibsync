# Packaging

Before publishing, check package contents:

```shell
cargo package --list
cargo package
```

Publish to crates.io from a release tag with the `publish.yml` workflow, or
manually with:

```shell
cargo publish
```
