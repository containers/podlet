# Contributing to Podlet

We'd love to have you join the community! Below summarizes the processes that we follow.

## Reporting Issues

Before reporting an issue, check our backlog of
[open issues](https://github.com/containers/podlet/issues)
to see if someone else has already reported it. If so, feel free to add
your scenario, or additional information, to the discussion. Or simply
"subscribe" to it to be notified when it is updated.

If you find a new issue with the project we'd love to hear about it! The most
important aspect of a bug report is that it includes enough information for
us to reproduce it. So, please include as much detail as possible and try
to remove the extra stuff that doesn't really relate to the issue itself.
The easier it is for us to reproduce it, the faster it'll be fixed!

Please don't include any private/sensitive information in your issue!

## Submitting Pull Requests

No [pull request] (PR) is too small! Typos, additional comments in the code,
new test cases, bug fixes, new features, more documentation, ... it's all
welcome!

While bug fixes can first be identified via an [issue], that is not required.
It's ok to just open up a PR with the fix, but make sure you include the same
information you would have included in an issue - like how to reproduce it.

PRs for new features should include some background on what use cases the
new code is trying to address. When possible and when it makes sense, try to break up
larger PRs into smaller ones - it's easier to review smaller
code changes. But only if those smaller ones make sense as stand-alone PRs.

Commits that fix issues should include one or more footers like `Closes: #XXX` or `Fixes: #XXX` at the
end of the commit message. GitHub will automatically close the referenced issue when the PR is merged
and the [changelog] will include the issue.

### Use Conventional Commits

While not a requirement, try to use [conventional commits](https://www.conventionalcommits.org) for
your commit messages. It makes creating the [changelog] via [git-cliff](https://git-cliff.org/) easier.

### Sign Your Commits

For a PR to be merged, each commit must contain a `Signed-off-by` footer. The sign-off is a
line at the end of the explanation for the commit. Your signature certifies that you wrote the patch
or otherwise have the right to pass it on as an open-source patch.

The rules are simple: if you can certify the following (from [developercertificate.org](https://developercertificate.org/)):

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
660 York Street, Suite 102,
San Francisco, CA 94110 USA

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.

Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

Then you just add a line to every git commit message:

```
Signed-off-by: Joe Smith <joe.smith@email.com>
```

Use your real name (sorry, no pseudonyms or anonymous contributions).

If you set your `user.name` and `user.email` git configs, you can sign your
commit automatically with `git commit -s`.

## Building

Podlet is a normal Rust project, so once [Rust is installed],
the source code can be cloned and built with:

```shell
git clone git@github.com:containers/podlet.git
cd podlet
cargo build
```

Release builds are created with the `dist` profile:

```shell
cargo build --profile dist
```

## Continuous Integration

A number of jobs are automatically run for each pull request and merge.
If you are submitting code changes and would like to run the CI jobs locally,
below is a list of all the jobs with explanations and the commands that they run. 

- format:
  - Ensures consistent formatting for all Rust code.
  - `cargo fmt --check`
- clippy:
  - [Clippy](https://github.com/rust-lang/rust-clippy) is a collection of lints for Rust.
  - If [Rust is installed] via `rustup`, install Clippy with `rustup component add clippy`.
  - Lints are configured in the [`Cargo.toml`](./Cargo.toml) file.
  - It's ok to use `#[allow(...)]` to override a lint,
    but try to document the reasoning if it's not obvious.
  - `cargo clippy`
- test:
  - Unit tests are defined in the source.
  - All tests should pass.
  - `cargo test`
- build-container:
  - Ensures that the [podlet container](./Containerfile) can build for both x86 and ARM platforms.
  - First, [install podman](https://podman.io/docs/installation).
  - `podman build --platform linux/amd64 -t podlet .`
  - `podman build --platform linux/arm64/v8 -t podlet .`

## Communication

The podlet project shares communication channels with other projects in the [containers organization](https://github.com/containers#-community).

For discussions about issues, bugs, or features, feel free to create an [issue], [discussion], or [pull request] on GitHub.


[changelog]: ./CHANGELOG.md
[discussion]: https://github.com/containers/podlet/discussions
[issue]: https://github.com/containers/podlet/issues
[pull request]: https://github.com/containers/podlet/pulls
[Rust is installed]: https://www.rust-lang.org/tools/install
