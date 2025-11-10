# fig2json

Convert Figma `.fig` files to clean, optimized JSON for AI-powered design implementation.

## Overview

`fig2json` extracts and transforms Figma design files into structured JSON, making it easy for LLMs to understand and implement your designs. The tool removes Figma-specific metadata and applies intelligent transformations to produce JSON optimized for HTML/CSS conversion.

## How to Use

### 1. Save Your Figma Design Locally

In Figma, go to the top-left menu:

- Click **File** → **Save local copy...**
- Save the `.fig` file to your computer

### 2. Convert to JSON

Run `fig2json` on your saved file:

```bash
fig2json your-design.fig output-directory
```

This will:

- Extract all contents to `output-directory/`
- Convert the design to `output-directory/canvas.json`

### 3. Implement with AI

Ask your LLM to implement the design:

```
Implement the design found in output-directory/canvas.json
```

The clean JSON structure makes it easier for AI to understand your design and hopefully generate accurate HTML/CSS.

**Tip:** For easier LLM consumption, consider using [jq](https://github.com/jqlang/jq) to extract specific parts of the design or to further process the JSON before sending it to your LLM.

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
# Binary will be at target/release/fig2json
```

## Usage

### Basic Usage

Convert a `.fig` file and extract to a directory:

```bash
fig2json design.fig output-dir
```

Convert a `.fig` file to stdout:

```bash
fig2json design.fig
```

Convert to a specific output file:

```bash
fig2json design.fig -o output.json
```

### Command Line Flags

| Flag                  | Description                                                                                                                      |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `-o, --output <FILE>` | Output JSON file path (default: stdout). Cannot be used with extract directory mode.                                             |
| `--compact`           | Output compact JSON instead of pretty-printed (default is indented).                                                             |
| `-v, --verbose`       | Enable verbose output for debugging.                                                                                             |
| `--raw`               | Generate both transformed `.json` and raw `.raw.json` files. The raw version contains the original data without transformations. |

### Examples

**Extract and convert with verbose output:**

```bash
fig2json design.fig output-dir --verbose
```

**Compact JSON output:**

```bash
fig2json design.fig -o compact.json --compact
```

**Generate both transformed and raw JSON:**

```bash
fig2json design.fig output-dir --raw
# Creates: output-dir/canvas.json and output-dir/canvas.raw.json
```

**Pipe to other tools:**

```bash
fig2json design.fig | jq '.document.children[0]'
```

## Transformations

The tool applies intelligent transformations to clean up the JSON:

- **Removes default values**: `blendMode: "NORMAL"`, default letter spacing, line height
- **Removes Figma-specific metadata**: Internal IDs, text data, image thumbnails
- **Removes redundant fields**: Derived layout sizes, empty font properties
- **Filters internal nodes**: Removes `internalOnly` elements
- **Preserves geometry**: Keeps SVG paths for icons and images
- **Optimizes structure**: Only essential fields for HTML/CSS rendering

Use the `--raw` flag to also generate the untransformed JSON for comparison.

## Output Structure

After extraction, you'll find:

```
output-directory/
├── canvas.json          # Main design file (transformed)
├── canvas.raw.json      # Raw untransformed data (if --raw flag used)
└── [other extracted files]
```

The `canvas.json` file contains the complete design tree with all layers, styles, and properties needed for implementation.

## FAQ

### What inspired this project?

This project was inspired by [Evan Wallace's Figma File Parser](https://madebyevan.com/figma/fig-file-parser/).

### How was this built?

This project was mostly vibe-coded with Claude Code.

### What's the difference between transformed and raw JSON?

- **Transformed JSON** (default): Clean, optimized structure with Figma metadata removed and defaults stripped. Best for AI consumption and HTML/CSS generation.
- **Raw JSON** (`--raw` flag): Original decoded data with all Figma-specific fields intact. Useful for debugging or advanced use cases.

### Does this work with Figma plugins?

No, this tool works with locally saved `.fig` files. You need to use Figma's "Save local copy" feature to export the file first.

## License

MIT
