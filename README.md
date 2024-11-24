# Socketboard Server

A Rust-based server application designed for debugging and managing remote projects by allowing clients to view and edit variables in real-time. This system supports multiple clients with customizable permissions, making it ideal for development and monitoring purposes.

---

## Features

- **Real-time Variable Management:** 
  View, modify, and monitor variables from a central dashboard.
  
- **Multi-language Client Support:** 
  Interfaces available (soon) for Python, Rust, JavaScript, and Java.
  
- **Permissions System (coming soon):** 
  Define client permissions for read-only, read/write, and restricted variable access.

- **Web Dashboard (coming soon):** 
  A simple and customizable UI for monitoring and editing variables.

---

## Getting Started

### Prerequisites

- Rust (for building the server)
- Web browser (for accessing the dashboard)
- Supported language tools for clients (Python, Node.js, Java, etc.)

### Installation

You can either download the pre-built binaries or build the server from source.

1. **Download Binaries:**
   - Download the latest release from the [Releases](https://github.com/socketboard/socketboard/releases) page.
    - Extract the contents to a directory of your choice.
    - Run the server executable for your platform.
      - In the future, the executable may be run with arguments to specify the host and port, as well as inital state files.

2. **Build from Source:**
    - Clone the repository: `git clone https://github.com/socketboard/socketboard.git`
    - Navigate to the project directory: `cd socketboard`
    - Build the server: `cargo build --release`
    - Run the server: `./target/release/socketboard`

---

### Usage

1. **Start the Server:**
    - Run the server executable.
    - By default, the server will start on `localhost:8080`.

2. **Access the Dashboard:**
    - Open a web browser and navigate to `http://localhost:8080`.
    - You should see the dashboard with a list of connected clients.
