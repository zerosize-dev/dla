package shop.codec

import java.nio.ByteBuffer

/** Serialises an order into the wire frame the codec library expects. */
object FramePacker {
    const val HEADER_BYTES = 16
    const val LINE_BYTES = 64
    private val MAGIC = byteArrayOf('O'.code.toByte(), 'R'.code.toByte(), 'D'.code.toByte(), '1'.code.toByte())

    fun frameSize(lineCount: Int): Int = HEADER_BYTES + lineCount * LINE_BYTES

    fun pack(order: Order): ByteArray {
        val buf = ByteBuffer.allocate(frameSize(order.lines.size))
        buf.put(MAGIC)
        buf.putShort(1)
        buf.putShort(0)
        buf.putInt(order.lines.size)
        buf.putInt(0)
        for (line in order.lines) {
            val start = buf.position()
            val sku = line.sku.toByteArray(Charsets.US_ASCII)
            buf.put(sku, 0, minOf(sku.size, 16))
            buf.position(start + 16)
            buf.putInt(line.quantity)
            buf.putLong(line.priceCents)
            buf.position(start + LINE_BYTES)
        }
        return buf.array()
    }
}
