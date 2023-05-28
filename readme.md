# changelogs

A set of (relatively) simple GitHub actions for changelog generation from git commit bodies.

- `necauqua/changelogs/extract@v1` - parses git commit bodies and produces a JSON file with changelog data. Absolute overkill, look at that thing :)

  It looks for tags that start with `v`, the `main` branch and any branches that start with `backport/`.

  Out of these it builds a tree structure, with tags being releases and branches being unreleased trunks.

  Then it reads bodies of git commits between every tag/branch and it's parent tag/root and looks for the pattern like below in them:
  ```
  ${section1}:
    - ${entry1}
    - ${entry2}
    - ..
    - ${entryM}
  ${section2}:
    - ..
  ..
  ${sectionN}:
    - ..
  ```

  Sections from different commits are then merged by their lowercase names and sorted.
  Common sections are *added*, *changed*, *fixed*, *security* etc. Entries are just some changes.

  The top-level list of per tag changes is sorted so be newest first.

  <details>
  <summary>The format example (not a schema cuz im lazy and this is for personal use anyway lol)</summary>

  ```json
   [
      {
        "is_release": "true if it was a tag",
        "name": "git short ref name, e.g. name of the tag or of the branch",
        "hash": "hash of the git commit (not the tag object)",
        "timestamp": "the git timestamp in seconds of the annotated tag if present, or of the commit otherwise",
        "sections": [
          {
            "name": "section name, e.g. 'added'",
            "entries": [
              "a line describing some change, e.g. 'added herobrine', see the pattern above",
              ..
            ]
          },
          ..
        ]
      },
      ..
   ]
  ```
  </details>

- `necauqua/changelogs/render@v1` - accepts the changelog json from above to render it into a simple markdown. See its action inputs for configuration options.

- `necauqua/changelogs/gen-forge-updates@v1` - A Minecraft Forge specific action, that generates the `updates.json` needed for the update checker API from the above changelog JSON.

### But why Rust tho
Idk, better than Python, and waaay better than JS. It's weird how GitHub Actions don't allow TS actions natively.
