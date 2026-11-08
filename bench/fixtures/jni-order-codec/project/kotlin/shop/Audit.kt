package shop.codec

class Entry(val id: String, val digest: Long, val sample: ByteArray)

/** Keeps a short sample of each audited frame for the nightly reconciliation job. */
object Audit {
    const val SAMPLE_BYTES = 1536

    val trail = ArrayList<Entry>()

    fun record(id: String, digest: Long, payload: ByteArray) {
        trail.add(Entry(id, digest, payload.copyOf(SAMPLE_BYTES)))
    }

    fun size(): Int = trail.size
}
