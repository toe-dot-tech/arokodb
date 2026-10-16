# ArokoDB Windows Vector Studio

A Windows desktop app written in Rust. Its Vector Studio uses the Rust
exact-search core directly, and its industry library explains possible uses
in everyday language. The industry pages are illustrative product ideas, not
claims of deployed integrations or measured business results.

## Run on Windows

Install the Rust MSVC toolchain, then from the repository root run:

```powershell
cargo run --release --manifest-path rust\aroko-windows\Cargo.toml
```

Cargo builds when needed and opens the app. To rebuild without launching:

```powershell
cargo build --release --manifest-path rust\aroko-windows\Cargo.toml
```

Launch the built executable directly:

```powershell
.\rust\aroko-windows\target\release\aroko-windows.exe
```

## Use

- The app opens in **Light** appearance by default. Use the sun/moon control at
  the bottom of the left activity bar to switch between light and dark modes.
- Use the VS Code-style activity bar on the left to switch between the vector
  workspace and industry ideas. Select the active icon again to collapse the
  adjacent navigation panel. Navigation icons and labels are left-aligned, and
  the navigation panel can be resized. The ArokoDB artwork is used as the
  Windows window/taskbar icon.
- Resize the window down to its smaller supported layout: Vector Studio stacks
  the add/search cards, industry examples adapt their grid to the available
  width, and the industry strip scrolls horizontally when needed. The sidebar
  can also be dragged narrower or wider.
- Open **Vector studio** to add vectors, search, and import/export CSV.
- Set a vector dimension before adding vectors. Clear the index before changing
  dimensions after vectors have been added.
- Enter a unique vector ID and exactly the configured number of comma-separated
  values, then select **Add vector**.
- Select the Sparkles icon beside the vector-values field to fill it with random
  sample numbers for testing. This does not create semantic embeddings; use
  vectors produced by an embedding model for meaning-based search.
- Enter a same-dimension query, select Top K and squared Euclidean, cosine, or
  Manhattan distance, then select **Search**. Lower distances are closer.
- The **Search particle map** is at the top of Vector Studio. After searching,
  it animates the exact scan: the query crawls through every vector, spokes show
  comparisons, and the nearest matches glow. It plots the first two vector
  dimensions, so it is an explanatory projection rather than the distance
  metric or an HNSW graph traversal.
- Use **Try a crawl search** to load eight sample vectors and the query
  `0.80, 0.20, 0.10`, then enter that query in Vector Studio and select
  **Search** to watch the scan. The samples are arbitrary numbers for
  demonstrating the backend, not semantic embeddings. The sample button only
  loads into an empty collection; clear your current vectors first.
- The interface uses the bundled Lucide icon font, so its navigation and
  action icons render consistently without depending on Windows emoji fonts.
- Enter a CSV path and select **Import CSV** or **Export CSV**. Files use a
  header like `id,v1,v2,v3`, followed by one ID/vector per line. Only plain
  numeric CSV fields are supported.
- Browse **Industry ideas** for plain-language examples across finance,
  cryptocurrency, healthcare, news and digital media, AI, social and
  community, fitness, and education. Each page explains what might be saved,
  what a person could search for, and why local search might help.
- Select **Try this example** to load made-up three-number vectors and see a
  nearest-match search in the studio. These values are teaching examples, not
  real text embeddings, production integrations, or industry performance
  claims.

Vectors are held in memory and are lost when the app closes unless exported.
The app is exact-search (`O(number of vectors × dimension)`), not HNSW.
