# R.O.S.E. (Rust Ortholog Search Engine)

**R.O.S.E.** is a high-performance, cross-platform tool designed to map orthologous genes between genomes. Utilizing a seed-and-extend approach with sparse alignment heuristics, it provides a fast and accurate way to identify genetic relationships in genomic data.

## 🚀 Key Features

* **Seed & Extend Architecture**: Employs an inverted k-mer index to filter candidates, followed by sparse alignment (`LCSk++`) and final pairwise verification.
* **Parallel Processing**: Built with `rayon` and `tokio` to fully utilize multi-core systems.
* **Cross-Platform GUI**: Built with `iced`, providing a native experience on Windows, macOS, and Linux.
* **Automated Pipeline**: Manages the entire process, from NCBI Entrez retrieval to CSV result export.

## 🛠 Prerequisites

* [Rust/Cargo](https://www.rust-lang.org/tools/install) (latest stable)
* **Linux Users**: Ensure you have GTK and XCB development libraries installed:
    ```bash
    sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
    ```

## 🚀 Build & Run

1.  Clone the repository:
    ```bash
    git clone [https://github.com/tomkys144/rose](https://github.com/tomkys144/rose)
    cd rose
    ```
2.  Build the project:
    ```bash
    cargo build --release
    ```
3.  Run the application:
    ```bash
    cargo run --release
    ```

```bash
git tag v1.0.0
git push origin v1.0.0
```

## ⚖️ License

This project is licensed under the MIT License - see the `LICENSE` file for details.