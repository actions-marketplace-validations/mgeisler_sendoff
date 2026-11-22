# Sendoff

A GitHub-native release tool, Rust-first.

> **Note**: `0.0.1` is an early bootstrap release under active
> development. The initial release provides working changelog
> extraction and updating commands while the full `prepare`,
> `propose`, and `publish` release workflow is being built.

## Overview

Sendoff automates software releases around pull requests:

- PR-native draft changelogs populated from merged pull requests.
- On-demand, maintainer-owned `release-*` branches.
- First-class Rust and `crates.io` support.

## CLI Usage

Extract the release notes for a version from `CHANGELOG.md`:

```sh
sendoff changelog extract 0.0.1
sendoff changelog extract 0.0.1 --file README.md --heading-level "###"
```

Insert or update a version section in `CHANGELOG.md`:

```sh
sendoff changelog update 0.1.0 --date 2026-11-22 --body "- Initial release."
```

## Planned GitHub Action Usage

Once the `prepare`, `propose`, and `publish` phases are wired up,
repositories will invoke Sendoff as a single step:

```yaml
- uses: mgeisler/sendoff@v1
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
