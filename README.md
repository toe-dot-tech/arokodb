# ArokoDB — Embedded Vector Database Engine

**RFC Specification:** RFC-0014  
**Status:** IMPLEMENTATION / ARCHITECTURAL SPEC  
**Author:** Emmanuel Orimoloye ([@toe-dot-tech](https://github.com/toe-dot-tech))  
**Core Language:** Rust (C/Dart FFI Bindings)  
**Target Runtimes:** iOS, Android, macOS, Linux, Windows  

---

## 1. System Overview & Problem Statement

Modern mobile and edge applications increasingly depend on real-time semantic search, local Retrieval-Augmented Generation (RAG), and on-device user embeddings. Existing vector databases (e.g., Qdrant, Milvus, Pinecone) are architected as network-attached, client-server cloud systems. When bundled into mobile runtimes, they suffer from severe operational constraints:

1. **High Active Memory Footprint:** Loading high-dimensional floating-point index structures (e.g., 100,000 vectors at 768 dimensions = ~300 MB raw `f32` data) into mobile process RAM triggers OS Out-Of-Memory (OOM) kills.
2. **Cold-Start Allocation Overhead:** Deserializing large vector graphs from disk into heap allocations at startup introduces multi-second main-thread stalls.
3. **IPC / Serialization Bottlenecks:** Passing high-dimensional embeddings across native application boundaries (e.g., via JSON or Protobuf IPC) saturates process memory bandwidth.

**ArokoDB** solves these constraints by providing an embedded, zero-copy, memory-mapped (`mmap`) vector engine written in Rust. It delivers <12ms p99 recall latency across 100,000 768-dimensional vectors while capping active RAM usage below **35 MB**.

---

## 2. System Architecture

ArokoDB operates in-process alongside the host application runtime (Flutter/Dart, Swift, or C++), communicating directly over FFI via shared memory pointers.


```

+-------------------------------------------------------------------------+
|                         Host Application Process                        |
|                                                                         |
|  +---------------------+      +--------------------------------------+  |
|  | Flutter / Swift UI  |      |   Local Embedding Model (e.g., ONNX) |  |
|  +----------+----------+      +------------------+-------------------+  |
|             |                                    |                      |
|             | Query Pointer                      | 768-dim f32 vector   |
|             v                                    v                      |
|  +-------------------------------------------------------------------+  |
|  |                    ArokoDB C / FFI Binding Layer                   |  |
|  +----------------------------------+--------------------------------+  |
|                                     |                                   |
+-------------------------------------|-----------------------------------+
| Direct In-Memory FFI Call
v
+-------------------------------------------------------------------------+
|                       ArokoDB Native Core Engine (Rust)                 |
|                                                                         |
|  +-------------------------------------------------------------------+  |
|  |                 SIMD Vector Distance Engine                       |  |
|  |        (ARM NEON / x86 AVX2 Dot Product, Cosine, L2 Distance)     |  |
|  +----------------------------------+--------------------------------+  |
|                                     |                                   |
|  +----------------------------------v--------------------------------+  |
|  |             Hierarchical Navigable Small World (HNSW)             |  |
|  |               Graph Index Execution Engine (M=16)                 |  |
|  +----------------------------------+--------------------------------+  |
|                                     |                                   |
|  +----------------------------------v--------------------------------+  |
|  |             Zero-Copy Memory-Mapped Storage Layer                 |  |
|  |              (mmap Engine with Scalar Quantization SQ8)           |  |
|  +----------------------------------+--------------------------------+  |
|                                     |                                   |
+-------------------------------------|-----------------------------------+
|
v
+---------------------------------+
| Persistent Disk File (`.aroko`) |
+---------------------------------+

```

---

## 3. Storage Engine & Memory Allocation Model

### 3.1 Memory-Mapped Page Layout (`mmap`)

Rather than allocating index vectors on the heap, ArokoDB maps its custom single-file storage format (`.aroko`) directly into the process's virtual address space using OS page tables.


```

+-----------------------------------------------------------------------+
| File Header (64 Bytes)   | Magic Bytes, Version, Dimension, Metric    |
+--------------------------+--------------------------------------------+
| Vector Data Segment      | Contiguous Quantized Vector Blobs (SQ8/FP16)|
+--------------------------+--------------------------------------------+
| HNSW Graph Entrypoints   | Layer-wise Node Link Lists & Offsets       |
+--------------------------+--------------------------------------------+
| Write-Ahead Log (WAL)    | Uncommitted Delta Ingestion Buffer         |
+-----------------------------------------------------------------------+

```

* **Virtual Address Alignment:** All vector data offsets are 64-byte aligned to match SIMD cache-line requirements.
* **Lazy Page Page-In:** The Operating System manages physical RAM allocations via `madvise(MADV_RANDOM)`. Unrequested vector clusters are never paged into physical memory.
* **Instant Cold Starts:** Initializing the database takes **< 1.5ms**, requiring only a file descriptor open and virtual space mapping without disk-to-heap parsing.

---

## 4. SIMD Acceleration & Vector Quantization

### 4.1 Scalar Quantization (SQ8)

To compress embedding sizes by 75% without compromising recall accuracy, ArokoDB converts standard 32-bit floating point values (`f32`) to 8-bit unsigned integers (`u8`):

$$q_i = \lfloor \frac{v_i - v_{\min}}{v_{\max} - v_{\min}} \times 255 \rceil$$

This reduces a 768-dimensional vector footprint from **3,072 bytes to 768 bytes**, allowing 4x more vectors to fit within the CPU L2/L3 cache.

### 4.2 SIMD Inner Product Engine

Distance calculations are executed via inline assembly / intrinsic bindings tailored to the host CPU architecture:

* **ARM v8+ (Mobile):** NEON vector instructions (`vdotq_u32`, `vld1q_u8`) computing 16 dot-product operations per clock cycle.
* **x86_64 (Desktop):** AVX2/FMA instructions computing 32 dot-product operations per clock cycle.

---

## 5. Concurrency & Thread Safety

ArokoDB utilizes a **Single-Writer, Multiple-Reader (SWMR)** concurrency model:



```
              +-----------------------------------+
              |          Writer Thread            |
              +-----------------+-----------------+
                                |
                                v
              +-----------------------------------+
              |      Write-Ahead Log (WAL)        |
              +-----------------+-----------------+
                                |
                                | Atomic Pointer Swap
                                v

+-----------------------------------------------------------------------+
|                      Read-Only HNSW Graph (mmap)                      |
|                                                                       |
|   +-------------------+  +-------------------+  +-------------------+ |
|   | Reader Thread 1   |  | Reader Thread 2   |  | Reader Thread N   | |
|   +-------------------+  +-------------------+  +-------------------+ |
+-----------------------------------------------------------------------+

```

1. **Lock-Free Reader Operations:** Reader threads traverse the memory-mapped HNSW graph without acquiring mutexes or rwlocks, eliminating reader-writer contention.
2. **Copy-On-Write (COW) Index Updates:** Graph mutations occur inside an isolated staging memory region. Once a batch mutation completes, the entry-node pointer is atomically updated using `std::sync::atomic::AtomicU64`.

---

## 6. Performance Benchmarks

### Benchmark Environment
* **Hardware:** Apple M2 Pro (Mobile Edge Profile) / Intel i7-13700K
* **Dataset:** 100,000 Vectors, 768 Dimensions (`f32` normalized)
* **Metric:** Cosine Distance, Top-10 Nearest Neighbors ($K=10$)

| Engine | Storage Mode | Cold Start Latency | p50 Latency | p99 Latency | Active Memory Footprint |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **ArokoDB (SQ8 + mmap)** | Memory-Mapped File | **1.2 ms** | **3.8 ms** | **11.4 ms** | **31.2 MB** |
| Standard In-Memory HNSW | Heap Memory Allocation | 840 ms | 2.1 ms | 8.2 ms | 342.0 MB |
| SQLite (Vector Extension) | Disk B-Tree | 12.0 ms | 48.6 ms | 124.0 ms | 45.0 MB |

---

## 7. FFI & API Usage Example

### Rust Core API
```rust
use arokodb::{Database, DistanceMetric, QueryOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open memory-mapped vector database
    let db = Database::open("./vectors.aroko", DistanceMetric::Cosine)?;

    // 768-dimensional query vector
    let query_vector: Vec<f32> = vec![0.023; 768];

    // Execute ANN search (Top-5)
    let results = db.query(
        &query_vector, 
        5, 
        QueryOptions { ef_search: 64 }
    )?;

    for result in results {
        println!("ID: {}, Distance: {:.4}", result.id, result.distance);
    }

    Ok(())
}

```

### Native C FFI Header (`arokodb.h`)

```c
#ifndef AROKODB_H
#define AROKODB_H

#include <stdint.h>

typedef struct ArokoDB ArokoDB;

typedef struct {
    uint64_t id;
    float distance;
} ArokoQueryResult;

ArokoDB* aroko_open(const char* path, uint8_t metric_type);
int32_t aroko_query(
    const ArokoDB* db, 
    const float* vector_ptr, 
    uint32_t dim, 
    uint32_t k, 
    ArokoQueryResult* out_results
);
void aroko_close(ArokoDB* db);

#endif

```
