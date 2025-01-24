# NeuroPAL Lens

[![dependency status](https://deps.rs/repo/github/lycantrope/neuropal_lens/status.svg)](https://deps.rs/repo/github/lycantrope/neuropal_lens)
[![Build Status](https://github.com/lycantrope/neuropal_lens/workflows/CI/badge.svg)](https://github.com/lycantrope/neuropal_lens/actions?workflow=CI)

NeuroPAL Lens is a Rust-based application designed to facilitate the visualization and analysis of neurons in *Caenorhabditis elegans* labeled using the NeuroPAL system. This app enables users to check neuron [colors](10.1016/j.cell.2020.12.012) and [positions](10.1186/s12859-022-04738-3).

## Features

- **NeuronPAL Color:** Inspect neuron color labeling using the NeuroPAL system.
- **Neuron Position:** Verify neuron positions within the organism.
- **Web-Based Interface:** Access a lightweight, interactive web app.

## Web App

Try the live version of NeuroPAL Lens: [https://lycantrope.github.io/neuropal_lens/](https://lycantrope.github.io/neuropal_lens/)

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) installed on your system.
- A modern web browser to access the web app.

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/lycantrope/neuropal_lens.git
   cd neuropal_lens
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. Run the application locally:
   ```bash
   cargo run
   ```

## Usage

- Launch the web app.
- Explore neuron positions and verify colors interactively.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **[NeuroPAL](https://www.hobertlab.org/neuropal/):** The NeuroPAL system for neuron identification.

---

