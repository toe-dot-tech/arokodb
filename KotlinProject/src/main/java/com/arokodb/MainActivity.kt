package com.arokodb

import android.app.Activity
import android.os.Bundle
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.EditText
import android.widget.Spinner
import android.widget.TextView
import java.util.Locale

class MainActivity : Activity() {
    private lateinit var database: ArokoDB
    private lateinit var status: TextView
    private lateinit var result: TextView
    private lateinit var dimension: EditText
    private lateinit var vectorId: EditText
    private lateinit var vector: EditText
    private lateinit var query: EditText
    private lateinit var topK: EditText
    private lateinit var metric: Spinner

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        status = findViewById(R.id.txt_status)
        result = findViewById(R.id.txt_result)
        dimension = findViewById(R.id.input_dimension)
        vectorId = findViewById(R.id.input_id)
        vector = findViewById(R.id.input_vector)
        query = findViewById(R.id.input_query)
        topK = findViewById(R.id.input_top_k)
        metric = findViewById(R.id.input_metric)

        metric.adapter = ArrayAdapter(
            this,
            android.R.layout.simple_spinner_dropdown_item,
            listOf("Squared Euclidean", "Cosine distance", "Manhattan")
        )

        database = ArokoDB()
        findViewById<Button>(R.id.btn_init).setOnClickListener { resetIndex() }
        findViewById<Button>(R.id.btn_store).setOnClickListener { addVector() }
        findViewById<Button>(R.id.btn_search).setOnClickListener { search() }
        resetIndex()
    }

    private fun resetIndex() {
        val requestedDimension = dimension.text.toString().toIntOrNull()
        if (requestedDimension == null || requestedDimension !in 1..4096) {
            status.text = "Dimension must be a whole number between 1 and 4096."
            return
        }

        try {
            database.initialize(requestedDimension)
            status.text = "Ready · ${database.size()} vectors · dimension $requestedDimension"
            result.text = "Add vectors here, then search for the nearest matches."
        } catch (error: IllegalStateException) {
            status.text = "Could not create index: ${error.message}"
        }
    }

    private fun addVector() {
        val id = vectorId.text.toString().toIntOrNull()
        if (id == null || id < 0) {
            status.text = "Vector ID must be a non-negative whole number."
            return
        }

        val values = parseVector(vector.text.toString()) ?: return
        try {
            database.insert(id, values)
            status.text = "Vector $id added · ${database.size()} vectors in the index"
            vector.text.clear()
            if (id < Int.MAX_VALUE) {
                vectorId.setText((id + 1).toString())
            }
        } catch (error: IllegalArgumentException) {
            status.text = error.message ?: "The vector could not be added."
        } catch (error: IllegalStateException) {
            status.text = "Index error: ${error.message}"
        }
    }

    private fun search() {
        val values = parseVector(query.text.toString()) ?: return
        val requestedK = topK.text.toString().toIntOrNull()
        if (requestedK == null || requestedK <= 0) {
            status.text = "Top K must be a positive whole number."
            return
        }

        try {
            val hits = database.search(values, requestedK, metric.selectedItemPosition)
            status.text = "Found ${hits.size} result(s) · ${database.size()} vectors searched"
            result.text = if (hits.isEmpty()) {
                "No vectors are stored yet."
            } else {
                hits.mapIndexed { index, hit ->
                    "${index + 1}. ID ${hit.id} · distance ${
                        String.format(Locale.US, "%.6f", hit.distance)
                    }"
                }.joinToString("\n")
            }
        } catch (error: IllegalStateException) {
            status.text = "Search error: ${error.message}"
        }
    }

    private fun parseVector(raw: String): FloatArray? {
        val values = raw.split(',').map { it.trim().toFloatOrNull() }
        if (values.any { it == null || !it.isFinite() }) {
            status.text = "Enter only finite numbers separated by commas."
            return null
        }
        val parsed = values.filterNotNull().toFloatArray()
        if (parsed.size != database.dimension) {
            status.text = "Expected ${database.dimension} values, got ${parsed.size}."
            return null
        }
        return parsed
    }
}
