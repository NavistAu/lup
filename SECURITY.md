# Security policy

## Supported versions

The most recent minor release gets security fixes. This project has no
long-term support branches.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |

## Report a vulnerability

Report vulnerabilities privately through
[GitHub Security Advisories](https://github.com/NavistAu/lup/security/advisories/new).
Do not open a public issue for a vulnerability.

Include the version, your platform, the exact invocation, and a directory layout
that reproduces the problem. Expect an acknowledgement within seven days.

## The threat that matters most

`lup` walks **upward** and returns the first match. Everything above your working
directory is therefore an input, including directories you do not control.

On a shared or multi-user host, `/`, `/Users`, `/home`, `/tmp` and similar
ancestors may be writable by someone else. If an attacker can create a file at a
path `lup` searches for, in an ancestor directory, `lup` will find it and return
it — and the documented usage is `source $(lup .env)` and `eval $(lup -e .env)`.
That turns a planted file into code execution in your shell.

This is the same class of problem as a version-control tool honouring
configuration from a parent directory. It is the reason to read this policy
before scripting `lup` into a login shell.

## What is in scope

- **Escaping an intended boundary.** Any path by which `lup` returns a match
  outside the hierarchy the caller asked it to search, or continues past a
  boundary it was told to stop at.
- **Symlink handling that contradicts the configured follow behaviour.** The
  library exposes follow settings; a match reached in a way those settings should
  have prevented is a bug in scope.
- **Time-of-check to time-of-use races** in the walk, where the path returned is
  not the path that was tested.
- **Path handling defects** — traversal via crafted components, embedded
  separators, or non-UTF-8 components producing a path the caller did not expect.
- **Output that is unsafe to interpolate.** `lup` is documented for use inside
  `$( )`. A returned path that breaks out of shell quoting is in scope.

## What is out of scope

- **Finding a file that legitimately exists in an ancestor directory.** That is
  the entire function of the tool. If an untrusted ancestor is writable, the
  result is untrusted; that is a property of the filesystem, not a defect here.
- **`eval` of found content.** `-e` evaluates what it is given, by design.
- **Anything you `source` being malicious** when it came from a directory you
  chose to search.

## Hardening advice

**Bound the search.** `lup` ships boundary flags for exactly this reason:

- `-g`/`--git` stops at the git root and errors outside a repository, which keeps
  the search inside a tree you presumably control.
- `-r`/`--root` does the opposite — it walks all the way to `/`. Do not combine
  it with `source` or `eval` on a shared host.

If any ancestor of your working directory is writable by another user, do not
`source` or `eval` `lup` output without first checking the returned path's
location and ownership. `-a`/`--all` is useful here: it shows every hit, so you
can see whether something unexpected sits above the one you wanted.
