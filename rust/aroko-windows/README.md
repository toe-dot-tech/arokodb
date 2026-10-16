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

- Use the VS Code-style activity bar on the left to switch between the vector
  workspace and industry ideas. Select the active icon again to collapse the
  adjacent navigation panel.
- Open **Vector studio** to add vectors, search, and import/export CSV.
- Set a vector dimension before adding vectors. Clear the index before changing
  dimensions after vectors have been added.
- Enter a unique vector ID and exactly the configured number of comma-separated
  values, then select **Add vector**.
- Select the ✦ button beside the vector-values field to fill it with random
  sample numbers for testing. This does not create semantic embeddings; use
  vectors produced by an embedding model for meaning-based search.
- Enter a same-dimension query, select Top K and squared Euclidean, cosine, or
  Manhattan distance, then select **Search**. Lower distances are closer.
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
