package com.arokodb

data class SearchHit(val id: Int, val distance: Float)

class ArokoDB {
    var dimension: Int = 0
        private set

    fun initialize(dimension: Int) {
        check(dimension in 1..4096) { "Dimension must be between 1 and 4096." }
        check(nativeInitialize(dimension)) { "The native Rust index could not be initialized." }
        this.dimension = dimension
    }

    fun insert(id: Int, vector: FloatArray) {
        require(id >= 0) { "Vector ID must be non-negative." }
        require(vector.size == dimension) { "Expected $dimension values, got ${vector.size}." }
        require(vector.all(Float::isFinite)) { "Vector values must be finite." }
        check(nativeInsert(id, vector)) { "Vector ID $id already exists or the vector was rejected." }
    }

    fun search(query: FloatArray, k: Int, metric: Int): List<SearchHit> {
        require(query.size == dimension) { "Expected $dimension values, got ${query.size}." }
        require(query.all(Float::isFinite)) { "Query values must be finite." }
        require(k > 0) { "Top K must be positive." }
        require(metric in 0..2) { "Unsupported distance metric." }

        val encoded = checkNotNull(nativeSearch(query, k, metric)) {
            "The native Rust search failed."
        }
        return encoded.asList().chunked(2).map { (id, distanceBits) ->
            SearchHit(id.toInt(), Float.fromBits(distanceBits.toInt()))
        }
    }

    fun size(): Int = nativeSize()

    private external fun nativeInitialize(dimension: Int): Boolean
    private external fun nativeInsert(id: Int, vector: FloatArray): Boolean
    private external fun nativeSearch(query: FloatArray, k: Int, metric: Int): LongArray?
    private external fun nativeSize(): Int

    companion object {
        init {
            System.loadLibrary("aroko_ffi")
        }
    }
}
