package shop.codec

class EncodeResult(val orderId: String, val checksum: Long, val frameBytes: Int)

/** Hands order frames to the codec library and records the outcome. */
class OrderEncoder(private val stats: Stats) {
    fun encode(order: Order): EncodeResult {
        val frame = FramePacker.pack(order)
        val handle = Native.open(frame)
        val sum = Native.checksum(handle)
        val size = Native.size(handle)
        if (size > MAX_INLINE_BYTES) {
            // Wholesale frames go to the batch lane and are audited there.
            stats.bulkFrames++
            return EncodeResult(order.id, sum, size)
        }
        Audit.record(order.id, sum, frame)
        Native.close(handle)
        stats.inlineFrames++
        return EncodeResult(order.id, sum, size)
    }

    companion object {
        const val MAX_INLINE_BYTES = 16384
    }
}
