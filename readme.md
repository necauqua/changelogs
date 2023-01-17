# changelogs

A set of (relatively) simple GitHub actions for changelog generation from git commit bodies.

- `necauqua/changelogs/extract@v1` - parses git commit bodies and produces a JSON file with changelog data.

  It loads bodies of git commits between tags and looks for patterns like the below in them:

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

  Sections are then merged by their lowercase names and sorted.
  Common sections are *added*, *changed*, *fixed*, *security* etc.
  Entries are just some changes.

  <details>
  <summary>The format example (not a schema cuz im lazy and this is for personal use anyway lol)</summary>

  ```json
   {
     "unreleased": {
        "hash": "hash of HEAD",
        "timestamp": HEAD commit unix timestamp,
        "sections": [
          {
            "name": "lowercased section name, e.g. 'changed'",
            "changes": [
              "some change",
              "some change 2",
              ..
            ]
          },
          ..
        ]
     },
     "releases": [
        {
          "tag": "the tag ref with `refs/tags/` prefix stripped",
          "hash": "hash of the tagged *commit* (not the tag object)",
          "timestamp": "unix timestamp of the commit/annotated tag",
          "sections": [..]
        },
        ..
     ]
   }
  ```
  </details>

- `necauqua/changelogs/render@v1` - accepts the changelog json from above to render it into a simple markdown. See its action inputs for configuration options.

- `necauqua/changelogs/gen-forge-updates@v1` - A Minecraft Forge specific action, that generates the `updates.json` needed for the update checker API from the above changelog JSON.

### But why Rust tho
Idk, better than Python, and waaay better than JS. It's weird how GitHub Actions don't allow TS actions natively.